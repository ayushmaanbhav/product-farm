//! DAG (Directed Acyclic Graph) for rule dependencies
//!
//! This module provides:
//! - Construction of dependency graph from rules
//! - Topological sorting for execution order
//! - Cycle detection
//! - Parallel execution group computation

use hashbrown::{HashMap, HashSet};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort;
use petgraph::Direction;
use product_farm_core::{Rule, RuleId};
use crate::error::{RuleEngineError, RuleEngineResult};

/// A node in the rule dependency graph
#[derive(Debug, Clone)]
pub struct RuleNode {
    /// The rule ID
    pub id: RuleId,
    /// Input attribute paths required by this rule
    pub inputs: Vec<String>,
    /// Output attribute paths produced by this rule
    pub outputs: Vec<String>,
    /// Order index for execution ordering (lower = earlier)
    pub order_index: i32,
}

impl RuleNode {
    /// Create from a Rule
    pub fn from_rule(rule: &Rule) -> Self {
        Self {
            id: rule.id.clone(),
            inputs: rule.input_attributes.iter().map(|p| p.path.as_str().to_string()).collect(),
            outputs: rule.output_attributes.iter().map(|p| p.path.as_str().to_string()).collect(),
            order_index: rule.order_index,
        }
    }
}

/// Dependency graph for rules
#[derive(Debug)]
pub struct RuleDag {
    /// The petgraph directed graph
    graph: DiGraph<RuleNode, ()>,
    /// Map from rule ID to node index
    rule_to_node: HashMap<RuleId, NodeIndex>,
    /// Map from output attribute to producing rule
    output_to_rule: HashMap<String, RuleId>,
}

impl RuleDag {
    /// Create a new empty DAG
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            rule_to_node: HashMap::new(),
            output_to_rule: HashMap::new(),
        }
    }

    /// Build a DAG from a list of rules
    pub fn from_rules(rules: &[Rule]) -> RuleEngineResult<Self> {
        let mut dag = Self::new();

        // First pass: add all nodes
        for rule in rules {
            dag.add_rule(rule)?;
        }

        // Second pass: add edges based on dependencies
        dag.build_edges()?;

        // Check for cycles using toposort (iterative, unlike is_cyclic_directed which is recursive)
        // This avoids stack overflow with deep dependency chains (100k+ levels)
        if let Err(cycle) = toposort(&dag.graph, None) {
            let node = &dag.graph[cycle.node_id()];
            return Err(RuleEngineError::CyclicDependency(format!(
                "Cycle detected at rule: {:?}",
                node.id
            )));
        }

        Ok(dag)
    }

    /// Add a rule to the DAG
    fn add_rule(&mut self, rule: &Rule) -> RuleEngineResult<()> {
        let node = RuleNode::from_rule(rule);

        // Check for duplicate outputs
        for output in &node.outputs {
            if self.output_to_rule.contains_key(output) {
                return Err(RuleEngineError::InvalidConfiguration(format!(
                    "Multiple rules produce the same output: {}",
                    output
                )));
            }
        }

        let idx = self.graph.add_node(node.clone());
        self.rule_to_node.insert(rule.id.clone(), idx);

        // Register all outputs
        for output in &node.outputs {
            self.output_to_rule.insert(output.clone(), rule.id.clone());
        }

        Ok(())
    }

    /// Build edges based on input/output dependencies
    fn build_edges(&mut self) -> RuleEngineResult<()> {
        // Collect edges to add (can't modify graph while iterating)
        let mut edges = Vec::new();

        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            let rule_id = node.id.clone();

            for input in &node.inputs {
                // Check if this input is produced by another rule
                if let Some(producer_id) = self.output_to_rule.get(input) {
                    if producer_id != &rule_id {
                        if let Some(&producer_idx) = self.rule_to_node.get(producer_id) {
                            // Edge from producer to consumer
                            edges.push((producer_idx, idx));
                        }
                    }
                }
            }
        }

        for (from, to) in edges {
            self.graph.add_edge(from, to, ());
        }

        Ok(())
    }

    /// Get topologically sorted rule IDs
    pub fn topological_order(&self) -> RuleEngineResult<Vec<RuleId>> {
        let sorted = toposort(&self.graph, None)
            .map_err(|cycle| {
                let node = &self.graph[cycle.node_id()];
                RuleEngineError::CyclicDependency(format!(
                    "Cycle detected at rule: {:?}",
                    node.id
                ))
            })?;

        Ok(sorted.into_iter().map(|idx| self.graph[idx].id.clone()).collect())
    }

    /// Get execution levels for parallel execution
    ///
    /// Rules in the same level have no dependencies on each other
    /// and can be executed in parallel.
    pub fn execution_levels(&self) -> RuleEngineResult<Vec<Vec<RuleId>>> {
        let order = self.topological_order()?;
        if order.is_empty() {
            return Ok(vec![]);
        }

        // Calculate level for each node
        let mut levels: HashMap<RuleId, usize> = HashMap::new();

        for rule_id in &order {
            let idx = self.rule_to_node[rule_id];

            // Level is max level of all dependencies + 1
            let max_dep_level = self.graph
                .neighbors_directed(idx, Direction::Incoming)
                .map(|dep_idx| levels.get(&self.graph[dep_idx].id).copied().unwrap_or(0))
                .max()
                .unwrap_or(0);

            let level = if self.graph.neighbors_directed(idx, Direction::Incoming).count() == 0 {
                0
            } else {
                max_dep_level + 1
            };

            levels.insert(rule_id.clone(), level);
        }

        // Group by level
        let max_level = levels.values().copied().max().unwrap_or(0);
        let mut result = vec![Vec::new(); max_level + 1];

        for (rule_id, level) in levels {
            result[level].push(rule_id);
        }

        // Sort within each level by order_index
        for level in &mut result {
            level.sort_by(|a, b| {
                let a_node = &self.graph[self.rule_to_node[a]];
                let b_node = &self.graph[self.rule_to_node[b]];
                a_node.order_index.cmp(&b_node.order_index) // Lower index first
            });
        }

        Ok(result)
    }

    /// Get the rule node by ID
    pub fn get_rule(&self, id: &RuleId) -> Option<&RuleNode> {
        self.rule_to_node.get(id).map(|&idx| &self.graph[idx])
    }

    /// Get dependencies of a rule (rules that must execute before it)
    pub fn dependencies(&self, id: &RuleId) -> Vec<RuleId> {
        self.rule_to_node
            .get(id)
            .map(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Incoming)
                    .map(|dep_idx| self.graph[dep_idx].id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get dependents of a rule (rules that depend on it)
    pub fn dependents(&self, id: &RuleId) -> Vec<RuleId> {
        self.rule_to_node
            .get(id)
            .map(|&idx| {
                self.graph
                    .neighbors_directed(idx, Direction::Outgoing)
                    .map(|dep_idx| self.graph[dep_idx].id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a rule ID exists in the DAG
    pub fn contains(&self, id: &RuleId) -> bool {
        self.rule_to_node.contains_key(id)
    }

    /// Get the number of rules in the DAG
    pub fn len(&self) -> usize {
        self.graph.node_count()
    }

    /// Check if the DAG is empty
    pub fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    /// Get all rule IDs
    pub fn rule_ids(&self) -> Vec<RuleId> {
        self.rule_to_node.keys().cloned().collect()
    }

    /// Find missing inputs (inputs not provided by any rule or external data)
    pub fn find_missing_inputs(&self, available_inputs: &HashSet<String>) -> Vec<(RuleId, String)> {
        let mut missing = Vec::new();

        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            for input in &node.inputs {
                // Check if input is produced by another rule or is available externally
                if !self.output_to_rule.contains_key(input) && !available_inputs.contains(input) {
                    missing.push((node.id.clone(), input.clone()));
                }
            }
        }

        missing
    }

    /// Generate a structured execution plan
    pub fn execution_plan(&self) -> RuleEngineResult<ExecutionPlan> {
        let levels = self.execution_levels()?;

        let stages: Vec<ExecutionStage> = levels
            .into_iter()
            .enumerate()
            .map(|(idx, rule_ids)| {
                let rules: Vec<ExecutionPlanRule> = rule_ids
                    .into_iter()
                    .map(|id| {
                        let node = self.get_rule(&id).unwrap();
                        ExecutionPlanRule {
                            id: id.to_string(),
                            inputs: node.inputs.clone(),
                            outputs: node.outputs.clone(),
                            dependencies: self.dependencies(&id).iter().map(|d| d.to_string()).collect(),
                        }
                    })
                    .collect();

                ExecutionStage {
                    level: idx,
                    parallel: rules.len() > 1,
                    rules,
                }
            })
            .collect();

        Ok(ExecutionPlan {
            total_rules: self.len(),
            total_stages: stages.len(),
            stages,
        })
    }

    /// Export to DOT format (for Graphviz visualization)
    pub fn to_dot(&self) -> String {
        let mut dot = String::from("digraph RuleDag {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=rounded];\n\n");

        // Add nodes with labels
        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            let node_id = node.id.to_string();
            let label = format!(
                "{}\\n[in: {}]\\n[out: {}]",
                node_id,
                node.inputs.join(", "),
                node.outputs.join(", ")
            );
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\"];\n",
                node_id, label
            ));
        }

        dot.push('\n');

        // Add edges
        for edge in self.graph.edge_indices() {
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            let from_id = self.graph[from].id.to_string();
            let to_id = self.graph[to].id.to_string();
            dot.push_str(&format!("  \"{}\" -> \"{}\";\n", from_id, to_id));
        }

        dot.push_str("}\n");
        dot
    }

    /// Export to Mermaid format (for markdown documentation)
    pub fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TD\n");

        // Add nodes with labels
        for idx in self.graph.node_indices() {
            let node = &self.graph[idx];
            let node_id = node.id.to_string();
            let safe_id = node_id.replace('-', "_");
            let label = format!(
                "{}\\n[{}] → [{}]",
                node_id,
                node.inputs.join(", "),
                node.outputs.join(", ")
            );
            mermaid.push_str(&format!("  {}[\"{}\"]\n", safe_id, label));
        }

        // Add edges
        for edge in self.graph.edge_indices() {
            let (from, to) = self.graph.edge_endpoints(edge).unwrap();
            let from_id = self.graph[from].id.to_string().replace('-', "_");
            let to_id = self.graph[to].id.to_string().replace('-', "_");
            mermaid.push_str(&format!("  {} --> {}\n", from_id, to_id));
        }

        mermaid
    }

    /// Export to ASCII art representation
    pub fn to_ascii(&self) -> RuleEngineResult<String> {
        let levels = self.execution_levels()?;
        let mut output = String::new();

        output.push_str("Execution Plan\n");
        output.push_str("==============\n\n");

        for (level_idx, rule_ids) in levels.iter().enumerate() {
            let parallel = if rule_ids.len() > 1 { " (parallel)" } else { "" };
            output.push_str(&format!("Stage {}{}:\n", level_idx, parallel));

            for rule_id in rule_ids {
                let node = self.get_rule(rule_id).unwrap();
                output.push_str(&format!("  ├─ {}\n", rule_id));
                output.push_str(&format!("  │   inputs:  [{}]\n", node.inputs.join(", ")));
                output.push_str(&format!("  │   outputs: [{}]\n", node.outputs.join(", ")));

                let deps = self.dependencies(rule_id);
                if !deps.is_empty() {
                    output.push_str(&format!(
                        "  │   depends: [{}]\n",
                        deps.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")
                    ));
                }
            }
            output.push('\n');
        }

        Ok(output)
    }
}

/// Structured execution plan for API responses
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionPlan {
    /// Total number of rules
    pub total_rules: usize,
    /// Total number of execution stages
    pub total_stages: usize,
    /// Execution stages in order
    pub stages: Vec<ExecutionStage>,
}

/// A stage of execution (rules at the same level)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStage {
    /// Stage level (0-indexed)
    pub level: usize,
    /// Whether rules in this stage can run in parallel
    pub parallel: bool,
    /// Rules in this stage
    pub rules: Vec<ExecutionPlanRule>,
}

/// A rule in the execution plan
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionPlanRule {
    /// Rule ID
    pub id: String,
    /// Input attributes
    pub inputs: Vec<String>,
    /// Output attributes
    pub outputs: Vec<String>,
    /// Dependencies (rule IDs that must execute before this)
    pub dependencies: Vec<String>,
}

impl Default for RuleDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_rule(product: &str, inputs: &[&str], outputs: &[&str]) -> Rule {
        Rule::from_json_logic(product, "test", json!({"var": inputs[0]}))
            .with_inputs(inputs.iter().map(|s| s.to_string()))
            .with_outputs(outputs.iter().map(|s| s.to_string()))
    }

    #[test]
    fn test_simple_dag() {
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
            make_rule("p", &["b"], &["c"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let order = dag.topological_order().unwrap();

        // Check that rules are in order
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_parallel_execution_levels() {
        // Diamond pattern: r1 -> r2, r3 -> r4
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
            make_rule("p", &["a"], &["c"]),
            make_rule("p", &["b", "c"], &["d"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let levels = dag.execution_levels().unwrap();

        assert_eq!(levels.len(), 3);
        assert_eq!(levels[0].len(), 1); // r1
        assert_eq!(levels[1].len(), 2); // r2, r3 (parallel)
        assert_eq!(levels[2].len(), 1); // r4
    }

    #[test]
    fn test_find_missing_inputs() {
        let rules = vec![
            make_rule("p", &["external_input"], &["a"]),
            make_rule("p", &["a", "missing_input"], &["b"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();

        // With only external_input available
        let available = HashSet::from(["external_input".to_string()]);
        let missing = dag.find_missing_inputs(&available);

        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].1, "missing_input");
    }

    #[test]
    fn test_execution_plan() {
        // Diamond pattern
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
            make_rule("p", &["a"], &["c"]),
            make_rule("p", &["b", "c"], &["d"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let plan = dag.execution_plan().unwrap();

        assert_eq!(plan.total_rules, 4);
        assert_eq!(plan.total_stages, 3);
        assert!(!plan.stages[0].parallel); // Single rule in stage 0
        assert!(plan.stages[1].parallel);   // Two parallel rules in stage 1
        assert!(!plan.stages[2].parallel); // Single rule in stage 2
    }

    #[test]
    fn test_to_dot() {
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let dot = dag.to_dot();

        assert!(dot.contains("digraph RuleDag"));
        assert!(dot.contains("->"));
        assert!(dot.contains("[in:"));
        assert!(dot.contains("[out:"));
    }

    #[test]
    fn test_to_mermaid() {
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let mermaid = dag.to_mermaid();

        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_to_ascii() {
        let rules = vec![
            make_rule("p", &["input"], &["a"]),
            make_rule("p", &["a"], &["b"]),
        ];

        let dag = RuleDag::from_rules(&rules).unwrap();
        let ascii = dag.to_ascii().unwrap();

        assert!(ascii.contains("Execution Plan"));
        assert!(ascii.contains("Stage 0"));
        assert!(ascii.contains("Stage 1"));
        assert!(ascii.contains("inputs:"));
        assert!(ascii.contains("outputs:"));
    }
}

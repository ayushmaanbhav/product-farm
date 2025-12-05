//! Graph Visualization for Rule Dependencies
//!
//! Generates visual representations of rule dependency graphs
//! in various formats (Mermaid, DOT, ASCII).

use crate::error::{AgentError, AgentResult};
use crate::tools::VisualizeGraphOutput;
use std::collections::{HashMap, HashSet};

/// Represents a node in the dependency graph
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeType {
    Product,
    Attribute,
    Rule,
    Input,
    Output,
}

impl NodeType {
    fn mermaid_style(&self) -> &'static str {
        match self {
            NodeType::Product => ":::product",
            NodeType::Attribute => ":::attribute",
            NodeType::Rule => ":::rule",
            NodeType::Input => ":::input",
            NodeType::Output => ":::output",
        }
    }

    fn dot_style(&self) -> &'static str {
        match self {
            NodeType::Product => "shape=box,style=filled,fillcolor=lightblue",
            NodeType::Attribute => "shape=ellipse,style=filled,fillcolor=lightyellow",
            NodeType::Rule => "shape=diamond,style=filled,fillcolor=lightgreen",
            NodeType::Input => "shape=ellipse,style=filled,fillcolor=lightgray",
            NodeType::Output => "shape=ellipse,style=filled,fillcolor=lightcoral",
        }
    }

    fn ascii_char(&self) -> char {
        match self {
            NodeType::Product => 'P',
            NodeType::Attribute => 'A',
            NodeType::Rule => 'R',
            NodeType::Input => 'I',
            NodeType::Output => 'O',
        }
    }
}

/// Represents an edge in the dependency graph
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

#[derive(Debug, Clone, Copy)]
pub enum EdgeType {
    DependsOn,
    Computes,
    BelongsTo,
}

impl EdgeType {
    fn mermaid_style(&self) -> &'static str {
        match self {
            EdgeType::DependsOn => "-->|depends on|",
            EdgeType::Computes => "-->|computes|",
            EdgeType::BelongsTo => "-.->|belongs to|",
        }
    }

    fn dot_label(&self) -> &'static str {
        match self {
            EdgeType::DependsOn => "depends on",
            EdgeType::Computes => "computes",
            EdgeType::BelongsTo => "belongs to",
        }
    }
}

/// Generates graph visualizations
pub struct GraphVisualizer {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
}

impl GraphVisualizer {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, id: impl Into<String>, label: impl Into<String>, node_type: NodeType) {
        self.nodes.push(GraphNode {
            id: id.into(),
            label: label.into(),
            node_type,
        });
    }

    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>, edge_type: EdgeType) {
        self.edges.push(GraphEdge {
            from: from.into(),
            to: to.into(),
            edge_type,
        });
    }

    /// Generate visualization in the specified format
    pub fn render(&self, format: &str) -> AgentResult<VisualizeGraphOutput> {
        let content = match format.to_lowercase().as_str() {
            "mermaid" => self.render_mermaid(),
            "dot" => self.render_dot(),
            "ascii" => self.render_ascii(),
            _ => return Err(AgentError::InvalidInput(format!(
                "Unknown format '{}'. Use 'mermaid', 'dot', or 'ascii'.",
                format
            ))),
        };

        Ok(VisualizeGraphOutput {
            format: format.to_string(),
            content,
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
        })
    }

    fn render_mermaid(&self) -> String {
        let mut output = String::new();
        output.push_str("```mermaid\n");
        output.push_str("graph TD\n");

        // Add style definitions
        output.push_str("    classDef product fill:#e1f5fe,stroke:#01579b\n");
        output.push_str("    classDef attribute fill:#fff9c4,stroke:#f57f17\n");
        output.push_str("    classDef rule fill:#c8e6c9,stroke:#2e7d32\n");
        output.push_str("    classDef input fill:#f5f5f5,stroke:#616161\n");
        output.push_str("    classDef output fill:#ffcdd2,stroke:#c62828\n");
        output.push('\n');

        // Add nodes
        for node in &self.nodes {
            let safe_id = self.sanitize_id(&node.id);
            let safe_label = self.escape_mermaid(&node.label);
            output.push_str(&format!(
                "    {}[\"{}\"]{}\n",
                safe_id,
                safe_label,
                node.node_type.mermaid_style()
            ));
        }

        output.push('\n');

        // Add edges
        for edge in &self.edges {
            let safe_from = self.sanitize_id(&edge.from);
            let safe_to = self.sanitize_id(&edge.to);
            output.push_str(&format!(
                "    {} {} {}\n",
                safe_from,
                edge.edge_type.mermaid_style(),
                safe_to
            ));
        }

        output.push_str("```\n");
        output
    }

    fn render_dot(&self) -> String {
        let mut output = String::new();
        output.push_str("digraph RuleDependencies {\n");
        output.push_str("    rankdir=TB;\n");
        output.push_str("    node [fontname=\"Arial\"];\n");
        output.push_str("    edge [fontname=\"Arial\",fontsize=10];\n");
        output.push('\n');

        // Add nodes
        for node in &self.nodes {
            let safe_id = self.sanitize_id(&node.id);
            let safe_label = self.escape_dot(&node.label);
            output.push_str(&format!(
                "    {} [label=\"{}\",{}];\n",
                safe_id,
                safe_label,
                node.node_type.dot_style()
            ));
        }

        output.push('\n');

        // Add edges
        for edge in &self.edges {
            let safe_from = self.sanitize_id(&edge.from);
            let safe_to = self.sanitize_id(&edge.to);
            output.push_str(&format!(
                "    {} -> {} [label=\"{}\"];\n",
                safe_from,
                safe_to,
                edge.edge_type.dot_label()
            ));
        }

        output.push_str("}\n");
        output
    }

    fn render_ascii(&self) -> String {
        let mut output = String::new();
        output.push_str("Rule Dependency Graph\n");
        output.push_str("=====================\n\n");

        // Legend
        output.push_str("Legend: [P]=Product [A]=Attribute [R]=Rule [I]=Input [O]=Output\n\n");

        // Build adjacency list for layout
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();
        let mut roots: HashSet<&str> = self.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &self.edges {
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
            roots.remove(edge.to.as_str());
        }

        // Node lookup
        let node_map: HashMap<&str, &GraphNode> =
            self.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        // Simple tree-like output
        let mut visited = HashSet::new();

        for root in roots {
            Self::render_ascii_node(root, 0, &adjacency, &node_map, &mut visited, &mut output);
        }

        if output.ends_with("=====================\n\n") {
            output.push_str("(empty graph)\n");
        }

        output
    }

    fn render_ascii_node(
        node_id: &str,
        depth: usize,
        adjacency: &HashMap<&str, Vec<&str>>,
        node_map: &HashMap<&str, &GraphNode>,
        visited: &mut HashSet<String>,
        output: &mut String,
    ) {
        if visited.contains(node_id) {
            let indent = "  ".repeat(depth);
            output.push_str(&format!("{}└── (cycle: {})\n", indent, node_id));
            return;
        }
        visited.insert(node_id.to_string());

        let indent = "  ".repeat(depth);
        if let Some(node) = node_map.get(node_id) {
            output.push_str(&format!(
                "{}[{}] {}\n",
                indent,
                node.node_type.ascii_char(),
                node.label
            ));
        } else {
            output.push_str(&format!("{}[?] {}\n", indent, node_id));
        }

        if let Some(children) = adjacency.get(node_id) {
            for child in children {
                Self::render_ascii_node(child, depth + 1, adjacency, node_map, visited, output);
            }
        }
    }

    fn sanitize_id(&self, id: &str) -> String {
        id.replace(".", "_")
            .replace("$", "_")
            .replace("-", "_")
            .replace(" ", "_")
    }

    fn escape_mermaid(&self, s: &str) -> String {
        s.replace("\"", "&quot;").replace("<", "&lt;").replace(">", "&gt;")
    }

    fn escape_dot(&self, s: &str) -> String {
        s.replace("\"", "\\\"").replace("\n", "\\n")
    }
}

impl Default for GraphVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a visualizer from rule dependencies
pub fn build_rule_graph(
    rules: &[(String, Vec<String>, Vec<String>)], // (rule_id, inputs, outputs)
) -> GraphVisualizer {
    let mut viz = GraphVisualizer::new();
    let mut all_attributes: HashSet<String> = HashSet::new();

    // Collect all attributes
    for (_, inputs, outputs) in rules {
        all_attributes.extend(inputs.iter().cloned());
        all_attributes.extend(outputs.iter().cloned());
    }

    // Add attribute nodes
    for attr in &all_attributes {
        let node_type = if rules.iter().any(|(_, _, outputs)| outputs.contains(attr)) {
            NodeType::Output
        } else {
            NodeType::Input
        };
        viz.add_node(attr.clone(), attr.clone(), node_type);
    }

    // Add rule nodes and edges
    for (rule_id, inputs, outputs) in rules {
        viz.add_node(rule_id.clone(), rule_id.clone(), NodeType::Rule);

        for input in inputs {
            viz.add_edge(input.clone(), rule_id.clone(), EdgeType::DependsOn);
        }

        for output in outputs {
            viz.add_edge(rule_id.clone(), output.clone(), EdgeType::Computes);
        }
    }

    viz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_graph_mermaid() {
        let rules = vec![
            (
                "calc_premium".to_string(),
                vec!["age".to_string(), "base_rate".to_string()],
                vec!["premium".to_string()],
            ),
        ];

        let viz = build_rule_graph(&rules);
        let result = viz.render("mermaid").unwrap();

        assert!(result.content.contains("mermaid"));
        assert!(result.content.contains("age"));
        assert!(result.content.contains("calc_premium"));
        assert!(result.content.contains("premium"));
        assert_eq!(result.node_count, 4); // 3 attributes + 1 rule
    }

    #[test]
    fn test_graph_dot() {
        let viz = GraphVisualizer::new();
        let result = viz.render("dot").unwrap();
        assert!(result.content.contains("digraph"));
    }

    #[test]
    fn test_graph_ascii() {
        let viz = GraphVisualizer::new();
        let result = viz.render("ascii").unwrap();
        assert!(result.content.contains("Legend"));
    }
}

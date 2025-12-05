// Enhanced Rule Canvas with Interactive DAG Visualization
// Features: Custom nodes, minimap, zoom controls, execution level coloring

import { useCallback, useMemo, useState, memo } from 'react';
import {
  ReactFlow,
  Controls,
  Background,
  BackgroundVariant,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  MarkerType,
  Handle,
  Position,
  Panel,
} from '@xyflow/react';
import type { Node, Edge, Connection, NodeProps } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import type { AbstractAttribute, Rule, ExecutionPlan } from '@/types';
import { useUIStore, useSimulationStore } from '@/store';
import { Button } from '@/components/ui/button';
import {
  Layers,
  Eye,
  EyeOff,
  Target,
  GitBranch,
} from 'lucide-react';
import { cn } from '@/lib/utils';

// =============================================================================
// CUSTOM NODE COMPONENTS
// =============================================================================

interface AttributeNodeData {
  label: string;
  path: string;
  datatype: string;
  isInput: boolean;
  tags: string[];
  value?: unknown;
  highlighted?: boolean;
  impactLevel?: 'source' | 'affected' | 'neutral';
}

interface RuleNodeData {
  label: string;
  ruleId: string;
  ruleType: string;
  displayExpression: string;
  enabled: boolean;
  executionLevel: number;
  inputCount: number;
  outputCount: number;
  highlighted?: boolean;
  executing?: boolean;
}

const AttributeNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as unknown as AttributeNodeData;
  const { impactAnalysisTarget, setImpactAnalysisTarget } = useUIStore();
  const isImpactSource = impactAnalysisTarget === nodeData.path;

  const getBorderColor = () => {
    if (nodeData.impactLevel === 'source') return 'border-red-500 ring-2 ring-red-200';
    if (nodeData.impactLevel === 'affected') return 'border-orange-500 ring-2 ring-orange-200';
    if (nodeData.isInput) return 'border-blue-400';
    return 'border-amber-400';
  };

  const getBackgroundColor = () => {
    if (nodeData.impactLevel === 'source') return 'bg-red-50';
    if (nodeData.impactLevel === 'affected') return 'bg-orange-50';
    if (nodeData.isInput) return 'bg-blue-50';
    return 'bg-amber-50';
  };

  return (
    <div
      className={cn(
        'relative min-w-[140px] rounded-lg border-2 px-3 py-2 shadow-sm transition-all',
        getBorderColor(),
        getBackgroundColor(),
        selected && 'ring-2 ring-primary',
        nodeData.highlighted && 'ring-2 ring-green-400'
      )}
    >
      {/* Input handle (left) */}
      {!nodeData.isInput && (
        <Handle
          type="target"
          position={Position.Left}
          className="!h-3 !w-3 !border-2 !border-white !bg-gray-400"
        />
      )}

      {/* Content */}
      <div className="flex flex-col gap-1">
        <div className="flex items-center justify-between gap-2">
          <span className="text-xs font-semibold text-gray-800 truncate max-w-[100px]">
            {nodeData.label}
          </span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              setImpactAnalysisTarget(isImpactSource ? null : nodeData.path);
            }}
            className={cn(
              'p-0.5 rounded transition-colors',
              isImpactSource ? 'bg-red-200 text-red-700' : 'hover:bg-gray-200 text-gray-400'
            )}
            title="Analyze impact"
          >
            <Target className="h-3 w-3" />
          </button>
        </div>
        <div className="flex items-center gap-1">
          <span className="rounded bg-gray-200 px-1.5 py-0.5 text-[10px] font-medium text-gray-600">
            {nodeData.datatype}
          </span>
          {nodeData.isInput && (
            <span className="rounded bg-blue-200 px-1.5 py-0.5 text-[10px] font-medium text-blue-700">
              INPUT
            </span>
          )}
        </div>
        {nodeData.value !== undefined && (
          <div className="mt-1 rounded bg-white px-1.5 py-0.5 text-[10px] font-mono text-gray-700 border">
            = {String(nodeData.value)}
          </div>
        )}
      </div>

      {/* Output handle (right) */}
      <Handle
        type="source"
        position={Position.Right}
        className="!h-3 !w-3 !border-2 !border-white !bg-gray-600"
      />
    </div>
  );
});

AttributeNode.displayName = 'AttributeNode';

const RuleNode = memo(({ data, selected }: NodeProps) => {
  const nodeData = data as unknown as RuleNodeData;

  const getLevelColor = () => {
    const colors = [
      'bg-emerald-50 border-emerald-400',
      'bg-teal-50 border-teal-400',
      'bg-cyan-50 border-cyan-400',
      'bg-sky-50 border-sky-400',
      'bg-indigo-50 border-indigo-400',
      'bg-purple-50 border-purple-400',
    ];
    return colors[nodeData.executionLevel % colors.length];
  };

  return (
    <div
      className={cn(
        'relative min-w-[180px] rounded-lg border-2 px-3 py-2 shadow-md transition-all',
        getLevelColor(),
        selected && 'ring-2 ring-primary',
        nodeData.highlighted && 'ring-2 ring-green-400',
        nodeData.executing && 'animate-pulse',
        !nodeData.enabled && 'opacity-50'
      )}
    >
      {/* Input handle (left) */}
      <Handle
        type="target"
        position={Position.Left}
        className="!h-4 !w-4 !border-2 !border-white !bg-emerald-500"
      />

      {/* Header */}
      <div className="flex items-center justify-between gap-2 mb-1">
        <div className="flex items-center gap-1.5">
          <GitBranch className="h-3.5 w-3.5 text-emerald-600" />
          <span className="text-[10px] font-semibold text-gray-800 uppercase tracking-wide">
            {nodeData.ruleType}
          </span>
        </div>
        <span className="rounded-full bg-white px-1.5 py-0.5 text-[10px] font-bold text-gray-600 border">
          L{nodeData.executionLevel}
        </span>
      </div>

      {/* Expression */}
      <div className="rounded bg-white/80 p-1.5 text-[11px] font-mono text-gray-700 border border-gray-200 line-clamp-2">
        {nodeData.displayExpression}
      </div>

      {/* Footer */}
      <div className="mt-1.5 flex items-center justify-between text-[10px] text-gray-500">
        <span>{nodeData.inputCount} in</span>
        <span>{nodeData.outputCount} out</span>
      </div>

      {/* Output handle (right) */}
      <Handle
        type="source"
        position={Position.Right}
        className="!h-4 !w-4 !border-2 !border-white !bg-emerald-600"
      />
    </div>
  );
});

RuleNode.displayName = 'RuleNode';

const nodeTypes = {
  attribute: AttributeNode,
  rule: RuleNode,
};

// =============================================================================
// LAYOUT UTILITIES
// =============================================================================

interface LayoutOptions {
  type: 'dagre' | 'hierarchical' | 'force';
  nodeWidth: number;
  nodeHeight: number;
}

function layoutNodes(
  attributes: AbstractAttribute[],
  rules: Rule[],
  executionPlan: ExecutionPlan | null,
  _options: LayoutOptions,
  simulationResults?: Record<string, unknown>,
  impactAnalysisTarget?: string | null
): { nodes: Node[]; edges: Edge[] } {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // Classify attributes
  const inputAttrs = attributes.filter((a) => a.tags.some((t) => t.name === 'input'));
  const outputAttrs = attributes.filter((a) => !a.tags.some((t) => t.name === 'input'));

  // Build output-to-rule mapping
  const outputToRuleId = new Map<string, string>();
  rules.forEach((r) => {
    r.outputAttributes.forEach((o) => {
      outputToRuleId.set(o.attributePath, r.id);
    });
  });

  // Calculate impact analysis
  const affectedPaths = new Set<string>();
  if (impactAnalysisTarget) {
    // Find all rules that use this attribute as input
    const queue = [impactAnalysisTarget];
    const visited = new Set<string>();

    while (queue.length > 0) {
      const current = queue.shift()!;
      if (visited.has(current)) continue;
      visited.add(current);

      // Find rules that use this attribute
      rules.forEach((r) => {
        const usesAsInput = r.inputAttributes.some((i) => i.attributePath === current);
        if (usesAsInput) {
          r.outputAttributes.forEach((o) => {
            affectedPaths.add(o.attributePath);
            queue.push(o.attributePath);
          });
        }
      });
    }
  }

  // Get rule execution levels
  const ruleToLevel = new Map<string, number>();
  executionPlan?.levels.forEach((level) => {
    level.ruleIds.forEach((ruleId) => {
      ruleToLevel.set(ruleId, level.level);
    });
  });

  const maxLevel = Math.max(...Array.from(ruleToLevel.values()), 0);

  // Column positions
  const inputX = 50;
  const levelWidth = 250;
  const outputX = inputX + levelWidth * (maxLevel + 2) + 100;
  const rowHeight = 100;

  // Input attribute nodes (left column)
  inputAttrs.forEach((attr, i) => {
    const attrName = attr.attributeName || attr.abstractPath.split(':').pop() || '';
    const isImpactSource = impactAnalysisTarget === attr.abstractPath;

    nodes.push({
      id: `attr-${attr.abstractPath}`,
      type: 'attribute',
      position: { x: inputX, y: i * rowHeight + 50 },
      data: {
        label: attrName,
        path: attr.abstractPath,
        datatype: attr.datatypeId,
        isInput: true,
        tags: attr.tags.map((t) => t.name),
        value: simulationResults?.[attrName],
        impactLevel: isImpactSource ? 'source' : undefined,
      },
    });
  });

  // Rule nodes (middle columns by execution level)
  const levelCounts = new Map<number, number>();

  rules.forEach((rule) => {
    const level = ruleToLevel.get(rule.id) ?? 0;
    const countAtLevel = levelCounts.get(level) ?? 0;
    levelCounts.set(level, countAtLevel + 1);

    const x = inputX + levelWidth + level * levelWidth;
    const y = countAtLevel * (rowHeight + 20) + 50;

    nodes.push({
      id: `rule-${rule.id}`,
      type: 'rule',
      position: { x, y },
      data: {
        label: rule.description || rule.displayExpression.substring(0, 30),
        ruleId: rule.id,
        ruleType: rule.ruleType,
        displayExpression: rule.displayExpression,
        enabled: rule.enabled,
        executionLevel: level,
        inputCount: rule.inputAttributes.length,
        outputCount: rule.outputAttributes.length,
      },
    });

    // Input edges
    rule.inputAttributes.forEach((input) => {
      const sourceId = outputToRuleId.has(input.attributePath)
        ? `rule-${outputToRuleId.get(input.attributePath)}`
        : `attr-${input.attributePath}`;

      edges.push({
        id: `e-${sourceId}-${rule.id}`,
        source: sourceId,
        target: `rule-${rule.id}`,
        type: 'smoothstep',
        animated: true,
        style: { stroke: '#6b7280', strokeWidth: 1.5 },
        markerEnd: { type: MarkerType.ArrowClosed, color: '#6b7280' },
      });
    });
  });

  // Output attribute nodes (right column)
  outputAttrs.forEach((attr, i) => {
    const attrName = attr.attributeName || attr.abstractPath.split(':').pop() || '';
    const isAffected = affectedPaths.has(attr.abstractPath);

    nodes.push({
      id: `attr-${attr.abstractPath}`,
      type: 'attribute',
      position: { x: outputX, y: i * rowHeight + 50 },
      data: {
        label: attrName,
        path: attr.abstractPath,
        datatype: attr.datatypeId,
        isInput: false,
        tags: attr.tags.map((t) => t.name),
        value: simulationResults?.[attrName],
        impactLevel: isAffected ? 'affected' : undefined,
      },
    });

    // Output edges from rules to output attributes
    const producingRuleId = outputToRuleId.get(attr.abstractPath);
    if (producingRuleId) {
      edges.push({
        id: `e-${producingRuleId}-${attr.abstractPath}`,
        source: `rule-${producingRuleId}`,
        target: `attr-${attr.abstractPath}`,
        type: 'smoothstep',
        style: { stroke: '#10b981', strokeWidth: 2 },
        markerEnd: { type: MarkerType.ArrowClosed, color: '#10b981' },
      });
    }
  });

  return { nodes, edges };
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

interface RuleCanvasProps {
  attributes: AbstractAttribute[];
  rules: Rule[];
  executionPlan: ExecutionPlan | null;
  onNodeClick?: (nodeId: string) => void;
  onRuleEdit?: (ruleId: string) => void;
}

export function RuleCanvas({
  attributes,
  rules,
  executionPlan,
  onNodeClick,
  onRuleEdit,
}: RuleCanvasProps) {
  const { graphLayout, impactAnalysisTarget, setImpactAnalysisTarget } = useUIStore();
  const { results } = useSimulationStore();
  const [showMinimap, setShowMinimap] = useState(true);
  const [showLevels, setShowLevels] = useState(true);

  // Build simulation results map
  const simulationResults = useMemo(() => {
    if (!results?.outputs) return undefined;
    const map: Record<string, unknown> = {};
    Object.entries(results.outputs).forEach(([key, val]) => {
      if (val && 'value' in val) {
        map[key] = (val as { value: unknown }).value;
      }
    });
    return map;
  }, [results]);

  // Layout nodes and edges
  const { nodes: initialNodes, edges: initialEdges } = useMemo(
    () =>
      layoutNodes(
        attributes,
        rules,
        executionPlan,
        { type: graphLayout, nodeWidth: 180, nodeHeight: 80 },
        simulationResults,
        impactAnalysisTarget
      ),
    [attributes, rules, executionPlan, graphLayout, simulationResults, impactAnalysisTarget]
  );

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // Update nodes when data changes
  useMemo(() => {
    setNodes(initialNodes);
    setEdges(initialEdges);
  }, [initialNodes, initialEdges, setNodes, setEdges]);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      if (node.id.startsWith('rule-') && onRuleEdit) {
        const ruleId = node.id.replace('rule-', '');
        onRuleEdit(ruleId);
      }
      onNodeClick?.(node.id);
    },
    [onNodeClick, onRuleEdit]
  );

  // Legend data
  const levelColors = [
    { level: 0, color: 'bg-emerald-100 border-emerald-400' },
    { level: 1, color: 'bg-teal-100 border-teal-400' },
    { level: 2, color: 'bg-cyan-100 border-cyan-400' },
    { level: 3, color: 'bg-sky-100 border-sky-400' },
  ];

  return (
    <div className="relative h-full w-full rounded-lg border bg-white">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeClick={handleNodeClick}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={0.1}
        maxZoom={2}
        attributionPosition="bottom-left"
      >
        <Background variant={BackgroundVariant.Dots} gap={16} size={1} color="#e5e7eb" />
        <Controls showInteractive={false} />

        {showMinimap && (
          <MiniMap
            nodeColor={(node) => {
              if (node.type === 'attribute') {
                const data = node.data as unknown as AttributeNodeData;
                if (data.impactLevel === 'source') return '#ef4444';
                if (data.impactLevel === 'affected') return '#f97316';
                return data.isInput ? '#3b82f6' : '#f59e0b';
              }
              return '#10b981';
            }}
            maskColor="rgba(0, 0, 0, 0.1)"
            pannable
            zoomable
          />
        )}

        {/* Toolbar Panel */}
        <Panel position="top-right" className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowMinimap(!showMinimap)}
            className="gap-1.5"
          >
            {showMinimap ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
            Map
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowLevels(!showLevels)}
            className="gap-1.5"
          >
            <Layers className="h-4 w-4" />
            Levels
          </Button>
          {impactAnalysisTarget && (
            <Button
              variant="destructive"
              size="sm"
              onClick={() => setImpactAnalysisTarget(null)}
              className="gap-1.5"
            >
              <Target className="h-4 w-4" />
              Clear Impact
            </Button>
          )}
        </Panel>

        {/* Legend Panel */}
        {showLevels && (
          <Panel position="bottom-right" className="bg-white rounded-lg border shadow-lg p-3">
            <p className="text-xs font-semibold text-gray-700 mb-2">Execution Levels</p>
            <div className="space-y-1.5">
              {levelColors.map((lc) => (
                <div key={lc.level} className="flex items-center gap-2">
                  <div className={cn('h-4 w-6 rounded border-2', lc.color)} />
                  <span className="text-xs text-gray-600">Level {lc.level}</span>
                </div>
              ))}
            </div>
            <div className="mt-3 pt-2 border-t space-y-1.5">
              <div className="flex items-center gap-2">
                <div className="h-4 w-6 rounded border-2 bg-blue-100 border-blue-400" />
                <span className="text-xs text-gray-600">Input Attribute</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-4 w-6 rounded border-2 bg-amber-100 border-amber-400" />
                <span className="text-xs text-gray-600">Computed Attribute</span>
              </div>
            </div>
            {impactAnalysisTarget && (
              <div className="mt-3 pt-2 border-t space-y-1.5">
                <p className="text-xs font-semibold text-gray-700">Impact Analysis</p>
                <div className="flex items-center gap-2">
                  <div className="h-4 w-6 rounded border-2 bg-red-100 border-red-500" />
                  <span className="text-xs text-gray-600">Source</span>
                </div>
                <div className="flex items-center gap-2">
                  <div className="h-4 w-6 rounded border-2 bg-orange-100 border-orange-500" />
                  <span className="text-xs text-gray-600">Affected</span>
                </div>
              </div>
            )}
          </Panel>
        )}
      </ReactFlow>
    </div>
  );
}

export default RuleCanvas;

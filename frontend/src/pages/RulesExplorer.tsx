/**
 * RulesExplorer - DAG Visualization with proper viewBox-based zoom
 * Uses real product data from the backend API
 */

import React, { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import {
  ZoomIn, ZoomOut, ChevronRight, ChevronLeft,
  Package, Activity, X, Layers, RotateCcw, AlertCircle, Loader2,
  Calculator, GitMerge, Filter, Database, Box, Target,
} from 'lucide-react';
import { useProductStore } from '@/store';
import type { Rule, ExecutionPlan, AbstractAttribute } from '@/types';

// ============================================================================
// TYPES & CONSTANTS
// ============================================================================

interface DAGNode {
  id: string;
  name: string;
  type: 'input' | 'rule' | 'output' | 'cluster';
  level: number;
  pattern: string;
  childCount: number;
  latency?: string;
  tier?: number;
  color: string;
  x: number;
  y: number;
  ruleId?: string;
  enabled?: boolean;
  inputCount?: number;
  outputCount?: number;
}

interface DAGEdge {
  from: string;
  to: string;
  isCritical: boolean;
}

const PATTERNS: Record<string, { name: string; color: string; icon: typeof Box }> = {
  INPUT: { name: 'Input', color: '#6366F1', icon: Box },
  CALCULATION: { name: 'Calculation', color: '#22C55E', icon: Calculator },
  CONDITIONAL: { name: 'Conditional', color: '#EAB308', icon: GitMerge },
  COMPARISON: { name: 'Comparison', color: '#3B82F6', icon: Filter },
  AGGREGATION: { name: 'Aggregation', color: '#EC4899', icon: Layers },
  LOOKUP: { name: 'Lookup', color: '#8B5CF6', icon: Database },
  OUTPUT: { name: 'Output', color: '#F97316', icon: Target },
};

// Node sizes per LOD level
const LOD_SIZES = {
  1: { w: 220, h: 88, gap: 35, levelGap: 140 },  // Domain clusters - largest
  2: { w: 200, h: 80, gap: 30, levelGap: 130 },  // Pattern groups - large
  3: { w: 180, h: 72, gap: 25, levelGap: 120 },  // Individual rules - normal
};

const LEVEL_GAP = 120;

// ============================================================================
// PATTERN DETECTION
// ============================================================================

function detectRulePattern(rule: Rule): string {
  const expr = rule.compiledExpression?.toLowerCase() || '';
  const display = rule.displayExpression?.toLowerCase() || '';
  const ruleType = rule.ruleType?.toLowerCase() || '';

  // Check for common patterns
  if (ruleType.includes('lookup') || expr.includes('lookup') || display.includes('lookup')) {
    return 'LOOKUP';
  }
  if (ruleType.includes('aggregate') || expr.includes('reduce') || expr.includes('sum') || display.includes('sum')) {
    return 'AGGREGATION';
  }
  if (expr.includes('"if"') || expr.includes('"?:"') || display.includes('if ') || display.includes('when ')) {
    return 'CONDITIONAL';
  }
  if (expr.includes('">"') || expr.includes('"<"') || expr.includes('"=="') || expr.includes('"!="')) {
    return 'COMPARISON';
  }
  if (expr.includes('"+"') || expr.includes('"-"') || expr.includes('"*"') || expr.includes('"/"')) {
    return 'CALCULATION';
  }

  // Default based on position
  return 'CALCULATION';
}

// ============================================================================
// DAG GENERATION FROM REAL DATA
// ============================================================================

interface GenerateDAGOptions {
  rules: Rule[];
  executionPlan: ExecutionPlan | null;
  abstractAttributes: AbstractAttribute[];
  lod: number;
}

function generateDAGFromData({ rules, executionPlan, abstractAttributes, lod }: GenerateDAGOptions): {
  nodes: DAGNode[];
  edges: DAGEdge[];
  nodeW: number;
  nodeH: number
} {
  const nodes: DAGNode[] = [];
  const edges: DAGEdge[] = [];

  if (!rules.length) {
    return { nodes, edges, nodeW: 180, nodeH: 72 };
  }

  // Get LOD-specific sizes
  const sizes = LOD_SIZES[lod as keyof typeof LOD_SIZES] || LOD_SIZES[3];
  const nodeW = sizes.w;
  const nodeH = sizes.h;
  const nodeGap = sizes.gap;
  const levelGap = sizes.levelGap;

  // Build rule-to-level mapping from execution plan
  const ruleToLevel = new Map<string, number>();
  let maxLevel = 0;

  if (executionPlan?.levels) {
    executionPlan.levels.forEach((level) => {
      level.ruleIds.forEach((ruleId) => {
        ruleToLevel.set(ruleId, level.level);
        maxLevel = Math.max(maxLevel, level.level);
      });
    });
  } else {
    // Fallback: assign levels based on orderIndex
    rules.forEach((rule, i) => {
      ruleToLevel.set(rule.id, Math.floor(i / Math.max(4, Math.ceil(rules.length / 5))));
      maxLevel = Math.max(maxLevel, ruleToLevel.get(rule.id) || 0);
    });
  }

  // Build dependencies map
  const dependencies = new Map<string, string[]>();
  if (executionPlan?.dependencies) {
    executionPlan.dependencies.forEach((dep) => {
      dependencies.set(dep.ruleId, dep.dependsOn);
    });
  }

  if (lod === 1) {
    // Domain clusters - group rules by detected domain/category
    const domainGroups = new Map<string, Rule[]>();

    rules.forEach(rule => {
      // Extract domain from rule description, display expression, or inputs
      let domain = 'General';
      const desc = (rule.description || rule.displayExpression || '').toLowerCase();

      if (desc.includes('risk') || desc.includes('score')) domain = 'Risk';
      else if (desc.includes('premium') || desc.includes('price') || desc.includes('rate')) domain = 'Premium';
      else if (desc.includes('discount') || desc.includes('rebate')) domain = 'Discounts';
      else if (desc.includes('valid') || desc.includes('check') || desc.includes('error')) domain = 'Validation';
      else if (desc.includes('elig') || desc.includes('qualify')) domain = 'Eligibility';
      else if (desc.includes('output') || desc.includes('result') || desc.includes('final')) domain = 'Output';
      else if (desc.includes('input') || desc.includes('data')) domain = 'Input';
      else if (desc.includes('calc') || desc.includes('compute')) domain = 'Calculation';

      if (!domainGroups.has(domain)) {
        domainGroups.set(domain, []);
      }
      domainGroups.get(domain)!.push(rule);
    });

    const domains = Array.from(domainGroups.keys());
    const cols = Math.min(3, domains.length);
    const colWidth = nodeW + nodeGap * 2;
    const totalWidth = cols * colWidth - nodeGap * 2;
    const startX = -totalWidth / 2;

    domains.forEach((domain, i) => {
      const rulesInDomain = domainGroups.get(domain) || [];
      const level = Math.floor(i / cols);
      const col = i % cols;
      const patternKeys = Object.keys(PATTERNS);
      const pattern = patternKeys[i % patternKeys.length];

      nodes.push({
        id: `d${i}`,
        name: `${domain} Domain`,
        type: 'cluster',
        level,
        pattern,
        childCount: rulesInDomain.length,
        tier: 1,
        color: PATTERNS[pattern].color,
        x: startX + col * colWidth,
        y: level * levelGap,
      });
    });

    // Connect domains vertically
    const numRows = Math.ceil(domains.length / cols);
    for (let row = 0; row < numRows - 1; row++) {
      for (let col = 0; col < cols; col++) {
        const fromIdx = row * cols + col;
        const toIdx = (row + 1) * cols + col;
        if (fromIdx < domains.length && toIdx < domains.length) {
          edges.push({ from: `d${fromIdx}`, to: `d${toIdx}`, isCritical: col === 0 });
        }
      }
    }

  } else if (lod === 2) {
    // Pattern groups - group by rule type/pattern
    const patternGroups = new Map<string, Rule[]>();

    rules.forEach(rule => {
      const pattern = detectRulePattern(rule);
      if (!patternGroups.has(pattern)) {
        patternGroups.set(pattern, []);
      }
      patternGroups.get(pattern)!.push(rule);
    });

    const patterns = Array.from(patternGroups.keys());
    const cols = Math.min(5, patterns.length);
    const colWidth = nodeW + nodeGap * 2;
    const totalWidth = cols * colWidth - nodeGap * 2;
    const startX = -totalWidth / 2;

    // Organize by level
    const levelToPatterns = new Map<number, { pattern: string; rules: Rule[] }[]>();

    patterns.forEach(pattern => {
      const rulesInPattern = patternGroups.get(pattern) || [];
      // Find average level for this pattern
      const avgLevel = rulesInPattern.reduce((sum, r) => sum + (ruleToLevel.get(r.id) || 0), 0) / rulesInPattern.length;
      const level = Math.round(avgLevel);

      if (!levelToPatterns.has(level)) {
        levelToPatterns.set(level, []);
      }
      levelToPatterns.get(level)!.push({ pattern, rules: rulesInPattern });
    });

    let nodeIdx = 0;
    const levels = Array.from(levelToPatterns.keys()).sort((a, b) => a - b);

    levels.forEach((level, li) => {
      const patternsAtLevel = levelToPatterns.get(level) || [];
      patternsAtLevel.forEach((pg, pi) => {
        nodes.push({
          id: `p${nodeIdx}`,
          name: `${PATTERNS[pg.pattern]?.name || pg.pattern}`,
          type: 'cluster',
          level: li,
          pattern: pg.pattern,
          childCount: pg.rules.length,
          tier: 1,
          color: PATTERNS[pg.pattern]?.color || '#6B7280',
          x: startX + (pi % cols) * colWidth,
          y: li * levelGap,
        });
        nodeIdx++;
      });
    });

    // Connect pattern nodes by level
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      // Find nodes in next level
      const nextLevelNodes = nodes.filter(n => n.level === node.level + 1);
      if (nextLevelNodes.length > 0) {
        // Connect to nearest node in next level
        const nearest = nextLevelNodes.reduce((a, b) =>
          Math.abs(a.x - node.x) < Math.abs(b.x - node.x) ? a : b
        );
        edges.push({
          from: node.id,
          to: nearest.id,
          isCritical: node.x === Math.min(...nodes.filter(n => n.level === node.level).map(n => n.x))
        });
      }
    }

  } else {
    // LOD 3: Individual rules
    const rulesByLevel = new Map<number, Rule[]>();

    rules.forEach(rule => {
      const level = ruleToLevel.get(rule.id) || 0;
      if (!rulesByLevel.has(level)) {
        rulesByLevel.set(level, []);
      }
      rulesByLevel.get(level)!.push(rule);
    });

    const levels = Array.from(rulesByLevel.keys()).sort((a, b) => a - b);
    const maxRulesPerLevel = Math.max(...Array.from(rulesByLevel.values()).map(r => r.length));
    const cols = Math.min(8, maxRulesPerLevel);
    const colWidth = nodeW + nodeGap;
    const totalWidth = cols * colWidth - nodeGap;
    const startX = -totalWidth / 2;

    levels.forEach((level, levelIdx) => {
      const rulesAtLevel = rulesByLevel.get(level) || [];
      rulesAtLevel.forEach((rule, ri) => {
        const pattern = detectRulePattern(rule);
        const col = ri % cols;
        const row = Math.floor(ri / cols);

        nodes.push({
          id: rule.id,
          name: rule.description || rule.displayExpression?.slice(0, 25) || `Rule ${rule.id.slice(0, 8)}`,
          type: 'rule',
          level: levelIdx,
          pattern,
          childCount: 0,
          tier: 1,
          color: PATTERNS[pattern]?.color || '#6B7280',
          x: startX + col * colWidth,
          y: (levelIdx * 1.5 + row * 0.3) * levelGap,
          ruleId: rule.id,
          enabled: rule.enabled,
          inputCount: rule.inputAttributes?.length || 0,
          outputCount: rule.outputAttributes?.length || 0,
        });
      });
    });

    // Create edges from dependencies
    const nodeIds = new Set(nodes.map(n => n.id));

    rules.forEach(rule => {
      const deps = dependencies.get(rule.id) || [];
      deps.forEach((depRuleId, di) => {
        if (nodeIds.has(depRuleId) && nodeIds.has(rule.id)) {
          edges.push({
            from: depRuleId,
            to: rule.id,
            isCritical: di === 0, // First dependency is critical path
          });
        }
      });
    });

    // If no edges from dependencies, create simple level-based edges
    if (edges.length === 0 && levels.length > 1) {
      for (let li = 0; li < levels.length - 1; li++) {
        const currentLevel = rulesByLevel.get(levels[li]) || [];
        const nextLevel = rulesByLevel.get(levels[li + 1]) || [];

        // Connect first rule of each level (critical path)
        if (currentLevel[0] && nextLevel[0]) {
          edges.push({
            from: currentLevel[0].id,
            to: nextLevel[0].id,
            isCritical: true,
          });
        }
      }
    }
  }

  return { nodes, edges, nodeW, nodeH };
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export default function RulesExplorer() {
  const {
    products,
    selectedProduct,
    selectProduct,
    rules,
    executionPlan,
    abstractAttributes,
    fetchProducts,
    isLoading,
    error,
  } = useProductStore();

  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [localLoading, setLocalLoading] = useState(false);

  // ViewBox-based zoom: smaller viewBox = zoomed in
  const [viewBox, setViewBox] = useState({ x: -500, y: -40, w: 1000, h: 700 });
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0, vbX: 0, vbY: 0 });
  const svgRef = useRef<SVGSVGElement>(null);

  // Fetch products on mount
  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  // LOD based on viewBox width (smaller = more zoomed in = more detail)
  const lod = useMemo(() => {
    if (viewBox.w > 1400) return 1;      // Very zoomed out - domains
    if (viewBox.w > 800) return 2;       // Medium - patterns
    return 3;                             // Zoomed in - rules
  }, [viewBox.w]);

  // Generate DAG from real data
  const dag = useMemo(() => {
    if (!selectedProduct) return null;
    return generateDAGFromData({ rules, executionPlan, abstractAttributes, lod });
  }, [selectedProduct, rules, executionPlan, abstractAttributes, lod]);

  const selectedNode = dag?.nodes.find(n => n.id === selectedNodeId);
  const selectedRule = selectedNode?.ruleId ? rules.find(r => r.id === selectedNode.ruleId) : null;

  // Handle product selection
  const handleSelectProduct = async (productId: string) => {
    setLocalLoading(true);
    try {
      await selectProduct(productId);
      setViewBox({ x: -500, y: -40, w: 1000, h: 700 });
      setSelectedNodeId(null);
    } finally {
      setLocalLoading(false);
    }
  };

  const handleBackToProducts = async () => {
    await selectProduct(null);
    setSelectedNodeId(null);
  };

  // Zoom by changing viewBox size
  const handleZoom = useCallback((direction: 'in' | 'out') => {
    setViewBox(vb => {
      const factor = direction === 'in' ? 0.8 : 1.25;
      const newW = Math.max(400, Math.min(2500, vb.w * factor));
      const newH = Math.max(300, Math.min(1800, vb.h * factor));
      // Keep center point
      const cx = vb.x + vb.w / 2;
      const cy = vb.y + vb.h / 2;
      return {
        x: cx - newW / 2,
        y: cy - newH / 2,
        w: newW,
        h: newH,
      };
    });
  }, []);

  // Mouse handlers for panning
  const handleMouseDown = (e: React.MouseEvent<SVGSVGElement>) => {
    if (e.button === 0) {
      setIsPanning(true);
      setPanStart({ x: e.clientX, y: e.clientY, vbX: viewBox.x, vbY: viewBox.y });
    }
  };

  const handleMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    if (!isPanning || !svgRef.current) return;
    const rect = svgRef.current.getBoundingClientRect();
    const scaleX = viewBox.w / rect.width;
    const scaleY = viewBox.h / rect.height;
    const dx = (e.clientX - panStart.x) * scaleX;
    const dy = (e.clientY - panStart.y) * scaleY;
    setViewBox(vb => ({ ...vb, x: panStart.vbX - dx, y: panStart.vbY - dy }));
  };

  const handleMouseUp = () => setIsPanning(false);

  // Wheel zoom
  const handleWheel = useCallback((e: React.WheelEvent) => {
    e.preventDefault();
    handleZoom(e.deltaY > 0 ? 'out' : 'in');
  }, [handleZoom]);

  const resetView = () => {
    setViewBox({ x: -500, y: -40, w: 1000, h: 700 });
    setSelectedNodeId(null);
  };

  const getLODLabel = (l: number) => {
    if (l === 1) return 'Domain Clusters';
    if (l === 2) return 'Pattern Groups';
    return 'Individual Rules';
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'ACTIVE': return 'bg-green-900 text-green-300';
      case 'DRAFT': return 'bg-yellow-900 text-yellow-300';
      case 'PENDING_APPROVAL': return 'bg-blue-900 text-blue-300';
      case 'DISCONTINUED': return 'bg-gray-700 text-gray-400';
      default: return 'bg-gray-700 text-gray-400';
    }
  };

  return (
    <div className="h-full flex flex-col bg-gray-900">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-4">
          {selectedProduct && (
            <button onClick={handleBackToProducts} className="p-2 hover:bg-gray-700 rounded-lg">
              <ChevronLeft className="w-5 h-5 text-gray-300" />
            </button>
          )}
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <Layers className="w-6 h-6 text-blue-400" />
            Rules Explorer
          </h1>
          {selectedProduct && (
            <div className="flex items-center gap-1 text-sm text-gray-400">
              <ChevronRight className="w-4 h-4" />
              <span className="text-gray-200">{selectedProduct.name}</span>
            </div>
          )}
        </div>
        {dag && (
          <div className="flex items-center gap-6 text-sm">
            <span className="text-gray-400">LOD: <span className="text-purple-400 font-semibold">{getLODLabel(lod)}</span></span>
            <span className="text-gray-400">
              <span className="font-mono font-semibold text-white">{rules.length}</span> rules
            </span>
            <span className="text-gray-400">
              <span className="font-mono font-semibold text-white">{dag.nodes.length}</span> nodes
            </span>
          </div>
        )}
      </div>

      {/* Error Display */}
      {error && (
        <div className="bg-red-900/50 border-b border-red-700 px-4 py-2 flex items-center gap-2 text-red-300">
          <AlertCircle className="w-4 h-4" />
          <span className="text-sm">{error}</span>
        </div>
      )}

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Loading State */}
        {(isLoading || localLoading) && (
          <div className="flex-1 flex items-center justify-center">
            <div className="flex flex-col items-center gap-4">
              <Loader2 className="w-10 h-10 text-blue-400 animate-spin" />
              <span className="text-gray-400">Loading...</span>
            </div>
          </div>
        )}

        {/* Product Selection */}
        {!selectedProduct && !isLoading && !localLoading && (
          <div className="flex-1 p-6 overflow-auto">
            <div className="grid grid-cols-4 gap-4 mb-6">
              {[
                { icon: Package, label: 'Products', value: products.length, color: 'blue' },
                { icon: Layers, label: 'Active', value: products.filter(p => p.status === 'ACTIVE').length, color: 'green' },
                { icon: Database, label: 'Draft', value: products.filter(p => p.status === 'DRAFT').length, color: 'yellow' },
                { icon: Activity, label: 'Pending', value: products.filter(p => p.status === 'PENDING_APPROVAL').length, color: 'purple' },
              ].map(({ icon: Icon, label, value, color }) => (
                <div key={label} className={`bg-${color}-900/30 border border-${color}-800/50 rounded-xl p-4`}>
                  <Icon className={`w-8 h-8 text-${color}-400 mb-2`} />
                  <div className="text-2xl font-bold text-white">{value}</div>
                  <div className="text-sm text-gray-400">{label}</div>
                </div>
              ))}
            </div>

            <h2 className="text-lg font-semibold text-white mb-4">Products</h2>

            {products.length === 0 ? (
              <div className="text-center py-12 text-gray-400">
                <Package className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No products found. Create a product to get started.</p>
              </div>
            ) : (
              <div className="grid grid-cols-3 gap-4">
                {products.map(product => (
                  <div
                    key={product.id}
                    onClick={() => handleSelectProduct(product.id)}
                    className="bg-gray-800 rounded-xl border border-gray-700 p-5 cursor-pointer hover:border-blue-500 transition-colors"
                  >
                    <div className="flex justify-between mb-3">
                      <span className={`px-2 py-1 rounded text-xs font-medium ${getStatusColor(product.status)}`}>
                        {product.status}
                      </span>
                      <span className="text-xs text-gray-500">{product.templateType}</span>
                    </div>
                    <h3 className="font-bold text-lg text-white mb-2">{product.name}</h3>
                    <p className="text-sm text-gray-400 mb-4 line-clamp-2">{product.description}</p>
                    <div className="text-xs text-gray-500">
                      Version {product.version} • Created {new Date(product.createdAt).toLocaleDateString()}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* DAG View */}
        {selectedProduct && dag && !isLoading && !localLoading && (
          <>
            <div className="flex-1 relative">
              {/* Controls */}
              <div className="absolute right-4 top-4 z-10 bg-gray-800 rounded-lg shadow-lg p-2 flex flex-col gap-1 border border-gray-700">
                <button onClick={() => handleZoom('in')} className="p-2 hover:bg-gray-700 rounded text-gray-300">
                  <ZoomIn className="w-5 h-5" />
                </button>
                <div className="text-xs text-center py-1 font-mono text-gray-400">
                  {Math.round(800 / viewBox.w * 100)}%
                </div>
                <button onClick={() => handleZoom('out')} className="p-2 hover:bg-gray-700 rounded text-gray-300">
                  <ZoomOut className="w-5 h-5" />
                </button>
                <hr className="my-1 border-gray-700" />
                <button onClick={resetView} className="p-2 hover:bg-gray-700 rounded text-gray-300">
                  <RotateCcw className="w-5 h-5" />
                </button>
              </div>

              {/* LOD Indicator - Clickable */}
              <div className="absolute left-4 top-4 z-10 bg-gray-800 rounded-lg shadow-lg p-3 border border-gray-700">
                <div className="text-xs font-medium text-gray-500 mb-2">Detail Level</div>
                {[
                  { name: 'Domains', targetW: 1600, targetH: 1100 },
                  { name: 'Patterns', targetW: 1000, targetH: 700 },
                  { name: 'Rules', targetW: 600, targetH: 450 },
                ].map((level, i) => (
                  <button
                    key={level.name}
                    onClick={() => setViewBox({
                      x: -level.targetW / 2,
                      y: -40,
                      w: level.targetW,
                      h: level.targetH,
                    })}
                    className={`block w-full text-left text-xs px-3 py-1.5 rounded transition-colors ${
                      lod === i + 1
                        ? 'bg-blue-600 text-white'
                        : 'text-gray-400 hover:bg-gray-700 hover:text-white cursor-pointer'
                    }`}
                  >
                    {level.name}
                  </button>
                ))}
              </div>

              {/* Empty state */}
              {dag.nodes.length === 0 && (
                <div className="absolute inset-0 flex items-center justify-center">
                  <div className="text-center text-gray-400">
                    <Layers className="w-16 h-16 mx-auto mb-4 opacity-30" />
                    <p className="text-lg mb-2">No rules in this product</p>
                    <p className="text-sm">Add rules to visualize the execution graph</p>
                  </div>
                </div>
              )}

              {/* SVG Canvas */}
              {dag.nodes.length > 0 && (
                <svg
                  ref={svgRef}
                  className="w-full h-full bg-gray-900"
                  viewBox={`${viewBox.x} ${viewBox.y} ${viewBox.w} ${viewBox.h}`}
                  onMouseDown={handleMouseDown}
                  onMouseMove={handleMouseMove}
                  onMouseUp={handleMouseUp}
                  onMouseLeave={handleMouseUp}
                  onWheel={handleWheel}
                  style={{ cursor: isPanning ? 'grabbing' : 'grab' }}
                >
                  <defs>
                    <marker id="arrow" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
                      <polygon points="0 0, 8 3, 0 6" fill="#64748B" />
                    </marker>
                    <marker id="arrow-crit" markerWidth="8" markerHeight="6" refX="8" refY="3" orient="auto">
                      <polygon points="0 0, 8 3, 0 6" fill="#3B82F6" />
                    </marker>
                  </defs>

                  {/* Level bands */}
                  {Array.from(new Set(dag.nodes.map(n => n.level))).map(level => (
                    <g key={`level-${level}`}>
                      <line
                        x1={-500}
                        y1={level * LEVEL_GAP - 15}
                        x2={500}
                        y2={level * LEVEL_GAP - 15}
                        stroke="#374151"
                        strokeDasharray="4,4"
                      />
                      <text x={-480} y={level * LEVEL_GAP + 20} fontSize="12" fill="#6B7280">L{level}</text>
                    </g>
                  ))}

                  {/* Edges */}
                  {dag.edges.map((edge, i) => {
                    const from = dag.nodes.find(n => n.id === edge.from);
                    const to = dag.nodes.find(n => n.id === edge.to);
                    if (!from || !to) return null;

                    return (
                      <line
                        key={i}
                        x1={from.x + dag.nodeW / 2}
                        y1={from.y + dag.nodeH}
                        x2={to.x + dag.nodeW / 2}
                        y2={to.y}
                        stroke={edge.isCritical ? '#3B82F6' : '#64748B'}
                        strokeWidth={edge.isCritical ? 2 : 1}
                        strokeDasharray={edge.isCritical ? undefined : '4,2'}
                        markerEnd={edge.isCritical ? 'url(#arrow-crit)' : 'url(#arrow)'}
                      />
                    );
                  })}

                  {/* Nodes */}
                  {dag.nodes.map(node => {
                    const Icon = PATTERNS[node.pattern]?.icon || Box;
                    const nW = dag.nodeW;
                    const nH = dag.nodeH;
                    const isDisabled = node.enabled === false;

                    return (
                      <g
                        key={node.id}
                        transform={`translate(${node.x}, ${node.y})`}
                        onClick={() => setSelectedNodeId(node.id)}
                        style={{ cursor: 'pointer', opacity: isDisabled ? 0.5 : 1 }}
                      >
                        <rect
                          width={nW}
                          height={nH}
                          rx={8}
                          fill={node.type === 'cluster' ? '#1F2937' : '#FFFFFF'}
                          stroke={selectedNodeId === node.id ? '#3B82F6' : node.color}
                          strokeWidth={selectedNodeId === node.id ? 3 : 2}
                        />
                        <rect x={0} y={0} width={6} height={nH} fill={node.color} rx={3} />
                        <foreignObject x={14} y={nH * 0.15} width={28} height={28}>
                          <Icon style={{ width: 24, height: 24, color: node.color }} />
                        </foreignObject>
                        <text x={48} y={nH * 0.38} fontSize="15" fontWeight="600" fill={node.type === 'cluster' ? '#E5E7EB' : '#1F2937'}>
                          {node.name.length > 20 ? node.name.slice(0, 18) + '..' : node.name}
                        </text>
                        <text x={14} y={nH * 0.72} fontSize="13" fill="#9CA3AF">
                          {node.type === 'cluster'
                            ? `${node.childCount >= 1000 ? (node.childCount/1000).toFixed(0) + 'K' : node.childCount} rules`
                            : node.inputCount !== undefined
                              ? `${node.inputCount} in → ${node.outputCount} out`
                              : 'Rule'}
                        </text>
                        {node.tier !== undefined && (
                          <g transform={`translate(${nW - 36}, 10)`}>
                            <rect width={28} height={20} rx={4} fill={node.tier === 1 ? '#3B82F6' : '#6B7280'} />
                            <text x={14} y={14} fontSize="11" fill="white" textAnchor="middle" fontWeight="600">T{node.tier}</text>
                          </g>
                        )}
                      </g>
                    );
                  })}
                </svg>
              )}

              {/* Legend */}
              <div className="absolute left-4 bottom-4 bg-gray-800 rounded-lg p-3 text-xs border border-gray-700">
                <div className="font-medium text-white mb-2">Legend</div>
                <div className="flex items-center gap-2 text-gray-400 mb-1">
                  <div className="w-4 h-0.5 bg-blue-500" /> Critical path
                </div>
                <div className="flex items-center gap-2 text-gray-400">
                  <div className="w-4 h-0.5 border-t border-dashed border-gray-500" /> Dependency
                </div>
              </div>
            </div>

            {/* Detail Panel */}
            {selectedNode && (
              <div className="w-80 bg-gray-800 border-l border-gray-700 overflow-auto shrink-0">
                <div className="p-4 border-b border-gray-700 flex justify-between items-center">
                  <h2 className="font-bold text-white truncate">{selectedNode.name}</h2>
                  <button onClick={() => setSelectedNodeId(null)} className="p-1 hover:bg-gray-700 rounded">
                    <X className="w-5 h-5 text-gray-400" />
                  </button>
                </div>
                <div className="p-4 space-y-4">
                  <div className="flex gap-2 flex-wrap">
                    <span className="px-2 py-0.5 rounded text-xs bg-gray-700 text-gray-300">{selectedNode.type}</span>
                    {selectedNode.tier !== undefined && (
                      <span className="px-2 py-0.5 rounded text-xs bg-blue-900 text-blue-300">Tier {selectedNode.tier}</span>
                    )}
                    {selectedNode.enabled === false && (
                      <span className="px-2 py-0.5 rounded text-xs bg-red-900 text-red-300">Disabled</span>
                    )}
                  </div>

                  {selectedNode.type === 'cluster' && (
                    <div>
                      <div className="text-sm text-gray-500">Contains</div>
                      <div className="text-2xl font-bold text-blue-400">
                        {selectedNode.childCount.toLocaleString()} <span className="text-sm font-normal text-gray-500">rules</span>
                      </div>
                    </div>
                  )}

                  <div>
                    <div className="text-sm text-gray-500">Execution Level</div>
                    <div className="text-lg font-semibold text-white">L{selectedNode.level}</div>
                  </div>

                  {selectedNode.inputCount !== undefined && (
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <div className="text-sm text-gray-500">Inputs</div>
                        <div className="text-lg font-semibold text-blue-400">{selectedNode.inputCount}</div>
                      </div>
                      <div>
                        <div className="text-sm text-gray-500">Outputs</div>
                        <div className="text-lg font-semibold text-green-400">{selectedNode.outputCount}</div>
                      </div>
                    </div>
                  )}

                  <div>
                    <div className="text-sm text-gray-500">Pattern</div>
                    <div className="flex items-center gap-2 mt-1">
                      <div className="w-3 h-3 rounded" style={{ backgroundColor: selectedNode.color }} />
                      <span className="text-white">{PATTERNS[selectedNode.pattern]?.name || selectedNode.pattern}</span>
                    </div>
                  </div>

                  {/* Show rule details if available */}
                  {selectedRule && (
                    <>
                      <hr className="border-gray-700" />
                      <div>
                        <div className="text-sm text-gray-500 mb-1">Rule ID</div>
                        <div className="text-xs font-mono text-gray-300 break-all">{selectedRule.id}</div>
                      </div>
                      {selectedRule.description && (
                        <div>
                          <div className="text-sm text-gray-500 mb-1">Description</div>
                          <div className="text-sm text-gray-300">{selectedRule.description}</div>
                        </div>
                      )}
                      <div>
                        <div className="text-sm text-gray-500 mb-1">Expression</div>
                        <div className="text-xs font-mono text-gray-400 bg-gray-900 p-2 rounded max-h-32 overflow-auto">
                          {selectedRule.displayExpression}
                        </div>
                      </div>
                      {selectedRule.inputAttributes && selectedRule.inputAttributes.length > 0 && (
                        <div>
                          <div className="text-sm text-gray-500 mb-1">Input Attributes</div>
                          <div className="space-y-1">
                            {selectedRule.inputAttributes.map((attr, i) => (
                              <div key={i} className="text-xs text-gray-400 font-mono truncate">
                                {attr.attributePath}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                      {selectedRule.outputAttributes && selectedRule.outputAttributes.length > 0 && (
                        <div>
                          <div className="text-sm text-gray-500 mb-1">Output Attributes</div>
                          <div className="space-y-1">
                            {selectedRule.outputAttributes.map((attr, i) => (
                              <div key={i} className="text-xs text-green-400 font-mono truncate">
                                {attr.attributePath}
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </>
                  )}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

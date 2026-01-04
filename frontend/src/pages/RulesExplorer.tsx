/**
 * RulesExplorer - DAG Visualization with proper viewBox-based zoom
 */

import React, { useState, useMemo, useCallback, useEffect, useRef } from 'react';
import {
  ZoomIn, ZoomOut, ChevronRight, ChevronLeft,
  Package, Activity, X, Layers, RotateCcw,
  Calculator, GitMerge, Filter, Database, Box, Target,
} from 'lucide-react';

// ============================================================================
// TYPES & CONSTANTS
// ============================================================================

interface Product {
  id: string;
  name: string;
  description: string;
  status: string;
  ruleCount: number;
  attributeCount: number;
}

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

// Default (used for rendering)
let NODE_W = 180;
let NODE_H = 72;
let LEVEL_GAP = 120;
let NODE_GAP = 25;

// ============================================================================
// DATA GENERATION
// ============================================================================

function generateProducts(): Product[] {
  return [
    { id: '1m-risk', name: 'Global Financial Risk Engine', description: '1M rules for risk modeling', status: 'ACTIVE', ruleCount: 1000000, attributeCount: 15000 },
    { id: 'motor', name: 'Motor Insurance', description: 'Premium calculation', status: 'ACTIVE', ruleCount: 47, attributeCount: 234 },
    { id: 'crisis', name: 'Crisis Assessment', description: 'Engineer evaluation', status: 'DRAFT', ruleCount: 12, attributeCount: 89 },
  ];
}

function generateDAG(lod: number): { nodes: DAGNode[]; edges: DAGEdge[]; nodeW: number; nodeH: number } {
  const nodes: DAGNode[] = [];
  const edges: DAGEdge[] = [];
  const patternKeys = Object.keys(PATTERNS);

  // Get LOD-specific sizes
  const sizes = LOD_SIZES[lod as keyof typeof LOD_SIZES] || LOD_SIZES[3];
  const nodeW = sizes.w;
  const nodeH = sizes.h;
  const nodeGap = sizes.gap;
  const levelGap = sizes.levelGap;

  if (lod === 1) {
    // Domain clusters - 9 nodes in 3x3 grid, CENTERED
    const domains = ['Risk', 'Premium', 'Eligibility', 'Discounts', 'Validation', 'Scoring', 'Classification', 'Aggregation', 'Output'];
    const cols = 3;
    const colWidth = nodeW + nodeGap * 2;
    const totalWidth = cols * colWidth - nodeGap * 2;
    const startX = -totalWidth / 2;

    domains.forEach((name, i) => {
      const level = Math.floor(i / 3);
      const col = i % 3;
      nodes.push({
        id: `d${i}`,
        name: `${name} Domain`,
        type: 'cluster',
        level,
        pattern: patternKeys[i % patternKeys.length],
        childCount: Math.floor(1000000 / 9),
        latency: `${10 + i * 5}ms`,
        tier: 1,
        color: PATTERNS[patternKeys[i % patternKeys.length]].color,
        x: startX + col * colWidth,
        y: level * levelGap,
      });
    });
    // Vertical edges only - each node connects to node directly below
    for (let i = 0; i < 6; i++) {
      edges.push({ from: `d${i}`, to: `d${i + 3}`, isCritical: i === 0 });
    }

  } else if (lod === 2) {
    // Pattern groups - 5 columns x 4 rows = 20 nodes, CENTERED
    const domains = ['Risk', 'Premium', 'Eligibility', 'Discounts', 'Validation'];
    const cols = domains.length;
    const colWidth = nodeW + nodeGap * 2;
    const totalWidth = cols * colWidth - nodeGap * 2;
    const startX = -totalWidth / 2;

    let idx = 0;
    domains.forEach((domain, di) => {
      patternKeys.slice(0, 4).forEach((pattern, pi) => {
        nodes.push({
          id: `p${idx}`,
          name: `${domain} ${PATTERNS[pattern].name}`,
          type: 'cluster',
          level: pi,
          pattern,
          childCount: Math.floor(1000000 / 20),
          latency: `${5 + pi * 3}ms`,
          tier: 1,
          color: PATTERNS[pattern].color,
          x: startX + di * colWidth,
          y: pi * levelGap,
        });
        // Vertical edge only within same column
        if (pi > 0) {
          edges.push({ from: `p${idx - 1}`, to: `p${idx}`, isCritical: di === 0 });
        }
        idx++;
      });
    });

  } else {
    // Individual rules - 4 nodes per level, clean vertical edges
    const cols = 4;
    const colWidth = nodeW + nodeGap;
    const totalWidth = cols * colWidth - nodeGap;
    const startX = -totalWidth / 2;

    const structure = [
      { type: 'input' as const, names: ['input_0', 'input_1', 'input_2', 'input_3'] },
      { type: 'rule' as const, names: ['calc_0', 'calc_1', 'calc_2', 'calc_3'] },
      { type: 'rule' as const, names: ['cond_0', 'cond_1', 'cond_2', 'cond_3'] },
      { type: 'rule' as const, names: ['agg_0', 'agg_1', 'agg_2', 'agg_3'] },
      { type: 'output' as const, names: ['result_0', 'result_1', 'result_2', 'result_3'] },
    ];

    let nodeIdx = 0;
    structure.forEach((levelDef, level) => {
      levelDef.names.forEach((name, col) => {
        const id = `r${nodeIdx}`;
        const patternIdx = levelDef.type === 'input' ? 0 : levelDef.type === 'output' ? 6 : (level + col) % 5;
        const pattern = patternKeys[patternIdx];

        nodes.push({
          id,
          name,
          type: levelDef.type,
          level,
          pattern,
          childCount: 0,
          latency: levelDef.type === 'input' ? undefined : `${100 + nodeIdx * 50}ns`,
          tier: levelDef.type === 'input' ? 0 : 1,
          color: PATTERNS[pattern].color,
          x: startX + col * colWidth,
          y: level * levelGap,
        });

        // Vertical edge only - connect to node directly above in same column
        if (level > 0) {
          const parentId = `r${nodeIdx - 4}`; // Previous level, same column
          edges.push({
            from: parentId,
            to: id,
            isCritical: col === 0, // First column is critical path
          });
        }

        nodeIdx++;
      });
    });
  }

  return { nodes, edges, nodeW, nodeH };
}

// ============================================================================
// MAIN COMPONENT
// ============================================================================

export default function RulesExplorer() {
  const [selectedProductId, setSelectedProductId] = useState<string | null>(null);
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null);
  const [products] = useState(generateProducts);

  // ViewBox-based zoom: smaller viewBox = zoomed in
  const [viewBox, setViewBox] = useState({ x: -500, y: -40, w: 1000, h: 700 });
  const [isPanning, setIsPanning] = useState(false);
  const [panStart, setPanStart] = useState({ x: 0, y: 0, vbX: 0, vbY: 0 });
  const svgRef = useRef<SVGSVGElement>(null);

  // LOD based on viewBox width (smaller = more zoomed in = more detail)
  const lod = useMemo(() => {
    if (viewBox.w > 1400) return 1;      // Very zoomed out - domains
    if (viewBox.w > 800) return 2;       // Medium - patterns
    return 3;                             // Zoomed in - rules
  }, [viewBox.w]);

  // Generate DAG
  const dag = useMemo(() => {
    if (!selectedProductId) return null;
    return generateDAG(lod);
  }, [selectedProductId, lod]);

  const selectedNode = dag?.nodes.find(n => n.id === selectedNodeId);

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

  // Reset view when product changes
  useEffect(() => {
    if (selectedProductId) {
      setViewBox({ x: -500, y: -40, w: 1000, h: 700 });
      setSelectedNodeId(null);
    }
  }, [selectedProductId]);

  const getLODLabel = (l: number) => {
    if (l === 1) return 'Domain Clusters';
    if (l === 2) return 'Pattern Groups';
    return 'Individual Rules';
  };

  return (
    <div className="h-full flex flex-col bg-gray-900">
      {/* Header */}
      <div className="bg-gray-800 border-b border-gray-700 px-4 py-3 flex items-center justify-between shrink-0">
        <div className="flex items-center gap-4">
          {selectedProductId && (
            <button onClick={() => setSelectedProductId(null)} className="p-2 hover:bg-gray-700 rounded-lg">
              <ChevronLeft className="w-5 h-5 text-gray-300" />
            </button>
          )}
          <h1 className="text-xl font-bold text-white flex items-center gap-2">
            <Layers className="w-6 h-6 text-blue-400" />
            Rules Explorer
          </h1>
          {selectedProductId && (
            <div className="flex items-center gap-1 text-sm text-gray-400">
              <ChevronRight className="w-4 h-4" />
              <span className="text-gray-200">{products.find(p => p.id === selectedProductId)?.name}</span>
            </div>
          )}
        </div>
        {dag && (
          <div className="flex items-center gap-6 text-sm">
            <span className="text-gray-400">LOD: <span className="text-purple-400 font-semibold">{getLODLabel(lod)}</span></span>
            <span className="text-gray-400"><span className="font-mono font-semibold text-white">{dag.nodes.length}</span> nodes</span>
          </div>
        )}
      </div>

      {/* Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Product Selection */}
        {!selectedProductId && (
          <div className="flex-1 p-6 overflow-auto">
            <div className="grid grid-cols-4 gap-4 mb-6">
              {[
                { icon: Package, label: 'Products', value: products.length, color: 'blue' },
                { icon: Layers, label: 'Total Rules', value: products.reduce((s, p) => s + p.ruleCount, 0).toLocaleString(), color: 'green' },
                { icon: Database, label: 'Attributes', value: products.reduce((s, p) => s + p.attributeCount, 0).toLocaleString(), color: 'purple' },
                { icon: Activity, label: 'Health', value: '99.2%', color: 'amber' },
              ].map(({ icon: Icon, label, value, color }) => (
                <div key={label} className={`bg-${color}-900/30 border border-${color}-800/50 rounded-xl p-4`}>
                  <Icon className={`w-8 h-8 text-${color}-400 mb-2`} />
                  <div className="text-2xl font-bold text-white">{value}</div>
                  <div className="text-sm text-gray-400">{label}</div>
                </div>
              ))}
            </div>
            <h2 className="text-lg font-semibold text-white mb-4">Products</h2>
            <div className="grid grid-cols-3 gap-4">
              {products.map(product => (
                <div
                  key={product.id}
                  onClick={() => setSelectedProductId(product.id)}
                  className="bg-gray-800 rounded-xl border border-gray-700 p-5 cursor-pointer hover:border-blue-500 transition-colors"
                >
                  <div className="flex justify-between mb-3">
                    <span className={`px-2 py-1 rounded text-xs font-medium ${product.status === 'ACTIVE' ? 'bg-green-900 text-green-300' : 'bg-yellow-900 text-yellow-300'}`}>
                      {product.status}
                    </span>
                  </div>
                  <h3 className="font-bold text-lg text-white mb-2">{product.name}</h3>
                  <p className="text-sm text-gray-400 mb-4">{product.description}</p>
                  <div className="grid grid-cols-2 gap-3">
                    <div className="bg-gray-900 rounded-lg p-2 text-center">
                      <div className="text-xl font-bold text-blue-400">
                        {product.ruleCount >= 1000000 ? `${(product.ruleCount / 1000000).toFixed(1)}M` : product.ruleCount}
                      </div>
                      <div className="text-xs text-gray-500">Rules</div>
                    </div>
                    <div className="bg-gray-900 rounded-lg p-2 text-center">
                      <div className="text-xl font-bold text-purple-400">
                        {product.attributeCount >= 1000 ? `${(product.attributeCount / 1000).toFixed(1)}K` : product.attributeCount}
                      </div>
                      <div className="text-xs text-gray-500">Attributes</div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* DAG View */}
        {selectedProductId && dag && (
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

              {/* SVG Canvas */}
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
                  return (
                    <g
                      key={node.id}
                      transform={`translate(${node.x}, ${node.y})`}
                      onClick={() => setSelectedNodeId(node.id)}
                      style={{ cursor: 'pointer' }}
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
                          : node.latency || 'Input'}
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
              <div className="w-72 bg-gray-800 border-l border-gray-700 overflow-auto shrink-0">
                <div className="p-4 border-b border-gray-700 flex justify-between items-center">
                  <h2 className="font-bold text-white truncate">{selectedNode.name}</h2>
                  <button onClick={() => setSelectedNodeId(null)} className="p-1 hover:bg-gray-700 rounded">
                    <X className="w-5 h-5 text-gray-400" />
                  </button>
                </div>
                <div className="p-4 space-y-4">
                  <div className="flex gap-2">
                    <span className="px-2 py-0.5 rounded text-xs bg-gray-700 text-gray-300">{selectedNode.type}</span>
                    {selectedNode.tier !== undefined && (
                      <span className="px-2 py-0.5 rounded text-xs bg-blue-900 text-blue-300">Tier {selectedNode.tier}</span>
                    )}
                  </div>
                  {selectedNode.type === 'cluster' && (
                    <div>
                      <div className="text-sm text-gray-500">Contains</div>
                      <div className="text-2xl font-bold text-blue-400">{selectedNode.childCount.toLocaleString()} <span className="text-sm font-normal text-gray-500">rules</span></div>
                    </div>
                  )}
                  <div>
                    <div className="text-sm text-gray-500">Execution Level</div>
                    <div className="text-lg font-semibold text-white">L{selectedNode.level}</div>
                  </div>
                  {selectedNode.latency && (
                    <div>
                      <div className="text-sm text-gray-500">Latency</div>
                      <div className="text-lg font-semibold text-green-400">{selectedNode.latency}</div>
                    </div>
                  )}
                  <div>
                    <div className="text-sm text-gray-500">Pattern</div>
                    <div className="flex items-center gap-2 mt-1">
                      <div className="w-3 h-3 rounded" style={{ backgroundColor: selectedNode.color }} />
                      <span className="text-white">{PATTERNS[selectedNode.pattern]?.name || selectedNode.pattern}</span>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}

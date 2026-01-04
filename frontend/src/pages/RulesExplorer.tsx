/**
 * RulesExplorer - Next-generation Rules Visualization
 *
 * A smart, pattern-aware rule exploration interface with:
 * - Semantic grouping via natural language analysis
 * - Multi-level zoom (Universe → Group → Rule → Trace)
 * - Efficient virtualized rendering for large datasets
 * - Responsive layout with toggleable panels
 * - Real-time metrics and insights
 */

import React, { useState, useCallback, useMemo, useEffect, useRef } from 'react';
import {
  Search, Filter, ZoomIn, ZoomOut, Maximize2, Minimize2,
  ChevronRight, ChevronDown, ChevronLeft, Layers, Grid3X3,
  BarChart3, Lightbulb, AlertCircle, CheckCircle2, Clock,
  Cpu, Database, GitBranch, Play, Pause, RotateCcw,
  PanelLeftClose, PanelLeft, PanelRightClose, PanelRight,
  Settings2, Eye, EyeOff, TrendingUp, TrendingDown,
  Sparkles, Brain, Zap, Target, Network, Box, Boxes,
  LayoutGrid, LayoutList, Workflow, ArrowUpRight, Info,
  Hash, Type, ToggleLeft, Calendar, List, Braces
} from 'lucide-react';

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

interface PatternCategory {
  type: 'calculation' | 'conditional' | 'transformation' | 'validation' | 'lookup' | 'aggregation';
  subtype: string;
}

interface SemanticGroup {
  id: string;
  name: string;
  description: string;
  keywords: string[];
  confidence: number;
  ruleIds: string[];
  patterns: PatternCategory[];
  metrics: GroupMetrics;
  color: string;
  icon: React.ReactNode;
}

interface GroupMetrics {
  ruleCount: number;
  avgComplexity: number;
  maxDepth: number;
  parallelismScore: number;
  estimatedTimeNs: number;
  bytecodeRate: number;
  llmRuleCount: number;
}

interface RuleInsight {
  ruleId: string;
  patterns: PatternCategory[];
  tags: string[];
  similarRules: SimilarityMatch[];
  complexity: ComplexityBreakdown;
  summary: string;
  suggestions: Suggestion[];
  facts: string[];
}

interface SimilarityMatch {
  ruleId: string;
  similarityScore: number;
  matchingPatterns: PatternCategory[];
  reason: string;
}

interface ComplexityBreakdown {
  totalNodes: number;
  maxDepth: number;
  variableCount: number;
  operatorCount: number;
  conditionCount: number;
  loopCount: number;
  cyclomaticComplexity: number;
}

interface Suggestion {
  priority: 'high' | 'medium' | 'low' | 'info';
  category: 'performance' | 'maintainability' | 'correctness' | 'simplification';
  title: string;
  description: string;
  estimatedImpact: string;
}

interface Rule {
  id: string;
  name: string;
  expression: object;
  outputAttribute: string;
  dependencies: string[];
  enabled: boolean;
  tier: 0 | 1 | 2;
  avgLatencyNs: number;
  evaluationCount: number;
  isLlm: boolean;
}

interface ZoomLevel {
  level: number;
  name: string;
  description: string;
}

// ============================================================================
// ZOOM LEVELS CONFIGURATION
// ============================================================================

const ZOOM_LEVELS: ZoomLevel[] = [
  { level: 0, name: 'Universe', description: 'All semantic groups overview' },
  { level: 1, name: 'Group', description: 'Rules within a group' },
  { level: 2, name: 'Rule', description: 'Individual rule details' },
  { level: 3, name: 'Trace', description: 'Execution trace & bytecode' },
];

// ============================================================================
// MOCK DATA GENERATOR (for demonstration)
// ============================================================================

const generateMockGroups = (): SemanticGroup[] => [
  {
    id: 'pricing',
    name: 'Pricing & Calculations',
    description: 'Rules that compute prices, premiums, rates, and financial amounts',
    keywords: ['price', 'premium', 'rate', 'cost', 'fee', 'amount'],
    confidence: 0.92,
    ruleIds: ['calc_premium', 'base_rate', 'total_cost', 'tax_amount', 'commission'],
    patterns: [{ type: 'calculation', subtype: 'multiplication' }],
    metrics: {
      ruleCount: 12,
      avgComplexity: 4.2,
      maxDepth: 5,
      parallelismScore: 0.75,
      estimatedTimeNs: 2500,
      bytecodeRate: 0.92,
      llmRuleCount: 0,
    },
    color: '#22C55E',
    icon: <TrendingUp className="w-5 h-5" />,
  },
  {
    id: 'risk',
    name: 'Risk Assessment',
    description: 'Rules that evaluate risk scores, factors, and ratings',
    keywords: ['risk', 'score', 'factor', 'rating', 'assessment'],
    confidence: 0.88,
    ruleIds: ['risk_factor', 'risk_score', 'hazard_level', 'exposure_rating'],
    patterns: [{ type: 'conditional', subtype: 'tiered' }],
    metrics: {
      ruleCount: 8,
      avgComplexity: 6.5,
      maxDepth: 7,
      parallelismScore: 0.45,
      estimatedTimeNs: 4200,
      bytecodeRate: 0.88,
      llmRuleCount: 1,
    },
    color: '#EAB308',
    icon: <Target className="w-5 h-5" />,
  },
  {
    id: 'discount',
    name: 'Discounts & Promotions',
    description: 'Rules that apply discounts, rebates, and promotional offers',
    keywords: ['discount', 'rebate', 'offer', 'promotion', 'loyalty'],
    confidence: 0.85,
    ruleIds: ['loyalty_discount', 'volume_rebate', 'promo_code', 'early_bird'],
    patterns: [{ type: 'calculation', subtype: 'percentage' }],
    metrics: {
      ruleCount: 6,
      avgComplexity: 3.1,
      maxDepth: 4,
      parallelismScore: 0.90,
      estimatedTimeNs: 1800,
      bytecodeRate: 1.0,
      llmRuleCount: 0,
    },
    color: '#8B5CF6',
    icon: <Sparkles className="w-5 h-5" />,
  },
  {
    id: 'eligibility',
    name: 'Eligibility & Validation',
    description: 'Rules that determine eligibility and validate business constraints',
    keywords: ['eligible', 'valid', 'approve', 'qualify', 'permitted'],
    confidence: 0.82,
    ruleIds: ['is_eligible', 'validate_age', 'check_status', 'approve_request'],
    patterns: [{ type: 'validation', subtype: 'business_rule' }],
    metrics: {
      ruleCount: 10,
      avgComplexity: 5.8,
      maxDepth: 6,
      parallelismScore: 0.60,
      estimatedTimeNs: 3500,
      bytecodeRate: 0.80,
      llmRuleCount: 2,
    },
    color: '#3B82F6',
    icon: <CheckCircle2 className="w-5 h-5" />,
  },
  {
    id: 'coverage',
    name: 'Coverage & Benefits',
    description: 'Rules for insurance coverage, benefits, and policy details',
    keywords: ['cover', 'benefit', 'policy', 'claim', 'limit'],
    confidence: 0.79,
    ruleIds: ['coverage_limit', 'benefit_amount', 'deductible', 'exclusion_check'],
    patterns: [{ type: 'lookup', subtype: 'table_lookup' }],
    metrics: {
      ruleCount: 11,
      avgComplexity: 4.9,
      maxDepth: 5,
      parallelismScore: 0.55,
      estimatedTimeNs: 3100,
      bytecodeRate: 0.73,
      llmRuleCount: 3,
    },
    color: '#EC4899',
    icon: <Box className="w-5 h-5" />,
  },
];

const generateMockRules = (): Rule[] => [
  {
    id: 'risk_factor',
    name: 'Risk Factor Calculator',
    expression: { if: [{ '<': [{ var: 'age' }, 25] }, 1.5, { '<': [{ var: 'age' }, 35] }, 1.2, 1.0] },
    outputAttribute: 'risk_factor',
    dependencies: ['age', 'claims_history'],
    enabled: true,
    tier: 1,
    avgLatencyNs: 892,
    evaluationCount: 1247892,
    isLlm: false,
  },
  {
    id: 'base_premium',
    name: 'Base Premium',
    expression: { '*': [{ var: 'vehicle_value' }, 0.02] },
    outputAttribute: 'base_premium',
    dependencies: ['vehicle_value'],
    enabled: true,
    tier: 1,
    avgLatencyNs: 330,
    evaluationCount: 1247892,
    isLlm: false,
  },
  {
    id: 'final_premium',
    name: 'Final Premium',
    expression: { '*': [{ var: 'base_premium' }, { var: 'risk_factor' }] },
    outputAttribute: 'final_premium',
    dependencies: ['base_premium', 'risk_factor'],
    enabled: true,
    tier: 1,
    avgLatencyNs: 445,
    evaluationCount: 1247892,
    isLlm: false,
  },
];

// ============================================================================
// UTILITY COMPONENTS
// ============================================================================

const MetricBadge: React.FC<{
  label: string;
  value: string | number;
  icon?: React.ReactNode;
  color?: string;
  trend?: 'up' | 'down' | 'neutral';
}> = ({ label, value, icon, color = 'gray', trend }) => (
  <div className="flex items-center gap-1.5 px-2 py-1 rounded-md bg-gray-100 dark:bg-gray-800">
    {icon}
    <span className="text-xs text-gray-500 dark:text-gray-400">{label}</span>
    <span className={`text-sm font-medium text-${color}-600 dark:text-${color}-400`}>
      {value}
    </span>
    {trend === 'up' && <TrendingUp className="w-3 h-3 text-green-500" />}
    {trend === 'down' && <TrendingDown className="w-3 h-3 text-red-500" />}
  </div>
);

const ProgressBar: React.FC<{
  value: number;
  max?: number;
  color?: string;
  showLabel?: boolean;
}> = ({ value, max = 100, color = 'blue', showLabel = true }) => {
  const percentage = Math.min((value / max) * 100, 100);
  return (
    <div className="flex items-center gap-2">
      <div className="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
        <div
          className={`h-full bg-${color}-500 rounded-full transition-all duration-300`}
          style={{ width: `${percentage}%` }}
        />
      </div>
      {showLabel && (
        <span className="text-xs text-gray-500 w-10 text-right">{percentage.toFixed(0)}%</span>
      )}
    </div>
  );
};

const TierBadge: React.FC<{ tier: 0 | 1 | 2 }> = ({ tier }) => {
  const configs = {
    0: { label: 'AST', color: 'bg-gray-200 text-gray-600', icon: '░░' },
    1: { label: 'Bytecode', color: 'bg-blue-100 text-blue-700', icon: '██' },
    2: { label: 'JIT', color: 'bg-purple-100 text-purple-700', icon: '▓▓' },
  };
  const config = configs[tier];
  return (
    <span className={`px-2 py-0.5 rounded text-xs font-mono ${config.color}`}>
      T{tier} {config.icon}
    </span>
  );
};

const PatternTag: React.FC<{ pattern: PatternCategory }> = ({ pattern }) => {
  const colorMap: Record<string, string> = {
    calculation: 'bg-green-100 text-green-700 border-green-200',
    conditional: 'bg-yellow-100 text-yellow-700 border-yellow-200',
    transformation: 'bg-blue-100 text-blue-700 border-blue-200',
    validation: 'bg-red-100 text-red-700 border-red-200',
    lookup: 'bg-purple-100 text-purple-700 border-purple-200',
    aggregation: 'bg-pink-100 text-pink-700 border-pink-200',
  };
  return (
    <span className={`px-2 py-0.5 rounded-full text-xs border ${colorMap[pattern.type] || 'bg-gray-100'}`}>
      {pattern.type}: {pattern.subtype}
    </span>
  );
};

// ============================================================================
// PANEL COMPONENTS
// ============================================================================

const InsightsPanel: React.FC<{
  visible: boolean;
  onToggle: () => void;
  insights: string[];
  suggestions: Suggestion[];
}> = ({ visible, onToggle, insights, suggestions }) => {
  if (!visible) return null;

  return (
    <div className="border-l border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 w-80 flex-shrink-0 overflow-hidden flex flex-col">
      <div className="p-4 border-b border-gray-200 dark:border-gray-700 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Lightbulb className="w-5 h-5 text-yellow-500" />
          <h3 className="font-semibold">Insights</h3>
        </div>
        <button onClick={onToggle} className="p-1 hover:bg-gray-100 rounded">
          <PanelRightClose className="w-4 h-4" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4">
        {/* Interesting Facts */}
        <div>
          <h4 className="text-sm font-medium text-gray-500 mb-2 flex items-center gap-1">
            <Sparkles className="w-4 h-4" />
            Interesting Facts
          </h4>
          <div className="space-y-2">
            {insights.map((insight, i) => (
              <div key={i} className="p-2 bg-blue-50 dark:bg-blue-900/20 rounded-lg text-sm">
                {insight}
              </div>
            ))}
          </div>
        </div>

        {/* Suggestions */}
        <div>
          <h4 className="text-sm font-medium text-gray-500 mb-2 flex items-center gap-1">
            <AlertCircle className="w-4 h-4" />
            Suggestions
          </h4>
          <div className="space-y-2">
            {suggestions.map((suggestion, i) => (
              <div
                key={i}
                className={`p-3 rounded-lg border ${
                  suggestion.priority === 'high'
                    ? 'border-red-200 bg-red-50'
                    : suggestion.priority === 'medium'
                    ? 'border-yellow-200 bg-yellow-50'
                    : 'border-gray-200 bg-gray-50'
                }`}
              >
                <div className="font-medium text-sm">{suggestion.title}</div>
                <div className="text-xs text-gray-600 mt-1">{suggestion.description}</div>
                <div className="text-xs text-gray-500 mt-1 flex items-center gap-1">
                  <Zap className="w-3 h-3" />
                  {suggestion.estimatedImpact}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

const MetricsPanel: React.FC<{
  visible: boolean;
  onToggle: () => void;
  globalMetrics: {
    totalRules: number;
    avgComplexity: number;
    bytecodeRate: number;
    llmRules: number;
    avgLatency: string;
    totalEvaluations: string;
  };
}> = ({ visible, onToggle, globalMetrics }) => {
  if (!visible) return null;

  return (
    <div className="border-b border-gray-200 dark:border-gray-700 bg-gradient-to-r from-gray-50 to-white dark:from-gray-800 dark:to-gray-900 p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-semibold flex items-center gap-2">
          <BarChart3 className="w-5 h-5 text-blue-500" />
          Global Metrics
        </h3>
        <button onClick={onToggle} className="p-1 hover:bg-gray-100 rounded text-gray-500">
          <Minimize2 className="w-4 h-4" />
        </button>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-blue-600">{globalMetrics.totalRules}</div>
          <div className="text-xs text-gray-500">Total Rules</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-green-600">{globalMetrics.avgComplexity.toFixed(1)}</div>
          <div className="text-xs text-gray-500">Avg Complexity</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-purple-600">{(globalMetrics.bytecodeRate * 100).toFixed(0)}%</div>
          <div className="text-xs text-gray-500">Bytecode Rate</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-amber-600">{globalMetrics.llmRules}</div>
          <div className="text-xs text-gray-500">LLM Rules</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-cyan-600">{globalMetrics.avgLatency}</div>
          <div className="text-xs text-gray-500">Avg Latency</div>
        </div>
        <div className="bg-white dark:bg-gray-800 rounded-lg p-3 shadow-sm">
          <div className="text-2xl font-bold text-rose-600">{globalMetrics.totalEvaluations}</div>
          <div className="text-xs text-gray-500">Evaluations</div>
        </div>
      </div>
    </div>
  );
};

// ============================================================================
// ZOOM LEVEL VIEWS
// ============================================================================

const UniverseView: React.FC<{
  groups: SemanticGroup[];
  onSelectGroup: (group: SemanticGroup) => void;
}> = ({ groups, onSelectGroup }) => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 p-4">
      {groups.map((group) => (
        <div
          key={group.id}
          onClick={() => onSelectGroup(group)}
          className="bg-white dark:bg-gray-800 rounded-xl border border-gray-200 dark:border-gray-700 p-4 cursor-pointer hover:shadow-lg hover:border-blue-300 transition-all duration-200 group"
        >
          {/* Header */}
          <div className="flex items-start justify-between mb-3">
            <div
              className="w-10 h-10 rounded-lg flex items-center justify-center"
              style={{ backgroundColor: `${group.color}20`, color: group.color }}
            >
              {group.icon}
            </div>
            <div className="flex items-center gap-1">
              <span className="text-2xl font-bold">{group.metrics.ruleCount}</span>
              <span className="text-xs text-gray-500">rules</span>
            </div>
          </div>

          {/* Title & Description */}
          <h3 className="font-semibold text-lg mb-1 group-hover:text-blue-600 transition-colors">
            {group.name}
          </h3>
          <p className="text-sm text-gray-500 mb-3 line-clamp-2">{group.description}</p>

          {/* Metrics */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Health</span>
              <span className="text-green-600">{(group.confidence * 100).toFixed(0)}%</span>
            </div>
            <ProgressBar value={group.confidence * 100} color="green" showLabel={false} />

            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Bytecode</span>
              <span className="text-blue-600">{(group.metrics.bytecodeRate * 100).toFixed(0)}%</span>
            </div>
            <ProgressBar value={group.metrics.bytecodeRate * 100} color="blue" showLabel={false} />
          </div>

          {/* Tags */}
          <div className="flex flex-wrap gap-1 mt-3">
            {group.keywords.slice(0, 3).map((kw) => (
              <span key={kw} className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs">
                {kw}
              </span>
            ))}
            {group.keywords.length > 3 && (
              <span className="px-2 py-0.5 bg-gray-100 dark:bg-gray-700 rounded text-xs">
                +{group.keywords.length - 3}
              </span>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-between mt-4 pt-3 border-t border-gray-100 dark:border-gray-700">
            <div className="flex items-center gap-2 text-xs text-gray-500">
              <Clock className="w-3 h-3" />
              {(group.metrics.estimatedTimeNs / 1000).toFixed(1)}μs avg
            </div>
            {group.metrics.llmRuleCount > 0 && (
              <div className="flex items-center gap-1 text-xs text-purple-600">
                <Brain className="w-3 h-3" />
                {group.metrics.llmRuleCount} LLM
              </div>
            )}
            <ChevronRight className="w-4 h-4 text-gray-400 group-hover:text-blue-500 transition-colors" />
          </div>
        </div>
      ))}
    </div>
  );
};

const GroupView: React.FC<{
  group: SemanticGroup;
  rules: Rule[];
  onSelectRule: (rule: Rule) => void;
  onBack: () => void;
}> = ({ group, rules, onSelectRule, onBack }) => {
  const [viewMode, setViewMode] = useState<'grid' | 'list' | 'dag'>('grid');
  const [sortBy, setSortBy] = useState<'name' | 'complexity' | 'latency'>('name');

  const groupRules = rules.filter((r) => group.ruleIds.includes(r.id));

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
        <div className="flex items-center gap-3 mb-3">
          <button
            onClick={onBack}
            className="p-2 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
          >
            <ChevronLeft className="w-5 h-5" />
          </button>
          <div
            className="w-10 h-10 rounded-lg flex items-center justify-center"
            style={{ backgroundColor: `${group.color}20`, color: group.color }}
          >
            {group.icon}
          </div>
          <div>
            <h2 className="text-xl font-bold">{group.name}</h2>
            <p className="text-sm text-gray-500">{group.description}</p>
          </div>
        </div>

        {/* Controls */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <button
              onClick={() => setViewMode('grid')}
              className={`p-2 rounded ${viewMode === 'grid' ? 'bg-blue-100 text-blue-600' : 'hover:bg-gray-100'}`}
            >
              <LayoutGrid className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewMode('list')}
              className={`p-2 rounded ${viewMode === 'list' ? 'bg-blue-100 text-blue-600' : 'hover:bg-gray-100'}`}
            >
              <LayoutList className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewMode('dag')}
              className={`p-2 rounded ${viewMode === 'dag' ? 'bg-blue-100 text-blue-600' : 'hover:bg-gray-100'}`}
            >
              <Workflow className="w-4 h-4" />
            </button>
          </div>

          <div className="flex items-center gap-2">
            <span className="text-sm text-gray-500">Sort by:</span>
            <select
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as any)}
              className="text-sm border rounded px-2 py-1"
            >
              <option value="name">Name</option>
              <option value="complexity">Complexity</option>
              <option value="latency">Latency</option>
            </select>
          </div>
        </div>
      </div>

      {/* Rules Grid */}
      <div className="flex-1 overflow-auto p-4">
        <div className={`grid gap-4 ${viewMode === 'grid' ? 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3' : 'grid-cols-1'}`}>
          {groupRules.map((rule) => (
            <div
              key={rule.id}
              onClick={() => onSelectRule(rule)}
              className="bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4 cursor-pointer hover:shadow-md hover:border-blue-300 transition-all"
            >
              <div className="flex items-start justify-between mb-2">
                <div>
                  <h4 className="font-medium">{rule.name}</h4>
                  <code className="text-xs text-gray-500">{rule.id}</code>
                </div>
                <TierBadge tier={rule.tier} />
              </div>

              <div className="text-sm text-gray-600 mb-3 font-mono bg-gray-50 dark:bg-gray-900 p-2 rounded">
                → {rule.outputAttribute}
              </div>

              <div className="flex flex-wrap gap-2 mb-3">
                {rule.dependencies.slice(0, 3).map((dep) => (
                  <span key={dep} className="px-2 py-0.5 bg-blue-50 text-blue-600 rounded text-xs">
                    {dep}
                  </span>
                ))}
                {rule.dependencies.length > 3 && (
                  <span className="px-2 py-0.5 bg-gray-100 rounded text-xs">
                    +{rule.dependencies.length - 3}
                  </span>
                )}
              </div>

              <div className="flex items-center justify-between text-xs text-gray-500">
                <span className="flex items-center gap-1">
                  <Clock className="w-3 h-3" />
                  {rule.avgLatencyNs}ns
                </span>
                <span className="flex items-center gap-1">
                  <Zap className="w-3 h-3" />
                  {(rule.evaluationCount / 1000000).toFixed(1)}M evals
                </span>
                {rule.isLlm && (
                  <span className="flex items-center gap-1 text-purple-600">
                    <Brain className="w-3 h-3" />
                    LLM
                  </span>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

const RuleDetailView: React.FC<{
  rule: Rule;
  insight: RuleInsight;
  onBack: () => void;
  onViewTrace: () => void;
}> = ({ rule, insight, onBack, onViewTrace }) => {
  const [activeTab, setActiveTab] = useState<'expression' | 'metrics' | 'similar' | 'tests'>('expression');

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-900">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-3 mb-2">
          <button onClick={onBack} className="p-2 hover:bg-gray-100 rounded-lg">
            <ChevronLeft className="w-5 h-5" />
          </button>
          <div className="flex-1">
            <div className="flex items-center gap-3">
              <h2 className="text-xl font-bold">{rule.name}</h2>
              <TierBadge tier={rule.tier} />
              {rule.isLlm && (
                <span className="px-2 py-0.5 bg-purple-100 text-purple-700 rounded text-xs flex items-center gap-1">
                  <Brain className="w-3 h-3" /> LLM-powered
                </span>
              )}
            </div>
            <code className="text-sm text-gray-500">{rule.id}</code>
          </div>
          <button
            onClick={onViewTrace}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 flex items-center gap-2"
          >
            <Play className="w-4 h-4" />
            View Trace
          </button>
        </div>

        {/* Summary */}
        <p className="text-gray-600 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 p-3 rounded-lg">
          {insight.summary}
        </p>

        {/* Tags */}
        <div className="flex flex-wrap gap-2 mt-3">
          {insight.tags.map((tag) => (
            <span key={tag} className="px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded-full text-sm">
              #{tag}
            </span>
          ))}
          {insight.patterns.map((pattern, i) => (
            <PatternTag key={i} pattern={pattern} />
          ))}
        </div>
      </div>

      {/* Tabs */}
      <div className="border-b border-gray-200 dark:border-gray-700">
        <div className="flex">
          {(['expression', 'metrics', 'similar', 'tests'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-3 text-sm font-medium border-b-2 transition-colors ${
                activeTab === tab
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700'
              }`}
            >
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'expression' && (
          <div className="space-y-4">
            {/* Expression */}
            <div>
              <h3 className="font-medium mb-2 flex items-center gap-2">
                <Braces className="w-4 h-4" />
                JSON Logic Expression
              </h3>
              <pre className="bg-gray-900 text-green-400 p-4 rounded-lg overflow-x-auto text-sm">
                {JSON.stringify(rule.expression, null, 2)}
              </pre>
            </div>

            {/* Dependencies */}
            <div className="grid grid-cols-2 gap-4">
              <div>
                <h3 className="font-medium mb-2 flex items-center gap-2">
                  <ArrowUpRight className="w-4 h-4" />
                  Inputs ({rule.dependencies.length})
                </h3>
                <div className="space-y-1">
                  {rule.dependencies.map((dep) => (
                    <div key={dep} className="flex items-center gap-2 p-2 bg-gray-50 dark:bg-gray-800 rounded">
                      <Database className="w-4 h-4 text-blue-500" />
                      <code className="text-sm">{dep}</code>
                    </div>
                  ))}
                </div>
              </div>
              <div>
                <h3 className="font-medium mb-2 flex items-center gap-2">
                  <Box className="w-4 h-4" />
                  Output
                </h3>
                <div className="flex items-center gap-2 p-2 bg-green-50 dark:bg-green-900/20 rounded">
                  <Target className="w-4 h-4 text-green-500" />
                  <code className="text-sm">{rule.outputAttribute}</code>
                </div>
              </div>
            </div>

            {/* Complexity Breakdown */}
            <div>
              <h3 className="font-medium mb-2 flex items-center gap-2">
                <Cpu className="w-4 h-4" />
                Complexity Breakdown
              </h3>
              <div className="grid grid-cols-3 md:grid-cols-6 gap-4">
                {Object.entries(insight.complexity).map(([key, value]) => (
                  <div key={key} className="bg-gray-50 dark:bg-gray-800 p-3 rounded-lg text-center">
                    <div className="text-xl font-bold text-blue-600">{value}</div>
                    <div className="text-xs text-gray-500">{key.replace(/([A-Z])/g, ' $1').trim()}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}

        {activeTab === 'metrics' && (
          <div className="space-y-4">
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
              <div className="bg-gradient-to-br from-blue-50 to-blue-100 dark:from-blue-900/20 dark:to-blue-800/20 p-4 rounded-xl">
                <div className="text-3xl font-bold text-blue-600">{rule.avgLatencyNs}ns</div>
                <div className="text-sm text-gray-600">Avg Latency</div>
              </div>
              <div className="bg-gradient-to-br from-green-50 to-green-100 dark:from-green-900/20 dark:to-green-800/20 p-4 rounded-xl">
                <div className="text-3xl font-bold text-green-600">
                  {(rule.evaluationCount / 1000000).toFixed(2)}M
                </div>
                <div className="text-sm text-gray-600">Evaluations</div>
              </div>
              <div className="bg-gradient-to-br from-purple-50 to-purple-100 dark:from-purple-900/20 dark:to-purple-800/20 p-4 rounded-xl">
                <div className="text-3xl font-bold text-purple-600">T{rule.tier}</div>
                <div className="text-sm text-gray-600">Compilation Tier</div>
              </div>
              <div className="bg-gradient-to-br from-amber-50 to-amber-100 dark:from-amber-900/20 dark:to-amber-800/20 p-4 rounded-xl">
                <div className="text-3xl font-bold text-amber-600">0%</div>
                <div className="text-sm text-gray-600">Error Rate</div>
              </div>
            </div>

            {/* Latency Distribution (placeholder) */}
            <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-xl">
              <h3 className="font-medium mb-3">Latency Distribution</h3>
              <div className="h-32 flex items-end justify-center gap-1">
                {[1, 2, 4, 7, 10, 8, 5, 3, 2, 1, 1, 0, 0, 0].map((h, i) => (
                  <div
                    key={i}
                    className="w-6 bg-blue-500 rounded-t"
                    style={{ height: `${h * 10}%` }}
                  />
                ))}
              </div>
              <div className="flex justify-between text-xs text-gray-500 mt-2">
                <span>100ns</span>
                <span>1μs</span>
                <span>10μs</span>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'similar' && (
          <div className="space-y-3">
            <h3 className="font-medium">Similar Rules ({insight.similarRules.length})</h3>
            {insight.similarRules.length === 0 ? (
              <p className="text-gray-500">No similar rules found</p>
            ) : (
              insight.similarRules.map((match) => (
                <div key={match.ruleId} className="flex items-center justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                  <div>
                    <code className="font-medium">{match.ruleId}</code>
                    <p className="text-sm text-gray-500">{match.reason}</p>
                  </div>
                  <div className="text-right">
                    <div className="text-lg font-bold text-blue-600">
                      {(match.similarityScore * 100).toFixed(0)}%
                    </div>
                    <div className="text-xs text-gray-500">similarity</div>
                  </div>
                </div>
              ))
            )}
          </div>
        )}

        {activeTab === 'tests' && (
          <div className="text-center py-8 text-gray-500">
            <Info className="w-12 h-12 mx-auto mb-2 opacity-50" />
            <p>Test scenarios will appear here</p>
          </div>
        )}
      </div>

      {/* Facts */}
      {insight.facts.length > 0 && (
        <div className="border-t border-gray-200 dark:border-gray-700 p-4 bg-gray-50 dark:bg-gray-800">
          <h4 className="text-sm font-medium text-gray-500 mb-2 flex items-center gap-1">
            <Sparkles className="w-4 h-4" />
            Interesting Facts
          </h4>
          <div className="flex flex-wrap gap-2">
            {insight.facts.map((fact, i) => (
              <span key={i} className="px-3 py-1 bg-white dark:bg-gray-700 rounded-full text-sm shadow-sm">
                {fact}
              </span>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

// ============================================================================
// MAIN COMPONENT
// ============================================================================

const RulesExplorer: React.FC = () => {
  // State
  const [zoomLevel, setZoomLevel] = useState(0);
  const [selectedGroup, setSelectedGroup] = useState<SemanticGroup | null>(null);
  const [selectedRule, setSelectedRule] = useState<Rule | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  // Panel visibility
  const [showMetrics, setShowMetrics] = useState(true);
  const [showInsights, setShowInsights] = useState(true);
  const [showFilters, setShowFilters] = useState(false);

  // Mock data
  const groups = useMemo(() => generateMockGroups(), []);
  const rules = useMemo(() => generateMockRules(), []);

  // Mock insight for selected rule
  const selectedRuleInsight: RuleInsight | null = selectedRule
    ? {
        ruleId: selectedRule.id,
        patterns: [{ type: 'conditional', subtype: 'tiered' }],
        tags: ['risk-assessment', 'pricing', 'age-based'],
        similarRules: [
          { ruleId: 'risk_score', similarityScore: 0.85, matchingPatterns: [], reason: 'Both use tiered age logic' },
          { ruleId: 'discount_factor', similarityScore: 0.72, matchingPatterns: [], reason: 'Similar threshold patterns' },
        ],
        complexity: {
          totalNodes: 12,
          maxDepth: 4,
          variableCount: 3,
          operatorCount: 5,
          conditionCount: 3,
          loopCount: 0,
          cyclomaticComplexity: 4,
        },
        summary: `Computes '${selectedRule.outputAttribute}' using ${selectedRule.dependencies.join(', ')} via tiered threshold logic`,
        suggestions: [
          {
            priority: 'low',
            category: 'simplification',
            title: 'Consider lookup table',
            description: 'The tiered conditions could be expressed as a lookup table for clarity.',
            estimatedImpact: 'Easier configuration updates',
          },
        ],
        facts: [
          'Uses 3 age thresholds',
          'Bytecode compiled for 3.5x speedup',
          'Input rule in DAG level 1',
        ],
      }
    : null;

  // Global metrics
  const globalMetrics = {
    totalRules: 47,
    avgComplexity: 4.8,
    bytecodeRate: 0.88,
    llmRules: 6,
    avgLatency: '23ms',
    totalEvaluations: '12.4M',
  };

  // Mock insights
  const insights = [
    'Most rules use tiered conditional logic (68%)',
    '6 rules could be consolidated into 2',
    'Pricing rules account for 45% of execution time',
    'LLM rules have 50x higher latency but handle edge cases',
  ];

  const suggestions: Suggestion[] = [
    {
      priority: 'medium',
      category: 'performance',
      title: 'Consolidate similar rules',
      description: 'risk_factor and risk_score share 85% logic',
      estimatedImpact: '15% latency reduction',
    },
  ];

  // Navigation handlers
  const handleSelectGroup = useCallback((group: SemanticGroup) => {
    setSelectedGroup(group);
    setZoomLevel(1);
  }, []);

  const handleSelectRule = useCallback((rule: Rule) => {
    setSelectedRule(rule);
    setZoomLevel(2);
  }, []);

  const handleBack = useCallback(() => {
    if (zoomLevel === 3) {
      setZoomLevel(2);
    } else if (zoomLevel === 2) {
      setSelectedRule(null);
      setZoomLevel(1);
    } else if (zoomLevel === 1) {
      setSelectedGroup(null);
      setZoomLevel(0);
    }
  }, [zoomLevel]);

  const handleViewTrace = useCallback(() => {
    setZoomLevel(3);
  }, []);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' || e.key === 'Backspace') {
        if (document.activeElement?.tagName !== 'INPUT') {
          e.preventDefault();
          handleBack();
        }
      }
      if (e.key === '1') setZoomLevel(0);
      if (e.key === '2' && selectedGroup) setZoomLevel(1);
      if (e.key === '3' && selectedRule) setZoomLevel(2);
      if (e.key === 'm') setShowMetrics((v) => !v);
      if (e.key === 'i') setShowInsights((v) => !v);
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleBack, selectedGroup, selectedRule]);

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900">
      {/* Top Bar */}
      <div className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 p-4">
        <div className="flex items-center justify-between">
          {/* Left: Title & Breadcrumb */}
          <div className="flex items-center gap-4">
            <h1 className="text-xl font-bold flex items-center gap-2">
              <Network className="w-6 h-6 text-blue-500" />
              Rules Explorer
            </h1>

            {/* Breadcrumb */}
            <div className="flex items-center gap-1 text-sm text-gray-500">
              <button
                onClick={() => setZoomLevel(0)}
                className={`hover:text-blue-600 ${zoomLevel === 0 ? 'text-blue-600 font-medium' : ''}`}
              >
                Universe
              </button>
              {selectedGroup && (
                <>
                  <ChevronRight className="w-4 h-4" />
                  <button
                    onClick={() => setZoomLevel(1)}
                    className={`hover:text-blue-600 ${zoomLevel === 1 ? 'text-blue-600 font-medium' : ''}`}
                  >
                    {selectedGroup.name}
                  </button>
                </>
              )}
              {selectedRule && (
                <>
                  <ChevronRight className="w-4 h-4" />
                  <button
                    onClick={() => setZoomLevel(2)}
                    className={`hover:text-blue-600 ${zoomLevel === 2 ? 'text-blue-600 font-medium' : ''}`}
                  >
                    {selectedRule.name}
                  </button>
                </>
              )}
              {zoomLevel === 3 && (
                <>
                  <ChevronRight className="w-4 h-4" />
                  <span className="text-blue-600 font-medium">Trace</span>
                </>
              )}
            </div>
          </div>

          {/* Center: Search */}
          <div className="flex-1 max-w-md mx-4">
            <div className="relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search rules, attributes, patterns..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full pl-10 pr-4 py-2 border border-gray-200 dark:border-gray-700 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>
          </div>

          {/* Right: Controls */}
          <div className="flex items-center gap-2">
            {/* Zoom Level Indicator */}
            <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
              {ZOOM_LEVELS.map((level) => (
                <button
                  key={level.level}
                  onClick={() => {
                    if (level.level <= zoomLevel) {
                      setZoomLevel(level.level);
                      if (level.level < 1) setSelectedGroup(null);
                      if (level.level < 2) setSelectedRule(null);
                    }
                  }}
                  disabled={
                    (level.level === 1 && !selectedGroup) ||
                    (level.level === 2 && !selectedRule) ||
                    (level.level === 3 && !selectedRule)
                  }
                  className={`px-3 py-1 text-sm rounded transition-colors ${
                    level.level === zoomLevel
                      ? 'bg-blue-500 text-white'
                      : level.level > zoomLevel
                      ? 'text-gray-400 cursor-not-allowed'
                      : 'hover:bg-gray-200 dark:hover:bg-gray-600'
                  }`}
                  title={level.description}
                >
                  {level.level}
                </button>
              ))}
            </div>

            <div className="w-px h-6 bg-gray-200 dark:bg-gray-700" />

            {/* Panel Toggles */}
            <button
              onClick={() => setShowMetrics((v) => !v)}
              className={`p-2 rounded-lg ${showMetrics ? 'bg-blue-100 text-blue-600' : 'hover:bg-gray-100'}`}
              title="Toggle Metrics (M)"
            >
              <BarChart3 className="w-5 h-5" />
            </button>
            <button
              onClick={() => setShowInsights((v) => !v)}
              className={`p-2 rounded-lg ${showInsights ? 'bg-yellow-100 text-yellow-600' : 'hover:bg-gray-100'}`}
              title="Toggle Insights (I)"
            >
              <Lightbulb className="w-5 h-5" />
            </button>
            <button
              onClick={() => setShowFilters((v) => !v)}
              className={`p-2 rounded-lg ${showFilters ? 'bg-purple-100 text-purple-600' : 'hover:bg-gray-100'}`}
              title="Toggle Filters"
            >
              <Filter className="w-5 h-5" />
            </button>
          </div>
        </div>
      </div>

      {/* Metrics Panel */}
      <MetricsPanel
        visible={showMetrics}
        onToggle={() => setShowMetrics(false)}
        globalMetrics={globalMetrics}
      />

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Main View */}
        <div className="flex-1 overflow-hidden">
          {zoomLevel === 0 && (
            <UniverseView groups={groups} onSelectGroup={handleSelectGroup} />
          )}

          {zoomLevel === 1 && selectedGroup && (
            <GroupView
              group={selectedGroup}
              rules={rules}
              onSelectRule={handleSelectRule}
              onBack={handleBack}
            />
          )}

          {zoomLevel === 2 && selectedRule && selectedRuleInsight && (
            <RuleDetailView
              rule={selectedRule}
              insight={selectedRuleInsight}
              onBack={handleBack}
              onViewTrace={handleViewTrace}
            />
          )}

          {zoomLevel === 3 && selectedRule && (
            <div className="h-full flex flex-col">
              {/* Trace View Header */}
              <div className="p-4 border-b bg-white dark:bg-gray-800">
                <div className="flex items-center gap-3">
                  <button onClick={handleBack} className="p-2 hover:bg-gray-100 rounded-lg">
                    <ChevronLeft className="w-5 h-5" />
                  </button>
                  <h2 className="text-xl font-bold">Bytecode Trace: {selectedRule.name}</h2>
                </div>
              </div>

              {/* Trace Content */}
              <div className="flex-1 overflow-auto p-4">
                <div className="bg-gray-900 text-green-400 font-mono text-sm p-4 rounded-lg">
                  <div className="text-gray-500 mb-4">// Bytecode execution trace for {selectedRule.id}</div>
                  <table className="w-full">
                    <thead>
                      <tr className="text-gray-500 border-b border-gray-700">
                        <th className="text-left p-2">IP</th>
                        <th className="text-left p-2">OpCode</th>
                        <th className="text-left p-2">Operands</th>
                        <th className="text-left p-2">Stack</th>
                        <th className="text-left p-2">Time</th>
                      </tr>
                    </thead>
                    <tbody>
                      {[
                        { ip: 0, op: 'LoadVar', args: 'age', stack: '[30]', time: '12ns' },
                        { ip: 1, op: 'LoadConst', args: '25', stack: '[30, 25]', time: '8ns' },
                        { ip: 2, op: 'Lt', args: '', stack: '[false]', time: '15ns' },
                        { ip: 3, op: 'JumpIfNot', args: '@7', stack: '[]', time: '5ns' },
                        { ip: 7, op: 'LoadVar', args: 'age', stack: '[30]', time: '12ns', current: true },
                        { ip: 8, op: 'LoadConst', args: '35', stack: '[30, 35]', time: '8ns' },
                        { ip: 9, op: 'Lt', args: '', stack: '[true]', time: '15ns' },
                        { ip: 10, op: 'LoadConst', args: '1.2', stack: '[1.2]', time: '8ns' },
                        { ip: 11, op: 'Return', args: '', stack: '→ 1.2', time: '3ns' },
                      ].map((row: any, i) => (
                        <tr
                          key={i}
                          className={`border-b border-gray-800 ${row.current ? 'bg-blue-900/30' : ''}`}
                        >
                          <td className="p-2 text-cyan-400">{row.ip}</td>
                          <td className="p-2 text-yellow-400">{row.op}</td>
                          <td className="p-2">{row.args}</td>
                          <td className="p-2 text-purple-400">{row.stack}</td>
                          <td className="p-2 text-gray-500">{row.time}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>

                <div className="mt-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg">
                  <h3 className="font-medium mb-2">Execution Summary</h3>
                  <div className="grid grid-cols-3 gap-4 text-sm">
                    <div>
                      <span className="text-gray-500">Instructions:</span> 9 executed, 3 skipped
                    </div>
                    <div>
                      <span className="text-gray-500">Total Time:</span> 86ns
                    </div>
                    <div>
                      <span className="text-gray-500">Result:</span>{' '}
                      <span className="text-green-600 font-mono">1.2</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Insights Panel */}
        <InsightsPanel
          visible={showInsights}
          onToggle={() => setShowInsights(false)}
          insights={insights}
          suggestions={suggestions}
        />
      </div>

      {/* Status Bar */}
      <div className="bg-gray-100 dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-4 py-2 flex items-center justify-between text-xs text-gray-500">
        <div className="flex items-center gap-4">
          <span>Zoom Level: {ZOOM_LEVELS[zoomLevel].name}</span>
          <span>•</span>
          <span>{groups.length} groups</span>
          <span>•</span>
          <span>{rules.length} rules loaded</span>
        </div>
        <div className="flex items-center gap-4">
          <span>Press <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">?</kbd> for shortcuts</span>
          <span>•</span>
          <span>
            <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded">M</kbd> Metrics
            <kbd className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded ml-2">I</kbd> Insights
          </span>
        </div>
      </div>
    </div>
  );
};

export default RulesExplorer;

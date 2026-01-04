/**
 * Smart Data Loading Utilities
 *
 * Efficient data loading strategies for large rule sets:
 * - Progressive loading with virtualization
 * - Semantic grouping on-the-fly
 * - Intelligent caching
 * - Background pattern analysis
 */

import { useState, useEffect, useCallback, useMemo, useRef } from 'react';

// ============================================================================
// TYPES
// ============================================================================

export interface LoadingState {
  isLoading: boolean;
  isLoadingMore: boolean;
  error: string | null;
  progress: number;
  totalItems: number;
  loadedItems: number;
}

export interface DataChunk<T> {
  items: T[];
  offset: number;
  total: number;
  hasMore: boolean;
}

export interface SemanticGroup {
  id: string;
  name: string;
  description: string;
  keywords: string[];
  confidence: number;
  itemIds: string[];
  metrics: GroupMetrics;
  color: string;
}

export interface GroupMetrics {
  itemCount: number;
  avgComplexity: number;
  maxDepth: number;
  parallelismScore: number;
  bytecodeRate: number;
  llmCount: number;
}

export interface PatternMatch {
  type: string;
  subtype: string;
  confidence: number;
}

export interface AnalyzedItem {
  id: string;
  patterns: PatternMatch[];
  tags: string[];
  complexity: number;
  groupIds: string[];
}

// ============================================================================
// KEYWORD PATTERNS FOR SEMANTIC GROUPING
// ============================================================================

const KEYWORD_PATTERNS = {
  pricing: {
    name: 'Pricing & Calculations',
    description: 'Rules that compute prices, premiums, rates, and financial amounts',
    keywords: ['price', 'premium', 'rate', 'cost', 'fee', 'charge', 'amount', 'total', 'tax', 'commission'],
    color: '#22C55E',
  },
  risk: {
    name: 'Risk Assessment',
    description: 'Rules that evaluate risk scores, factors, and ratings',
    keywords: ['risk', 'score', 'factor', 'rating', 'assessment', 'level', 'hazard', 'exposure'],
    color: '#EAB308',
  },
  discount: {
    name: 'Discounts & Promotions',
    description: 'Rules that apply discounts, rebates, and promotional offers',
    keywords: ['discount', 'rebate', 'reduction', 'offer', 'promotion', 'loyalty', 'bonus', 'credit'],
    color: '#8B5CF6',
  },
  eligibility: {
    name: 'Eligibility & Validation',
    description: 'Rules that determine eligibility and validate business constraints',
    keywords: ['eligible', 'valid', 'qualify', 'entitled', 'permitted', 'approved', 'check', 'verify'],
    color: '#3B82F6',
  },
  coverage: {
    name: 'Coverage & Benefits',
    description: 'Rules for insurance coverage, benefits, and policy details',
    keywords: ['cover', 'coverage', 'benefit', 'policy', 'claim', 'limit', 'deductible', 'exclusion'],
    color: '#EC4899',
  },
  customer: {
    name: 'Customer Data',
    description: 'Rules related to customer information and profiles',
    keywords: ['customer', 'user', 'client', 'member', 'profile', 'account', 'person', 'applicant'],
    color: '#06B6D4',
  },
  calculation: {
    name: 'General Calculations',
    description: 'Rules with arithmetic and computational logic',
    keywords: ['calculate', 'compute', 'derive', 'formula', 'sum', 'average', 'multiply', 'divide'],
    color: '#F97316',
  },
};

// ============================================================================
// PATTERN DETECTION
// ============================================================================

const detectPatterns = (expression: any): PatternMatch[] => {
  const patterns: PatternMatch[] = [];

  const detectRecursive = (expr: any) => {
    if (!expr || typeof expr !== 'object') return;

    if (Array.isArray(expr)) {
      expr.forEach(detectRecursive);
      return;
    }

    Object.entries(expr).forEach(([op, args]) => {
      switch (op) {
        case '*':
          // Check for percentage
          if (Array.isArray(args) && args.some((a: any) => typeof a === 'number' && a > 0 && a < 1)) {
            patterns.push({ type: 'calculation', subtype: 'percentage', confidence: 0.9 });
          } else {
            patterns.push({ type: 'calculation', subtype: 'multiplication', confidence: 0.95 });
          }
          break;
        case '/':
          patterns.push({ type: 'calculation', subtype: 'division', confidence: 0.95 });
          break;
        case '+':
          patterns.push({ type: 'calculation', subtype: 'addition', confidence: 0.95 });
          break;
        case '-':
          patterns.push({ type: 'calculation', subtype: 'subtraction', confidence: 0.95 });
          break;
        case 'if':
          if (Array.isArray(args) && args.length > 3) {
            patterns.push({ type: 'conditional', subtype: 'tiered', confidence: 0.9 });
          } else {
            patterns.push({ type: 'conditional', subtype: 'simple', confidence: 0.85 });
          }
          break;
        case '<':
        case '<=':
        case '>':
        case '>=':
          patterns.push({ type: 'conditional', subtype: 'threshold', confidence: 0.9 });
          break;
        case '==':
        case '===':
        case '!=':
        case '!==':
          patterns.push({ type: 'conditional', subtype: 'equality', confidence: 0.9 });
          break;
        case 'and':
        case 'or':
          patterns.push({ type: 'conditional', subtype: 'logical', confidence: 0.85 });
          break;
        case 'var':
          if (typeof args === 'string' && args.includes('.')) {
            patterns.push({ type: 'lookup', subtype: 'nested', confidence: 0.9 });
          } else {
            patterns.push({ type: 'lookup', subtype: 'variable', confidence: 0.95 });
          }
          break;
        case 'min':
        case 'max':
          patterns.push({ type: 'aggregation', subtype: op, confidence: 0.95 });
          break;
        case 'map':
        case 'filter':
        case 'reduce':
          patterns.push({ type: 'transformation', subtype: op, confidence: 0.9 });
          break;
        case 'all':
        case 'some':
        case 'none':
          patterns.push({ type: 'aggregation', subtype: op, confidence: 0.9 });
          break;
        case 'cat':
          patterns.push({ type: 'transformation', subtype: 'concatenation', confidence: 0.9 });
          break;
        case 'in':
          patterns.push({ type: 'validation', subtype: 'membership', confidence: 0.85 });
          break;
      }

      detectRecursive(args);
    });
  };

  detectRecursive(expression);

  // Deduplicate patterns
  const seen = new Set<string>();
  return patterns.filter((p) => {
    const key = `${p.type}:${p.subtype}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
};

// ============================================================================
// SEMANTIC GROUPING
// ============================================================================

const assignToGroups = (item: { id: string; name?: string; outputAttribute?: string }): string[] => {
  const text = `${item.id} ${item.name || ''} ${item.outputAttribute || ''}`.toLowerCase();
  const groups: string[] = [];

  Object.entries(KEYWORD_PATTERNS).forEach(([groupId, config]) => {
    if (config.keywords.some((kw) => text.includes(kw))) {
      groups.push(groupId);
    }
  });

  return groups.length > 0 ? groups : ['uncategorized'];
};

const extractTags = (item: { id: string; name?: string }): string[] => {
  const text = `${item.id} ${item.name || ''}`;
  const tags: string[] = [];

  // Extract from snake_case and camelCase
  const parts = text
    .split(/[_\-\s]/)
    .flatMap((p) => p.split(/(?=[A-Z])/))
    .filter((p) => p.length > 2)
    .map((p) => p.toLowerCase());

  // Take unique meaningful parts
  const seen = new Set<string>();
  parts.forEach((p) => {
    if (!seen.has(p) && p.length > 3 && tags.length < 5) {
      seen.add(p);
      tags.push(p);
    }
  });

  return tags;
};

// ============================================================================
// COMPLEXITY ANALYSIS
// ============================================================================

const analyzeComplexity = (expression: any): number => {
  let complexity = 1; // Base complexity

  const countRecursive = (expr: any, depth: number) => {
    if (!expr || typeof expr !== 'object') return;

    if (Array.isArray(expr)) {
      expr.forEach((e) => countRecursive(e, depth));
      return;
    }

    Object.entries(expr).forEach(([op, args]) => {
      complexity += 1;

      // Conditional operators add complexity
      if (['if', 'and', 'or'].includes(op)) {
        complexity += 1;
      }

      // Loops add significant complexity
      if (['map', 'filter', 'reduce', 'all', 'some', 'none'].includes(op)) {
        complexity += 2;
      }

      countRecursive(args, depth + 1);
    });
  };

  countRecursive(expression, 0);
  return complexity;
};

// ============================================================================
// HOOKS
// ============================================================================

/**
 * Hook for progressive data loading with smart chunking
 */
export function useProgressiveLoader<T extends { id: string }>(
  fetchFn: (offset: number, limit: number) => Promise<DataChunk<T>>,
  options: {
    initialPageSize?: number;
    loadMoreThreshold?: number;
    autoLoadMore?: boolean;
  } = {}
) {
  const { initialPageSize = 50, loadMoreThreshold = 0.8, autoLoadMore = true } = options;

  const [items, setItems] = useState<T[]>([]);
  const [state, setState] = useState<LoadingState>({
    isLoading: true,
    isLoadingMore: false,
    error: null,
    progress: 0,
    totalItems: 0,
    loadedItems: 0,
  });

  const offsetRef = useRef(0);
  const hasMoreRef = useRef(true);

  const loadInitial = useCallback(async () => {
    try {
      setState((s) => ({ ...s, isLoading: true, error: null }));
      const chunk = await fetchFn(0, initialPageSize);

      setItems(chunk.items);
      offsetRef.current = chunk.items.length;
      hasMoreRef.current = chunk.hasMore;

      setState((s) => ({
        ...s,
        isLoading: false,
        totalItems: chunk.total,
        loadedItems: chunk.items.length,
        progress: chunk.total > 0 ? (chunk.items.length / chunk.total) * 100 : 100,
      }));
    } catch (err) {
      setState((s) => ({
        ...s,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to load data',
      }));
    }
  }, [fetchFn, initialPageSize]);

  const loadMore = useCallback(async () => {
    if (!hasMoreRef.current || state.isLoadingMore) return;

    try {
      setState((s) => ({ ...s, isLoadingMore: true }));
      const chunk = await fetchFn(offsetRef.current, initialPageSize);

      setItems((prev) => [...prev, ...chunk.items]);
      offsetRef.current += chunk.items.length;
      hasMoreRef.current = chunk.hasMore;

      setState((s) => ({
        ...s,
        isLoadingMore: false,
        loadedItems: offsetRef.current,
        progress: chunk.total > 0 ? (offsetRef.current / chunk.total) * 100 : 100,
      }));
    } catch (err) {
      setState((s) => ({
        ...s,
        isLoadingMore: false,
        error: err instanceof Error ? err.message : 'Failed to load more',
      }));
    }
  }, [fetchFn, initialPageSize, state.isLoadingMore]);

  const reset = useCallback(() => {
    setItems([]);
    offsetRef.current = 0;
    hasMoreRef.current = true;
    loadInitial();
  }, [loadInitial]);

  useEffect(() => {
    loadInitial();
  }, [loadInitial]);

  return {
    items,
    ...state,
    hasMore: hasMoreRef.current,
    loadMore,
    reset,
  };
}

/**
 * Hook for semantic grouping of items
 */
export function useSemanticGroups<T extends { id: string; name?: string; outputAttribute?: string; expression?: any }>(
  items: T[]
) {
  return useMemo(() => {
    const groupMap = new Map<string, SemanticGroup>();

    // Initialize groups
    Object.entries(KEYWORD_PATTERNS).forEach(([id, config]) => {
      groupMap.set(id, {
        id,
        name: config.name,
        description: config.description,
        keywords: config.keywords,
        confidence: 0,
        itemIds: [],
        metrics: {
          itemCount: 0,
          avgComplexity: 0,
          maxDepth: 0,
          parallelismScore: 0,
          bytecodeRate: 0,
          llmCount: 0,
        },
        color: config.color,
      });
    });

    // Add uncategorized group
    groupMap.set('uncategorized', {
      id: 'uncategorized',
      name: 'Uncategorized',
      description: 'Rules that don\'t match any known pattern',
      keywords: [],
      confidence: 0,
      itemIds: [],
      metrics: {
        itemCount: 0,
        avgComplexity: 0,
        maxDepth: 0,
        parallelismScore: 0,
        bytecodeRate: 0,
        llmCount: 0,
      },
      color: '#6B7280',
    });

    // Assign items to groups
    let totalComplexity = 0;
    items.forEach((item) => {
      const groupIds = assignToGroups(item);
      const complexity = item.expression ? analyzeComplexity(item.expression) : 1;
      totalComplexity += complexity;

      groupIds.forEach((gid) => {
        const group = groupMap.get(gid);
        if (group) {
          group.itemIds.push(item.id);
          group.metrics.itemCount++;
          group.metrics.avgComplexity += complexity;
        }
      });
    });

    // Calculate final metrics
    groupMap.forEach((group) => {
      if (group.metrics.itemCount > 0) {
        group.metrics.avgComplexity /= group.metrics.itemCount;
        group.confidence = Math.min(group.metrics.itemCount / items.length, 1);
        group.metrics.bytecodeRate = 0.85 + Math.random() * 0.15; // Mock
        group.metrics.parallelismScore = 0.5 + Math.random() * 0.5; // Mock
      }
    });

    // Filter out empty groups and sort by item count
    return Array.from(groupMap.values())
      .filter((g) => g.itemIds.length > 0)
      .sort((a, b) => b.itemIds.length - a.itemIds.length);
  }, [items]);
}

/**
 * Hook for analyzing items and extracting patterns
 */
export function usePatternAnalysis<T extends { id: string; name?: string; expression?: any }>(items: T[]) {
  return useMemo(() => {
    const analyzed: Map<string, AnalyzedItem> = new Map();

    items.forEach((item) => {
      const patterns = item.expression ? detectPatterns(item.expression) : [];
      const tags = extractTags(item);
      const complexity = item.expression ? analyzeComplexity(item.expression) : 1;
      const groupIds = assignToGroups(item as any);

      analyzed.set(item.id, {
        id: item.id,
        patterns,
        tags,
        complexity,
        groupIds,
      });
    });

    return analyzed;
  }, [items]);
}

/**
 * Hook for generating insights from analyzed data
 */
export function useInsights<T extends { id: string; expression?: any }>(
  items: T[],
  groups: SemanticGroup[],
  analysis: Map<string, AnalyzedItem>
) {
  return useMemo(() => {
    const insights: string[] = [];
    const suggestions: Array<{
      priority: 'high' | 'medium' | 'low' | 'info';
      category: string;
      title: string;
      description: string;
      estimatedImpact: string;
    }> = [];

    // Pattern distribution
    const patternCounts = new Map<string, number>();
    analysis.forEach((a) => {
      a.patterns.forEach((p) => {
        const key = `${p.type}:${p.subtype}`;
        patternCounts.set(key, (patternCounts.get(key) || 0) + 1);
      });
    });

    // Most common pattern
    let maxPattern = '';
    let maxCount = 0;
    patternCounts.forEach((count, pattern) => {
      if (count > maxCount) {
        maxCount = count;
        maxPattern = pattern;
      }
    });
    if (maxPattern) {
      insights.push(`Most common pattern: ${maxPattern.replace(':', ' - ')} (${maxCount} rules)`);
    }

    // Complexity distribution
    const complexities = Array.from(analysis.values()).map((a) => a.complexity);
    const avgComplexity = complexities.reduce((a, b) => a + b, 0) / complexities.length;
    const highComplexity = complexities.filter((c) => c > 10).length;

    if (highComplexity > 0) {
      insights.push(`${highComplexity} rules have high complexity (>10)`);
      suggestions.push({
        priority: 'medium',
        category: 'maintainability',
        title: 'Simplify complex rules',
        description: `${highComplexity} rules could be broken into smaller, more testable units`,
        estimatedImpact: 'Improved maintainability and debugging',
      });
    }

    // Group insights
    groups.forEach((g) => {
      if (g.itemIds.length >= 5) {
        insights.push(`"${g.name}" contains ${g.itemIds.length} related rules`);
      }
    });

    // Find similar rules
    const patternSignatures = new Map<string, string[]>();
    analysis.forEach((a, id) => {
      const sig = a.patterns.map((p) => `${p.type}:${p.subtype}`).sort().join('|');
      if (!patternSignatures.has(sig)) {
        patternSignatures.set(sig, []);
      }
      patternSignatures.get(sig)!.push(id);
    });

    patternSignatures.forEach((ids, sig) => {
      if (ids.length >= 3 && sig.length > 0) {
        suggestions.push({
          priority: 'low',
          category: 'consolidation',
          title: `${ids.length} rules share similar patterns`,
          description: `Rules ${ids.slice(0, 3).join(', ')}${ids.length > 3 ? '...' : ''} use identical patterns`,
          estimatedImpact: 'Potential for consolidation or abstraction',
        });
      }
    });

    // General stats
    insights.push(`Average rule complexity: ${avgComplexity.toFixed(1)}`);
    insights.push(`${items.length} total rules across ${groups.length} semantic groups`);

    return { insights, suggestions };
  }, [items, groups, analysis]);
}

/**
 * Hook for virtualized list rendering
 */
export function useVirtualizedList<T>(
  items: T[],
  options: {
    containerHeight: number;
    itemHeight: number;
    overscan?: number;
  }
) {
  const { containerHeight, itemHeight, overscan = 5 } = options;
  const [scrollTop, setScrollTop] = useState(0);

  const visibleCount = Math.ceil(containerHeight / itemHeight);
  const startIndex = Math.max(0, Math.floor(scrollTop / itemHeight) - overscan);
  const endIndex = Math.min(items.length, startIndex + visibleCount + overscan * 2);

  const visibleItems = useMemo(
    () =>
      items.slice(startIndex, endIndex).map((item, i) => ({
        item,
        index: startIndex + i,
        style: {
          position: 'absolute' as const,
          top: (startIndex + i) * itemHeight,
          height: itemHeight,
          width: '100%',
        },
      })),
    [items, startIndex, endIndex, itemHeight]
  );

  const totalHeight = items.length * itemHeight;

  const handleScroll = useCallback((e: React.UIEvent<HTMLElement>) => {
    setScrollTop(e.currentTarget.scrollTop);
  }, []);

  return {
    visibleItems,
    totalHeight,
    handleScroll,
    startIndex,
    endIndex,
  };
}

export default {
  useProgressiveLoader,
  useSemanticGroups,
  usePatternAnalysis,
  useInsights,
  useVirtualizedList,
};

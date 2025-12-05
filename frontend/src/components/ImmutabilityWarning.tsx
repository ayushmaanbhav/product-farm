// Immutability Warning Component
// Shows warnings when modifications affect immutable attributes

import { useState, useEffect } from 'react';
import { useProductStore, useUIStore } from '@/store';
import { api } from '@/services/api';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
  AlertTriangle,
  Lock,
  Copy,
  X,
  ChevronDown,
  ChevronUp,
  GitBranch,
  ArrowRight,
} from 'lucide-react';
import type { ImpactAnalysis } from '@/types';

// =============================================================================
// INLINE WARNING BADGE
// =============================================================================

interface ImmutabilityBadgeProps {
  isImmutable: boolean;
  size?: 'sm' | 'md';
}

export function ImmutabilityBadge({ isImmutable, size = 'sm' }: ImmutabilityBadgeProps) {
  if (!isImmutable) return null;

  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-5 w-5',
  };

  return (
    <span className="inline-flex items-center gap-1 text-amber-600" title="Immutable - cannot be modified directly">
      <Lock className={sizeClasses[size]} />
    </span>
  );
}

// =============================================================================
// IMPACT ANALYSIS VIEWER
// =============================================================================

interface ImpactAnalysisViewerProps {
  analysis: ImpactAnalysis;
  onClose: () => void;
  onClone?: () => void;
}

export function ImpactAnalysisViewer({ analysis, onClose, onClone }: ImpactAnalysisViewerProps) {
  const [showTransitive, setShowTransitive] = useState(false);

  const targetName = analysis.targetPath.split(':').pop() || analysis.targetPath;

  return (
    <div className="border rounded-lg bg-white shadow-lg overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b bg-gray-50 flex items-center justify-between">
        <h3 className="font-medium text-sm flex items-center gap-2">
          <GitBranch className="h-4 w-4 text-indigo-500" />
          Impact Analysis: {targetName}
        </h3>
        <button onClick={onClose} className="p-1 hover:bg-gray-200 rounded">
          <X className="h-4 w-4 text-gray-500" />
        </button>
      </div>

      <div className="p-4 space-y-4">
        {/* Warning if has immutable dependents */}
        {analysis.hasImmutableDependents && (
          <div className="p-3 bg-amber-50 border border-amber-200 rounded-lg flex items-start gap-3">
            <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0" />
            <div>
              <p className="text-sm font-medium text-amber-800">
                Modification would affect immutable attributes
              </p>
              <p className="text-xs text-amber-700 mt-1">
                {analysis.immutablePaths.length} immutable attribute(s) depend on this attribute.
                Clone the product to make changes.
              </p>
              {onClone && (
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-2 h-7 text-xs border-amber-300 text-amber-700 hover:bg-amber-100"
                  onClick={onClone}
                >
                  <Copy className="h-3 w-3 mr-1" />
                  Clone Product
                </Button>
              )}
            </div>
          </div>
        )}

        {/* Direct Dependencies */}
        <div>
          <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
            Direct Dependencies ({analysis.directDependencies.length})
          </h4>
          {analysis.directDependencies.length > 0 ? (
            <div className="space-y-1">
              {analysis.directDependencies.map((dep, idx) => (
                <div
                  key={idx}
                  className={cn(
                    'flex items-center justify-between px-3 py-2 rounded text-sm',
                    dep.isImmutable ? 'bg-amber-50 border border-amber-200' : 'bg-gray-50'
                  )}
                >
                  <div className="flex items-center gap-2">
                    {dep.direction === 'upstream' ? (
                      <ArrowRight className="h-3 w-3 text-gray-400 rotate-180" />
                    ) : (
                      <ArrowRight className="h-3 w-3 text-gray-400" />
                    )}
                    <span className="font-medium">{dep.attributeName}</span>
                    {dep.isImmutable && <Lock className="h-3 w-3 text-amber-500" />}
                  </div>
                  <span className="text-xs text-gray-500">{dep.direction}</span>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-sm text-gray-400 italic">No direct dependencies</p>
          )}
        </div>

        {/* Transitive Dependencies */}
        {analysis.transitiveDependencies.length > 0 && (
          <div>
            <button
              onClick={() => setShowTransitive(!showTransitive)}
              className="flex items-center gap-1 text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 hover:text-gray-700"
            >
              {showTransitive ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
              Transitive Dependencies ({analysis.transitiveDependencies.length})
            </button>
            {showTransitive && (
              <div className="space-y-1">
                {analysis.transitiveDependencies.map((dep, idx) => (
                  <div
                    key={idx}
                    className={cn(
                      'flex items-center justify-between px-3 py-2 rounded text-sm',
                      dep.isImmutable ? 'bg-amber-50 border border-amber-200' : 'bg-gray-50'
                    )}
                  >
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-400">({dep.distance} steps)</span>
                      <span className="font-medium">{dep.attributeName}</span>
                      {dep.isImmutable && <Lock className="h-3 w-3 text-amber-500" />}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Affected Rules */}
        {analysis.affectedRules.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
              Affected Rules ({analysis.affectedRules.length})
            </h4>
            <div className="text-xs text-gray-600">
              {analysis.affectedRules.join(', ').substring(0, 100)}
              {analysis.affectedRules.join(', ').length > 100 && '...'}
            </div>
          </div>
        )}

        {/* Affected Functionalities */}
        {analysis.affectedFunctionalities.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
              Affected Functionalities ({analysis.affectedFunctionalities.length})
            </h4>
            <div className="flex flex-wrap gap-1">
              {analysis.affectedFunctionalities.map((funcId, idx) => (
                <span key={idx} className="px-2 py-0.5 bg-indigo-100 text-indigo-700 rounded text-xs">
                  {funcId}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// MODIFICATION CHECK HOOK
// =============================================================================

interface ModificationCheckResult {
  canModify: boolean;
  reason?: string;
  requiresClone: boolean;
  affectedImmutablePaths: string[];
}

export function useModificationCheck(productId: string | undefined, targetPath: string | null) {
  const [result, setResult] = useState<ModificationCheckResult | null>(null);
  const [isChecking, setIsChecking] = useState(false);

  useEffect(() => {
    if (!productId || !targetPath) {
      setResult(null);
      return;
    }

    let cancelled = false;

    const check = async () => {
      setIsChecking(true);
      try {
        const checkResult = await api.impact.checkModification(productId, targetPath);
        if (!cancelled) {
          setResult(checkResult);
        }
      } catch (error) {
        console.error('Modification check failed:', error);
      } finally {
        if (!cancelled) {
          setIsChecking(false);
        }
      }
    };

    check();

    return () => {
      cancelled = true;
    };
  }, [productId, targetPath]);

  return { result, isChecking };
}

// =============================================================================
// IMPACT ANALYSIS HOOK
// =============================================================================

export function useImpactAnalysis(productId: string | undefined, targetPath: string | null) {
  const [analysis, setAnalysis] = useState<ImpactAnalysis | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    if (!productId || !targetPath) {
      setAnalysis(null);
      return;
    }

    let cancelled = false;

    const analyze = async () => {
      setIsLoading(true);
      try {
        const result = await api.impact.analyze(productId, targetPath);
        if (!cancelled) {
          setAnalysis(result);
        }
      } catch (error) {
        console.error('Impact analysis failed:', error);
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    analyze();

    return () => {
      cancelled = true;
    };
  }, [productId, targetPath]);

  return { analysis, isLoading };
}

// =============================================================================
// FLOATING IMPACT PANEL
// =============================================================================

export function FloatingImpactPanel() {
  const { selectedProduct } = useProductStore();
  const { impactAnalysisTarget, setImpactAnalysisTarget, setCloneDialogOpen } = useUIStore();

  const { analysis } = useImpactAnalysis(
    selectedProduct?.id,
    impactAnalysisTarget
  );

  if (!impactAnalysisTarget || !analysis) return null;

  return (
    <div className="fixed bottom-4 right-4 z-40 w-96 max-h-[60vh] overflow-auto">
      <ImpactAnalysisViewer
        analysis={analysis}
        onClose={() => setImpactAnalysisTarget(null)}
        onClone={() => {
          setCloneDialogOpen(true);
          setImpactAnalysisTarget(null);
        }}
      />
    </div>
  );
}

export default FloatingImpactPanel;

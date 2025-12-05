// Rules Page - Visual DAG, Simulation, and Rule Management
// Central workspace for managing and testing product rules

import { useEffect, useState, useCallback, useMemo } from 'react';
import { useProductStore, useUIStore } from '@/store';
import { RuleCanvas } from '@/components/RuleCanvas';
import { SimulationPanel } from '@/components/SimulationPanel';
import { AttributeExplorer } from '@/components/AttributeExplorer';
import { RuleBuilder } from '@/components/RuleBuilder';
import { FunctionalityPanel } from '@/components/FunctionalityPanel';
import { CloneProductDialog } from '@/components/CloneProductDialog';
import { FloatingImpactPanel, ImmutabilityBadge } from '@/components/ImmutabilityWarning';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import type { Rule, ProductFunctionality } from '@/types';
import { cn } from '@/lib/utils';
import {
  Plus,
  GitBranch,
  Layers,
  FlaskConical,
  Grid3X3,
  LayoutGrid,
  Maximize2,
  ChevronDown,
  Edit3,
  Trash2,
  Check,
  Eye,
  EyeOff,
  Loader2,
  Boxes,
  Lock,
  X,
  Copy,
  Send,
  CheckCircle,
  Clock,
} from 'lucide-react';

// =============================================================================
// STATUS BADGE
// =============================================================================

function ProductStatusBadge({ status }: { status: string }) {
  const config: Record<string, { className: string; icon: typeof Check }> = {
    DRAFT: { className: 'bg-gray-100 text-gray-700', icon: Edit3 },
    PENDING_APPROVAL: { className: 'bg-amber-100 text-amber-700', icon: Clock },
    ACTIVE: { className: 'bg-green-100 text-green-700', icon: CheckCircle },
    DISCONTINUED: { className: 'bg-red-100 text-red-700', icon: X },
  };

  const { className, icon: Icon } = config[status] || config.DRAFT;

  return (
    <span className={cn('inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium', className)}>
      <Icon className="h-3 w-3" />
      {status.replace('_', ' ')}
    </span>
  );
}

// =============================================================================
// FUNCTIONALITY FILTER
// =============================================================================

interface FunctionalityFilterProps {
  functionalities: ProductFunctionality[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}

function FunctionalityFilter({ functionalities, selectedId, onSelect }: FunctionalityFilterProps) {
  const [isOpen, setIsOpen] = useState(false);

  const selected = functionalities.find(f => f.id === selectedId);

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          'flex items-center gap-2 rounded-lg border px-3 py-1.5 text-sm font-medium transition-colors',
          selectedId
            ? 'bg-indigo-50 border-indigo-200 text-indigo-700'
            : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
        )}
      >
        <Boxes className="h-4 w-4" />
        <span className="truncate max-w-[120px]">
          {selected?.displayName || 'All Functionalities'}
        </span>
        {selectedId ? (
          <button
            onClick={(e) => {
              e.stopPropagation();
              onSelect(null);
            }}
            className="p-0.5 hover:bg-indigo-200 rounded"
          >
            <X className="h-3 w-3" />
          </button>
        ) : (
          <ChevronDown className="h-4 w-4 text-gray-400" />
        )}
      </button>

      {isOpen && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setIsOpen(false)} />
          <div className="absolute top-full left-0 mt-1 z-20 w-64 rounded-lg border bg-white shadow-lg">
            <div className="p-2 space-y-1 max-h-[300px] overflow-auto">
              <button
                onClick={() => {
                  onSelect(null);
                  setIsOpen(false);
                }}
                className={cn(
                  'flex items-center justify-between w-full rounded-md px-3 py-2 text-sm transition-colors',
                  !selectedId
                    ? 'bg-primary text-primary-foreground'
                    : 'hover:bg-gray-100 text-gray-700'
                )}
              >
                <span>All Functionalities</span>
                {!selectedId && <Check className="h-4 w-4" />}
              </button>
              {functionalities.map((func) => (
                <button
                  key={func.id}
                  onClick={() => {
                    onSelect(func.id);
                    setIsOpen(false);
                  }}
                  className={cn(
                    'flex items-center justify-between w-full rounded-md px-3 py-2 text-sm transition-colors',
                    func.id === selectedId
                      ? 'bg-primary text-primary-foreground'
                      : 'hover:bg-gray-100 text-gray-700'
                  )}
                >
                  <div className="flex items-center gap-2">
                    <span>{func.displayName}</span>
                    {func.immutable && <Lock className="h-3 w-3 text-amber-500" />}
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="text-[10px] opacity-70">
                      {func.requiredAttributes.length} attrs
                    </span>
                    {func.id === selectedId && <Check className="h-4 w-4" />}
                  </div>
                </button>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// =============================================================================
// RULE LIST ITEM
// =============================================================================

interface RuleListItemProps {
  rule: Rule;
  isSelected: boolean;
  hasImmutableOutputs: boolean;
  onSelect: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onToggle: () => void;
}

function RuleListItem({
  rule,
  isSelected,
  hasImmutableOutputs,
  onSelect,
  onEdit,
  onDelete,
  onToggle,
}: RuleListItemProps) {
  return (
    <div
      className={cn(
        'group relative rounded-lg border p-3 transition-all cursor-pointer',
        isSelected
          ? 'border-primary bg-primary/5 ring-1 ring-primary'
          : 'border-gray-200 hover:border-gray-300 bg-white'
      )}
      onClick={onSelect}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-2 mb-2">
        <div className="flex items-center gap-2 min-w-0">
          <GitBranch
            className={cn('h-4 w-4 shrink-0', rule.enabled ? 'text-emerald-500' : 'text-gray-400')}
          />
          <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
            {rule.ruleType}
          </span>
          {hasImmutableOutputs && (
            <ImmutabilityBadge isImmutable size="sm" />
          )}
        </div>
        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={(e) => {
              e.stopPropagation();
              onToggle();
            }}
            className="p-1 rounded hover:bg-gray-100"
            title={rule.enabled ? 'Disable' : 'Enable'}
          >
            {rule.enabled ? (
              <Eye className="h-3.5 w-3.5 text-emerald-500" />
            ) : (
              <EyeOff className="h-3.5 w-3.5 text-gray-400" />
            )}
          </button>
          {!hasImmutableOutputs && (
            <>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onEdit();
                }}
                className="p-1 rounded hover:bg-gray-100"
                title="Edit"
              >
                <Edit3 className="h-3.5 w-3.5 text-gray-500" />
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete();
                }}
                className="p-1 rounded hover:bg-red-100"
                title="Delete"
              >
                <Trash2 className="h-3.5 w-3.5 text-red-500" />
              </button>
            </>
          )}
        </div>
      </div>

      {/* Expression */}
      <p className="text-sm text-gray-800 font-medium line-clamp-2 mb-2">
        {rule.displayExpression}
      </p>

      {/* Footer */}
      <div className="flex items-center justify-between text-[10px] text-gray-500">
        <div className="flex items-center gap-2">
          <span>{rule.inputAttributes.length} inputs</span>
          <span>•</span>
          <span>{rule.outputAttributes.length} outputs</span>
        </div>
        {rule.description && (
          <span className="truncate max-w-[100px]" title={rule.description}>
            {rule.description}
          </span>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// PRODUCT SELECTOR
// =============================================================================

function ProductSelector() {
  const { products, selectedProduct, selectProduct, fetchProducts, isLoading } = useProductStore();
  const { setCloneDialogOpen } = useUIStore();
  const [isOpen, setIsOpen] = useState(false);

  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  useEffect(() => {
    // Auto-select first product if none selected
    if (products.length > 0 && !selectedProduct) {
      selectProduct(products[0].id);
    }
  }, [products, selectedProduct, selectProduct]);

  return (
    <div className="flex items-center gap-2">
      <div className="relative">
        <button
          onClick={() => setIsOpen(!isOpen)}
          className="flex items-center gap-2 rounded-lg border bg-white px-3 py-2 text-sm font-medium text-gray-700 hover:bg-gray-50 transition-colors min-w-[200px]"
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : (
            <>
              <span className="flex-1 text-left truncate">
                {selectedProduct?.name || 'Select Product'}
              </span>
              <ChevronDown className="h-4 w-4 text-gray-400" />
            </>
          )}
        </button>

        {isOpen && (
          <>
            <div className="fixed inset-0 z-10" onClick={() => setIsOpen(false)} />
            <div className="absolute top-full left-0 mt-1 z-20 w-72 rounded-lg border bg-white shadow-lg">
              <div className="p-2 space-y-1 max-h-[300px] overflow-auto">
                {products.map((product) => (
                  <button
                    key={product.id}
                    onClick={() => {
                      selectProduct(product.id);
                      setIsOpen(false);
                    }}
                    className={cn(
                      'flex items-center justify-between w-full rounded-md px-3 py-2 text-sm transition-colors',
                      product.id === selectedProduct?.id
                        ? 'bg-primary text-primary-foreground'
                        : 'hover:bg-gray-100 text-gray-700'
                    )}
                  >
                    <div className="flex flex-col items-start">
                      <span className="font-medium">{product.name}</span>
                      <div className="flex items-center gap-2">
                        <span className="text-[10px] opacity-70">{product.templateType}</span>
                        <ProductStatusBadge status={product.status} />
                      </div>
                    </div>
                    {product.id === selectedProduct?.id && <Check className="h-4 w-4" />}
                  </button>
                ))}
              </div>
            </div>
          </>
        )}
      </div>

      {selectedProduct && (
        <div className="flex items-center gap-1">
          <ProductStatusBadge status={selectedProduct.status} />
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={() => setCloneDialogOpen(true)}
            title="Clone Product"
          >
            <Copy className="h-3.5 w-3.5" />
          </Button>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// MAIN RULES PAGE
// =============================================================================

export function Rules() {
  const {
    selectedProduct,
    rules,
    abstractAttributes,
    functionalities,
    executionPlan,
    updateRule,
    deleteRule,
    submitProduct,
    approveProduct,
  } = useProductStore();
  const {
    simulationPanelOpen,
    attributeExplorerOpen,
    functionalityPanelOpen,
    cloneDialogOpen,
    toggleSimulationPanel,
    toggleAttributeExplorer,
    toggleFunctionalityPanel,
    setCloneDialogOpen,
    viewMode,
    setViewMode,
    selectedFunctionalityId,
    setSelectedFunctionality,
  } = useUIStore();

  const [selectedRuleId, setSelectedRuleId] = useState<string | null>(null);
  const [showRuleBuilder, setShowRuleBuilder] = useState(false);
  const [editingRule, setEditingRule] = useState<Rule | undefined>();

  // Get immutable attribute paths
  const immutablePaths = useMemo(() => {
    return new Set(
      abstractAttributes.filter(a => a.immutable).map(a => a.abstractPath)
    );
  }, [abstractAttributes]);

  // Filter rules by functionality
  const filteredRules = useMemo(() => {
    if (!selectedFunctionalityId) return rules;

    const func = functionalities.find(f => f.id === selectedFunctionalityId);
    if (!func) return rules;

    const funcPaths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));

    // Show rules that have outputs in the functionality
    return rules.filter(r =>
      r.outputAttributes.some(oa => funcPaths.has(oa.attributePath))
    );
  }, [rules, functionalities, selectedFunctionalityId]);

  // Filter attributes by functionality
  const filteredAttributes = useMemo(() => {
    if (!selectedFunctionalityId) return abstractAttributes;

    const func = functionalities.find(f => f.id === selectedFunctionalityId);
    if (!func) return abstractAttributes;

    const funcPaths = new Set(func.requiredAttributes.map(ra => ra.abstractPath));
    return abstractAttributes.filter(a => funcPaths.has(a.abstractPath));
  }, [abstractAttributes, functionalities, selectedFunctionalityId]);

  // Check if rule has immutable outputs
  const hasImmutableOutputs = useCallback((rule: Rule) => {
    return rule.outputAttributes.some(oa => immutablePaths.has(oa.attributePath));
  }, [immutablePaths]);

  const handleRuleSelect = useCallback((ruleId: string) => {
    setSelectedRuleId(ruleId);
  }, []);

  const handleRuleEdit = useCallback(
    (ruleId: string) => {
      const rule = rules.find((r) => r.id === ruleId);
      if (rule && hasImmutableOutputs(rule)) {
        // Show clone dialog instead of editing
        setCloneDialogOpen(true);
        return;
      }
      setEditingRule(rule);
      setShowRuleBuilder(true);
    },
    [rules, hasImmutableOutputs, setCloneDialogOpen]
  );

  const handleRuleDelete = useCallback(
    async (ruleId: string) => {
      const rule = rules.find(r => r.id === ruleId);
      if (rule && hasImmutableOutputs(rule)) {
        // Show clone dialog instead
        setCloneDialogOpen(true);
        return;
      }

      if (window.confirm('Are you sure you want to delete this rule?')) {
        await deleteRule(ruleId);
        if (selectedRuleId === ruleId) {
          setSelectedRuleId(null);
        }
      }
    },
    [deleteRule, selectedRuleId, rules, hasImmutableOutputs, setCloneDialogOpen]
  );

  const handleRuleToggle = useCallback(
    async (rule: Rule) => {
      await updateRule(rule.id, { enabled: !rule.enabled });
    },
    [updateRule]
  );

  const handleCreateRule = useCallback(() => {
    setEditingRule(undefined);
    setShowRuleBuilder(true);
  }, []);

  const handleNodeClick = useCallback((nodeId: string) => {
    if (nodeId.startsWith('rule-')) {
      setSelectedRuleId(nodeId.replace('rule-', ''));
    }
  }, []);

  const handleSubmitProduct = useCallback(async () => {
    if (selectedProduct) {
      await submitProduct(selectedProduct.id);
    }
  }, [selectedProduct, submitProduct]);

  const handleApproveProduct = useCallback(async () => {
    if (selectedProduct) {
      await approveProduct(selectedProduct.id);
    }
  }, [selectedProduct, approveProduct]);

  // Rule builder mode
  if (showRuleBuilder) {
    return (
      <div className="h-full">
        <RuleBuilder
          rule={editingRule}
          onSave={async () => {
            setShowRuleBuilder(false);
            setEditingRule(undefined);
          }}
          onCancel={() => {
            setShowRuleBuilder(false);
            setEditingRule(undefined);
          }}
        />
      </div>
    );
  }

  if (!selectedProduct) {
    return (
      <div className="h-full flex items-center justify-center">
        <Card className="max-w-md">
          <CardContent className="pt-6">
            <div className="text-center">
              <GitBranch className="h-12 w-12 mx-auto text-gray-300 mb-4" />
              <h3 className="text-lg font-semibold text-gray-800 mb-2">No Product Selected</h3>
              <p className="text-sm text-gray-500 mb-4">
                Select a product to view and manage its business rules.
              </p>
              <ProductSelector />
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Toolbar */}
      <div className="flex-shrink-0 flex items-center justify-between border-b bg-white px-4 py-2">
        <div className="flex items-center gap-3">
          <ProductSelector />
          <div className="h-6 border-l" />
          <FunctionalityFilter
            functionalities={functionalities}
            selectedId={selectedFunctionalityId}
            onSelect={setSelectedFunctionality}
          />
          <div className="h-6 border-l" />
          <span className="text-sm text-gray-500">
            {filteredRules.length}{selectedFunctionalityId ? `/${rules.length}` : ''} rules •{' '}
            {filteredAttributes.length}{selectedFunctionalityId ? `/${abstractAttributes.length}` : ''} attributes
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Approval Actions */}
          {selectedProduct.status === 'DRAFT' && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleSubmitProduct}
              className="gap-1.5"
            >
              <Send className="h-4 w-4" />
              Submit
            </Button>
          )}
          {selectedProduct.status === 'PENDING_APPROVAL' && (
            <Button
              variant="default"
              size="sm"
              onClick={handleApproveProduct}
              className="gap-1.5 bg-green-600 hover:bg-green-700"
            >
              <CheckCircle className="h-4 w-4" />
              Approve
            </Button>
          )}

          <div className="h-6 border-l" />

          {/* View Mode Toggle */}
          <div className="flex items-center border rounded-lg">
            <Button
              variant={viewMode === 'graph' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('graph')}
              className="rounded-r-none"
            >
              <Maximize2 className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'split' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('split')}
              className="rounded-none border-x"
            >
              <LayoutGrid className="h-4 w-4" />
            </Button>
            <Button
              variant={viewMode === 'table' ? 'default' : 'ghost'}
              size="sm"
              onClick={() => setViewMode('table')}
              className="rounded-l-none"
            >
              <Grid3X3 className="h-4 w-4" />
            </Button>
          </div>

          <div className="h-6 border-l" />

          {/* Panel Toggles */}
          <Button
            variant={functionalityPanelOpen ? 'default' : 'outline'}
            size="sm"
            onClick={toggleFunctionalityPanel}
            className="gap-1.5"
          >
            <Boxes className="h-4 w-4" />
            Functions
          </Button>
          <Button
            variant={attributeExplorerOpen ? 'default' : 'outline'}
            size="sm"
            onClick={toggleAttributeExplorer}
            className="gap-1.5"
          >
            <Layers className="h-4 w-4" />
            Attributes
          </Button>
          <Button
            variant={simulationPanelOpen ? 'default' : 'outline'}
            size="sm"
            onClick={toggleSimulationPanel}
            className="gap-1.5"
          >
            <FlaskConical className="h-4 w-4" />
            Simulate
          </Button>

          <div className="h-6 border-l" />

          <Button onClick={handleCreateRule} className="gap-1.5">
            <Plus className="h-4 w-4" />
            New Rule
          </Button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Far Left Panel - Functionality Panel */}
        {functionalityPanelOpen && viewMode !== 'graph' && (
          <div className="w-80 border-r bg-gray-50 overflow-hidden">
            <FunctionalityPanel />
          </div>
        )}

        {/* Left Panel - Attribute Explorer */}
        {attributeExplorerOpen && viewMode !== 'graph' && (
          <div className="w-72 border-r bg-gray-50 overflow-hidden">
            <AttributeExplorer />
          </div>
        )}

        {/* Center - Rule Canvas or Table */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {viewMode === 'table' ? (
            // Table View
            <div className="flex-1 overflow-auto p-4">
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {filteredRules.map((rule) => (
                  <RuleListItem
                    key={rule.id}
                    rule={rule}
                    isSelected={selectedRuleId === rule.id}
                    hasImmutableOutputs={hasImmutableOutputs(rule)}
                    onSelect={() => handleRuleSelect(rule.id)}
                    onEdit={() => handleRuleEdit(rule.id)}
                    onDelete={() => handleRuleDelete(rule.id)}
                    onToggle={() => handleRuleToggle(rule)}
                  />
                ))}
                {filteredRules.length === 0 && (
                  <div className="col-span-full py-12 text-center text-gray-500">
                    <GitBranch className="h-12 w-12 mx-auto text-gray-300 mb-3" />
                    <p className="text-sm">
                      {selectedFunctionalityId ? 'No rules for this functionality' : 'No rules defined yet'}
                    </p>
                    {!selectedFunctionalityId && (
                      <Button onClick={handleCreateRule} variant="outline" className="mt-4 gap-1.5">
                        <Plus className="h-4 w-4" />
                        Create First Rule
                      </Button>
                    )}
                  </div>
                )}
              </div>
            </div>
          ) : (
            // Graph View
            <div className="flex-1 p-4">
              <RuleCanvas
                attributes={filteredAttributes}
                rules={filteredRules}
                executionPlan={executionPlan}
                onNodeClick={handleNodeClick}
                onRuleEdit={handleRuleEdit}
              />
            </div>
          )}

          {/* Rule Details (in split view) */}
          {viewMode === 'split' && selectedRuleId && (
            <div className="h-64 border-t bg-white overflow-auto">
              {(() => {
                const selectedRule = rules.find((r) => r.id === selectedRuleId);
                if (!selectedRule) return null;

                let parsedExpression = {};
                try {
                  parsedExpression = JSON.parse(selectedRule.compiledExpression);
                } catch {
                  parsedExpression = {};
                }

                const ruleHasImmutableOutputs = hasImmutableOutputs(selectedRule);

                return (
                  <div className="p-4">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <div className="flex items-center gap-2">
                          <h3 className="text-lg font-semibold text-gray-800">
                            {selectedRule.description || 'Rule Details'}
                          </h3>
                          {ruleHasImmutableOutputs && (
                            <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-amber-100 text-amber-700 text-xs">
                              <Lock className="h-3 w-3" />
                              Has Immutable Outputs
                            </span>
                          )}
                        </div>
                        <p className="text-sm text-gray-500">{selectedRule.ruleType}</p>
                      </div>
                      <div className="flex items-center gap-2">
                        {ruleHasImmutableOutputs ? (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setCloneDialogOpen(true)}
                          >
                            <Copy className="h-4 w-4 mr-1" />
                            Clone to Edit
                          </Button>
                        ) : (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleRuleEdit(selectedRuleId)}
                          >
                            <Edit3 className="h-4 w-4 mr-1" />
                            Edit
                          </Button>
                        )}
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => handleRuleToggle(selectedRule)}
                        >
                          {selectedRule.enabled ? 'Disable' : 'Enable'}
                        </Button>
                      </div>
                    </div>

                    <div className="grid md:grid-cols-2 gap-4">
                      <div>
                        <h4 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                          Expression
                        </h4>
                        <p className="text-sm text-gray-800 bg-gray-50 rounded-lg p-3 font-mono">
                          {selectedRule.displayExpression}
                        </p>
                      </div>
                      <div>
                        <h4 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                          JSON Logic
                        </h4>
                        <pre className="text-xs bg-gray-900 text-gray-100 rounded-lg p-3 overflow-auto max-h-[100px]">
                          {JSON.stringify(parsedExpression, null, 2)}
                        </pre>
                      </div>
                    </div>

                    <div className="grid md:grid-cols-2 gap-4 mt-4">
                      <div>
                        <h4 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                          Inputs ({selectedRule.inputAttributes.length})
                        </h4>
                        <div className="flex flex-wrap gap-1">
                          {selectedRule.inputAttributes.map((attr) => {
                            const isImmutable = immutablePaths.has(attr.attributePath);
                            return (
                              <span
                                key={attr.attributePath}
                                className={cn(
                                  'rounded px-2 py-0.5 text-xs flex items-center gap-1',
                                  isImmutable
                                    ? 'bg-amber-100 text-amber-700'
                                    : 'bg-blue-100 text-blue-700'
                                )}
                              >
                                {attr.attributePath.split(':').pop()}
                                {isImmutable && <Lock className="h-3 w-3" />}
                              </span>
                            );
                          })}
                        </div>
                      </div>
                      <div>
                        <h4 className="text-xs font-semibold text-gray-500 uppercase mb-2">
                          Outputs ({selectedRule.outputAttributes.length})
                        </h4>
                        <div className="flex flex-wrap gap-1">
                          {selectedRule.outputAttributes.map((attr) => {
                            const isImmutable = immutablePaths.has(attr.attributePath);
                            return (
                              <span
                                key={attr.attributePath}
                                className={cn(
                                  'rounded px-2 py-0.5 text-xs flex items-center gap-1',
                                  isImmutable
                                    ? 'bg-amber-100 text-amber-700'
                                    : 'bg-emerald-100 text-emerald-700'
                                )}
                              >
                                {attr.attributePath.split(':').pop()}
                                {isImmutable && <Lock className="h-3 w-3" />}
                              </span>
                            );
                          })}
                        </div>
                      </div>
                    </div>
                  </div>
                );
              })()}
            </div>
          )}
        </div>

        {/* Right Panel - Simulation */}
        {simulationPanelOpen && (
          <div className="w-80 border-l overflow-hidden">
            <SimulationPanel />
          </div>
        )}
      </div>

      {/* Clone Dialog */}
      <CloneProductDialog
        open={cloneDialogOpen}
        onClose={() => setCloneDialogOpen(false)}
        reason="This product contains immutable attributes that cannot be modified directly."
        affectedPaths={Array.from(immutablePaths)}
      />

      {/* Floating Impact Panel */}
      <FloatingImpactPanel />
    </div>
  );
}

export default Rules;

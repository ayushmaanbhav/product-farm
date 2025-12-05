// Functionalities Page - Manage Product Functionalities
// Full CRUD with approval workflow and attribute assignment

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useProductStore } from '@/store';
import { api } from '@/services/api';
import { Link } from 'react-router-dom';
import type { ProductFunctionality, AbstractAttribute, FunctionalityRequiredAttribute } from '@/types';
import {
  Boxes,
  Plus,
  Pencil,
  Trash2,
  Search,
  Package,
  Lock,
  CheckCircle,
  Clock,
  FileText,
  X,
  ChevronRight,
  ChevronDown,
  Play,
  Send,
  Check,
  ArrowDownToLine,
  Cog,
} from 'lucide-react';

// =============================================================================
// STATUS BADGE
// =============================================================================

function StatusBadge({ status }: { status: string }) {
  const config = {
    ACTIVE: { bg: 'bg-emerald-100', text: 'text-emerald-700', icon: CheckCircle },
    PENDING_APPROVAL: { bg: 'bg-amber-100', text: 'text-amber-700', icon: Clock },
    DRAFT: { bg: 'bg-gray-100', text: 'text-gray-700', icon: FileText },
  }[status] || { bg: 'bg-gray-100', text: 'text-gray-700', icon: FileText };

  const Icon = config.icon;

  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium ${config.bg} ${config.text}`}>
      <Icon className="h-3 w-3" />
      {status.replace('_', ' ')}
    </span>
  );
}

// =============================================================================
// ATTRIBUTE SELECTOR
// =============================================================================

interface AttributeSelectorProps {
  productId: string;
  selectedPaths: string[];
  onToggle: (path: string) => void;
}

function AttributeSelector({ productId, selectedPaths, onToggle }: AttributeSelectorProps) {
  const [attributes, setAttributes] = useState<AbstractAttribute[]>([]);
  const [search, setSearch] = useState('');
  const [expandedComponents, setExpandedComponents] = useState<Set<string>>(new Set());

  useEffect(() => {
    api.abstractAttributes.list(productId).then(setAttributes);
  }, [productId]);

  const filtered = attributes.filter(
    (a) =>
      a.attributeName.toLowerCase().includes(search.toLowerCase()) ||
      a.componentType.toLowerCase().includes(search.toLowerCase())
  );

  const grouped = filtered.reduce(
    (acc, attr) => {
      const component = attr.componentType;
      if (!acc[component]) acc[component] = [];
      acc[component].push(attr);
      return acc;
    },
    {} as Record<string, AbstractAttribute[]>
  );

  const toggleComponent = (component: string) => {
    const next = new Set(expandedComponents);
    if (next.has(component)) {
      next.delete(component);
    } else {
      next.add(component);
    }
    setExpandedComponents(next);
  };

  return (
    <div className="border rounded-lg overflow-hidden">
      <div className="p-3 border-b bg-gray-50">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search attributes..."
            className="pl-10 h-8 text-sm"
          />
        </div>
      </div>
      <div className="max-h-64 overflow-y-auto">
        {Object.entries(grouped).map(([component, attrs]) => (
          <div key={component}>
            <button
              onClick={() => toggleComponent(component)}
              className="w-full px-3 py-2 flex items-center justify-between hover:bg-gray-50 text-left"
            >
              <span className="font-medium text-sm">{component}</span>
              <div className="flex items-center gap-2">
                <span className="text-xs text-gray-500">
                  {attrs.filter((a) => selectedPaths.includes(a.abstractPath)).length}/{attrs.length}
                </span>
                {expandedComponents.has(component) ? (
                  <ChevronDown className="h-4 w-4 text-gray-400" />
                ) : (
                  <ChevronRight className="h-4 w-4 text-gray-400" />
                )}
              </div>
            </button>
            {expandedComponents.has(component) && (
              <div className="border-t bg-white">
                {attrs.map((attr) => (
                  <label
                    key={attr.abstractPath}
                    className="flex items-center gap-3 px-4 py-2 hover:bg-gray-50 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={selectedPaths.includes(attr.abstractPath)}
                      onChange={() => onToggle(attr.abstractPath)}
                      className="rounded border-gray-300"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm">{attr.attributeName}</span>
                        {attr.immutable && <Lock className="h-3 w-3 text-amber-500" />}
                      </div>
                      <span className="text-xs text-gray-500">{attr.datatypeId}</span>
                    </div>
                    {attr.tags.some((t) => t.name === 'input') ? (
                      <span className="px-1.5 py-0.5 bg-blue-100 text-blue-700 rounded text-[10px] font-medium">
                        IN
                      </span>
                    ) : (
                      <span className="px-1.5 py-0.5 bg-amber-100 text-amber-700 rounded text-[10px] font-medium">
                        OUT
                      </span>
                    )}
                  </label>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// FUNCTIONALITY EDITOR DIALOG
// =============================================================================

interface FunctionalityEditorProps {
  productId: string;
  functionality: ProductFunctionality | null;
  onSave: (data: Partial<ProductFunctionality>) => Promise<void>;
  onCancel: () => void;
}

function FunctionalityEditor({ productId, functionality, onSave, onCancel }: FunctionalityEditorProps) {
  const [formData, setFormData] = useState({
    name: functionality?.name || '',
    displayName: functionality?.displayName || '',
    description: functionality?.description || '',
    requiredAttributePaths: functionality?.requiredAttributes.map((ra) => ra.abstractPath) || [],
  });
  const [isSaving, setIsSaving] = useState(false);

  const handleToggleAttribute = (path: string) => {
    const next = formData.requiredAttributePaths.includes(path)
      ? formData.requiredAttributePaths.filter((p) => p !== path)
      : [...formData.requiredAttributePaths, path];
    setFormData({ ...formData, requiredAttributePaths: next });
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const requiredAttributes: FunctionalityRequiredAttribute[] = formData.requiredAttributePaths.map(
        (path, idx) => ({
          functionalityId: functionality?.id || '',
          abstractPath: path,
          description: '',
          orderIndex: idx,
        })
      );

      await onSave({
        productId,
        name: formData.name,
        displayName: formData.displayName,
        description: formData.description,
        requiredAttributes,
      });
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-2xl p-6 max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold">
            {functionality ? 'Edit Functionality' : 'Create Functionality'}
          </h3>
          <button onClick={onCancel} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        <div className="space-y-4">
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Identifier (snake_case)
              </label>
              <Input
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., premium_calculation"
                disabled={!!functionality}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">Display Name</label>
              <Input
                value={formData.displayName}
                onChange={(e) => setFormData({ ...formData, displayName: e.target.value })}
                placeholder="e.g., Premium Calculation"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Describe what this functionality does..."
              rows={2}
              className="w-full px-3 py-2 border rounded-md text-sm resize-none"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Required Attributes ({formData.requiredAttributePaths.length} selected)
            </label>
            <AttributeSelector
              productId={productId}
              selectedPaths={formData.requiredAttributePaths}
              onToggle={handleToggleAttribute}
            />
          </div>
        </div>

        <div className="flex gap-3 mt-6">
          <Button variant="outline" className="flex-1" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            className="flex-1"
            onClick={handleSave}
            disabled={isSaving || !formData.name || !formData.displayName}
          >
            {isSaving ? 'Saving...' : 'Save'}
          </Button>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// FUNCTIONALITY CARD
// =============================================================================

interface FunctionalityCardProps {
  functionality: ProductFunctionality;
  abstractAttributes: AbstractAttribute[];
  onEdit: () => void;
  onDelete: () => void;
  onSubmit: () => void;
  onApprove: () => void;
}

function FunctionalityCard({ functionality, abstractAttributes, onEdit, onDelete, onSubmit, onApprove }: FunctionalityCardProps) {
  const [expanded, setExpanded] = useState(false);

  // Calculate input vs computed attribute counts
  const attributeCounts = functionality.requiredAttributes.reduce(
    (acc, ra) => {
      const attr = abstractAttributes.find((a) => a.abstractPath === ra.abstractPath);
      if (attr) {
        const isInput = attr.tags.some((t) => t.name === 'input');
        if (isInput) {
          acc.input++;
        } else {
          acc.computed++;
        }
      }
      return acc;
    },
    { input: 0, computed: 0 }
  );

  return (
    <Card className="overflow-hidden">
      <div className="px-4 py-3 flex items-start justify-between">
        <div
          className="flex items-start gap-3 flex-1 cursor-pointer"
          onClick={() => setExpanded(!expanded)}
        >
          <div className="p-2 bg-indigo-100 rounded-lg">
            <Boxes className="h-5 w-5 text-indigo-600" />
          </div>
          <div className="flex-1">
            <div className="flex items-center gap-2">
              <h4 className="font-medium">{functionality.displayName}</h4>
              {functionality.immutable && (
                <span title="Immutable">
                  <Lock className="h-4 w-4 text-amber-500" />
                </span>
              )}
            </div>
            <p className="text-xs text-gray-500">{functionality.name}</p>
            <p className="text-sm text-gray-600 mt-1">{functionality.description}</p>
            <div className="flex items-center gap-3 mt-2">
              <StatusBadge status={functionality.status} />
              <div className="flex items-center gap-2 text-xs">
                <span className="flex items-center gap-1 text-blue-600" title="Input attributes">
                  <ArrowDownToLine className="h-3 w-3" />
                  {attributeCounts.input}
                </span>
                <span className="flex items-center gap-1 text-amber-600" title="Computed attributes">
                  <Cog className="h-3 w-3" />
                  {attributeCounts.computed}
                </span>
                <span className="text-gray-400">
                  ({functionality.requiredAttributes.length} total)
                </span>
              </div>
            </div>
          </div>
          {expanded ? (
            <ChevronDown className="h-5 w-5 text-gray-400" />
          ) : (
            <ChevronRight className="h-5 w-5 text-gray-400" />
          )}
        </div>
      </div>

      {expanded && (
        <div className="border-t">
          {/* Required Attributes */}
          <div className="p-4 bg-gray-50">
            <h5 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2">
              Required Attributes
            </h5>
            {functionality.requiredAttributes.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {functionality.requiredAttributes.map((ra) => {
                  const attr = abstractAttributes.find((a) => a.abstractPath === ra.abstractPath);
                  const isInput = attr?.tags.some((t) => t.name === 'input');
                  return (
                    <span
                      key={ra.abstractPath}
                      className={`px-2 py-1 border rounded text-sm flex items-center gap-1.5 ${
                        isInput
                          ? 'bg-blue-50 border-blue-200 text-blue-700'
                          : 'bg-amber-50 border-amber-200 text-amber-700'
                      }`}
                    >
                      {isInput ? (
                        <ArrowDownToLine className="h-3 w-3" />
                      ) : (
                        <Cog className="h-3 w-3" />
                      )}
                      {ra.abstractPath.split(':').pop()}
                    </span>
                  );
                })}
              </div>
            ) : (
              <p className="text-sm text-gray-400">No attributes assigned</p>
            )}
          </div>

          {/* Actions */}
          <div className="p-4 border-t flex items-center justify-between">
            <div className="flex gap-2">
              {!functionality.immutable && (
                <>
                  <Button variant="outline" size="sm" onClick={onEdit}>
                    <Pencil className="h-4 w-4 mr-1" />
                    Edit
                  </Button>
                  <Button variant="outline" size="sm" onClick={onDelete} className="text-red-600 hover:text-red-700">
                    <Trash2 className="h-4 w-4 mr-1" />
                    Delete
                  </Button>
                </>
              )}
            </div>
            <div className="flex gap-2">
              {functionality.status === 'DRAFT' && (
                <Button size="sm" variant="outline" onClick={onSubmit}>
                  <Send className="h-4 w-4 mr-1" />
                  Submit
                </Button>
              )}
              {functionality.status === 'PENDING_APPROVAL' && (
                <Button size="sm" onClick={onApprove}>
                  <Check className="h-4 w-4 mr-1" />
                  Approve
                </Button>
              )}
              <Link to="/rules">
                <Button size="sm" variant="outline">
                  <Play className="h-4 w-4 mr-1" />
                  Test
                </Button>
              </Link>
            </div>
          </div>
        </div>
      )}
    </Card>
  );
}

// =============================================================================
// MAIN PAGE
// =============================================================================

export function Functionalities() {
  const { selectedProduct, products, selectProduct, fetchProducts } = useProductStore();
  const [functionalities, setFunctionalities] = useState<ProductFunctionality[]>([]);
  const [abstractAttributes, setAbstractAttributes] = useState<AbstractAttribute[]>([]);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string | null>(null);
  const [editingFunc, setEditingFunc] = useState<ProductFunctionality | null | 'new'>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  useEffect(() => {
    if (selectedProduct) {
      loadFunctionalities();
      loadAbstractAttributes();
    }
  }, [selectedProduct]);

  const loadFunctionalities = async () => {
    if (!selectedProduct) return;
    setLoading(true);
    try {
      const data = await api.functionalities.list(selectedProduct.id);
      setFunctionalities(data);
    } finally {
      setLoading(false);
    }
  };

  const loadAbstractAttributes = async () => {
    if (!selectedProduct) return;
    try {
      const data = await api.abstractAttributes.list(selectedProduct.id);
      setAbstractAttributes(data);
    } catch (e) {
      console.error('Failed to load abstract attributes:', e);
    }
  };

  const handleSave = async (data: Partial<ProductFunctionality>) => {
    if (!selectedProduct) return;

    if (editingFunc === 'new') {
      await api.functionalities.create(data);
    } else if (editingFunc) {
      await api.functionalities.update(selectedProduct.id, editingFunc.id, data);
    }
    setEditingFunc(null);
    loadFunctionalities();
  };

  const handleDelete = async (id: string) => {
    if (!selectedProduct) return;
    if (!confirm('Are you sure you want to delete this functionality?')) return;
    try {
      await api.functionalities.delete(selectedProduct.id, id);
      loadFunctionalities();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  const handleSubmit = async (id: string) => {
    if (!selectedProduct) return;
    await api.functionalities.submit(selectedProduct.id, id);
    loadFunctionalities();
  };

  const handleApprove = async (id: string) => {
    if (!selectedProduct) return;
    await api.functionalities.approve(selectedProduct.id, id);
    loadFunctionalities();
  };

  const filtered = functionalities.filter((f) => {
    const matchesSearch =
      f.name.toLowerCase().includes(search.toLowerCase()) ||
      f.displayName.toLowerCase().includes(search.toLowerCase());
    const matchesStatus = !statusFilter || f.status === statusFilter;
    return matchesSearch && matchesStatus;
  });

  const statusCounts = functionalities.reduce(
    (acc, f) => {
      acc[f.status] = (acc[f.status] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  // Product selector if none selected
  if (!selectedProduct) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Functionalities</h2>
          <p className="text-muted-foreground">Select a product to manage its functionalities</p>
        </div>

        {products.length > 0 ? (
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {products.map((product) => (
              <Card
                key={product.id}
                className="cursor-pointer transition-colors hover:border-primary"
                onClick={() => selectProduct(product.id)}
              >
                <CardHeader className="flex flex-row items-start gap-3 space-y-0">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                    <Package className="h-5 w-5 text-primary" />
                  </div>
                  <div>
                    <CardTitle className="text-base">{product.name}</CardTitle>
                    <CardDescription>{product.templateType}</CardDescription>
                  </div>
                </CardHeader>
                <CardContent>
                  <StatusBadge status={product.status} />
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-12">
              <Boxes className="h-12 w-12 text-muted-foreground/50" />
              <h3 className="mt-4 text-lg font-semibold">No Products Yet</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Create a product first to manage its functionalities.
              </p>
              <Button asChild className="mt-4">
                <Link to="/products">Go to Products</Link>
              </Button>
            </CardContent>
          </Card>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Functionalities</h2>
          <p className="text-muted-foreground">
            {selectedProduct.name} - {functionalities.length} functionalities
          </p>
        </div>
        <Button onClick={() => setEditingFunc('new')} className="gap-2">
          <Plus className="h-4 w-4" />
          New Functionality
        </Button>
      </div>

      {/* Stats */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Total</CardTitle>
            <Boxes className="h-4 w-4 text-indigo-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{functionalities.length}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Active</CardTitle>
            <CheckCircle className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts['ACTIVE'] || 0}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Pending</CardTitle>
            <Clock className="h-4 w-4 text-amber-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts['PENDING_APPROVAL'] || 0}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Draft</CardTitle>
            <FileText className="h-4 w-4 text-gray-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts['DRAFT'] || 0}</div>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search functionalities..."
            className="pl-10"
          />
        </div>
        <div className="flex gap-1 p-1 bg-gray-100 rounded-lg">
          {[null, 'ACTIVE', 'PENDING_APPROVAL', 'DRAFT'].map((status) => (
            <button
              key={status || 'all'}
              onClick={() => setStatusFilter(status)}
              className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
                statusFilter === status
                  ? 'bg-white shadow text-gray-900'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              {status ? status.replace('_', ' ') : 'All'}
            </button>
          ))}
        </div>
      </div>

      {/* List */}
      {loading ? (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      ) : filtered.length === 0 ? (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Boxes className="h-12 w-12 text-muted-foreground/50" />
            <h3 className="mt-4 text-lg font-semibold">
              {search || statusFilter ? 'No matching functionalities' : 'No Functionalities Yet'}
            </h3>
            <p className="mt-2 text-sm text-muted-foreground">
              {search || statusFilter
                ? 'Try adjusting your filters'
                : 'Create a functionality to group related attributes together.'}
            </p>
            {!search && !statusFilter && (
              <Button className="mt-4" onClick={() => setEditingFunc('new')}>
                <Plus className="h-4 w-4 mr-2" />
                Create Functionality
              </Button>
            )}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-3">
          {filtered.map((func) => (
            <FunctionalityCard
              key={func.id}
              functionality={func}
              abstractAttributes={abstractAttributes}
              onEdit={() => setEditingFunc(func)}
              onDelete={() => handleDelete(func.id)}
              onSubmit={() => handleSubmit(func.id)}
              onApprove={() => handleApprove(func.id)}
            />
          ))}
        </div>
      )}

      {/* Editor Dialog */}
      {editingFunc && (
        <FunctionalityEditor
          productId={selectedProduct.id}
          functionality={editingFunc === 'new' ? null : editingFunc}
          onSave={handleSave}
          onCancel={() => setEditingFunc(null)}
        />
      )}
    </div>
  );
}

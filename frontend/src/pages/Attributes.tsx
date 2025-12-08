// Attributes Page - Browse and manage abstract and concrete attributes
// Enhanced with functionality filtering, tabs, and CRUD operations

import { useEffect, useState, useMemo } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useProductStore } from '@/store';
import { api } from '@/services/api';
import { Link } from 'react-router-dom';
import type {
  AbstractAttribute,
  Attribute,
  ProductFunctionality,
  DataType,
  TemplateEnumeration,
  Rule,
} from '@/types';
import {
  Layers,
  Database,
  Calculator,
  ArrowRight,
  Package,
  Plus,
  Pencil,
  Trash2,
  Search,
  Lock,
  ChevronRight,
  ChevronDown,
  Boxes,
  FileText,
  Box,
} from 'lucide-react';
import { AttributeExplorer } from '@/components/AttributeExplorer';
import { AbstractAttributeForm, AttributeInstanceForm } from '@/components/forms';

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

// =============================================================================
// FUNCTIONALITY FILTER
// =============================================================================

interface FunctionalityFilterProps {
  productId: string;
  selected: string | null;
  onChange: (funcId: string | null) => void;
}

function FunctionalityFilter({ productId, selected, onChange }: FunctionalityFilterProps) {
  const [functionalities, setFunctionalities] = useState<ProductFunctionality[]>([]);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    api.functionalities.list(productId).then(setFunctionalities);
  }, [productId]);

  const selectedFunc = functionalities.find(f => f.id === selected);

  return (
    <div className="relative">
      <Button
        variant="outline"
        onClick={() => setOpen(!open)}
        className="gap-2"
      >
        <Boxes className="h-4 w-4" />
        {selectedFunc ? selectedFunc.displayName : 'All Functionalities'}
        <ChevronDown className="h-4 w-4 ml-1" />
      </Button>

      {open && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setOpen(false)} />
          <div className="absolute top-full left-0 mt-1 w-64 bg-white border rounded-lg shadow-lg z-20 overflow-hidden">
            <div className="p-2 border-b">
              <button
                onClick={() => { onChange(null); setOpen(false); }}
                className={`w-full px-3 py-2 rounded text-left text-sm ${!selected ? 'bg-indigo-50 text-indigo-700' : 'hover:bg-gray-50'}`}
              >
                All Functionalities
              </button>
            </div>
            <div className="max-h-64 overflow-y-auto p-2">
              {functionalities.map(func => (
                <button
                  key={func.id}
                  onClick={() => { onChange(func.id); setOpen(false); }}
                  className={`w-full px-3 py-2 rounded text-left text-sm ${selected === func.id ? 'bg-indigo-50 text-indigo-700' : 'hover:bg-gray-50'}`}
                >
                  <div className="font-medium">{func.displayName}</div>
                  <div className="text-xs text-gray-500">{func.requiredAttributes.length} attributes</div>
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
// ATTRIBUTE TABLE
// =============================================================================

interface AttributeTableProps {
  attributes: AbstractAttribute[];
  onEdit: (attr: AbstractAttribute) => void;
  onDelete: (path: string) => void;
}

function AttributeTable({ attributes, onEdit, onDelete }: AttributeTableProps) {
  const [expandedComponent, setExpandedComponent] = useState<string | null>(null);

  const grouped = attributes.reduce((acc, attr) => {
    const component = attr.componentType || 'default';
    if (!acc[component]) acc[component] = [];
    acc[component].push(attr);
    return acc;
  }, {} as Record<string, AbstractAttribute[]>);

  return (
    <div className="space-y-2">
      {Object.entries(grouped).map(([component, attrs]) => (
        <div key={component} className="border rounded-lg overflow-hidden">
          <button
            onClick={() => setExpandedComponent(expandedComponent === component ? null : component)}
            className="w-full px-4 py-3 bg-gray-50 flex items-center justify-between hover:bg-gray-100"
          >
            <div className="flex items-center gap-2">
              <Package className="h-4 w-4 text-gray-500" />
              <span className="font-medium capitalize">{component}</span>
              <span className="text-xs text-gray-500">({attrs.length})</span>
            </div>
            {expandedComponent === component ? (
              <ChevronDown className="h-4 w-4 text-gray-400" />
            ) : (
              <ChevronRight className="h-4 w-4 text-gray-400" />
            )}
          </button>

          {expandedComponent === component && (
            <div className="divide-y">
              {attrs.map(attr => (
                <div
                  key={attr.abstractPath}
                  className="px-4 py-3 flex items-center justify-between hover:bg-gray-50"
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm">{attr.attributeName}</span>
                      {attr.immutable && (
                        <span title="Immutable">
                          <Lock className="h-3 w-3 text-amber-500" />
                        </span>
                      )}
                      <span className="px-1.5 py-0.5 bg-gray-100 rounded text-[10px] font-medium text-gray-600">
                        {attr.datatypeId}
                      </span>
                      {attr.tags.some(t => t.name === 'input') && (
                        <span className="px-1.5 py-0.5 bg-blue-100 rounded text-[10px] font-medium text-blue-600">
                          INPUT
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-gray-500 mt-0.5">{attr.description}</p>
                    {attr.tags.length > 0 && (
                      <div className="flex gap-1 mt-1">
                        {attr.tags.slice(0, 4).map(tag => (
                          <span key={tag.name} className="px-1.5 py-0.5 bg-gray-50 border rounded text-[10px] text-gray-500">
                            {tag.name}
                          </span>
                        ))}
                        {attr.tags.length > 4 && (
                          <span className="text-[10px] text-gray-400">+{attr.tags.length - 4}</span>
                        )}
                      </div>
                    )}
                  </div>
                  {!attr.immutable && (
                    <div className="flex gap-1">
                      <button
                        onClick={() => onEdit(attr)}
                        className="p-1.5 hover:bg-gray-100 rounded"
                      >
                        <Pencil className="h-4 w-4 text-gray-500" />
                      </button>
                      <button
                        onClick={() => onDelete(attr.abstractPath)}
                        className="p-1.5 hover:bg-gray-100 rounded"
                      >
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// =============================================================================
// CONCRETE ATTRIBUTE TABLE
// =============================================================================

interface ConcreteAttributeTableProps {
  attributes: Attribute[];
  abstractAttributes: AbstractAttribute[];
  datatypes: DataType[];
  onEdit: (attr: Attribute) => void;
  onDelete: (path: string) => void;
}

function ConcreteAttributeTable({
  attributes,
  abstractAttributes,
  datatypes,
  onEdit,
  onDelete,
}: ConcreteAttributeTableProps) {
  const [expandedComponent, setExpandedComponent] = useState<string | null>(null);

  // Group by component type
  const grouped = attributes.reduce((acc, attr) => {
    const component = attr.componentType || 'default';
    if (!acc[component]) acc[component] = [];
    acc[component].push(attr);
    return acc;
  }, {} as Record<string, Attribute[]>);

  // Get abstract attribute for a concrete one
  const getAbstract = (abstractPath: string) =>
    abstractAttributes.find((a) => a.abstractPath === abstractPath);

  // Get datatype for an abstract attribute
  const getDatatype = (datatypeId: string) =>
    datatypes.find((dt) => dt.id === datatypeId);

  // Format value for display
  const formatValue = (attr: Attribute) => {
    if (attr.valueType === 'JUST_DEFINITION') return <span className="text-gray-400 italic">No value</span>;
    if (attr.valueType === 'RULE_DRIVEN') return <span className="text-amber-600">Rule-driven</span>;
    if (!attr.value) return <span className="text-gray-400">null</span>;
    if (attr.value.type === 'null') return <span className="text-gray-400">null</span>;

    const val = (attr.value as { value: unknown }).value;
    if (typeof val === 'boolean') return val ? 'true' : 'false';
    if (typeof val === 'object') return JSON.stringify(val).slice(0, 30) + '...';
    return String(val).slice(0, 50);
  };

  const getValueTypeIcon = (valueType: string) => {
    switch (valueType) {
      case 'FIXED_VALUE':
        return <FileText className="h-3.5 w-3.5 text-blue-500" />;
      case 'RULE_DRIVEN':
        return <Calculator className="h-3.5 w-3.5 text-amber-500" />;
      case 'JUST_DEFINITION':
        return <Box className="h-3.5 w-3.5 text-gray-400" />;
      default:
        return null;
    }
  };

  return (
    <div className="space-y-2">
      {Object.entries(grouped).map(([component, attrs]) => (
        <div key={component} className="border rounded-lg overflow-hidden">
          <button
            onClick={() => setExpandedComponent(expandedComponent === component ? null : component)}
            className="w-full px-4 py-3 bg-gray-50 flex items-center justify-between hover:bg-gray-100"
          >
            <div className="flex items-center gap-2">
              <Package className="h-4 w-4 text-gray-500" />
              <span className="font-medium capitalize">{component}</span>
              <span className="text-xs text-gray-500">({attrs.length})</span>
            </div>
            {expandedComponent === component ? (
              <ChevronDown className="h-4 w-4 text-gray-400" />
            ) : (
              <ChevronRight className="h-4 w-4 text-gray-400" />
            )}
          </button>

          {expandedComponent === component && (
            <div className="divide-y">
              {attrs.map((attr) => {
                const abstractAttr = getAbstract(attr.abstractPath);
                const datatype = abstractAttr ? getDatatype(abstractAttr.datatypeId) : null;

                return (
                  <div
                    key={attr.path}
                    className="px-4 py-3 flex items-center justify-between hover:bg-gray-50"
                  >
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        {getValueTypeIcon(attr.valueType)}
                        <span className="font-medium text-sm">{attr.attributeName}</span>
                        <span className="px-1.5 py-0.5 bg-purple-100 rounded text-[10px] font-medium text-purple-600">
                          {attr.componentId}
                        </span>
                        {datatype && (
                          <span className="px-1.5 py-0.5 bg-gray-100 rounded text-[10px] font-medium text-gray-600">
                            {datatype.primitiveType}
                          </span>
                        )}
                      </div>
                      <div className="text-xs text-gray-600 mt-0.5 font-mono">
                        {formatValue(attr)}
                      </div>
                    </div>
                    <div className="flex gap-1">
                      <button
                        onClick={() => onEdit(attr)}
                        className="p-1.5 hover:bg-gray-100 rounded"
                      >
                        <Pencil className="h-4 w-4 text-gray-500" />
                      </button>
                      <button
                        onClick={() => onDelete(attr.path)}
                        className="p-1.5 hover:bg-gray-100 rounded"
                      >
                        <Trash2 className="h-4 w-4 text-red-500" />
                      </button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      ))}

      {Object.keys(grouped).length === 0 && (
        <div className="text-center py-12 text-gray-500">
          <Box className="h-12 w-12 mx-auto text-gray-300" />
          <p className="mt-2">No concrete attributes yet</p>
          <p className="text-sm">Create a concrete attribute to instantiate an abstract attribute</p>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// MAIN PAGE
// =============================================================================

export function Attributes() {
  const { abstractAttributes, selectedProduct, products, selectProduct, fetchProducts, fetchAttributes } = useProductStore();
  const [search, setSearch] = useState('');
  const [functionalityFilter, setFunctionalityFilter] = useState<string | null>(null);
  const [filteredAttributes, setFilteredAttributes] = useState<AbstractAttribute[]>([]);
  const [editingAttr, setEditingAttr] = useState<AbstractAttribute | null | 'new'>(null);
  const [datatypes, setDatatypes] = useState<DataType[]>([]);
  const [enumerations, setEnumerations] = useState<TemplateEnumeration[]>([]);
  const [viewMode, setViewMode] = useState<'tree' | 'table'>('table');

  // New state for concrete attributes
  const [activeTab, setActiveTab] = useState<'abstract' | 'concrete'>('abstract');
  const [concreteAttributes, setConcreteAttributes] = useState<Attribute[]>([]);
  const [rules, setRules] = useState<Rule[]>([]);
  const [editingConcreteAttr, setEditingConcreteAttr] = useState<Attribute | null | 'new'>(null);

  // Fetch products and datatypes on mount
  useEffect(() => {
    fetchProducts();
    api.datatypes.list().then(setDatatypes);
  }, [fetchProducts]);

  // Fetch enumerations, rules, and concrete attributes when product changes
  useEffect(() => {
    if (selectedProduct) {
      api.templates.listEnumerations(selectedProduct.templateType)
        .then(setEnumerations)
        .catch(e => console.error('Failed to load enumerations:', e));
      api.rules.list(selectedProduct.id)
        .then(setRules)
        .catch(e => console.error('Failed to load rules:', e));
      api.attributes.list(selectedProduct.id)
        .then(setConcreteAttributes)
        .catch(e => console.error('Failed to load concrete attributes:', e));
    }
  }, [selectedProduct]);

  // Filter by functionality
  useEffect(() => {
    const filterAttrs = async () => {
      if (!selectedProduct) return;

      if (functionalityFilter) {
        const funcAttrs = await api.abstractAttributes.getByFunctionality(
          selectedProduct.id,
          functionalityFilter
        );
        setFilteredAttributes(funcAttrs);
      } else {
        setFilteredAttributes(abstractAttributes);
      }
    };

    filterAttrs();
  }, [selectedProduct, functionalityFilter, abstractAttributes]);

  // Search filter
  const displayedAttributes = filteredAttributes.filter(
    a =>
      a.attributeName.toLowerCase().includes(search.toLowerCase()) ||
      a.componentType.toLowerCase().includes(search.toLowerCase()) ||
      a.description?.toLowerCase().includes(search.toLowerCase())
  );

  const inputCount = displayedAttributes.filter(a => a.tags.some(t => t.name === 'input')).length;
  const computedCount = displayedAttributes.length - inputCount;
  const immutableCount = displayedAttributes.filter(a => a.immutable).length;

  const componentGroups = displayedAttributes.reduce(
    (acc, attr) => {
      const component = attr.componentType || 'default';
      if (!acc[component]) acc[component] = [];
      acc[component].push(attr);
      return acc;
    },
    {} as Record<string, typeof displayedAttributes>
  );

  const handleSave = async (data: Partial<AbstractAttribute>) => {
    if (!selectedProduct) return;

    try {
      if (editingAttr === 'new') {
        await api.abstractAttributes.create(data);
      } else if (editingAttr) {
        await api.abstractAttributes.update(editingAttr.abstractPath, data);
      }
      setEditingAttr(null);
      fetchAttributes(selectedProduct.id);
    } catch (e) {
      alert(`Failed to save attribute: ${(e as Error).message}`);
    }
  };

  const handleDelete = async (path: string) => {
    if (!confirm('Are you sure you want to delete this attribute?')) return;
    try {
      await api.abstractAttributes.delete(path);
      if (selectedProduct) {
        fetchAttributes(selectedProduct.id);
      }
    } catch (e) {
      alert((e as Error).message);
    }
  };

  // Concrete attribute handlers
  const handleSaveConcrete = async (data: Partial<Attribute>) => {
    if (!selectedProduct) return;

    try {
      if (editingConcreteAttr === 'new') {
        await api.attributes.create(data);
      } else if (editingConcreteAttr) {
        await api.attributes.update(editingConcreteAttr.path, data);
      }
      setEditingConcreteAttr(null);
      api.attributes.list(selectedProduct.id)
        .then(setConcreteAttributes)
        .catch(e => console.error('Failed to refresh attributes:', e));
    } catch (e) {
      alert(`Failed to save concrete attribute: ${(e as Error).message}`);
    }
  };

  const handleDeleteConcrete = async (path: string) => {
    if (!confirm('Are you sure you want to delete this concrete attribute?')) return;
    try {
      await api.attributes.delete(path);
      if (selectedProduct) {
        api.attributes.list(selectedProduct.id)
          .then(setConcreteAttributes)
          .catch(e => console.error('Failed to refresh attributes:', e));
      }
    } catch (e) {
      alert(`Failed to delete concrete attribute: ${(e as Error).message}`);
    }
  };

  // Filtered concrete attributes
  const displayedConcreteAttributes = useMemo(() => {
    return concreteAttributes.filter(
      (a) =>
        a.attributeName.toLowerCase().includes(search.toLowerCase()) ||
        a.componentType.toLowerCase().includes(search.toLowerCase()) ||
        a.componentId.toLowerCase().includes(search.toLowerCase())
    );
  }, [concreteAttributes, search]);

  // Concrete stats
  const fixedValueCount = displayedConcreteAttributes.filter((a) => a.valueType === 'FIXED_VALUE').length;
  const ruleDrivenCount = displayedConcreteAttributes.filter((a) => a.valueType === 'RULE_DRIVEN').length;
  const definitionOnlyCount = displayedConcreteAttributes.filter((a) => a.valueType === 'JUST_DEFINITION').length;

  if (!selectedProduct) {
    return (
      <div className="space-y-6">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Attributes</h2>
          <p className="text-muted-foreground">
            Select a product to explore its attributes
          </p>
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
                  <span
                    className={`px-2 py-0.5 rounded text-xs font-medium ${
                      product.status === 'ACTIVE'
                        ? 'bg-emerald-100 text-emerald-700'
                        : product.status === 'DRAFT'
                          ? 'bg-gray-100 text-gray-700'
                          : 'bg-amber-100 text-amber-700'
                    }`}
                  >
                    {product.status}
                  </span>
                </CardContent>
              </Card>
            ))}
          </div>
        ) : (
          <Card>
            <CardContent className="flex flex-col items-center justify-center py-12">
              <Layers className="h-12 w-12 text-muted-foreground/50" />
              <h3 className="mt-4 text-lg font-semibold">No Products Yet</h3>
              <p className="mt-2 text-sm text-muted-foreground">
                Create a product first to manage its attributes.
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
          <h2 className="text-2xl font-bold tracking-tight">Attributes</h2>
          <p className="text-muted-foreground">
            {selectedProduct.name} - {activeTab === 'abstract' ? displayedAttributes.length : displayedConcreteAttributes.length} attributes
            {functionalityFilter && activeTab === 'abstract' && ' (filtered)'}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {activeTab === 'abstract' ? (
            <Button onClick={() => setEditingAttr('new')} className="gap-2">
              <Plus className="h-4 w-4" />
              Add Abstract Attribute
            </Button>
          ) : (
            <Button onClick={() => setEditingConcreteAttr('new')} className="gap-2">
              <Plus className="h-4 w-4" />
              Add Concrete Attribute
            </Button>
          )}
          <Button asChild variant="outline" className="gap-2">
            <Link to="/rules">
              <ArrowRight className="h-4 w-4" />
              Rule Canvas
            </Link>
          </Button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex gap-1 p-1 bg-gray-100 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab('abstract')}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 ${
            activeTab === 'abstract' ? 'bg-white shadow text-gray-900' : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <Layers className="h-4 w-4" />
          Abstract ({abstractAttributes.length})
        </button>
        <button
          onClick={() => setActiveTab('concrete')}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2 ${
            activeTab === 'concrete' ? 'bg-white shadow text-gray-900' : 'text-gray-600 hover:text-gray-900'
          }`}
        >
          <Box className="h-4 w-4" />
          Concrete ({concreteAttributes.length})
        </button>
      </div>

      {/* Stats - Different for each tab */}
      {activeTab === 'abstract' ? (
        <div className="grid gap-4 md:grid-cols-5">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Total</CardTitle>
              <Layers className="h-4 w-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{displayedAttributes.length}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Inputs</CardTitle>
              <Database className="h-4 w-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{inputCount}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Computed</CardTitle>
              <Calculator className="h-4 w-4 text-amber-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{computedCount}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Immutable</CardTitle>
              <Lock className="h-4 w-4 text-amber-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{immutableCount}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Components</CardTitle>
              <Package className="h-4 w-4 text-emerald-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{Object.keys(componentGroups).length}</div>
            </CardContent>
          </Card>
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Total</CardTitle>
              <Box className="h-4 w-4 text-purple-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{displayedConcreteAttributes.length}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Fixed Value</CardTitle>
              <FileText className="h-4 w-4 text-blue-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{fixedValueCount}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Rule Driven</CardTitle>
              <Calculator className="h-4 w-4 text-amber-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{ruleDrivenCount}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">Definition Only</CardTitle>
              <Box className="h-4 w-4 text-gray-400" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{definitionOnlyCount}</div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Filters */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder={activeTab === 'abstract' ? 'Search abstract attributes...' : 'Search concrete attributes...'}
            className="pl-10"
          />
        </div>
        {activeTab === 'abstract' && (
          <>
            <FunctionalityFilter
              productId={selectedProduct.id}
              selected={functionalityFilter}
              onChange={setFunctionalityFilter}
            />
            <div className="flex gap-1 p-1 bg-gray-100 rounded-lg">
              <button
                onClick={() => setViewMode('table')}
                className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
                  viewMode === 'table' ? 'bg-white shadow text-gray-900' : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                Table
              </button>
              <button
                onClick={() => setViewMode('tree')}
                className={`px-3 py-1.5 rounded text-sm font-medium transition-colors ${
                  viewMode === 'tree' ? 'bg-white shadow text-gray-900' : 'text-gray-600 hover:text-gray-900'
                }`}
              >
                Tree
              </button>
            </div>
          </>
        )}
      </div>

      {/* Content - Conditional based on activeTab */}
      {activeTab === 'abstract' ? (
        // Abstract attributes content
        viewMode === 'table' ? (
          <AttributeTable
            attributes={displayedAttributes}
            onEdit={setEditingAttr}
            onDelete={handleDelete}
          />
        ) : (
          <div className="grid gap-6 lg:grid-cols-3">
            {/* Explorer Tree */}
            <div className="lg:col-span-1">
              <div className="h-[600px]">
                <AttributeExplorer />
              </div>
            </div>

            {/* Attribute Groups */}
            <div className="lg:col-span-2 space-y-4">
              <h3 className="text-lg font-semibold">Attributes by Component</h3>
              <div className="grid gap-4 md:grid-cols-2">
                {Object.entries(componentGroups).map(([component, attrs]) => (
                  <Card key={component}>
                    <CardHeader className="pb-3">
                      <CardTitle className="text-base flex items-center gap-2">
                        <Package className="h-4 w-4 text-gray-500" />
                        {component.charAt(0).toUpperCase() + component.slice(1)}
                      </CardTitle>
                      <CardDescription>{attrs.length} attributes</CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-2">
                        {attrs.slice(0, 5).map((attr) => (
                          <div
                            key={attr.abstractPath}
                            className="flex items-center justify-between text-sm"
                          >
                            <div className="flex items-center gap-1.5">
                              <span className="truncate">{attr.attributeName}</span>
                              {attr.immutable && <Lock className="h-3 w-3 text-amber-500" />}
                            </div>
                            <div className="flex items-center gap-2">
                              <span className="rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-medium text-gray-600">
                                {attr.datatypeId}
                              </span>
                              {attr.tags.some((t) => t.name === 'input') && (
                                <span className="rounded bg-blue-100 px-1.5 py-0.5 text-[10px] font-medium text-blue-600">
                                  IN
                                </span>
                              )}
                            </div>
                          </div>
                        ))}
                        {attrs.length > 5 && (
                          <p className="text-xs text-muted-foreground">
                            +{attrs.length - 5} more
                          </p>
                        )}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          </div>
        )
      ) : (
        // Concrete attributes content
        <ConcreteAttributeTable
          attributes={displayedConcreteAttributes}
          abstractAttributes={abstractAttributes}
          datatypes={datatypes}
          onEdit={setEditingConcreteAttr}
          onDelete={handleDeleteConcrete}
        />
      )}

      {/* Abstract Editor Dialog */}
      {editingAttr && (
        <AbstractAttributeForm
          productId={selectedProduct.id}
          attribute={editingAttr === 'new' ? null : editingAttr}
          datatypes={datatypes}
          enumerations={enumerations}
          existingComponentTypes={[...new Set(abstractAttributes.map(a => a.componentType))]}
          existingTags={[...new Set(abstractAttributes.flatMap(a => a.tags.map(t => t.name)))]}
          existingAttributes={abstractAttributes}
          onSave={handleSave}
          onCancel={() => setEditingAttr(null)}
        />
      )}

      {/* Concrete Editor Dialog */}
      {editingConcreteAttr && (
        <AttributeInstanceForm
          productId={selectedProduct.id}
          attribute={editingConcreteAttr === 'new' ? null : editingConcreteAttr}
          abstractAttributes={abstractAttributes}
          datatypes={datatypes}
          rules={rules}
          enumerations={enumerations}
          onSave={handleSaveConcrete}
          onCancel={() => setEditingConcreteAttr(null)}
        />
      )}
    </div>
  );
}

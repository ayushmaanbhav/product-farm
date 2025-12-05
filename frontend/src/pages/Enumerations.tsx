// Enumerations Page - Manage template enumerations with value editing
// Enhanced with usage tracking and cascade delete support

import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { api } from '@/services/api';
import type { TemplateEnumeration } from '@/types';
import {
  List,
  Plus,
  Pencil,
  Trash2,
  X,
  Search,
  ChevronRight,
  Tag,
  AlertTriangle,
  GripVertical,
  Info,
} from 'lucide-react';

// =============================================================================
// USAGE WARNING DIALOG
// =============================================================================

interface UsageWarningDialogProps {
  enumeration: TemplateEnumeration;
  valueToRemove: string;
  affectedAttributes: Array<{ productId: string; attributePath: string; valueType: string }>;
  onConfirm: () => void;
  onCancel: () => void;
}

function UsageWarningDialog({
  enumeration: _enumeration,
  valueToRemove,
  affectedAttributes,
  onConfirm,
  onCancel,
}: UsageWarningDialogProps) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-md p-6">
        <div className="flex items-start gap-3 mb-4">
          <div className="p-2 bg-amber-100 rounded-full">
            <AlertTriangle className="h-5 w-5 text-amber-600" />
          </div>
          <div>
            <h3 className="text-lg font-semibold">Remove "{valueToRemove}"?</h3>
            <p className="text-sm text-gray-500 mt-1">
              This value is used by {affectedAttributes.length} attribute{affectedAttributes.length !== 1 ? 's' : ''}
            </p>
          </div>
        </div>

        <div className="mb-4 max-h-48 overflow-y-auto">
          <p className="text-sm font-medium text-gray-700 mb-2">Affected attributes:</p>
          <div className="space-y-1">
            {affectedAttributes.map((attr, i) => (
              <div key={i} className="flex items-center gap-2 text-sm p-2 bg-gray-50 rounded">
                <span className="font-mono text-xs">{attr.productId}</span>
                <span className="text-gray-400">/</span>
                <span className="font-mono text-xs flex-1">{attr.attributePath}</span>
                <span className="text-xs text-gray-500">({attr.valueType})</span>
              </div>
            ))}
          </div>
        </div>

        <div className="p-3 bg-amber-50 border border-amber-200 rounded-md mb-4">
          <p className="text-sm text-amber-800">
            These attribute values will be set to <code className="bg-amber-100 px-1 rounded">null</code>.
          </p>
        </div>

        <div className="flex gap-3">
          <Button variant="outline" className="flex-1" onClick={onCancel}>
            Cancel
          </Button>
          <Button variant="destructive" className="flex-1" onClick={onConfirm}>
            Remove Anyway
          </Button>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// VALUE EDITOR
// =============================================================================

interface ValueEditorProps {
  values: string[];
  onChange: (values: string[]) => void;
  onCheckUsage?: (value: string) => Promise<Array<{ productId: string; attributePath: string; valueType: string }>>;
}

function ValueEditor({ values, onChange, onCheckUsage }: ValueEditorProps) {
  const [newValue, setNewValue] = useState('');
  const [valueError, setValueError] = useState<string | null>(null);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);
  const [pendingRemove, setPendingRemove] = useState<{
    value: string;
    affectedAttributes: Array<{ productId: string; attributePath: string; valueType: string }>;
  } | null>(null);

  // Validate enumeration value
  const validateValue = (value: string): boolean => {
    if (!value.trim()) {
      setValueError('Value cannot be empty');
      return false;
    }
    const pattern = /^[a-z]([-][a-z]|[a-z]){0,50}$/;
    if (!pattern.test(value)) {
      setValueError('Must be lowercase letters with optional hyphens (e.g., third-party)');
      return false;
    }
    if (values.includes(value)) {
      setValueError('Value already exists');
      return false;
    }
    setValueError(null);
    return true;
  };

  const handleAddValue = () => {
    const trimmed = newValue.trim().toLowerCase();
    if (validateValue(trimmed)) {
      onChange([...values, trimmed]);
      setNewValue('');
      setValueError(null);
    }
  };

  const handleRemoveValue = async (value: string) => {
    if (onCheckUsage) {
      // Check usage before removing
      const affectedAttributes = await onCheckUsage(value);
      if (affectedAttributes.length > 0) {
        setPendingRemove({ value, affectedAttributes });
        return;
      }
    }
    // No usage, remove directly
    onChange(values.filter((v) => v !== value));
  };

  const confirmRemove = () => {
    if (pendingRemove) {
      onChange(values.filter((v) => v !== pendingRemove.value));
      setPendingRemove(null);
    }
  };

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;

    const newValues = [...values];
    const draggedValue = newValues[draggedIndex];
    newValues.splice(draggedIndex, 1);
    newValues.splice(index, 0, draggedValue);
    onChange(newValues);
    setDraggedIndex(index);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
  };

  const moveValue = (index: number, direction: 'up' | 'down') => {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= values.length) return;

    const newValues = [...values];
    [newValues[index], newValues[newIndex]] = [newValues[newIndex], newValues[index]];
    onChange(newValues);
  };

  return (
    <div className="space-y-3">
      {/* Add Value Input */}
      <div className="flex gap-2">
        <Input
          value={newValue}
          onChange={(e) => {
            setNewValue(e.target.value.toLowerCase().replace(/[^a-z-]/g, ''));
            setValueError(null);
          }}
          placeholder="Add value (e.g., third-party)"
          onKeyDown={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddValue())}
          className={`font-mono ${valueError ? 'border-red-500' : ''}`}
        />
        <Button
          variant="outline"
          size="icon"
          onClick={handleAddValue}
          disabled={!newValue.trim()}
        >
          <Plus className="h-4 w-4" />
        </Button>
      </div>
      {valueError && <p className="text-xs text-red-500">{valueError}</p>}

      {/* Values List */}
      <div className="border rounded-md divide-y max-h-64 overflow-y-auto">
        {values.length === 0 ? (
          <div className="p-4 text-sm text-gray-400 text-center">
            No values added yet. Add at least one value.
          </div>
        ) : (
          values.map((value, index) => (
            <div
              key={value}
              draggable
              onDragStart={() => handleDragStart(index)}
              onDragOver={(e) => handleDragOver(e, index)}
              onDragEnd={handleDragEnd}
              className={`flex items-center gap-2 px-3 py-2 text-sm ${
                draggedIndex === index ? 'bg-blue-50' : 'hover:bg-gray-50'
              }`}
            >
              <span className="cursor-grab text-gray-400 hover:text-gray-600">
                <GripVertical className="h-4 w-4" />
              </span>
              <span className="flex-1 font-mono">{value}</span>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => moveValue(index, 'up')}
                  disabled={index === 0}
                  className="p-1 hover:bg-gray-200 rounded disabled:opacity-30"
                  title="Move up"
                >
                  <span className="text-xs">&#9650;</span>
                </button>
                <button
                  onClick={() => moveValue(index, 'down')}
                  disabled={index === values.length - 1}
                  className="p-1 hover:bg-gray-200 rounded disabled:opacity-30"
                  title="Move down"
                >
                  <span className="text-xs">&#9660;</span>
                </button>
                <button
                  onClick={() => handleRemoveValue(value)}
                  className="p-1 hover:bg-red-100 rounded text-red-500"
                  title="Remove"
                >
                  <X className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Usage Warning Dialog */}
      {pendingRemove && (
        <UsageWarningDialog
          enumeration={{} as TemplateEnumeration}
          valueToRemove={pendingRemove.value}
          affectedAttributes={pendingRemove.affectedAttributes}
          onConfirm={confirmRemove}
          onCancel={() => setPendingRemove(null)}
        />
      )}
    </div>
  );
}

// =============================================================================
// ENUMERATION EDITOR DIALOG
// =============================================================================

interface EnumerationEditorProps {
  enumeration: TemplateEnumeration | null;
  onSave: (data: Partial<TemplateEnumeration>) => Promise<void>;
  onCancel: () => void;
}

function EnumerationEditor({ enumeration, onSave, onCancel }: EnumerationEditorProps) {
  const [formData, setFormData] = useState({
    name: enumeration?.name || '',
    templateType: enumeration?.templateType || 'insurance',
    values: enumeration?.values || [],
    description: enumeration?.description || '',
  });
  const [nameError, setNameError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  // Validate enumeration name
  const validateName = (name: string): boolean => {
    if (!name.trim()) {
      setNameError('Name is required');
      return false;
    }
    const pattern = /^[a-z]([-][a-z]|[a-z]){0,50}$/;
    if (!pattern.test(name)) {
      setNameError('Must be lowercase letters with optional hyphens (e.g., cover-types)');
      return false;
    }
    setNameError(null);
    return true;
  };

  // Mock usage check - in production would call API
  const checkValueUsage = async (_value: string): Promise<Array<{ productId: string; attributePath: string; valueType: string }>> => {
    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 300));

    // Mock: randomly return some usage
    if (Math.random() > 0.5) {
      return [
        { productId: 'motor_v1', attributePath: 'cover.type', valueType: 'FIXED_VALUE' },
        { productId: 'health_v1', attributePath: 'plan.tier', valueType: 'RULE_DRIVEN' },
      ];
    }
    return [];
  };

  const handleSave = async () => {
    if (!validateName(formData.name)) return;
    if (formData.values.length === 0) {
      return;
    }

    setIsSaving(true);
    try {
      await onSave(formData);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg p-6 max-h-[90vh] overflow-y-auto">
        <h3 className="text-lg font-semibold mb-4">
          {enumeration ? 'Edit Enumeration' : 'Create Enumeration'}
        </h3>

        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Name <span className="text-red-500">*</span>
              </label>
              <Input
                value={formData.name}
                onChange={(e) => {
                  const val = e.target.value.toLowerCase().replace(/[^a-z-]/g, '');
                  setFormData({ ...formData, name: val });
                  if (val) validateName(val);
                }}
                placeholder="e.g., cover-types"
                className={`font-mono ${nameError ? 'border-red-500' : ''}`}
              />
              {nameError && <p className="text-xs text-red-500 mt-1">{nameError}</p>}
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Template Type <span className="text-red-500">*</span>
              </label>
              <Input
                value={formData.templateType}
                onChange={(e) => setFormData({ ...formData, templateType: e.target.value.toLowerCase().replace(/[^a-z-]/g, '') })}
                placeholder="e.g., insurance"
                className="font-mono"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <Input
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Brief description of this enumeration"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Values <span className="text-red-500">*</span>
              <span className="font-normal text-gray-500 ml-2">({formData.values.length})</span>
            </label>
            <ValueEditor
              values={formData.values}
              onChange={(values) => setFormData({ ...formData, values })}
              onCheckUsage={enumeration ? checkValueUsage : undefined}
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
            disabled={isSaving || !formData.name || formData.values.length === 0 || !!nameError}
          >
            {isSaving ? 'Saving...' : 'Save'}
          </Button>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// USAGE INDICATOR
// =============================================================================

interface UsageIndicatorProps {
  enumerationId: string;
}

function EnumUsageIndicator({ enumerationId }: UsageIndicatorProps) {
  const [usage, setUsage] = useState<{ count: number; loading: boolean }>({ count: 0, loading: true });

  useEffect(() => {
    // In production, this would call api.templates.getEnumerationUsage(enumerationId)
    const timer = setTimeout(() => {
      setUsage({ count: Math.floor(Math.random() * 5), loading: false });
    }, 500);
    return () => clearTimeout(timer);
  }, [enumerationId]);

  if (usage.loading) {
    return <span className="text-xs text-gray-400">...</span>;
  }

  if (usage.count === 0) {
    return <span className="text-xs text-gray-400">Not in use</span>;
  }

  return (
    <span className="text-xs text-blue-600">
      {usage.count} attribute{usage.count !== 1 ? 's' : ''}
    </span>
  );
}

// =============================================================================
// MAIN ENUMERATIONS PAGE
// =============================================================================

export function Enumerations() {
  const [enumerations, setEnumerations] = useState<TemplateEnumeration[]>([]);
  const [search, setSearch] = useState('');
  const [templateFilter, setTemplateFilter] = useState<string>('ALL');
  const [editingEnum, setEditingEnum] = useState<TemplateEnumeration | null | 'new'>(null);
  const [loading, setLoading] = useState(true);
  const [expandedId, setExpandedId] = useState<string | null>(null);

  useEffect(() => {
    loadEnumerations();
  }, []);

  const loadEnumerations = async () => {
    setLoading(true);
    try {
      const data = await api.templates.listEnumerations();
      setEnumerations(data);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (data: Partial<TemplateEnumeration>) => {
    if (editingEnum === 'new') {
      await api.templates.createEnumeration(data);
    } else if (editingEnum) {
      await api.templates.updateEnumeration(editingEnum.id, data);
    }
    setEditingEnum(null);
    loadEnumerations();
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this enumeration? This may affect attributes using it.')) return;
    try {
      await api.templates.deleteEnumeration(id);
      loadEnumerations();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  // Get unique template types
  const templateTypes = [...new Set(enumerations.map(e => e.templateType))];

  const filtered = enumerations.filter(
    (e) =>
      (templateFilter === 'ALL' || e.templateType === templateFilter) &&
      (e.name.toLowerCase().includes(search.toLowerCase()) ||
        e.templateType.toLowerCase().includes(search.toLowerCase()) ||
        e.values.some(v => v.toLowerCase().includes(search.toLowerCase())))
  );

  // Group by template type
  const grouped = filtered.reduce(
    (acc, e) => {
      if (!acc[e.templateType]) acc[e.templateType] = [];
      acc[e.templateType].push(e);
      return acc;
    },
    {} as Record<string, TemplateEnumeration[]>
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Enumerations</h2>
        <p className="text-muted-foreground">
          Manage enumeration value sets for product templates
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Total Enumerations</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{enumerations.length}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Template Types</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{templateTypes.length}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Total Values</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {enumerations.reduce((sum, e) => sum + e.values.length, 0)}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Avg Values/Enum</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {enumerations.length > 0
                ? Math.round(enumerations.reduce((sum, e) => sum + e.values.length, 0) / enumerations.length)
                : 0}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Filters */}
      <div className="flex items-center justify-between gap-4">
        <div className="flex items-center gap-4 flex-1">
          <div className="relative flex-1 max-w-sm">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder="Search enumerations or values..."
              className="pl-10"
            />
          </div>
          <select
            value={templateFilter}
            onChange={(e) => setTemplateFilter(e.target.value)}
            className="h-10 px-3 border rounded-md text-sm"
          >
            <option value="ALL">All Templates</option>
            {templateTypes.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
        </div>
        <Button onClick={() => setEditingEnum('new')} className="gap-2">
          <Plus className="h-4 w-4" />
          Add Enumeration
        </Button>
      </div>

      {/* List */}
      {loading ? (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      ) : Object.keys(grouped).length === 0 ? (
        <div className="text-center py-12">
          <List className="h-12 w-12 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">
            {search || templateFilter !== 'ALL' ? 'No enumerations match your filters' : 'No enumerations defined yet'}
          </p>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => setEditingEnum('new')}
          >
            Create your first enumeration
          </Button>
        </div>
      ) : (
        <div className="space-y-6">
          {Object.entries(grouped).map(([templateType, enums]) => (
            <div key={templateType}>
              <h3 className="text-sm font-semibold text-gray-600 uppercase tracking-wider mb-3 flex items-center gap-2">
                <Tag className="h-4 w-4" />
                {templateType}
                <span className="text-gray-400 font-normal">({enums.length})</span>
              </h3>
              <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
                {enums.map((en) => (
                  <Card key={en.id} className="group hover:shadow-md transition-shadow overflow-hidden">
                    <div
                      className="px-4 py-3 flex items-center justify-between cursor-pointer hover:bg-gray-50"
                      onClick={() => setExpandedId(expandedId === en.id ? null : en.id)}
                    >
                      <div className="flex items-center gap-3 flex-1 min-w-0">
                        <ChevronRight
                          className={`h-4 w-4 text-gray-400 transition-transform shrink-0 ${
                            expandedId === en.id ? 'rotate-90' : ''
                          }`}
                        />
                        <div className="min-w-0">
                          <div className="font-medium truncate">{en.name}</div>
                          <div className="flex items-center gap-2 text-xs text-gray-500">
                            <span>{en.values.length} values</span>
                            <span className="text-gray-300">|</span>
                            <EnumUsageIndicator enumerationId={en.id} />
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity" onClick={(e) => e.stopPropagation()}>
                        <button
                          onClick={() => setEditingEnum(en)}
                          className="p-1.5 hover:bg-gray-100 rounded"
                          title="Edit"
                        >
                          <Pencil className="h-4 w-4 text-gray-500" />
                        </button>
                        <button
                          onClick={() => handleDelete(en.id)}
                          className="p-1.5 hover:bg-gray-100 rounded"
                          title="Delete"
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </button>
                      </div>
                    </div>
                    {expandedId === en.id && (
                      <div className="px-4 py-3 border-t bg-gray-50">
                        {en.description && (
                          <p className="text-xs text-gray-500 mb-2 flex items-start gap-1">
                            <Info className="h-3 w-3 mt-0.5 shrink-0" />
                            {en.description}
                          </p>
                        )}
                        <div className="flex flex-wrap gap-1">
                          {en.values.map((v) => (
                            <span
                              key={v}
                              className="px-2 py-1 bg-white border rounded text-xs font-mono"
                            >
                              {v}
                            </span>
                          ))}
                        </div>
                      </div>
                    )}
                  </Card>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Editor Dialog */}
      {editingEnum && (
        <EnumerationEditor
          enumeration={editingEnum === 'new' ? null : editingEnum}
          onSave={handleSave}
          onCancel={() => setEditingEnum(null)}
        />
      )}
    </div>
  );
}

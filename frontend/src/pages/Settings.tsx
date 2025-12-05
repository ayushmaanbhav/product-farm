// Settings Page - System-wide configuration
// Manages Datatypes, Template Enumerations, and other global settings

import { useState, useEffect } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { api } from '@/services/api';
import type { DataType, TemplateEnumeration, PrimitiveType, DatatypeConstraints } from '@/types';
import { PRIMITIVE_TYPES } from '@/types';
import {
  Database,
  List,
  Plus,
  Pencil,
  Trash2,
  X,
  Search,
  ChevronRight,
  Hash,
  Type,
  ToggleLeft,
  Braces,
  Tag,
  ListOrdered,
  Binary,
  Clock,
} from 'lucide-react';

// =============================================================================
// TABS
// =============================================================================

type TabId = 'datatypes' | 'templates';

const tabs: { id: TabId; label: string; icon: typeof Database }[] = [
  { id: 'datatypes', label: 'Datatypes', icon: Database },
  { id: 'templates', label: 'Template Enumerations', icon: List },
];

// =============================================================================
// DATATYPE ICON HELPER
// =============================================================================

function DatatypeIcon({ primitiveType }: { primitiveType: PrimitiveType }) {
  switch (primitiveType) {
    case 'INT':
    case 'FLOAT':
      return <Hash className="h-4 w-4 text-blue-500" />;
    case 'DECIMAL':
      return <Binary className="h-4 w-4 text-indigo-500" />;
    case 'STRING':
      return <Type className="h-4 w-4 text-emerald-500" />;
    case 'BOOL':
      return <ToggleLeft className="h-4 w-4 text-purple-500" />;
    case 'DATETIME':
      return <Clock className="h-4 w-4 text-orange-500" />;
    case 'ENUM':
      return <ListOrdered className="h-4 w-4 text-pink-500" />;
    case 'ARRAY':
      return <List className="h-4 w-4 text-cyan-500" />;
    case 'OBJECT':
      return <Braces className="h-4 w-4 text-amber-500" />;
    default:
      return <Braces className="h-4 w-4 text-gray-500" />;
  }
}

// =============================================================================
// DATATYPE EDITOR DIALOG
// =============================================================================

interface DatatypeEditorProps {
  datatype: DataType | null;
  onSave: (data: Partial<DataType>) => Promise<void>;
  onCancel: () => void;
}

function DatatypeEditor({ datatype, onSave, onCancel }: DatatypeEditorProps) {
  const [formData, setFormData] = useState({
    id: datatype?.id || '',
    name: datatype?.name || '',
    primitiveType: datatype?.primitiveType || 'STRING' as PrimitiveType,
    constraints: datatype?.constraints || {} as DatatypeConstraints,
    description: datatype?.description || '',
  });
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // Update constraint field
  const updateConstraint = (key: keyof DatatypeConstraints, value: number | string | boolean | undefined) => {
    const newConstraints = { ...formData.constraints };
    if (value === undefined || value === '' || (typeof value === 'number' && isNaN(value))) {
      delete newConstraints[key];
    } else {
      (newConstraints as Record<string, unknown>)[key] = value;
    }
    setFormData({ ...formData, constraints: newConstraints });
  };

  // Reset constraints when primitive type changes
  const handlePrimitiveTypeChange = (newType: PrimitiveType) => {
    setFormData({
      ...formData,
      primitiveType: newType,
      constraints: {}, // Reset constraints on type change
    });
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      await onSave(formData);
    } finally {
      setIsSaving(false);
    }
  };

  // Render constraint fields based on primitive type
  const renderConstraintFields = () => {
    const { primitiveType, constraints } = formData;

    switch (primitiveType) {
      case 'INT':
      case 'FLOAT':
        return (
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">Min Value</label>
              <Input
                type="number"
                value={constraints.min ?? ''}
                onChange={(e) => updateConstraint('min', e.target.value ? Number(e.target.value) : undefined)}
                placeholder="No minimum"
                className="h-9"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">Max Value</label>
              <Input
                type="number"
                value={constraints.max ?? ''}
                onChange={(e) => updateConstraint('max', e.target.value ? Number(e.target.value) : undefined)}
                placeholder="No maximum"
                className="h-9"
              />
            </div>
          </div>
        );

      case 'DECIMAL':
        return (
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Precision (total digits)</label>
                <Input
                  type="number"
                  min={1}
                  max={38}
                  value={constraints.precision ?? ''}
                  onChange={(e) => updateConstraint('precision', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="e.g., 10"
                  className="h-9"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Scale (decimal places)</label>
                <Input
                  type="number"
                  min={0}
                  max={38}
                  value={constraints.scale ?? ''}
                  onChange={(e) => updateConstraint('scale', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="e.g., 2"
                  className="h-9"
                />
              </div>
            </div>
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Min Value</label>
                <Input
                  type="number"
                  step="any"
                  value={constraints.min ?? ''}
                  onChange={(e) => updateConstraint('min', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No minimum"
                  className="h-9"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Max Value</label>
                <Input
                  type="number"
                  step="any"
                  value={constraints.max ?? ''}
                  onChange={(e) => updateConstraint('max', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No maximum"
                  className="h-9"
                />
              </div>
            </div>
          </div>
        );

      case 'STRING':
        return (
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Min Length</label>
                <Input
                  type="number"
                  min={0}
                  value={constraints.minLength ?? ''}
                  onChange={(e) => updateConstraint('minLength', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No minimum"
                  className="h-9"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Max Length</label>
                <Input
                  type="number"
                  min={0}
                  value={constraints.maxLength ?? ''}
                  onChange={(e) => updateConstraint('maxLength', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No maximum"
                  className="h-9"
                />
              </div>
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">Pattern (regex)</label>
              <Input
                value={constraints.pattern ?? ''}
                onChange={(e) => updateConstraint('pattern', e.target.value || undefined)}
                placeholder="e.g., ^[A-Z]{2}[0-9]+$"
                className="h-9 font-mono text-xs"
              />
            </div>
          </div>
        );

      case 'ARRAY':
        return (
          <div className="space-y-3">
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Min Items</label>
                <Input
                  type="number"
                  min={0}
                  value={constraints.minItems ?? ''}
                  onChange={(e) => updateConstraint('minItems', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No minimum"
                  className="h-9"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 mb-1">Max Items</label>
                <Input
                  type="number"
                  min={0}
                  value={constraints.maxItems ?? ''}
                  onChange={(e) => updateConstraint('maxItems', e.target.value ? Number(e.target.value) : undefined)}
                  placeholder="No maximum"
                  className="h-9"
                />
              </div>
            </div>
            <div className="flex items-center gap-2">
              <input
                type="checkbox"
                id="uniqueItems"
                checked={constraints.uniqueItems ?? false}
                onChange={(e) => updateConstraint('uniqueItems', e.target.checked || undefined)}
                className="rounded border-gray-300"
              />
              <label htmlFor="uniqueItems" className="text-xs text-gray-600">
                Require unique items
              </label>
            </div>
          </div>
        );

      case 'BOOL':
      case 'DATETIME':
      case 'ENUM':
      case 'OBJECT':
      default:
        return (
          <p className="text-xs text-gray-500 italic">
            No additional constraints for {primitiveType} type
          </p>
        );
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg p-6 max-h-[90vh] overflow-y-auto">
        <h3 className="text-lg font-semibold mb-4">
          {datatype ? 'Edit Datatype' : 'Create Datatype'}
        </h3>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">ID</label>
            <Input
              value={formData.id}
              onChange={(e) => setFormData({ ...formData, id: e.target.value.toLowerCase().replace(/[^a-z-]/g, '') })}
              placeholder="e.g., currency, percentage"
              disabled={!!datatype}
              className="font-mono"
            />
            <p className="text-xs text-gray-500 mt-1">Lowercase letters and hyphens only</p>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Display Name</label>
            <Input
              value={formData.name}
              onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              placeholder="e.g., Currency Amount"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Primitive Type</label>
            <select
              value={formData.primitiveType}
              onChange={(e) => handlePrimitiveTypeChange(e.target.value as PrimitiveType)}
              className="w-full h-10 px-3 border rounded-md text-sm"
            >
              {PRIMITIVE_TYPES.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>

          {/* Constraint Fields */}
          <div className="p-3 bg-gray-50 rounded-md">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Constraints
            </label>
            {renderConstraintFields()}

            {/* JSON Preview Toggle */}
            <button
              type="button"
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="mt-3 text-xs text-blue-600 hover:text-blue-800"
            >
              {showAdvanced ? 'Hide' : 'Show'} JSON preview
            </button>
            {showAdvanced && (
              <pre className="mt-2 p-2 bg-gray-100 rounded text-xs font-mono overflow-x-auto">
                {JSON.stringify(formData.constraints, null, 2) || '{}'}
              </pre>
            )}
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">Description</label>
            <Input
              value={formData.description}
              onChange={(e) => setFormData({ ...formData, description: e.target.value })}
              placeholder="Brief description"
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
            disabled={isSaving || !formData.id || !formData.name}
          >
            {isSaving ? 'Saving...' : 'Save'}
          </Button>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// DATATYPES TAB
// =============================================================================

function DatatypesTab() {
  const [datatypes, setDatatypes] = useState<DataType[]>([]);
  const [search, setSearch] = useState('');
  const [editingDatatype, setEditingDatatype] = useState<DataType | null | 'new'>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadDatatypes();
  }, []);

  const loadDatatypes = async () => {
    setLoading(true);
    try {
      const data = await api.datatypes.list();
      setDatatypes(data);
    } finally {
      setLoading(false);
    }
  };

  const handleSave = async (data: Partial<DataType>) => {
    if (editingDatatype === 'new') {
      await api.datatypes.create(data);
    } else if (editingDatatype) {
      await api.datatypes.update(editingDatatype.id, data);
    }
    setEditingDatatype(null);
    loadDatatypes();
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this datatype?')) return;
    try {
      await api.datatypes.delete(id);
      loadDatatypes();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  const filtered = datatypes.filter(
    (d) =>
      d.id.toLowerCase().includes(search.toLowerCase()) ||
      d.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search datatypes..."
            className="pl-10"
          />
        </div>
        <Button onClick={() => setEditingDatatype('new')} className="gap-2">
          <Plus className="h-4 w-4" />
          Add Datatype
        </Button>
      </div>

      {/* List */}
      {loading ? (
        <div className="text-center py-8 text-gray-500">Loading...</div>
      ) : filtered.length === 0 ? (
        <div className="text-center py-8 text-gray-500">
          {search ? 'No datatypes match your search' : 'No datatypes defined'}
        </div>
      ) : (
        <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
          {filtered.map((dt) => (
            <Card key={dt.id} className="group">
              <CardHeader className="pb-2">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    <DatatypeIcon primitiveType={dt.primitiveType} />
                    <CardTitle className="text-base">{dt.name}</CardTitle>
                  </div>
                  <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                    <button
                      onClick={() => setEditingDatatype(dt)}
                      className="p-1 hover:bg-gray-100 rounded"
                    >
                      <Pencil className="h-4 w-4 text-gray-500" />
                    </button>
                    <button
                      onClick={() => handleDelete(dt.id)}
                      className="p-1 hover:bg-gray-100 rounded"
                    >
                      <Trash2 className="h-4 w-4 text-red-500" />
                    </button>
                  </div>
                </div>
                <CardDescription className="text-xs">{dt.id}</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center gap-2 text-sm">
                    <span className="text-gray-500">Type:</span>
                    <span className="px-2 py-0.5 bg-gray-100 rounded text-xs font-medium">
                      {dt.primitiveType}
                    </span>
                  </div>
                  {dt.description && (
                    <p className="text-xs text-gray-500">{dt.description}</p>
                  )}
                  {dt.constraints && Object.keys(dt.constraints).length > 0 && (
                    <div className="text-xs font-mono bg-gray-50 p-2 rounded overflow-x-auto">
                      {JSON.stringify(dt.constraints, null, 2)}
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Editor Dialog */}
      {editingDatatype && (
        <DatatypeEditor
          datatype={editingDatatype === 'new' ? null : editingDatatype}
          onSave={handleSave}
          onCancel={() => setEditingDatatype(null)}
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
  const [newValue, setNewValue] = useState('');
  const [valueError, setValueError] = useState<string | null>(null);
  const [nameError, setNameError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [draggedIndex, setDraggedIndex] = useState<number | null>(null);

  // Validate enumeration name (same pattern as enumerationName)
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
    if (formData.values.includes(value)) {
      setValueError('Value already exists');
      return false;
    }
    setValueError(null);
    return true;
  };

  const handleAddValue = () => {
    const trimmed = newValue.trim().toLowerCase();
    if (validateValue(trimmed)) {
      setFormData({ ...formData, values: [...formData.values, trimmed] });
      setNewValue('');
      setValueError(null);
    }
  };

  const handleRemoveValue = (value: string) => {
    setFormData({ ...formData, values: formData.values.filter((v) => v !== value) });
  };

  const handleDragStart = (index: number) => {
    setDraggedIndex(index);
  };

  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === index) return;

    const newValues = [...formData.values];
    const draggedValue = newValues[draggedIndex];
    newValues.splice(draggedIndex, 1);
    newValues.splice(index, 0, draggedValue);
    setFormData({ ...formData, values: newValues });
    setDraggedIndex(index);
  };

  const handleDragEnd = () => {
    setDraggedIndex(null);
  };

  const moveValue = (index: number, direction: 'up' | 'down') => {
    const newIndex = direction === 'up' ? index - 1 : index + 1;
    if (newIndex < 0 || newIndex >= formData.values.length) return;

    const newValues = [...formData.values];
    [newValues[index], newValues[newIndex]] = [newValues[newIndex], newValues[index]];
    setFormData({ ...formData, values: newValues });
  };

  const handleSave = async () => {
    if (!validateName(formData.name)) return;
    if (formData.values.length === 0) {
      setValueError('At least one value is required');
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
            <p className="text-xs text-gray-500 mt-1">Lowercase letters and hyphens only</p>
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
            <div className="flex gap-2 mb-2">
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
            {valueError && <p className="text-xs text-red-500 mb-2">{valueError}</p>}

            {/* Values List with reordering */}
            <div className="border rounded-md divide-y max-h-48 overflow-y-auto">
              {formData.values.length === 0 ? (
                <div className="p-4 text-sm text-gray-400 text-center">
                  No values added yet. Add at least one value.
                </div>
              ) : (
                formData.values.map((value, index) => (
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
                    <span className="cursor-grab text-gray-400 hover:text-gray-600">⋮⋮</span>
                    <span className="flex-1 font-mono">{value}</span>
                    <div className="flex items-center gap-1">
                      <button
                        onClick={() => moveValue(index, 'up')}
                        disabled={index === 0}
                        className="p-1 hover:bg-gray-200 rounded disabled:opacity-30"
                        title="Move up"
                      >
                        ↑
                      </button>
                      <button
                        onClick={() => moveValue(index, 'down')}
                        disabled={index === formData.values.length - 1}
                        className="p-1 hover:bg-gray-200 rounded disabled:opacity-30"
                        title="Move down"
                      >
                        ↓
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
// TEMPLATES TAB
// =============================================================================

function TemplatesTab() {
  const [enumerations, setEnumerations] = useState<TemplateEnumeration[]>([]);
  const [search, setSearch] = useState('');
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
    if (!confirm('Are you sure you want to delete this enumeration?')) return;
    try {
      await api.templates.deleteEnumeration(id);
      loadEnumerations();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  const filtered = enumerations.filter(
    (e) =>
      e.name.toLowerCase().includes(search.toLowerCase()) ||
      e.templateType.toLowerCase().includes(search.toLowerCase())
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
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search enumerations..."
            className="pl-10"
          />
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
        <div className="text-center py-8 text-gray-500">
          {search ? 'No enumerations match your search' : 'No enumerations defined'}
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
              <div className="space-y-2">
                {enums.map((en) => (
                  <Card key={en.id} className="overflow-hidden">
                    <div
                      className="px-4 py-3 flex items-center justify-between cursor-pointer hover:bg-gray-50"
                      onClick={() => setExpandedId(expandedId === en.id ? null : en.id)}
                    >
                      <div className="flex items-center gap-3">
                        <ChevronRight
                          className={`h-4 w-4 text-gray-400 transition-transform ${
                            expandedId === en.id ? 'rotate-90' : ''
                          }`}
                        />
                        <div>
                          <div className="font-medium">{en.name}</div>
                          <div className="text-xs text-gray-500">
                            {en.values.length} values
                            {en.description && ` - ${en.description}`}
                          </div>
                        </div>
                      </div>
                      <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
                        <button
                          onClick={() => setEditingEnum(en)}
                          className="p-1.5 hover:bg-gray-100 rounded"
                        >
                          <Pencil className="h-4 w-4 text-gray-500" />
                        </button>
                        <button
                          onClick={() => handleDelete(en.id)}
                          className="p-1.5 hover:bg-gray-100 rounded"
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </button>
                      </div>
                    </div>
                    {expandedId === en.id && (
                      <div className="px-4 py-3 border-t bg-gray-50">
                        <div className="flex flex-wrap gap-2">
                          {en.values.map((v) => (
                            <span
                              key={v}
                              className="px-2 py-1 bg-white border rounded text-sm"
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

// =============================================================================
// MAIN SETTINGS PAGE
// =============================================================================

export function Settings() {
  const [activeTab, setActiveTab] = useState<TabId>('datatypes');

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Settings</h2>
        <p className="text-muted-foreground">
          Manage system-wide configuration and data types
        </p>
      </div>

      {/* Tabs */}
      <div className="flex gap-1 p-1 bg-gray-100 rounded-lg w-fit">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex items-center gap-2 px-4 py-2 rounded-md text-sm font-medium transition-colors ${
              activeTab === tab.id
                ? 'bg-white shadow text-gray-900'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <tab.icon className="h-4 w-4" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab Content */}
      <div className="mt-6">
        {activeTab === 'datatypes' && <DatatypesTab />}
        {activeTab === 'templates' && <TemplatesTab />}
      </div>
    </div>
  );
}

// Datatypes Page - Manage data types with constraint rules
// Enhanced version with constraint rule builder and usage tracking

import { useState, useEffect, useCallback } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { api } from '@/services/api';
import type { DataType, PrimitiveType, DatatypeConstraints } from '@/types';
import { PRIMITIVE_TYPES } from '@/types';
import {
  Database,
  Plus,
  Pencil,
  Trash2,
  Search,
  Hash,
  Type,
  ToggleLeft,
  Braces,
  List,
  ListOrdered,
  Binary,
  Clock,
  CheckCircle2,
  XCircle,
  Code,
  Play,
} from 'lucide-react';

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
// CONSTRAINT RULE BUILDER
// =============================================================================

interface ConstraintRuleBuilderProps {
  expression: string;
  errorMessage: string;
  onChange: (expression: string, errorMessage: string) => void;
}

function ConstraintRuleBuilder({ expression, errorMessage, onChange }: ConstraintRuleBuilderProps) {
  const [testValue, setTestValue] = useState('');
  const [testResult, setTestResult] = useState<{ valid: boolean; error?: string } | null>(null);
  const [showHelp, setShowHelp] = useState(false);

  // Test the constraint expression
  const handleTest = useCallback(() => {
    if (!expression || !testValue) {
      setTestResult(null);
      return;
    }

    try {
      // Parse the JSON Logic expression
      const parsedExpr = JSON.parse(expression);

      // Simple evaluation for common patterns
      // In production, this would call the backend validation service
      let value: unknown;
      try {
        value = JSON.parse(testValue);
      } catch {
        value = testValue;
      }

      // Build context
      const context = { $value: value };

      // Basic JSON Logic evaluation (simplified)
      const result = evaluateJsonLogic(parsedExpr, context);
      setTestResult({ valid: !!result });
    } catch (e) {
      setTestResult({ valid: false, error: (e as Error).message });
    }
  }, [expression, testValue]);

  // Simplified JSON Logic evaluator for testing
  const evaluateJsonLogic = (logic: unknown, data: Record<string, unknown>): unknown => {
    if (logic === null || logic === undefined) return logic;
    if (typeof logic !== 'object') return logic;
    if (Array.isArray(logic)) return logic.map(item => evaluateJsonLogic(item, data));

    const operators = Object.keys(logic as object);
    if (operators.length !== 1) return logic;

    const operator = operators[0];
    const values = (logic as Record<string, unknown>)[operator];

    switch (operator) {
      case 'var': {
        const path = values as string;
        return path.split('.').reduce((obj: unknown, key) =>
          (obj as Record<string, unknown>)?.[key], data);
      }
      case '>': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return Number(a) > Number(b);
      }
      case '>=': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return Number(a) >= Number(b);
      }
      case '<': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return Number(a) < Number(b);
      }
      case '<=': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return Number(a) <= Number(b);
      }
      case '==': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return a == b;
      }
      case '===': {
        const [a, b] = (values as unknown[]).map(v => evaluateJsonLogic(v, data));
        return a === b;
      }
      case 'and': {
        return (values as unknown[]).every(v => evaluateJsonLogic(v, data));
      }
      case 'or': {
        return (values as unknown[]).some(v => evaluateJsonLogic(v, data));
      }
      case '!': {
        return !evaluateJsonLogic(values, data);
      }
      default:
        return logic;
    }
  };

  return (
    <div className="space-y-3 p-3 border rounded-lg bg-blue-50/50">
      <div className="flex items-center justify-between">
        <label className="flex items-center gap-2 text-sm font-medium text-gray-700">
          <Code className="h-4 w-4 text-blue-600" />
          Constraint Rule (JSON Logic)
        </label>
        <button
          type="button"
          onClick={() => setShowHelp(!showHelp)}
          className="text-xs text-blue-600 hover:text-blue-800"
        >
          {showHelp ? 'Hide help' : 'Show help'}
        </button>
      </div>

      {showHelp && (
        <div className="p-3 bg-blue-100/50 rounded-md text-xs space-y-2">
          <p className="font-medium">Available Variables:</p>
          <ul className="list-disc list-inside space-y-1 text-gray-600">
            <li><code className="bg-white px-1 rounded">$value</code> - The value being validated</li>
            <li>Other attribute paths for cross-field validation</li>
          </ul>
          <p className="font-medium mt-2">Example Expressions:</p>
          <div className="space-y-1 text-gray-600">
            <p><code className="bg-white px-1 rounded">{`{">=": [{"var": "$value"}, 0]}`}</code> - Value must be &gt;= 0</p>
            <p><code className="bg-white px-1 rounded">{`{"and": [{">=": [{"var": "$value"}, 18]}, {"<=": [{"var": "$value"}, 65]}]}`}</code> - Value between 18 and 65</p>
          </div>
        </div>
      )}

      <div>
        <textarea
          value={expression}
          onChange={(e) => onChange(e.target.value, errorMessage)}
          placeholder='{">=": [{"var": "$value"}, 0]}'
          className="w-full h-24 px-3 py-2 border rounded-md text-sm font-mono bg-white"
        />
      </div>

      <div>
        <label className="block text-xs font-medium text-gray-600 mb-1">Error Message</label>
        <Input
          value={errorMessage}
          onChange={(e) => onChange(expression, e.target.value)}
          placeholder="Value must meet the constraint"
          className="h-9"
        />
      </div>

      {/* Test Section */}
      <div className="pt-2 border-t">
        <label className="block text-xs font-medium text-gray-600 mb-1">Test Expression</label>
        <div className="flex gap-2">
          <Input
            value={testValue}
            onChange={(e) => setTestValue(e.target.value)}
            placeholder="Enter test value..."
            className="h-9 flex-1"
          />
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={handleTest}
            disabled={!expression || !testValue}
            className="gap-1"
          >
            <Play className="h-3 w-3" />
            Test
          </Button>
        </div>
        {testResult && (
          <div className={`mt-2 flex items-center gap-2 text-sm ${testResult.valid ? 'text-green-600' : 'text-red-600'}`}>
            {testResult.valid ? (
              <><CheckCircle2 className="h-4 w-4" /> Valid</>
            ) : (
              <><XCircle className="h-4 w-4" /> Invalid{testResult.error && `: ${testResult.error}`}</>
            )}
          </div>
        )}
      </div>
    </div>
  );
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
  const [showConstraintRule, setShowConstraintRule] = useState(
    !!(datatype?.constraints?.constraintRuleExpression)
  );
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
    setShowConstraintRule(false);
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
              <p className="text-xs text-gray-500 mt-1">Regular expression pattern for validation</p>
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
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-2xl p-6 max-h-[90vh] overflow-y-auto">
        <h3 className="text-lg font-semibold mb-4">
          {datatype ? 'Edit Datatype' : 'Create Datatype'}
        </h3>

        <div className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
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

          {/* Basic Constraint Fields */}
          <div className="p-3 bg-gray-50 rounded-md">
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Basic Constraints
            </label>
            {renderConstraintFields()}
          </div>

          {/* Constraint Rule Section */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="block text-sm font-medium text-gray-700">
                Advanced Constraint Rule
              </label>
              <button
                type="button"
                onClick={() => {
                  setShowConstraintRule(!showConstraintRule);
                  if (showConstraintRule) {
                    // Clear constraint rule when hiding
                    updateConstraint('constraintRuleExpression', undefined);
                    updateConstraint('constraintErrorMessage', undefined);
                  }
                }}
                className="text-xs text-blue-600 hover:text-blue-800"
              >
                {showConstraintRule ? 'Remove' : 'Add constraint rule'}
              </button>
            </div>
            {showConstraintRule && (
              <ConstraintRuleBuilder
                expression={formData.constraints.constraintRuleExpression || ''}
                errorMessage={formData.constraints.constraintErrorMessage || ''}
                onChange={(expr, msg) => {
                  const newConstraints = { ...formData.constraints };
                  if (expr) {
                    newConstraints.constraintRuleExpression = expr;
                  } else {
                    delete newConstraints.constraintRuleExpression;
                  }
                  if (msg) {
                    newConstraints.constraintErrorMessage = msg;
                  } else {
                    delete newConstraints.constraintErrorMessage;
                  }
                  setFormData({ ...formData, constraints: newConstraints });
                }}
              />
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

          {/* JSON Preview Toggle */}
          <button
            type="button"
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="text-xs text-gray-500 hover:text-gray-700"
          >
            {showAdvanced ? 'Hide' : 'Show'} JSON preview
          </button>
          {showAdvanced && (
            <pre className="p-3 bg-gray-100 rounded text-xs font-mono overflow-x-auto">
              {JSON.stringify({ ...formData, constraints: formData.constraints }, null, 2)}
            </pre>
          )}
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
// USAGE INDICATOR
// =============================================================================

interface UsageIndicatorProps {
  datatypeId: string;
}

function UsageIndicator({ datatypeId }: UsageIndicatorProps) {
  const [usage, setUsage] = useState<{ count: number; loading: boolean }>({ count: 0, loading: true });

  useEffect(() => {
    // In production, this would call api.datatypes.getUsage(datatypeId)
    // For now, simulate with mock data
    const timer = setTimeout(() => {
      setUsage({ count: Math.floor(Math.random() * 10), loading: false });
    }, 500);
    return () => clearTimeout(timer);
  }, [datatypeId]);

  if (usage.loading) {
    return <span className="text-xs text-gray-400">...</span>;
  }

  if (usage.count === 0) {
    return <span className="text-xs text-gray-400">Not in use</span>;
  }

  return (
    <span className="text-xs text-blue-600">
      Used by {usage.count} attribute{usage.count !== 1 ? 's' : ''}
    </span>
  );
}

// =============================================================================
// MAIN DATATYPES PAGE
// =============================================================================

export function Datatypes() {
  const [datatypes, setDatatypes] = useState<DataType[]>([]);
  const [search, setSearch] = useState('');
  const [typeFilter, setTypeFilter] = useState<PrimitiveType | 'ALL'>('ALL');
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
    if (!confirm('Are you sure you want to delete this datatype? This may affect attributes using it.')) return;
    try {
      await api.datatypes.delete(id);
      loadDatatypes();
    } catch (e) {
      alert((e as Error).message);
    }
  };

  const filtered = datatypes.filter(
    (d) =>
      (typeFilter === 'ALL' || d.primitiveType === typeFilter) &&
      (d.id.toLowerCase().includes(search.toLowerCase()) ||
        d.name.toLowerCase().includes(search.toLowerCase()))
  );

  // Group by primitive type
  const grouped = filtered.reduce(
    (acc, d) => {
      if (!acc[d.primitiveType]) acc[d.primitiveType] = [];
      acc[d.primitiveType].push(d);
      return acc;
    },
    {} as Record<string, DataType[]>
  );

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Datatypes</h2>
        <p className="text-muted-foreground">
          Define data types with validation constraints and rules
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Total Types</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{datatypes.length}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">With Constraints</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {datatypes.filter(d => d.constraints && Object.keys(d.constraints).length > 0).length}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">With Rules</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">
              {datatypes.filter(d => d.constraints?.constraintRuleExpression).length}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-gray-500">Primitive Types</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-2xl font-bold">{Object.keys(grouped).length}</p>
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
              placeholder="Search datatypes..."
              className="pl-10"
            />
          </div>
          <select
            value={typeFilter}
            onChange={(e) => setTypeFilter(e.target.value as PrimitiveType | 'ALL')}
            className="h-10 px-3 border rounded-md text-sm"
          >
            <option value="ALL">All Types</option>
            {PRIMITIVE_TYPES.map((t) => (
              <option key={t} value={t}>{t}</option>
            ))}
          </select>
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
        <div className="text-center py-12">
          <Database className="h-12 w-12 text-gray-300 mx-auto mb-4" />
          <p className="text-gray-500">
            {search || typeFilter !== 'ALL' ? 'No datatypes match your filters' : 'No datatypes defined yet'}
          </p>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => setEditingDatatype('new')}
          >
            Create your first datatype
          </Button>
        </div>
      ) : (
        <div className="grid gap-3 md:grid-cols-2 lg:grid-cols-3">
          {filtered.map((dt) => (
            <Card key={dt.id} className="group hover:shadow-md transition-shadow">
              <CardHeader className="pb-2">
                <div className="flex items-start justify-between">
                  <div className="flex items-center gap-2">
                    <DatatypeIcon primitiveType={dt.primitiveType} />
                    <div>
                      <CardTitle className="text-base">{dt.name}</CardTitle>
                      <CardDescription className="text-xs font-mono">{dt.id}</CardDescription>
                    </div>
                  </div>
                  <div className="opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
                    <button
                      onClick={() => setEditingDatatype(dt)}
                      className="p-1.5 hover:bg-gray-100 rounded"
                      title="Edit"
                    >
                      <Pencil className="h-4 w-4 text-gray-500" />
                    </button>
                    <button
                      onClick={() => handleDelete(dt.id)}
                      className="p-1.5 hover:bg-gray-100 rounded"
                      title="Delete"
                    >
                      <Trash2 className="h-4 w-4 text-red-500" />
                    </button>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="px-2 py-0.5 bg-gray-100 rounded text-xs font-medium">
                      {dt.primitiveType}
                    </span>
                    <UsageIndicator datatypeId={dt.id} />
                  </div>

                  {dt.description && (
                    <p className="text-xs text-gray-500 line-clamp-2">{dt.description}</p>
                  )}

                  {dt.constraints && Object.keys(dt.constraints).length > 0 && (
                    <div className="flex flex-wrap gap-1">
                      {dt.constraints.min !== undefined && (
                        <span className="px-1.5 py-0.5 bg-blue-50 text-blue-700 rounded text-xs">min: {dt.constraints.min}</span>
                      )}
                      {dt.constraints.max !== undefined && (
                        <span className="px-1.5 py-0.5 bg-blue-50 text-blue-700 rounded text-xs">max: {dt.constraints.max}</span>
                      )}
                      {dt.constraints.pattern && (
                        <span className="px-1.5 py-0.5 bg-purple-50 text-purple-700 rounded text-xs font-mono">regex</span>
                      )}
                      {dt.constraints.constraintRuleExpression && (
                        <span className="px-1.5 py-0.5 bg-amber-50 text-amber-700 rounded text-xs flex items-center gap-1">
                          <Code className="h-3 w-3" /> rule
                        </span>
                      )}
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

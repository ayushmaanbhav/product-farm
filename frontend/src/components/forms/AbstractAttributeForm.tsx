// AbstractAttributeForm - Complete form for creating/editing abstract attributes
// Includes all fields: basic info, type config, classification, relationships, display names

import { useState, useCallback, useEffect, useMemo } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { ValidatedInput } from './ValidatedInput';
import { SelectDropdown, type SelectOption } from './SelectDropdown';
import { TagMultiSelect, type TagItem } from './TagMultiSelect';
import { cn } from '@/lib/utils';
import {
  X,
  Plus,
  Trash2,
  ChevronDown,
  ChevronUp,
  Lock,
  AlertTriangle,
  GripVertical,
  Info,
} from 'lucide-react';
import type {
  AbstractAttribute,
  DataType,
  TemplateEnumeration,
  DisplayName,
  DisplayNameFormat,
  RelatedAttribute,
  AttributeRelationshipType,
} from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface AbstractAttributeFormProps {
  /** Product ID for this attribute */
  productId: string;
  /** Existing attribute for editing, null for create mode */
  attribute: AbstractAttribute | null;
  /** Available datatypes */
  datatypes: DataType[];
  /** Available enumerations (filtered by template type) */
  enumerations: TemplateEnumeration[];
  /** Existing component types for suggestion */
  existingComponentTypes: string[];
  /** Existing tags for suggestion */
  existingTags: string[];
  /** Other abstract attributes for relationship selection */
  existingAttributes: AbstractAttribute[];
  /** Callback when form is saved */
  onSave: (data: Partial<AbstractAttribute>) => Promise<void>;
  /** Callback when form is cancelled */
  onCancel: () => void;
}

interface FormData {
  componentType: string;
  componentId: string;
  attributeName: string;
  description: string;
  datatypeId: string;
  enumName: string;
  constraintExpression: string;
  tags: TagItem[];
  displayNames: DisplayName[];
  relatedAttributes: RelatedAttribute[];
  immutable: boolean;
}

// =============================================================================
// CONSTANTS
// =============================================================================

const DISPLAY_FORMATS: DisplayNameFormat[] = ['SYSTEM', 'HUMAN', 'ORIGINAL'];
const RELATIONSHIP_TYPES: AttributeRelationshipType[] = ['ENUMERATION', 'KEY_ENUMERATION', 'VALUE_ENUMERATION'];

// =============================================================================
// HELPER COMPONENTS
// =============================================================================

interface DisplayNameEditorProps {
  displayNames: DisplayName[];
  onChange: (names: DisplayName[]) => void;
  disabled?: boolean;
}

function DisplayNameEditor({ displayNames, onChange, disabled }: DisplayNameEditorProps) {
  const addDisplayName = () => {
    const newName: DisplayName = {
      name: '',
      format: 'HUMAN',
      orderIndex: displayNames.length,
    };
    onChange([...displayNames, newName]);
  };

  const updateDisplayName = (index: number, updates: Partial<DisplayName>) => {
    const updated = [...displayNames];
    updated[index] = { ...updated[index], ...updates };
    onChange(updated);
  };

  const removeDisplayName = (index: number) => {
    const updated = displayNames
      .filter((_, i) => i !== index)
      .map((dn, i) => ({ ...dn, orderIndex: i }));
    onChange(updated);
  };

  const moveDisplayName = (index: number, direction: 'up' | 'down') => {
    if (
      (direction === 'up' && index === 0) ||
      (direction === 'down' && index === displayNames.length - 1)
    ) {
      return;
    }

    const updated = [...displayNames];
    const targetIndex = direction === 'up' ? index - 1 : index + 1;
    [updated[index], updated[targetIndex]] = [updated[targetIndex], updated[index]];
    onChange(updated.map((dn, i) => ({ ...dn, orderIndex: i })));
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-700">Display Names</label>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={addDisplayName}
          disabled={disabled}
          className="h-7 text-xs"
        >
          <Plus className="h-3 w-3 mr-1" />
          Add
        </Button>
      </div>

      {displayNames.length === 0 ? (
        <p className="text-sm text-gray-500 italic">No display names configured</p>
      ) : (
        <div className="space-y-2">
          {displayNames.map((dn, index) => (
            <div
              key={index}
              className="flex items-center gap-2 p-2 bg-gray-50 rounded-md border"
            >
              <GripVertical className="h-4 w-4 text-gray-400 cursor-grab" />

              <Input
                value={dn.name}
                onChange={(e) => updateDisplayName(index, { name: e.target.value })}
                placeholder="Display name..."
                disabled={disabled}
                className="flex-1 h-8 text-sm"
              />

              <select
                value={dn.format}
                onChange={(e) => updateDisplayName(index, { format: e.target.value as DisplayNameFormat })}
                disabled={disabled}
                className="h-8 px-2 border rounded text-sm"
              >
                {DISPLAY_FORMATS.map((fmt) => (
                  <option key={fmt} value={fmt}>{fmt}</option>
                ))}
              </select>

              <div className="flex gap-1">
                <button
                  type="button"
                  onClick={() => moveDisplayName(index, 'up')}
                  disabled={disabled || index === 0}
                  className="p-1 hover:bg-gray-200 rounded disabled:opacity-50"
                >
                  <ChevronUp className="h-4 w-4" />
                </button>
                <button
                  type="button"
                  onClick={() => moveDisplayName(index, 'down')}
                  disabled={disabled || index === displayNames.length - 1}
                  className="p-1 hover:bg-gray-200 rounded disabled:opacity-50"
                >
                  <ChevronDown className="h-4 w-4" />
                </button>
                <button
                  type="button"
                  onClick={() => removeDisplayName(index)}
                  disabled={disabled}
                  className="p-1 hover:bg-red-100 rounded text-red-500"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

interface RelatedAttributeEditorProps {
  relatedAttributes: RelatedAttribute[];
  availableAttributes: AbstractAttribute[];
  currentPath?: string;
  onChange: (attrs: RelatedAttribute[]) => void;
  disabled?: boolean;
}

function RelatedAttributeEditor({
  relatedAttributes,
  availableAttributes,
  currentPath,
  onChange,
  disabled,
}: RelatedAttributeEditorProps) {
  const [showAdd, setShowAdd] = useState(false);
  const [newRelated, setNewRelated] = useState<Partial<RelatedAttribute>>({
    relationshipType: 'ENUMERATION',
    relatedPath: '',
  });

  // Filter out already selected and self
  const availableOptions = useMemo(() => {
    const selectedPaths = new Set(relatedAttributes.map((ra) => ra.relatedPath));
    return availableAttributes
      .filter((attr) => attr.abstractPath !== currentPath && !selectedPaths.has(attr.abstractPath))
      .map((attr) => ({
        value: attr.abstractPath,
        label: `${attr.componentType}:${attr.attributeName}`,
        description: attr.description,
      }));
  }, [availableAttributes, relatedAttributes, currentPath]);

  const addRelated = () => {
    if (!newRelated.relatedPath) return;

    const related: RelatedAttribute = {
      relationshipType: newRelated.relationshipType as AttributeRelationshipType,
      relatedPath: newRelated.relatedPath,
      orderIndex: relatedAttributes.length,
    };
    onChange([...relatedAttributes, related]);
    setNewRelated({ relationshipType: 'ENUMERATION', relatedPath: '' });
    setShowAdd(false);
  };

  const removeRelated = (index: number) => {
    const updated = relatedAttributes
      .filter((_, i) => i !== index)
      .map((ra, i) => ({ ...ra, orderIndex: i }));
    onChange(updated);
  };

  const updateRelated = (index: number, updates: Partial<RelatedAttribute>) => {
    const updated = [...relatedAttributes];
    updated[index] = { ...updated[index], ...updates };
    onChange(updated);
  };

  const getAttributeName = (path: string) => {
    const attr = availableAttributes.find((a) => a.abstractPath === path);
    return attr ? `${attr.componentType}:${attr.attributeName}` : path;
  };

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <label className="text-sm font-medium text-gray-700">Related Attributes</label>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => setShowAdd(true)}
          disabled={disabled || availableOptions.length === 0}
          className="h-7 text-xs"
        >
          <Plus className="h-3 w-3 mr-1" />
          Add Relationship
        </Button>
      </div>

      {relatedAttributes.length === 0 && !showAdd ? (
        <p className="text-sm text-gray-500 italic">No related attributes configured</p>
      ) : (
        <div className="space-y-2">
          {relatedAttributes.map((ra, index) => (
            <div
              key={ra.relatedPath}
              className="flex items-center gap-2 p-2 bg-gray-50 rounded-md border"
            >
              <span className="flex-1 text-sm font-medium truncate">
                {getAttributeName(ra.relatedPath)}
              </span>

              <select
                value={ra.relationshipType}
                onChange={(e) => updateRelated(index, { relationshipType: e.target.value as AttributeRelationshipType })}
                disabled={disabled}
                className="h-8 px-2 border rounded text-sm"
              >
                {RELATIONSHIP_TYPES.map((type) => (
                  <option key={type} value={type}>{type.replace(/_/g, ' ')}</option>
                ))}
              </select>

              <button
                type="button"
                onClick={() => removeRelated(index)}
                disabled={disabled}
                className="p-1 hover:bg-red-100 rounded text-red-500"
              >
                <Trash2 className="h-4 w-4" />
              </button>
            </div>
          ))}

          {showAdd && (
            <div className="p-3 bg-blue-50 rounded-md border border-blue-200 space-y-3">
              <SelectDropdown
                options={availableOptions}
                value={newRelated.relatedPath}
                onChange={(val) => setNewRelated({ ...newRelated, relatedPath: val })}
                placeholder="Select attribute..."
                searchable
              />

              <div className="flex items-center gap-2">
                <select
                  value={newRelated.relationshipType}
                  onChange={(e) => setNewRelated({ ...newRelated, relationshipType: e.target.value as AttributeRelationshipType })}
                  className="h-8 px-2 border rounded text-sm flex-1"
                >
                  {RELATIONSHIP_TYPES.map((type) => (
                    <option key={type} value={type}>{type.replace(/_/g, ' ')}</option>
                  ))}
                </select>

                <Button size="sm" onClick={addRelated} disabled={!newRelated.relatedPath}>
                  Add
                </Button>
                <Button size="sm" variant="outline" onClick={() => setShowAdd(false)}>
                  Cancel
                </Button>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function AbstractAttributeForm({
  productId,
  attribute,
  datatypes,
  enumerations,
  existingComponentTypes,
  existingTags,
  existingAttributes,
  onSave,
  onCancel,
}: AbstractAttributeFormProps) {
  const isEditMode = !!attribute;

  // Initialize form data
  const [formData, setFormData] = useState<FormData>(() => {
    if (attribute) {
      return {
        componentType: attribute.componentType,
        componentId: attribute.componentId || '',
        attributeName: attribute.attributeName,
        description: attribute.description,
        datatypeId: attribute.datatypeId,
        enumName: attribute.enumName || '',
        constraintExpression: attribute.constraintExpression || '',
        tags: attribute.tags,
        displayNames: attribute.displayNames,
        relatedAttributes: attribute.relatedAttributes,
        immutable: attribute.immutable,
      };
    }
    return {
      componentType: '',
      componentId: '',
      attributeName: '',
      description: '',
      datatypeId: 'string',
      enumName: '',
      constraintExpression: '',
      tags: [],
      displayNames: [],
      relatedAttributes: [],
      immutable: false,
    };
  });

  const [isSaving, setIsSaving] = useState(false);
  const [validationState, setValidationState] = useState({
    componentType: true,
    attributeName: true,
  });
  const [showConstraintEditor, setShowConstraintEditor] = useState(false);

  // Get selected datatype
  const selectedDatatype = datatypes.find((dt) => dt.id === formData.datatypeId);
  const isEnumType = selectedDatatype?.primitiveType === 'ENUM';

  // Build component type options
  const componentTypeOptions: SelectOption[] = useMemo(() => {
    const types = new Set(existingComponentTypes);
    return Array.from(types).map((type) => ({
      value: type,
      label: type.charAt(0).toUpperCase() + type.slice(1),
    }));
  }, [existingComponentTypes]);

  // Build datatype options grouped by primitive type
  const datatypeOptions: SelectOption[] = useMemo(() => {
    return datatypes.map((dt) => ({
      value: dt.id,
      label: dt.name,
      description: dt.description,
      group: dt.primitiveType,
    }));
  }, [datatypes]);

  // Build enumeration options
  const enumerationOptions: SelectOption[] = useMemo(() => {
    return enumerations.map((en) => ({
      value: en.name,
      label: en.name,
      description: `${en.values.length} values`,
    }));
  }, [enumerations]);

  // Auto-generate display names when attribute name changes
  useEffect(() => {
    if (!isEditMode && formData.attributeName && formData.displayNames.length === 0) {
      const humanName = formData.attributeName
        .replace(/_/g, ' ')
        .replace(/-/g, ' ')
        .replace(/\b\w/g, (c) => c.toUpperCase());

      setFormData((prev) => ({
        ...prev,
        displayNames: [
          { name: formData.attributeName, format: 'SYSTEM', orderIndex: 0 },
          { name: humanName, format: 'HUMAN', orderIndex: 1 },
        ],
      }));
    }
  }, [formData.attributeName, isEditMode]);

  // Update form field
  const updateField = useCallback(<K extends keyof FormData>(field: K, value: FormData[K]) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  }, []);

  // Handle save
  const handleSave = async () => {
    // Build abstract path
    const abstractPath = attribute?.abstractPath ||
      `${productId}:abstract-path:${formData.componentType}:${formData.attributeName}`;

    setIsSaving(true);
    try {
      await onSave({
        abstractPath,
        productId,
        componentType: formData.componentType,
        componentId: formData.componentId || undefined,
        attributeName: formData.attributeName,
        datatypeId: formData.datatypeId,
        enumName: isEnumType ? formData.enumName : undefined,
        description: formData.description,
        constraintExpression: formData.constraintExpression || undefined,
        displayExpression: formData.attributeName.replace(/_/g, ' '),
        displayNames: formData.displayNames,
        tags: formData.tags,
        relatedAttributes: formData.relatedAttributes,
        immutable: formData.immutable,
      });
    } finally {
      setIsSaving(false);
    }
  };

  // Validation
  const isValid =
    validationState.componentType &&
    validationState.attributeName &&
    formData.componentType.trim() !== '' &&
    formData.attributeName.trim() !== '' &&
    (!isEnumType || formData.enumName.trim() !== '');

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <h3 className="text-lg font-semibold">
            {isEditMode ? 'Edit Abstract Attribute' : 'Create Abstract Attribute'}
          </h3>
          <button onClick={onCancel} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        {/* Form Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
          {/* Section 1: Basic Info */}
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">1</span>
              Basic Information
            </h4>

            <div className="grid gap-4 grid-cols-2">
              <div>
                <SelectDropdown
                  label="Component Type"
                  required
                  options={componentTypeOptions}
                  value={formData.componentType}
                  onChange={(val) => updateField('componentType', val || '')}
                  placeholder="Select or create..."
                  searchable
                  creatable
                  onCreate={(val) => updateField('componentType', val)}
                  disabled={isEditMode}
                  error={!validationState.componentType ? 'Invalid format' : undefined}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1.5">
                  Component ID <span className="text-gray-400">(optional)</span>
                </label>
                <Input
                  value={formData.componentId}
                  onChange={(e) => updateField('componentId', e.target.value)}
                  placeholder="e.g., default"
                  disabled={isEditMode}
                />
              </div>
            </div>

            <ValidatedInput
              label="Attribute Name"
              required
              patternKey="attributeName"
              value={formData.attributeName}
              onChange={(val, isValid) => {
                updateField('attributeName', val);
                setValidationState((prev) => ({ ...prev, attributeName: isValid }));
              }}
              placeholder="e.g., base_premium"
              disabled={isEditMode}
              description="Lowercase letters, numbers, underscores, dots, and hyphens"
            />

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1.5">Description</label>
              <textarea
                value={formData.description}
                onChange={(e) => updateField('description', e.target.value)}
                placeholder="Describe the purpose of this attribute..."
                rows={2}
                className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1"
              />
            </div>
          </div>

          {/* Section 2: Type Configuration */}
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">2</span>
              Type Configuration
            </h4>

            <div className="grid gap-4 grid-cols-2">
              <SelectDropdown
                label="Datatype"
                required
                options={datatypeOptions}
                value={formData.datatypeId}
                onChange={(val) => updateField('datatypeId', val || 'string')}
                placeholder="Select datatype..."
                searchable
                grouped
              />

              {isEnumType && (
                <SelectDropdown
                  label="Enumeration"
                  required
                  options={enumerationOptions}
                  value={formData.enumName}
                  onChange={(val) => updateField('enumName', val || '')}
                  placeholder="Select enumeration..."
                  searchable
                  error={isEnumType && !formData.enumName ? 'Required for enum type' : undefined}
                />
              )}
            </div>

            {/* Constraint Expression */}
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <label className="text-sm font-medium text-gray-700">
                  Constraint Expression <span className="text-gray-400">(optional)</span>
                </label>
                <button
                  type="button"
                  onClick={() => setShowConstraintEditor(!showConstraintEditor)}
                  className="text-xs text-primary hover:underline"
                >
                  {showConstraintEditor ? 'Hide' : 'Show'} JSON Editor
                </button>
              </div>

              {showConstraintEditor && (
                <textarea
                  value={formData.constraintExpression}
                  onChange={(e) => updateField('constraintExpression', e.target.value)}
                  placeholder='{"min": 0, "max": 100}'
                  rows={3}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm font-mono resize-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1"
                />
              )}

              {selectedDatatype && (
                <p className="mt-1 text-xs text-gray-500 flex items-center gap-1">
                  <Info className="h-3 w-3" />
                  Base type: <strong>{selectedDatatype.primitiveType}</strong>
                  {selectedDatatype.constraints && Object.keys(selectedDatatype.constraints).length > 0 && (
                    <> with constraints: {JSON.stringify(selectedDatatype.constraints)}</>
                  )}
                </p>
              )}
            </div>
          </div>

          {/* Section 3: Classification */}
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">3</span>
              Classification
            </h4>

            <TagMultiSelect
              label="Tags"
              value={formData.tags}
              options={existingTags}
              onChange={(tags) => updateField('tags', tags)}
              placeholder="Add tags..."
              creatable
            />

            <div className={cn(
              'p-3 rounded-md border',
              formData.immutable ? 'bg-amber-50 border-amber-200' : 'bg-gray-50 border-gray-200'
            )}>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="immutable"
                  checked={formData.immutable}
                  onChange={(e) => updateField('immutable', e.target.checked)}
                  className="rounded border-gray-300"
                  disabled={isEditMode && attribute?.immutable}
                />
                <label htmlFor="immutable" className="text-sm font-medium text-gray-700 flex items-center gap-2">
                  <Lock className={cn("h-4 w-4", formData.immutable ? "text-amber-500" : "text-gray-400")} />
                  Mark as Immutable
                </label>
              </div>
              {formData.immutable && (
                <div className="mt-2 flex items-start gap-2 text-amber-700">
                  <AlertTriangle className="h-4 w-4 mt-0.5 shrink-0" />
                  <p className="text-xs">
                    <strong>Warning:</strong> Immutable attributes cannot be modified or deleted after creation.
                    Rules targeting this attribute will also be locked. To make changes, you'll need to clone the product.
                  </p>
                </div>
              )}
            </div>
          </div>

          {/* Section 4: Relationships */}
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">4</span>
              Relationships
            </h4>

            <RelatedAttributeEditor
              relatedAttributes={formData.relatedAttributes}
              availableAttributes={existingAttributes}
              currentPath={attribute?.abstractPath}
              onChange={(attrs) => updateField('relatedAttributes', attrs)}
            />
          </div>

          {/* Section 5: Display Names */}
          <div className="space-y-4">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">5</span>
              Display Names
            </h4>

            <DisplayNameEditor
              displayNames={formData.displayNames}
              onChange={(names) => updateField('displayNames', names)}
            />
          </div>
        </div>

        {/* Footer */}
        <div className="flex gap-3 px-6 py-4 border-t bg-gray-50">
          <Button variant="outline" className="flex-1" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            className="flex-1"
            onClick={handleSave}
            disabled={isSaving || !isValid}
          >
            {isSaving ? 'Saving...' : isEditMode ? 'Update Attribute' : 'Create Attribute'}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default AbstractAttributeForm;

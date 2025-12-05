// AttributeInstanceForm - Form for creating/editing concrete attribute instances
// Workflow: Select abstract attribute → Enter componentId → Select valueType → Configure value

import { useState, useCallback, useMemo, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { SelectDropdown, type SelectOption } from './SelectDropdown';
import { ValueEditor } from './ValueEditor';
import { InlineRuleBuilder } from './InlineRuleBuilder';
import { cn } from '@/lib/utils';
import {
  X,
  FileText,
  Calculator,
  Box,
  Info,
} from 'lucide-react';
import type {
  Attribute,
  AbstractAttribute,
  DataType,
  Rule,
  TemplateEnumeration,
  AttributeValueType,
  AttributeValue,
} from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface AttributeInstanceFormProps {
  /** Product ID for this attribute */
  productId: string;
  /** Existing attribute for editing, null for create mode */
  attribute: Attribute | null;
  /** Available abstract attributes */
  abstractAttributes: AbstractAttribute[];
  /** Available datatypes */
  datatypes: DataType[];
  /** Available rules for rule-driven values */
  rules: Rule[];
  /** Available enumerations */
  enumerations: TemplateEnumeration[];
  /** Callback when form is saved */
  onSave: (data: Partial<Attribute>) => Promise<void>;
  /** Callback when form is cancelled */
  onCancel: () => void;
  /** Optional callback when a rule is created inline */
  onRuleCreated?: (rule: Rule) => void;
  /** Optional callback to open the advanced rule builder */
  onOpenAdvancedRuleBuilder?: (abstractAttribute: AbstractAttribute) => void;
}

interface FormData {
  abstractPath: string;
  componentId: string;
  valueType: AttributeValueType;
  value: AttributeValue | undefined;
  ruleId: string;
}

// =============================================================================
// CONSTANTS
// =============================================================================

const VALUE_TYPE_OPTIONS: SelectOption<AttributeValueType>[] = [
  {
    value: 'FIXED_VALUE',
    label: 'Fixed Value',
    description: 'Value is directly set and stored',
    icon: <FileText className="h-4 w-4 text-blue-500" />,
  },
  {
    value: 'RULE_DRIVEN',
    label: 'Rule Driven',
    description: 'Value computed by a rule',
    icon: <Calculator className="h-4 w-4 text-amber-500" />,
  },
  {
    value: 'JUST_DEFINITION',
    label: 'Just Definition',
    description: 'No value, only schema definition',
    icon: <Box className="h-4 w-4 text-gray-500" />,
  },
];

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function AttributeInstanceForm({
  productId,
  attribute,
  abstractAttributes,
  datatypes,
  rules,
  enumerations,
  onSave,
  onCancel,
  onRuleCreated,
  onOpenAdvancedRuleBuilder,
}: AttributeInstanceFormProps) {
  const isEditMode = !!attribute;

  // Initialize form data
  const [formData, setFormData] = useState<FormData>(() => {
    if (attribute) {
      return {
        abstractPath: attribute.abstractPath,
        componentId: attribute.componentId,
        valueType: attribute.valueType,
        value: attribute.value,
        ruleId: attribute.ruleId || '',
      };
    }
    return {
      abstractPath: '',
      componentId: 'default',
      valueType: 'JUST_DEFINITION',
      value: undefined,
      ruleId: '',
    };
  });

  const [isSaving, setIsSaving] = useState(false);

  // Get selected abstract attribute
  const selectedAbstract = useMemo(() => {
    return abstractAttributes.find((a) => a.abstractPath === formData.abstractPath);
  }, [abstractAttributes, formData.abstractPath]);

  // Get datatype for selected abstract attribute
  const selectedDatatype = useMemo(() => {
    if (!selectedAbstract) return null;
    return datatypes.find((dt) => dt.id === selectedAbstract.datatypeId);
  }, [selectedAbstract, datatypes]);

  // Get enumeration values if datatype is ENUM
  const enumValues = useMemo(() => {
    if (!selectedAbstract || selectedDatatype?.primitiveType !== 'ENUM') return [];
    const enumeration = enumerations.find((e) => e.name === selectedAbstract.enumName);
    return enumeration?.values || [];
  }, [selectedAbstract, selectedDatatype, enumerations]);

  // Build abstract attribute options grouped by component
  const abstractOptions: SelectOption[] = useMemo(() => {
    return abstractAttributes.map((attr) => ({
      value: attr.abstractPath,
      label: attr.attributeName,
      description: `${attr.componentType} - ${attr.datatypeId}`,
      group: attr.componentType,
    }));
  }, [abstractAttributes]);

  // Build rule options - only rules that output to this attribute
  const ruleOptions: SelectOption[] = useMemo(() => {
    if (!formData.abstractPath) return [];
    return rules
      .filter((r) =>
        r.outputAttributes.some((oa) => oa.attributePath === formData.abstractPath)
      )
      .map((rule) => ({
        value: rule.id,
        label: rule.displayExpression?.substring(0, 50) || rule.id,
        description: rule.description,
      }));
  }, [rules, formData.abstractPath]);

  // Reset value when abstract or value type changes
  useEffect(() => {
    if (!isEditMode) {
      setFormData((prev) => ({
        ...prev,
        value: undefined,
        ruleId: '',
      }));
    }
  }, [formData.abstractPath, formData.valueType, isEditMode]);

  // Update form field
  const updateField = useCallback(<K extends keyof FormData>(field: K, value: FormData[K]) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  }, []);

  // Build path from abstract and componentId
  const buildPath = useCallback(() => {
    if (!selectedAbstract) return '';
    return `${productId}:${selectedAbstract.componentType}:${formData.componentId}:${selectedAbstract.attributeName}`;
  }, [productId, selectedAbstract, formData.componentId]);

  // Handle save
  const handleSave = async () => {
    if (!selectedAbstract) return;

    setIsSaving(true);
    try {
      const path = attribute?.path || buildPath();

      await onSave({
        path,
        abstractPath: formData.abstractPath,
        productId,
        componentType: selectedAbstract.componentType,
        componentId: formData.componentId,
        attributeName: selectedAbstract.attributeName,
        valueType: formData.valueType,
        value: formData.valueType === 'FIXED_VALUE' ? formData.value : undefined,
        ruleId: formData.valueType === 'RULE_DRIVEN' ? formData.ruleId : undefined,
      });
    } finally {
      setIsSaving(false);
    }
  };

  // Validation
  const isValid =
    formData.abstractPath &&
    formData.componentId.trim() !== '' &&
    (formData.valueType !== 'RULE_DRIVEN' || formData.ruleId);

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <h3 className="text-lg font-semibold">
            {isEditMode ? 'Edit Concrete Attribute' : 'Create Concrete Attribute'}
          </h3>
          <button onClick={onCancel} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        {/* Form Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
          {/* Step 1: Select Abstract Attribute */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">1</span>
              Select Abstract Attribute
            </h4>

            <SelectDropdown
              options={abstractOptions}
              value={formData.abstractPath}
              onChange={(val) => updateField('abstractPath', val || '')}
              placeholder="Select abstract attribute..."
              searchable
              grouped
              disabled={isEditMode}
            />

            {selectedAbstract && (
              <div className="p-3 bg-gray-50 rounded-md text-sm">
                <div className="flex items-center gap-2 text-gray-600">
                  <Info className="h-4 w-4" />
                  <span>
                    <strong>{selectedAbstract.componentType}</strong>:{selectedAbstract.attributeName}
                  </span>
                </div>
                <p className="text-gray-500 mt-1">{selectedAbstract.description}</p>
                {selectedDatatype && (
                  <p className="text-xs text-gray-400 mt-1">
                    Type: {selectedDatatype.name} ({selectedDatatype.primitiveType})
                  </p>
                )}
              </div>
            )}
          </div>

          {/* Step 2: Enter Component ID */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">2</span>
              Component Instance ID
            </h4>

            <Input
              value={formData.componentId}
              onChange={(e) => updateField('componentId', e.target.value)}
              placeholder="e.g., default, primary, instance_1"
              disabled={isEditMode}
            />

            {selectedAbstract && formData.componentId && (
              <p className="text-xs text-gray-500">
                Full path: <code className="bg-gray-100 px-1 py-0.5 rounded">{buildPath()}</code>
              </p>
            )}
          </div>

          {/* Step 3: Select Value Type */}
          <div className="space-y-3">
            <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">3</span>
              Value Type
            </h4>

            <div className="grid gap-2">
              {VALUE_TYPE_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  type="button"
                  onClick={() => updateField('valueType', opt.value)}
                  className={cn(
                    'flex items-center gap-3 p-3 rounded-md border text-left transition-colors',
                    formData.valueType === opt.value
                      ? 'border-primary bg-primary/5'
                      : 'border-gray-200 hover:border-gray-300'
                  )}
                >
                  {opt.icon}
                  <div>
                    <div className="font-medium text-sm">{opt.label}</div>
                    <div className="text-xs text-gray-500">{opt.description}</div>
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Step 4: Configure Value */}
          {formData.valueType === 'FIXED_VALUE' && selectedAbstract && selectedDatatype && (
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
                <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">4</span>
                Set Value
              </h4>

              <ValueEditor
                value={formData.value}
                primitiveType={selectedDatatype.primitiveType}
                constraints={selectedDatatype.constraints}
                enumValues={enumValues}
                onChange={(val) => updateField('value', val)}
              />
            </div>
          )}

          {formData.valueType === 'RULE_DRIVEN' && (
            <div className="space-y-3">
              <h4 className="text-sm font-semibold text-gray-900 flex items-center gap-2">
                <span className="w-6 h-6 rounded-full bg-primary/10 text-primary flex items-center justify-center text-xs">4</span>
                Select Rule
              </h4>

              {ruleOptions.length > 0 ? (
                <SelectDropdown
                  options={ruleOptions}
                  value={formData.ruleId}
                  onChange={(val) => updateField('ruleId', val || '')}
                  placeholder="Select rule..."
                  searchable
                />
              ) : selectedAbstract ? (
                <InlineRuleBuilder
                  abstractAttribute={selectedAbstract}
                  productId={productId}
                  availableInputs={abstractAttributes.filter(
                    (a) => a.abstractPath !== selectedAbstract.abstractPath
                  )}
                  onRuleCreated={(newRule) => {
                    // Update the rule ID in form data
                    updateField('ruleId', newRule.id);
                    // Notify parent if callback provided
                    onRuleCreated?.(newRule);
                  }}
                  onOpenAdvancedBuilder={() => {
                    onOpenAdvancedRuleBuilder?.(selectedAbstract);
                  }}
                />
              ) : null}
            </div>
          )}

          {formData.valueType === 'JUST_DEFINITION' && (
            <div className="p-4 bg-gray-50 rounded-md">
              <p className="text-sm text-gray-600">
                This attribute will be created without a value. It serves as a schema definition
                that can be populated later or through external processes.
              </p>
            </div>
          )}
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
            {isSaving ? 'Saving...' : isEditMode ? 'Update' : 'Create'}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default AttributeInstanceForm;

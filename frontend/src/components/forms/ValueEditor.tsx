// ValueEditor - Type-aware value editor component
// Adapts UI based on datatype's primitiveType

import { useState, useCallback, useMemo } from 'react';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import {
  Plus,
  Trash2,
  Calendar,
  ToggleLeft,
  ToggleRight,
  ChevronDown,
} from 'lucide-react';
import type {
  AttributeValue,
  PrimitiveType,
  DatatypeConstraints,
} from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface ValueEditorProps {
  /** Current value */
  value: AttributeValue | undefined;
  /** Primitive type of the value */
  primitiveType: PrimitiveType;
  /** Constraints from datatype */
  constraints?: DatatypeConstraints;
  /** Enumeration values (for ENUM type) */
  enumValues?: string[];
  /** Callback when value changes */
  onChange: (value: AttributeValue | undefined) => void;
  /** Label for the editor */
  label?: string;
  /** Whether the field is required */
  required?: boolean;
  /** Disabled state */
  disabled?: boolean;
  /** Error message */
  error?: string;
  /** Class name */
  className?: string;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

function extractValue(attrValue: AttributeValue | undefined): unknown {
  if (!attrValue) return undefined;
  if (attrValue.type === 'null') return null;
  return (attrValue as { value: unknown }).value;
}

function createAttributeValue(type: PrimitiveType, value: unknown): AttributeValue {
  if (value === null || value === undefined) {
    return { type: 'null' };
  }

  switch (type) {
    case 'STRING':
      return { type: 'string', value: String(value) };
    case 'INT':
      return { type: 'int', value: Number(value) };
    case 'FLOAT':
      return { type: 'float', value: Number(value) };
    case 'DECIMAL':
      return { type: 'decimal', value: String(value) };
    case 'BOOL':
      return { type: 'bool', value: Boolean(value) };
    case 'DATETIME':
      return { type: 'string', value: String(value) };
    case 'ENUM':
      return { type: 'string', value: String(value) };
    case 'ARRAY':
      return { type: 'array', value: value as AttributeValue[] };
    case 'OBJECT':
      return { type: 'object', value: value as Record<string, AttributeValue> };
    default:
      return { type: 'string', value: String(value) };
  }
}

// =============================================================================
// SUB-COMPONENTS
// =============================================================================

interface StringEditorProps {
  value: string;
  onChange: (value: string) => void;
  constraints?: DatatypeConstraints;
  disabled?: boolean;
  placeholder?: string;
}

function StringEditor({ value, onChange, constraints, disabled, placeholder }: StringEditorProps) {
  const maxLength = constraints?.maxLength;
  const pattern = constraints?.pattern;

  return (
    <div className="space-y-1">
      <Input
        value={value}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        placeholder={placeholder || 'Enter text...'}
        maxLength={maxLength}
        pattern={pattern}
        className="w-full"
      />
      {maxLength && (
        <p className="text-xs text-gray-500 text-right">
          {value.length} / {maxLength}
        </p>
      )}
    </div>
  );
}

interface NumberEditorProps {
  value: number | '';
  onChange: (value: number | null) => void;
  constraints?: DatatypeConstraints;
  disabled?: boolean;
  isInteger?: boolean;
}

function NumberEditor({ value, onChange, constraints, disabled, isInteger }: NumberEditorProps) {
  const min = constraints?.min;
  const max = constraints?.max;

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    if (val === '') {
      onChange(null);
    } else {
      const num = isInteger ? parseInt(val, 10) : parseFloat(val);
      if (!isNaN(num)) {
        onChange(num);
      }
    }
  };

  return (
    <Input
      type="number"
      value={value}
      onChange={handleChange}
      disabled={disabled}
      min={min}
      max={max}
      step={isInteger ? 1 : 'any'}
      placeholder={isInteger ? 'Enter integer...' : 'Enter number...'}
      className="w-full"
    />
  );
}

interface DecimalEditorProps {
  value: string;
  onChange: (value: string) => void;
  constraints?: DatatypeConstraints;
  disabled?: boolean;
}

function DecimalEditor({ value, onChange, constraints, disabled }: DecimalEditorProps) {
  const precision = constraints?.precision;
  const scale = constraints?.scale;

  const validateDecimal = useCallback((val: string): boolean => {
    if (!precision && !scale) return true;
    const parts = val.split('.');
    const intPart = parts[0].replace('-', '');
    const decPart = parts[1] || '';

    if (precision && intPart.length + decPart.length > precision) return false;
    if (scale && decPart.length > scale) return false;
    return true;
  }, [precision, scale]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value;
    // Allow empty, numbers, decimal point, and negative sign
    if (!/^-?\d*\.?\d*$/.test(val)) return;
    if (validateDecimal(val)) {
      onChange(val);
    }
  };

  return (
    <div className="space-y-1">
      <Input
        type="text"
        value={value}
        onChange={handleChange}
        disabled={disabled}
        placeholder="Enter decimal value..."
        className="w-full font-mono"
      />
      {(precision || scale) && (
        <p className="text-xs text-gray-500">
          Precision: {precision || 'any'}, Scale: {scale || 'any'}
        </p>
      )}
    </div>
  );
}

interface BoolEditorProps {
  value: boolean;
  onChange: (value: boolean) => void;
  disabled?: boolean;
}

function BoolEditor({ value, onChange, disabled }: BoolEditorProps) {
  return (
    <button
      type="button"
      onClick={() => !disabled && onChange(!value)}
      disabled={disabled}
      className={cn(
        'flex items-center gap-2 px-3 py-2 rounded-md border transition-colors',
        value
          ? 'bg-primary/10 border-primary text-primary'
          : 'bg-gray-50 border-gray-200 text-gray-600',
        disabled && 'opacity-50 cursor-not-allowed'
      )}
    >
      {value ? (
        <ToggleRight className="h-5 w-5" />
      ) : (
        <ToggleLeft className="h-5 w-5" />
      )}
      <span className="font-medium">{value ? 'True' : 'False'}</span>
    </button>
  );
}

interface DateTimeEditorProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

function DateTimeEditor({ value, onChange, disabled }: DateTimeEditorProps) {
  // Convert timestamp or ISO string to datetime-local format
  const displayValue = useMemo(() => {
    if (!value) return '';
    try {
      const date = new Date(value);
      return date.toISOString().slice(0, 16);
    } catch {
      return value;
    }
  }, [value]);

  return (
    <div className="relative">
      <Input
        type="datetime-local"
        value={displayValue}
        onChange={(e) => onChange(e.target.value)}
        disabled={disabled}
        className="w-full"
      />
      <Calendar className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 pointer-events-none" />
    </div>
  );
}

interface EnumEditorProps {
  value: string;
  onChange: (value: string) => void;
  options: string[];
  disabled?: boolean;
}

function EnumEditor({ value, onChange, options, disabled }: EnumEditorProps) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={cn(
          'w-full flex items-center justify-between h-10 px-3 border rounded-md bg-white text-sm',
          'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1',
          disabled && 'bg-gray-100 cursor-not-allowed opacity-60'
        )}
      >
        <span className={cn(!value && 'text-gray-400')}>
          {value || 'Select value...'}
        </span>
        <ChevronDown className="h-4 w-4 text-gray-400" />
      </button>

      {isOpen && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setIsOpen(false)} />
          <div className="absolute z-20 w-full mt-1 bg-white border rounded-md shadow-lg max-h-60 overflow-y-auto">
            {options.map((opt) => (
              <button
                key={opt}
                type="button"
                onClick={() => {
                  onChange(opt);
                  setIsOpen(false);
                }}
                className={cn(
                  'w-full px-3 py-2 text-left text-sm',
                  opt === value ? 'bg-primary/10 text-primary' : 'hover:bg-gray-50'
                )}
              >
                {opt}
              </button>
            ))}
            {options.length === 0 && (
              <div className="px-3 py-4 text-sm text-gray-500 text-center">
                No options available
              </div>
            )}
          </div>
        </>
      )}
    </div>
  );
}

interface ArrayEditorProps {
  value: AttributeValue[];
  onChange: (value: AttributeValue[]) => void;
  constraints?: DatatypeConstraints;
  disabled?: boolean;
}

function ArrayEditor({ value, onChange, constraints, disabled }: ArrayEditorProps) {
  const addItem = () => {
    if (constraints?.maxItems && value.length >= constraints.maxItems) return;
    onChange([...value, { type: 'string', value: '' }]);
  };

  const updateItem = (index: number, newValue: string) => {
    const updated = [...value];
    updated[index] = { type: 'string', value: newValue };
    onChange(updated);
  };

  const removeItem = (index: number) => {
    if (constraints?.minItems && value.length <= constraints.minItems) return;
    onChange(value.filter((_, i) => i !== index));
  };

  const canAdd = !constraints?.maxItems || value.length < constraints.maxItems;
  const canRemove = !constraints?.minItems || value.length > constraints.minItems;

  return (
    <div className="space-y-2">
      {value.map((item, index) => (
        <div key={index} className="flex items-center gap-2">
          <Input
            value={(item as { value: string }).value || ''}
            onChange={(e) => updateItem(index, e.target.value)}
            disabled={disabled}
            placeholder={`Item ${index + 1}`}
            className="flex-1"
          />
          {canRemove && !disabled && (
            <button
              type="button"
              onClick={() => removeItem(index)}
              className="p-2 hover:bg-red-50 rounded text-red-500"
            >
              <Trash2 className="h-4 w-4" />
            </button>
          )}
        </div>
      ))}

      {canAdd && !disabled && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={addItem}
          className="w-full"
        >
          <Plus className="h-4 w-4 mr-1" />
          Add Item
        </Button>
      )}

      {constraints && (
        <p className="text-xs text-gray-500">
          {value.length} items
          {constraints.minItems && ` (min: ${constraints.minItems})`}
          {constraints.maxItems && ` (max: ${constraints.maxItems})`}
        </p>
      )}
    </div>
  );
}

interface ObjectEditorProps {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

function ObjectEditor({ value, onChange, disabled }: ObjectEditorProps) {
  const [error, setError] = useState<string | null>(null);

  const handleChange = (newValue: string) => {
    onChange(newValue);
    try {
      JSON.parse(newValue);
      setError(null);
    } catch {
      setError('Invalid JSON');
    }
  };

  return (
    <div className="space-y-1">
      <textarea
        value={value}
        onChange={(e) => handleChange(e.target.value)}
        disabled={disabled}
        placeholder='{"key": "value"}'
        rows={4}
        className={cn(
          'w-full px-3 py-2 border rounded-md text-sm font-mono resize-y',
          'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1',
          error && 'border-red-500'
        )}
      />
      {error && <p className="text-xs text-red-600">{error}</p>}
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function ValueEditor({
  value,
  primitiveType,
  constraints,
  enumValues,
  onChange,
  label,
  required,
  disabled,
  error,
  className,
}: ValueEditorProps) {
  const handleChange = useCallback((newValue: unknown) => {
    onChange(createAttributeValue(primitiveType, newValue));
  }, [primitiveType, onChange]);

  const rawValue = extractValue(value);

  const renderEditor = () => {
    switch (primitiveType) {
      case 'STRING':
        return (
          <StringEditor
            value={(rawValue as string) || ''}
            onChange={handleChange}
            constraints={constraints}
            disabled={disabled}
          />
        );

      case 'INT':
        return (
          <NumberEditor
            value={rawValue !== undefined && rawValue !== null ? (rawValue as number) : ''}
            onChange={handleChange}
            constraints={constraints}
            disabled={disabled}
            isInteger
          />
        );

      case 'FLOAT':
        return (
          <NumberEditor
            value={rawValue !== undefined && rawValue !== null ? (rawValue as number) : ''}
            onChange={handleChange}
            constraints={constraints}
            disabled={disabled}
          />
        );

      case 'DECIMAL':
        return (
          <DecimalEditor
            value={(rawValue as string) || ''}
            onChange={handleChange}
            constraints={constraints}
            disabled={disabled}
          />
        );

      case 'BOOL':
        return (
          <BoolEditor
            value={(rawValue as boolean) || false}
            onChange={handleChange}
            disabled={disabled}
          />
        );

      case 'DATETIME':
        return (
          <DateTimeEditor
            value={(rawValue as string) || ''}
            onChange={handleChange}
            disabled={disabled}
          />
        );

      case 'ENUM':
        return (
          <EnumEditor
            value={(rawValue as string) || ''}
            onChange={handleChange}
            options={enumValues || []}
            disabled={disabled}
          />
        );

      case 'ARRAY':
        return (
          <ArrayEditor
            value={(rawValue as AttributeValue[]) || []}
            onChange={(arr) => onChange({ type: 'array', value: arr })}
            constraints={constraints}
            disabled={disabled}
          />
        );

      case 'OBJECT':
        return (
          <ObjectEditor
            value={typeof rawValue === 'object' ? JSON.stringify(rawValue, null, 2) : '{}'}
            onChange={(str) => {
              try {
                const parsed = JSON.parse(str);
                onChange({ type: 'object', value: parsed });
              } catch {
                // Keep invalid JSON as string
              }
            }}
            disabled={disabled}
          />
        );

      default:
        return (
          <StringEditor
            value={String(rawValue || '')}
            onChange={handleChange}
            disabled={disabled}
          />
        );
    }
  };

  return (
    <div className={cn('space-y-1.5', className)}>
      {label && (
        <label className="block text-sm font-medium text-gray-700">
          {label}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}

      {renderEditor()}

      {error && <p className="text-xs text-red-600">{error}</p>}
    </div>
  );
}

export default ValueEditor;

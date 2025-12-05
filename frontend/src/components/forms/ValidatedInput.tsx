// ValidatedInput - Input with real-time pattern validation
// Uses validation patterns from src/utils/validation.ts

import { useState, useCallback, useEffect, forwardRef } from 'react';
import { Input } from '@/components/ui/input';
import { validatePattern, type PatternKey, type ValidationResult } from '@/utils/validation';
import { cn } from '@/lib/utils';
import { CheckCircle, XCircle, AlertCircle } from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

export interface ValidatedInputProps
  extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onChange'> {
  /** The validation pattern to use */
  patternKey: PatternKey;
  /** Whether the field is required */
  required?: boolean;
  /** Callback when value changes - called with (value, isValid) */
  onChange?: (value: string, isValid: boolean) => void;
  /** Callback with full validation result */
  onValidationChange?: (result: ValidationResult) => void;
  /** Show validation feedback immediately (default: after blur) */
  validateOnChange?: boolean;
  /** Custom error message to override pattern error */
  customError?: string;
  /** Label for the input */
  label?: string;
  /** Additional description below the input */
  description?: string;
  /** Show success indicator when valid */
  showSuccess?: boolean;
  /** Unique values to check against for uniqueness validation */
  existingValues?: string[];
  /** External validation state to override internal */
  externalError?: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

export const ValidatedInput = forwardRef<HTMLInputElement, ValidatedInputProps>(
  function ValidatedInput(
    {
      patternKey,
      required = true,
      onChange,
      onValidationChange,
      validateOnChange = false,
      customError,
      label,
      description,
      showSuccess = true,
      existingValues,
      externalError,
      className,
      value: propValue,
      ...props
    },
    ref
  ) {
    const [internalValue, setInternalValue] = useState(propValue?.toString() || '');
    const [touched, setTouched] = useState(false);
    const [validation, setValidation] = useState<ValidationResult>({ isValid: true });

    // Sync with prop value
    useEffect(() => {
      if (propValue !== undefined) {
        setInternalValue(propValue.toString());
      }
    }, [propValue]);

    const validate = useCallback(
      (value: string): ValidationResult => {
        // First check pattern validation
        const result = validatePattern(value, patternKey, { required });

        // If pattern is valid, check uniqueness
        if (result.isValid && existingValues && value.trim()) {
          const isDuplicate = existingValues.some(
            (existing) => existing.toLowerCase() === value.toLowerCase()
          );
          if (isDuplicate) {
            return {
              isValid: false,
              error: 'This value already exists',
            };
          }
        }

        return result;
      },
      [patternKey, required, existingValues]
    );

    const handleChange = useCallback(
      (e: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = e.target.value;
        setInternalValue(newValue);

        const result = validate(newValue);
        if (validateOnChange || touched) {
          setValidation(result);
          onValidationChange?.(result);
        }

        onChange?.(newValue, result.isValid);
      },
      [validate, validateOnChange, touched, onChange, onValidationChange]
    );

    const handleBlur = useCallback(
      (e: React.FocusEvent<HTMLInputElement>) => {
        setTouched(true);
        const result = validate(internalValue);
        setValidation(result);
        onValidationChange?.(result);
        props.onBlur?.(e);
      },
      [validate, internalValue, onValidationChange, props]
    );

    // Determine which error to show
    const displayError = externalError || (touched ? (customError || validation.error) : undefined);
    const isValid = !externalError && validation.isValid;
    const showValidIcon = showSuccess && touched && isValid && internalValue.trim();

    return (
      <div className="space-y-1.5">
        {label && (
          <label className="block text-sm font-medium text-gray-700">
            {label}
            {required && <span className="text-red-500 ml-1">*</span>}
          </label>
        )}

        <div className="relative">
          <Input
            ref={ref}
            value={internalValue}
            onChange={handleChange}
            onBlur={handleBlur}
            className={cn(
              'pr-10',
              displayError && 'border-red-500 focus-visible:ring-red-500',
              showValidIcon && 'border-green-500 focus-visible:ring-green-500',
              className
            )}
            {...props}
          />

          {/* Validation Icon */}
          <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
            {displayError ? (
              <XCircle className="h-4 w-4 text-red-500" />
            ) : showValidIcon ? (
              <CheckCircle className="h-4 w-4 text-green-500" />
            ) : null}
          </div>
        </div>

        {/* Description */}
        {description && !displayError && (
          <p className="text-xs text-gray-500">{description}</p>
        )}

        {/* Error Message */}
        {displayError && (
          <div className="flex items-start gap-1.5 text-red-600">
            <AlertCircle className="h-3.5 w-3.5 mt-0.5 shrink-0" />
            <div>
              <p className="text-xs">{displayError}</p>
              {validation.hint && !customError && (
                <p className="text-xs text-gray-500 mt-0.5">{validation.hint}</p>
              )}
            </div>
          </div>
        )}
      </div>
    );
  }
);

export default ValidatedInput;

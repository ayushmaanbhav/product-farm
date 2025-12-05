// Validation Utilities for Product-FARM
// Aligned with backend gRPC validation patterns

import { useState, useCallback, useMemo } from 'react';

// =============================================================================
// BACKEND REGEX PATTERNS
// =============================================================================

/**
 * Validation patterns from backend proto definitions
 * These must match exactly with backend validation rules
 */
export const VALIDATION_PATTERNS = {
  // Product ID: Start with letter, then letters/numbers/underscores, max 51 chars
  productId: /^[a-zA-Z]([_][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}$/,

  // Component type: lowercase letters with optional hyphens, max 51 chars
  componentType: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Component ID: lowercase letters/numbers with optional hyphens, max 51 chars
  componentId: /^[a-z]([-][a-z0-9]|[a-z0-9]){0,50}$/,

  // Attribute name: lowercase with dots, hyphens, numbers, max 101 chars
  attributeName: /^[a-z]([.][a-z]|[-][a-z0-9]|[a-z0-9]){0,100}$/,

  // Display name: similar to attribute but longer, max 201 chars
  displayName: /^[a-z]([.][a-z]|[-][a-z0-9]|[a-z0-9]){0,200}$/,

  // Tag: lowercase letters with optional hyphens, max 51 chars
  tag: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Datatype ID: lowercase letters with optional hyphens, max 51 chars
  datatypeId: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Functionality name: lowercase letters with optional hyphens, max 51 chars
  functionalityName: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Enumeration name: lowercase letters with optional hyphens, max 51 chars
  enumerationName: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Enumeration value: lowercase letters with optional hyphens, max 51 chars
  enumerationValue: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Rule type: lowercase letters with optional hyphens, max 51 chars
  ruleType: /^[a-z]([-][a-z]|[a-z]){0,50}$/,

  // Product name: letters, numbers, punctuation, spaces, max 50 chars
  productName: /^[a-zA-Z0-9,.\-_:' ]{1,50}$/,
} as const;

export type PatternKey = keyof typeof VALIDATION_PATTERNS;

// =============================================================================
// HUMAN-READABLE ERROR MESSAGES
// =============================================================================

export const ERROR_MESSAGES: Record<PatternKey, { invalid: string; format: string }> = {
  productId: {
    invalid: 'Product ID must start with a letter and contain only letters, numbers, and underscores',
    format: 'e.g., "myProduct", "insurance_v1", "ProductA123"',
  },
  componentType: {
    invalid: 'Component type must be lowercase letters with optional hyphens',
    format: 'e.g., "cover", "customer-info", "pricing"',
  },
  componentId: {
    invalid: 'Component ID must be lowercase letters/numbers with optional hyphens',
    format: 'e.g., "basic", "premium-v2", "tier1"',
  },
  attributeName: {
    invalid: 'Attribute name must be lowercase with dots, hyphens, or numbers',
    format: 'e.g., "premium", "base-rate", "customer.age"',
  },
  displayName: {
    invalid: 'Display name must be lowercase with dots, hyphens, or numbers',
    format: 'e.g., "premium-amount", "customer.full-name"',
  },
  tag: {
    invalid: 'Tag must be lowercase letters with optional hyphens',
    format: 'e.g., "input", "computed", "financial-calc"',
  },
  datatypeId: {
    invalid: 'Datatype ID must be lowercase letters with optional hyphens',
    format: 'e.g., "decimal", "integer", "date-time"',
  },
  functionalityName: {
    invalid: 'Functionality name must be lowercase letters with optional hyphens',
    format: 'e.g., "premium-calculation", "eligibility-check"',
  },
  enumerationName: {
    invalid: 'Enumeration name must be lowercase letters with optional hyphens',
    format: 'e.g., "cover-types", "payment-frequency"',
  },
  enumerationValue: {
    invalid: 'Value must be lowercase letters with optional hyphens',
    format: 'e.g., "comprehensive", "third-party", "monthly"',
  },
  ruleType: {
    invalid: 'Rule type must be lowercase letters with optional hyphens',
    format: 'e.g., "calculation", "validation", "pricing-rule"',
  },
  productName: {
    invalid: 'Product name can contain letters, numbers, and common punctuation',
    format: 'e.g., "Motor Insurance V1", "Health Plan: Basic"',
  },
};

// =============================================================================
// VALIDATION FUNCTIONS
// =============================================================================

export interface ValidationResult {
  isValid: boolean;
  error?: string;
  hint?: string;
}

/**
 * Validate a value against a named pattern
 */
export function validatePattern(
  value: string,
  patternKey: PatternKey,
  options: { required?: boolean } = {}
): ValidationResult {
  const { required = true } = options;

  // Handle empty values
  if (!value || value.trim() === '') {
    if (required) {
      return { isValid: false, error: 'This field is required' };
    }
    return { isValid: true };
  }

  const pattern = VALIDATION_PATTERNS[patternKey];
  const messages = ERROR_MESSAGES[patternKey];

  if (!pattern.test(value)) {
    return {
      isValid: false,
      error: messages.invalid,
      hint: messages.format,
    };
  }

  return { isValid: true };
}

/**
 * Validate JSON string
 */
export function validateJson(value: string, options: { required?: boolean } = {}): ValidationResult {
  const { required = false } = options;

  if (!value || value.trim() === '') {
    if (required) {
      return { isValid: false, error: 'This field is required' };
    }
    return { isValid: true };
  }

  try {
    JSON.parse(value);
    return { isValid: true };
  } catch (e) {
    return {
      isValid: false,
      error: 'Invalid JSON format',
      hint: 'Must be valid JSON (e.g., {"key": "value"})',
    };
  }
}

/**
 * Validate number within range
 */
export function validateNumber(
  value: string | number,
  options: { min?: number; max?: number; integer?: boolean; required?: boolean } = {}
): ValidationResult {
  const { min, max, integer = false, required = false } = options;

  const strValue = String(value);
  if (!strValue || strValue.trim() === '') {
    if (required) {
      return { isValid: false, error: 'This field is required' };
    }
    return { isValid: true };
  }

  const num = Number(value);

  if (isNaN(num)) {
    return { isValid: false, error: 'Must be a valid number' };
  }

  if (integer && !Number.isInteger(num)) {
    return { isValid: false, error: 'Must be a whole number' };
  }

  if (min !== undefined && num < min) {
    return { isValid: false, error: `Must be at least ${min}` };
  }

  if (max !== undefined && num > max) {
    return { isValid: false, error: `Must be at most ${max}` };
  }

  return { isValid: true };
}

/**
 * Validate string length
 */
export function validateLength(
  value: string,
  options: { min?: number; max?: number; required?: boolean } = {}
): ValidationResult {
  const { min, max, required = false } = options;

  if (!value || value.trim() === '') {
    if (required) {
      return { isValid: false, error: 'This field is required' };
    }
    return { isValid: true };
  }

  if (min !== undefined && value.length < min) {
    return { isValid: false, error: `Must be at least ${min} characters` };
  }

  if (max !== undefined && value.length > max) {
    return { isValid: false, error: `Must be at most ${max} characters` };
  }

  return { isValid: true };
}

/**
 * Validate uniqueness against existing values
 */
export function validateUnique(
  value: string,
  existingValues: string[],
  options: { caseSensitive?: boolean; fieldName?: string } = {}
): ValidationResult {
  const { caseSensitive = false, fieldName = 'Value' } = options;

  const normalizedValue = caseSensitive ? value : value.toLowerCase();
  const normalizedExisting = existingValues.map((v) =>
    caseSensitive ? v : v.toLowerCase()
  );

  if (normalizedExisting.includes(normalizedValue)) {
    return { isValid: false, error: `${fieldName} already exists` };
  }

  return { isValid: true };
}

// =============================================================================
// REACT HOOKS
// =============================================================================

export interface FieldValidationState {
  value: string;
  error?: string;
  hint?: string;
  touched: boolean;
  isValid: boolean;
}

/**
 * Hook for single field validation with pattern
 */
export function useFieldValidation(
  patternKey: PatternKey,
  options: { required?: boolean; initialValue?: string } = {}
) {
  const { required = true, initialValue = '' } = options;

  const [state, setState] = useState<FieldValidationState>({
    value: initialValue,
    touched: false,
    isValid: !required || initialValue !== '',
  });

  const validate = useCallback(
    (value: string): ValidationResult => {
      return validatePattern(value, patternKey, { required });
    },
    [patternKey, required]
  );

  const setValue = useCallback(
    (value: string) => {
      const result = validate(value);
      setState({
        value,
        touched: true,
        isValid: result.isValid,
        error: result.error,
        hint: result.hint,
      });
    },
    [validate]
  );

  const setTouched = useCallback(() => {
    setState((prev) => {
      if (prev.touched) return prev;
      const result = validate(prev.value);
      return { ...prev, touched: true, error: result.error, hint: result.hint };
    });
  }, [validate]);

  const reset = useCallback(() => {
    setState({
      value: initialValue,
      touched: false,
      isValid: !required || initialValue !== '',
    });
  }, [initialValue, required]);

  return {
    ...state,
    setValue,
    setTouched,
    reset,
    validate,
  };
}

export interface FormValidationState<T extends Record<string, string>> {
  values: T;
  errors: Partial<Record<keyof T, string>>;
  touched: Partial<Record<keyof T, boolean>>;
  isValid: boolean;
}

export interface FieldConfig {
  pattern?: PatternKey;
  required?: boolean;
  customValidator?: (value: string, allValues: Record<string, string>) => ValidationResult;
}

/**
 * Hook for multi-field form validation
 */
export function useFormValidation<T extends Record<string, string>>(
  initialValues: T,
  fieldConfigs: Partial<Record<keyof T, FieldConfig>>
) {
  const [values, setValuesState] = useState<T>(initialValues);
  const [touched, setTouched] = useState<Partial<Record<keyof T, boolean>>>({});

  const validateField = useCallback(
    (field: keyof T, value: string, allValues: T): ValidationResult => {
      const config = fieldConfigs[field];
      if (!config) return { isValid: true };

      // Custom validator takes precedence
      if (config.customValidator) {
        return config.customValidator(value, allValues);
      }

      // Pattern validation
      if (config.pattern) {
        return validatePattern(value, config.pattern, { required: config.required });
      }

      // Required only
      if (config.required && (!value || value.trim() === '')) {
        return { isValid: false, error: 'This field is required' };
      }

      return { isValid: true };
    },
    [fieldConfigs]
  );

  const errors = useMemo(() => {
    const errs: Partial<Record<keyof T, string>> = {};
    for (const field of Object.keys(values) as Array<keyof T>) {
      const result = validateField(field, values[field], values);
      if (!result.isValid) {
        errs[field] = result.error;
      }
    }
    return errs;
  }, [values, validateField]);

  const isValid = useMemo(() => Object.keys(errors).length === 0, [errors]);

  const setValue = useCallback((field: keyof T, value: string) => {
    setValuesState((prev) => ({ ...prev, [field]: value }));
    setTouched((prev) => ({ ...prev, [field]: true }));
  }, []);

  const setValues = useCallback((newValues: Partial<T>) => {
    setValuesState((prev) => ({ ...prev, ...newValues }));
  }, []);

  const setFieldTouched = useCallback((field: keyof T) => {
    setTouched((prev) => ({ ...prev, [field]: true }));
  }, []);

  const touchAll = useCallback(() => {
    const allTouched: Partial<Record<keyof T, boolean>> = {};
    for (const field of Object.keys(values) as Array<keyof T>) {
      allTouched[field] = true;
    }
    setTouched(allTouched);
  }, [values]);

  const reset = useCallback(() => {
    setValuesState(initialValues);
    setTouched({});
  }, [initialValues]);

  return {
    values,
    errors,
    touched,
    isValid,
    setValue,
    setValues,
    setFieldTouched,
    touchAll,
    reset,
    validateField,
  };
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Convert a display string to a valid ID format
 * e.g., "My Product Name" -> "my-product-name"
 */
export function toIdFormat(value: string): string {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 50);
}

/**
 * Convert a display string to snake_case
 * e.g., "My Product Name" -> "my_product_name"
 */
export function toSnakeCase(value: string): string {
  return value
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, 50);
}

/**
 * Convert snake_case or kebab-case to Title Case
 * e.g., "my_product_name" -> "My Product Name"
 */
export function toTitleCase(value: string): string {
  return value
    .replace(/[-_]/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase());
}

/**
 * Build abstract path from components
 */
export function buildAbstractPath(
  productId: string,
  componentType: string,
  attributeName: string,
  componentId?: string
): string {
  const parts = [productId, 'abstract-path', componentType];
  if (componentId) {
    parts.push(componentId);
  }
  parts.push(attributeName);
  return parts.join(':');
}

/**
 * Build concrete path from components
 */
export function buildConcretePath(
  productId: string,
  componentType: string,
  componentId: string,
  attributeName: string
): string {
  return `${productId}:${componentType}:${componentId}:${attributeName}`;
}

/**
 * Parse an abstract path into its components
 */
export function parseAbstractPath(path: string): {
  productId: string;
  componentType: string;
  componentId?: string;
  attributeName: string;
} | null {
  const parts = path.split(':');
  if (parts.length < 4 || parts[1] !== 'abstract-path') {
    return null;
  }

  if (parts.length === 4) {
    return {
      productId: parts[0],
      componentType: parts[2],
      attributeName: parts[3],
    };
  }

  if (parts.length === 5) {
    return {
      productId: parts[0],
      componentType: parts[2],
      componentId: parts[3],
      attributeName: parts[4],
    };
  }

  return null;
}

/**
 * Parse a concrete path into its components
 */
export function parseConcretePath(path: string): {
  productId: string;
  componentType: string;
  componentId: string;
  attributeName: string;
} | null {
  const parts = path.split(':');
  if (parts.length !== 4) {
    return null;
  }

  return {
    productId: parts[0],
    componentType: parts[1],
    componentId: parts[2],
    attributeName: parts[3],
  };
}

// ProductForm - Enhanced form for creating/editing products
// Includes validation for product ID, template type selection, and date pickers

import { useState, useCallback, useMemo } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { SelectDropdown, type SelectOption } from './SelectDropdown';
import { ValidatedInput } from './ValidatedInput';
import { X, Package, Calendar, Info } from 'lucide-react';
import { ERROR_MESSAGES } from '@/utils/validation';
import type { Product } from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface ProductFormProps {
  /** Existing product for editing, null for create mode */
  product: Product | null;
  /** Available template types from existing products */
  existingTemplateTypes: string[];
  /** Callback when form is saved */
  onSave: (data: Partial<Product>) => Promise<void>;
  /** Callback when form is cancelled */
  onCancel: () => void;
}

interface FormData {
  id: string;
  name: string;
  description: string;
  templateType: string;
  effectiveFrom: string;
  expiryAt: string;
}

// =============================================================================
// CONSTANTS
// =============================================================================

const DEFAULT_TEMPLATE_TYPES = [
  { value: 'insurance', label: 'Insurance', description: 'General insurance products' },
  { value: 'life-insurance', label: 'Life Insurance', description: 'Life and health insurance' },
  { value: 'motor', label: 'Motor', description: 'Vehicle insurance products' },
  { value: 'property', label: 'Property', description: 'Home and property insurance' },
  { value: 'travel', label: 'Travel', description: 'Travel insurance products' },
  { value: 'loan', label: 'Loan', description: 'Lending and loan products' },
  { value: 'credit', label: 'Credit', description: 'Credit card and line of credit' },
  { value: 'investment', label: 'Investment', description: 'Investment and savings products' },
];

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

function formatDateForInput(dateValue: string | number | undefined): string {
  if (!dateValue) return '';
  try {
    const date = typeof dateValue === 'number' ? new Date(dateValue) : new Date(dateValue);
    return date.toISOString().split('T')[0];
  } catch {
    return '';
  }
}

function formatDateForApi(dateStr: string): number | undefined {
  if (!dateStr) return undefined;
  try {
    return new Date(dateStr).getTime();
  } catch {
    return undefined;
  }
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function ProductForm({
  product,
  existingTemplateTypes,
  onSave,
  onCancel,
}: ProductFormProps) {
  const isEditMode = !!product;

  // Initialize form data
  const [formData, setFormData] = useState<FormData>(() => {
    if (product) {
      return {
        id: product.id,
        name: product.name,
        description: product.description || '',
        templateType: product.templateType,
        effectiveFrom: formatDateForInput(product.effectiveFrom),
        expiryAt: formatDateForInput(product.expiryAt),
      };
    }
    return {
      id: '',
      name: '',
      description: '',
      templateType: 'insurance',
      effectiveFrom: new Date().toISOString().split('T')[0],
      expiryAt: '',
    };
  });

  const [isSaving, setIsSaving] = useState(false);
  const [newTemplateType, setNewTemplateType] = useState('');
  const [showNewTemplateInput, setShowNewTemplateInput] = useState(false);
  const [errors, setErrors] = useState<Record<string, string>>({});

  // Build template type options
  const templateOptions: SelectOption[] = useMemo(() => {
    const options: SelectOption[] = [];

    DEFAULT_TEMPLATE_TYPES.forEach((t) => {
      options.push(t);
    });

    existingTemplateTypes.forEach((t) => {
      if (!DEFAULT_TEMPLATE_TYPES.find((dt) => dt.value === t)) {
        options.push({
          value: t,
          label: t.charAt(0).toUpperCase() + t.slice(1).replace(/-/g, ' '),
          description: 'Custom template type',
        });
      }
    });

    return options;
  }, [existingTemplateTypes]);

  // Update form field
  const updateField = useCallback(<K extends keyof FormData>(field: K, value: FormData[K]) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    setErrors((prev) => ({ ...prev, [field]: '' }));
  }, []);

  // Validate form
  const validate = useCallback((): boolean => {
    const newErrors: Record<string, string> = {};

    if (!isEditMode) {
      if (!formData.id.trim()) {
        newErrors.id = 'Product ID is required';
      }
      // Pattern validation is handled by ValidatedInput
    }

    if (!formData.name.trim()) {
      newErrors.name = 'Product name is required';
    }

    if (!formData.templateType) {
      newErrors.templateType = 'Template type is required';
    }

    if (!formData.effectiveFrom) {
      newErrors.effectiveFrom = 'Effective from date is required';
    }

    if (formData.expiryAt && formData.effectiveFrom) {
      const effectiveDate = new Date(formData.effectiveFrom);
      const expiryDate = new Date(formData.expiryAt);
      if (expiryDate <= effectiveDate) {
        newErrors.expiryAt = 'Expiry date must be after effective date';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  }, [formData, isEditMode]);

  // Handle save
  const handleSave = async () => {
    if (!validate()) return;

    setIsSaving(true);
    try {
      await onSave({
        id: isEditMode ? product.id : formData.id,
        name: formData.name,
        description: formData.description || undefined,
        templateType: formData.templateType,
        effectiveFrom: formatDateForApi(formData.effectiveFrom),
        expiryAt: formatDateForApi(formData.expiryAt),
      });
    } finally {
      setIsSaving(false);
    }
  };

  // Add new template type
  const handleAddTemplateType = () => {
    if (!newTemplateType.trim()) return;
    const slug = newTemplateType.toLowerCase().replace(/\s+/g, '-');
    updateField('templateType', slug);
    setNewTemplateType('');
    setShowNewTemplateInput(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <Package className="h-5 w-5 text-primary" />
            </div>
            <h3 className="text-lg font-semibold">
              {isEditMode ? 'Edit Product' : 'Create Product'}
            </h3>
          </div>
          <button onClick={onCancel} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        {/* Form Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-5">
          {/* Product ID */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Product ID <span className="text-red-500">*</span>
            </label>
            {isEditMode ? (
              <div className="flex items-center gap-2 px-3 py-2 bg-gray-50 border rounded-md">
                <code className="text-sm font-mono">{product.id}</code>
                <span className="text-xs text-gray-500">(cannot be changed)</span>
              </div>
            ) : (
              <>
                <ValidatedInput
                  value={formData.id}
                  onChange={(val) => updateField('id', val)}
                  placeholder="e.g., termLife2024, motorStandard"
                  patternKey="productId"
                  validateOnChange
                  description={ERROR_MESSAGES.productId.format}
                />
                {errors.id && <p className="text-xs text-red-600">{errors.id}</p>}
              </>
            )}
          </div>

          {/* Product Name */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Product Name <span className="text-red-500">*</span>
            </label>
            <Input
              value={formData.name}
              onChange={(e) => updateField('name', e.target.value)}
              placeholder="e.g., Term Life Insurance 2024"
            />
            {errors.name && <p className="text-xs text-red-600">{errors.name}</p>}
          </div>

          {/* Description */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">Description</label>
            <textarea
              value={formData.description}
              onChange={(e) => updateField('description', e.target.value)}
              placeholder="Describe this product..."
              rows={3}
              className="w-full px-3 py-2 border rounded-md text-sm resize-none focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1"
            />
          </div>

          {/* Template Type */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Template Type <span className="text-red-500">*</span>
            </label>
            {showNewTemplateInput ? (
              <div className="flex gap-2">
                <Input
                  value={newTemplateType}
                  onChange={(e) => setNewTemplateType(e.target.value)}
                  placeholder="New template type name..."
                  className="flex-1"
                  autoFocus
                />
                <Button size="sm" onClick={handleAddTemplateType}>
                  Add
                </Button>
                <Button size="sm" variant="outline" onClick={() => setShowNewTemplateInput(false)}>
                  Cancel
                </Button>
              </div>
            ) : (
              <div className="flex gap-2">
                <div className="flex-1">
                  <SelectDropdown
                    options={templateOptions}
                    value={formData.templateType}
                    onChange={(val) => updateField('templateType', val || '')}
                    placeholder="Select template type..."
                    searchable
                    disabled={isEditMode}
                  />
                </div>
                {!isEditMode && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setShowNewTemplateInput(true)}
                  >
                    New
                  </Button>
                )}
              </div>
            )}
            {errors.templateType && <p className="text-xs text-red-600">{errors.templateType}</p>}
            {isEditMode && (
              <p className="text-xs text-gray-500">Template type cannot be changed after creation</p>
            )}
          </div>

          {/* Effective From */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Effective From <span className="text-red-500">*</span>
            </label>
            <div className="relative">
              <Input
                type="date"
                value={formData.effectiveFrom}
                onChange={(e) => updateField('effectiveFrom', e.target.value)}
              />
              <Calendar className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 pointer-events-none" />
            </div>
            {errors.effectiveFrom && <p className="text-xs text-red-600">{errors.effectiveFrom}</p>}
          </div>

          {/* Expiry At */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">Expiry At</label>
            <div className="relative">
              <Input
                type="date"
                value={formData.expiryAt}
                onChange={(e) => updateField('expiryAt', e.target.value)}
              />
              <Calendar className="absolute right-3 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400 pointer-events-none" />
            </div>
            {errors.expiryAt && <p className="text-xs text-red-600">{errors.expiryAt}</p>}
            <p className="text-xs text-gray-500">Optional. Leave empty for no expiry.</p>
          </div>

          {/* Info Banner */}
          <div className="p-3 bg-blue-50 border border-blue-100 rounded-md">
            <div className="flex items-start gap-2 text-blue-700">
              <Info className="h-4 w-4 mt-0.5 shrink-0" />
              <div className="text-sm">
                <p className="font-medium">Product Lifecycle</p>
                <p className="text-xs mt-1">
                  Products start in DRAFT status. Submit for approval when ready, then an
                  approver can activate the product.
                </p>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex gap-3 px-6 py-4 border-t bg-gray-50">
          <Button variant="outline" className="flex-1" onClick={onCancel}>
            Cancel
          </Button>
          <Button className="flex-1" onClick={handleSave} disabled={isSaving}>
            {isSaving ? 'Saving...' : isEditMode ? 'Update Product' : 'Create Product'}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default ProductForm;

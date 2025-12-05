// ProductCreationWizard - Multi-step wizard for creating products from templates
// Supports two modes:
// - 'template': Create new product from template (abstract attributes only)
// - 'clone': Clone existing product (copies concrete attribute values)

import { useState, useEffect, useMemo, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { api } from '@/services/api';
import type {
  ProductTemplate,
  TemplateAbstractAttribute,
  DataType,
  Product,
} from '@/types';
import {
  X,
  ChevronRight,
  ChevronLeft,
  Check,
  Package,
  Layers,
  Database,
  List,
  Zap,
  GitBranch,
  AlertCircle,
  Lock,
  Unlock,
  CheckCircle2,
  Circle,
  FileText,
  Copy,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

type WizardMode = 'template' | 'clone';

interface ProductCreationWizardProps {
  mode?: WizardMode;
  sourceProduct?: Product; // Required for clone mode
  onComplete: (productId: string) => void;
  onCancel: () => void;
}

interface WizardState {
  step: number;
  templateId: string | null;
  productInfo: {
    id: string;
    name: string;
    description: string;
    effectiveFrom: string;
    expiryAt: string;
  };
  selectedComponents: Set<string>;
  selectedDatatypes: Set<string>;
  selectedEnumerations: Set<string>;
  selectedFunctionalities: Set<string>;
  selectedAbstractAttributes: Set<string>;
}

const WIZARD_STEPS = [
  { id: 1, name: 'Template & Info', icon: Package, description: 'Select template and enter product details' },
  { id: 2, name: 'Components', icon: Layers, description: 'Choose which components to include' },
  { id: 3, name: 'Datatypes', icon: Database, description: 'Select datatypes for your product' },
  { id: 4, name: 'Enumerations', icon: List, description: 'Choose enumerations to include' },
  { id: 5, name: 'Functionalities', icon: Zap, description: 'Select product functionalities' },
  { id: 6, name: 'Attributes', icon: GitBranch, description: 'Choose abstract attributes' },
  { id: 7, name: 'Review', icon: CheckCircle2, description: 'Review and create product' },
];

// =============================================================================
// HELPER COMPONENTS
// =============================================================================

function SelectableCard({
  selected,
  required,
  disabled,
  onClick,
  children,
}: {
  selected: boolean;
  required?: boolean;
  disabled?: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <div
      onClick={disabled ? undefined : onClick}
      className={`
        relative p-4 rounded-lg border-2 transition-all cursor-pointer
        ${selected ? 'border-primary bg-primary/5' : 'border-gray-200 hover:border-gray-300'}
        ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
        ${required ? 'ring-1 ring-amber-200' : ''}
      `}
    >
      <div className="absolute top-2 right-2 flex items-center gap-1">
        {required && (
          <span className="text-xs bg-amber-100 text-amber-700 px-1.5 py-0.5 rounded">Required</span>
        )}
        {selected ? (
          <CheckCircle2 className="h-5 w-5 text-primary" />
        ) : (
          <Circle className="h-5 w-5 text-gray-300" />
        )}
      </div>
      {children}
    </div>
  );
}

function WizardProgress({ currentStep, steps }: { currentStep: number; steps: typeof WIZARD_STEPS }) {
  return (
    <div className="flex items-center justify-between mb-8">
      {steps.map((step, idx) => {
        const isActive = step.id === currentStep;
        const isCompleted = step.id < currentStep;
        const Icon = step.icon;

        return (
          <div key={step.id} className="flex items-center">
            <div className="flex flex-col items-center">
              <div
                className={`
                  w-10 h-10 rounded-full flex items-center justify-center border-2 transition-all
                  ${isCompleted ? 'bg-primary border-primary text-white' : ''}
                  ${isActive ? 'border-primary text-primary' : ''}
                  ${!isActive && !isCompleted ? 'border-gray-300 text-gray-400' : ''}
                `}
              >
                {isCompleted ? <Check className="h-5 w-5" /> : <Icon className="h-5 w-5" />}
              </div>
              <span className={`text-xs mt-1 ${isActive ? 'text-primary font-medium' : 'text-gray-500'}`}>
                {step.name}
              </span>
            </div>
            {idx < steps.length - 1 && (
              <div
                className={`w-12 h-0.5 mx-2 ${
                  step.id < currentStep ? 'bg-primary' : 'bg-gray-200'
                }`}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function ProductCreationWizard({
  mode = 'template',
  sourceProduct,
  onComplete,
  onCancel
}: ProductCreationWizardProps) {
  const isCloneMode = mode === 'clone';
  const [templates, setTemplates] = useState<ProductTemplate[]>([]);
  const [datatypes, setDatatypes] = useState<DataType[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize state based on mode
  const getInitialState = (): WizardState => {
    if (isCloneMode && sourceProduct) {
      // Clone mode: pre-populate from source product
      // Note: templateId will be set in useEffect once templates are loaded
      return {
        step: 1,
        templateId: null, // Will be set once templates are loaded
        productInfo: {
          id: `${sourceProduct.id}_v2`,
          name: `${sourceProduct.name} (v2)`,
          description: sourceProduct.description,
          effectiveFrom: new Date().toISOString().split('T')[0],
          expiryAt: '',
        },
        selectedComponents: new Set<string>(),
        selectedDatatypes: new Set<string>(),
        selectedEnumerations: new Set<string>(),
        selectedFunctionalities: new Set<string>(),
        selectedAbstractAttributes: new Set<string>(),
      };
    }
    // Template mode: empty initial state
    return {
      step: 1,
      templateId: null,
      productInfo: {
        id: '',
        name: '',
        description: '',
        effectiveFrom: new Date().toISOString().split('T')[0],
        expiryAt: '',
      },
      selectedComponents: new Set(),
      selectedDatatypes: new Set(),
      selectedEnumerations: new Set(),
      selectedFunctionalities: new Set(),
      selectedAbstractAttributes: new Set(),
    };
  };

  const [state, setState] = useState<WizardState>(getInitialState);

  // In clone mode, auto-select the template matching source product's templateType
  useEffect(() => {
    if (isCloneMode && sourceProduct && templates.length > 0 && !state.templateId) {
      const matchingTemplate = templates.find(t => t.type === sourceProduct.templateType);
      if (matchingTemplate) {
        setState(prev => ({ ...prev, templateId: matchingTemplate.id }));
      }
    }
  }, [isCloneMode, sourceProduct, templates, state.templateId]);

  const selectedTemplate = useMemo(() => {
    return templates.find((t) => t.id === state.templateId) || null;
  }, [templates, state.templateId]);

  // Load data
  useEffect(() => {
    async function loadData() {
      try {
        const [templatesData, datatypesData] = await Promise.all([
          api.templates.listProductTemplates(),
          api.datatypes.list(),
        ]);
        setTemplates(templatesData);
        setDatatypes(datatypesData);
      } catch (e) {
        setError((e as Error).message);
      } finally {
        setIsLoading(false);
      }
    }
    loadData();
  }, []);

  // Initialize selections when template is selected
  useEffect(() => {
    if (!selectedTemplate) return;

    if (isCloneMode && sourceProduct) {
      // Clone mode: select ALL items from source product
      // In clone mode, we want to preserve the source product's configuration
      const components = new Set(
        selectedTemplate.components.map((c) => c.id)
      );
      const dts = new Set(
        selectedTemplate.datatypes.map((d) => d.datatypeId)
      );
      const enums = new Set(
        selectedTemplate.enumerations.map((e) => e.id)
      );
      const funcs = new Set(
        selectedTemplate.functionalities.map((f) => f.id)
      );
      const attrs = new Set(
        selectedTemplate.abstractAttributes.map((a) => a.name)
      );

      setState((prev) => ({
        ...prev,
        selectedComponents: components,
        selectedDatatypes: dts,
        selectedEnumerations: enums,
        selectedFunctionalities: funcs,
        selectedAbstractAttributes: attrs,
      }));
    } else {
      // Template mode: select required and default included
      const components = new Set(
        selectedTemplate.components
          .filter((c) => c.isRequired)
          .map((c) => c.id)
      );

      const dts = new Set(
        selectedTemplate.datatypes
          .filter((d) => d.isRequired || d.defaultIncluded)
          .map((d) => d.datatypeId)
      );

      const enums = new Set(selectedTemplate.enumerations.map((e) => e.id));

      const funcs = new Set(
        selectedTemplate.functionalities
          .filter((f) => f.isRequired || f.defaultIncluded)
          .map((f) => f.id)
      );

      const attrs = new Set(
        selectedTemplate.abstractAttributes
          .filter((a) => a.isRequired || a.defaultIncluded)
          .map((a) => a.name)
      );

      setState((prev) => ({
        ...prev,
        selectedComponents: components,
        selectedDatatypes: dts,
        selectedEnumerations: enums,
        selectedFunctionalities: funcs,
        selectedAbstractAttributes: attrs,
      }));
    }
  }, [selectedTemplate, isCloneMode, sourceProduct]);

  // Toggle selection helper
  const toggleSelection = useCallback(
    (
      key: 'selectedComponents' | 'selectedDatatypes' | 'selectedEnumerations' | 'selectedFunctionalities' | 'selectedAbstractAttributes',
      id: string,
      isRequired: boolean
    ) => {
      if (isRequired) return; // Can't toggle required items
      setState((prev) => {
        const newSet = new Set(prev[key]);
        if (newSet.has(id)) {
          newSet.delete(id);
        } else {
          newSet.add(id);
        }
        return { ...prev, [key]: newSet };
      });
    },
    []
  );

  // Navigation
  const goNext = () => setState((prev) => ({ ...prev, step: Math.min(prev.step + 1, 7) }));
  const goBack = () => setState((prev) => ({ ...prev, step: Math.max(prev.step - 1, 1) }));

  // Validation
  const canProceed = useMemo(() => {
    switch (state.step) {
      case 1:
        // In clone mode, template is auto-selected from source product
        const hasTemplate = isCloneMode ? !!sourceProduct : state.templateId !== null;
        return (
          hasTemplate &&
          state.productInfo.id.trim() !== '' &&
          state.productInfo.name.trim() !== '' &&
          /^[a-z][a-z0-9_]*$/.test(state.productInfo.id)
        );
      case 2:
        return state.selectedComponents.size > 0;
      case 3:
        return state.selectedDatatypes.size > 0;
      case 4:
        return true; // Enumerations are optional
      case 5:
        return true; // Functionalities are optional (but recommended)
      case 6:
        return state.selectedAbstractAttributes.size > 0;
      case 7:
        return true;
      default:
        return false;
    }
  }, [state]);

  // Create product (or clone in clone mode)
  const handleCreate = async () => {
    if (!selectedTemplate) return;
    setIsCreating(true);
    setError(null);

    try {
      if (isCloneMode && sourceProduct) {
        // Clone mode: clone the product with concrete attribute values
        const result = await api.products.clone({
          sourceProductId: sourceProduct.id,
          newProductId: state.productInfo.id,
          newProductName: state.productInfo.name,
          newProductDescription: state.productInfo.description,
          // Pass selected items to allow customization during clone
          selectedComponents: Array.from(state.selectedComponents),
          selectedDatatypes: Array.from(state.selectedDatatypes),
          selectedEnumerations: Array.from(state.selectedEnumerations),
          selectedFunctionalities: Array.from(state.selectedFunctionalities),
          selectedAbstractAttributes: Array.from(state.selectedAbstractAttributes),
        });
        onComplete(result.newProductId);
      } else {
        // Template mode: create from template (abstract attributes only)
        const result = await api.templates.createProductFromTemplate(
          selectedTemplate.id,
          {
            id: state.productInfo.id,
            name: state.productInfo.name,
            description: state.productInfo.description,
            effectiveFrom: new Date(state.productInfo.effectiveFrom).getTime(),
            expiryAt: state.productInfo.expiryAt
              ? new Date(state.productInfo.expiryAt).getTime()
              : undefined,
          },
          Array.from(state.selectedComponents),
          Array.from(state.selectedDatatypes),
          Array.from(state.selectedEnumerations),
          Array.from(state.selectedFunctionalities),
          Array.from(state.selectedAbstractAttributes)
        );
        onComplete(result.product.id);
      }
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setIsCreating(false);
    }
  };

  // =============================================
  // STEP RENDERERS
  // =============================================

  const renderStep1 = () => (
    <div className="space-y-6">
      {isCloneMode && sourceProduct ? (
        // Clone mode: show source product info instead of template selection
        <div>
          <h3 className="text-lg font-semibold mb-4">Cloning Product</h3>
          <div className="p-4 bg-gradient-to-r from-indigo-50 to-purple-50 border border-indigo-200 rounded-lg">
            <div className="flex items-center gap-4">
              <div className="p-3 bg-white rounded-lg shadow-sm">
                <Copy className="h-6 w-6 text-indigo-500" />
              </div>
              <div className="flex-1">
                <p className="text-xs text-gray-500 uppercase tracking-wide">Source Product</p>
                <h4 className="font-semibold text-lg">{sourceProduct.name}</h4>
                <p className="text-sm text-gray-600">{sourceProduct.description}</p>
                <div className="flex gap-2 mt-2">
                  <span className="text-xs bg-white px-2 py-0.5 rounded border">
                    v{sourceProduct.version}
                  </span>
                  <span className="text-xs bg-indigo-100 text-indigo-700 px-2 py-0.5 rounded">
                    {sourceProduct.status}
                  </span>
                </div>
              </div>
            </div>
            <div className="mt-4 pt-4 border-t border-indigo-200">
              <p className="text-sm text-indigo-700">
                <strong>Note:</strong> Cloning will copy all concrete attribute values from the source product.
                You can customize which components and attributes to include in the following steps.
              </p>
            </div>
          </div>
        </div>
      ) : (
        // Template mode: show template selection
        <div>
          <h3 className="text-lg font-semibold mb-4">Select a Product Template</h3>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {templates.map((template) => (
              <SelectableCard
                key={template.id}
                selected={state.templateId === template.id}
                onClick={() => setState((prev) => ({ ...prev, templateId: template.id }))}
              >
                <div className="pr-8">
                  <h4 className="font-medium">{template.name}</h4>
                  <p className="text-sm text-gray-500 mt-1">{template.description}</p>
                  <div className="flex flex-wrap gap-1 mt-2">
                    <span className="text-xs bg-gray-100 px-2 py-0.5 rounded">{template.type}</span>
                    <span className="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded">
                      v{template.version}
                    </span>
                  </div>
                  <div className="flex gap-2 mt-2 text-xs text-gray-500">
                    <span>{template.components.length} components</span>
                    <span>{template.abstractAttributes.length} attributes</span>
                  </div>
                </div>
              </SelectableCard>
            ))}
          </div>
        </div>
      )}

      {(state.templateId || (isCloneMode && sourceProduct)) && (
        <div className={isCloneMode ? '' : 'border-t pt-6'}>
          <h3 className="text-lg font-semibold mb-4">
            {isCloneMode ? 'New Product Information' : 'Product Information'}
          </h3>
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <label className="block text-sm font-medium mb-1">
                Product ID <span className="text-red-500">*</span>
              </label>
              <Input
                value={state.productInfo.id}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    productInfo: { ...prev.productInfo, id: e.target.value.toLowerCase() },
                  }))
                }
                placeholder="e.g., motor_insurance_v2"
                className={
                  state.productInfo.id && !/^[a-z][a-z0-9_]*$/.test(state.productInfo.id)
                    ? 'border-red-300'
                    : ''
                }
              />
              <p className="text-xs text-gray-500 mt-1">
                Lowercase letters, numbers, and underscores. Must start with a letter.
              </p>
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">
                Product Name <span className="text-red-500">*</span>
              </label>
              <Input
                value={state.productInfo.name}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    productInfo: { ...prev.productInfo, name: e.target.value },
                  }))
                }
                placeholder="e.g., Motor Insurance V2"
              />
            </div>
            <div className="md:col-span-2">
              <label className="block text-sm font-medium mb-1">Description</label>
              <Input
                value={state.productInfo.description}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    productInfo: { ...prev.productInfo, description: e.target.value },
                  }))
                }
                placeholder="Brief description of the product"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">
                Effective From <span className="text-red-500">*</span>
              </label>
              <Input
                type="date"
                value={state.productInfo.effectiveFrom}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    productInfo: { ...prev.productInfo, effectiveFrom: e.target.value },
                  }))
                }
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Expiry Date (optional)</label>
              <Input
                type="date"
                value={state.productInfo.expiryAt}
                onChange={(e) =>
                  setState((prev) => ({
                    ...prev,
                    productInfo: { ...prev.productInfo, expiryAt: e.target.value },
                  }))
                }
              />
            </div>
          </div>
        </div>
      )}
    </div>
  );

  const renderStep2 = () => {
    if (!selectedTemplate) return null;

    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Select Components</h3>
            <p className="text-sm text-gray-500">
              Components are logical groupings of attributes. Required components cannot be deselected.
            </p>
          </div>
          <div className="text-sm text-gray-500">
            {state.selectedComponents.size} of {selectedTemplate.components.length} selected
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {selectedTemplate.components
            .sort((a, b) => a.orderIndex - b.orderIndex)
            .map((component) => {
              const attrCount = selectedTemplate.abstractAttributes.filter(
                (a) => a.componentId === component.id
              ).length;

              return (
                <SelectableCard
                  key={component.id}
                  selected={state.selectedComponents.has(component.id)}
                  required={component.isRequired}
                  disabled={component.isRequired}
                  onClick={() => toggleSelection('selectedComponents', component.id, component.isRequired)}
                >
                  <div className="pr-8">
                    <div className="flex items-center gap-2">
                      {component.isRequired ? (
                        <Lock className="h-4 w-4 text-amber-500" />
                      ) : (
                        <Unlock className="h-4 w-4 text-gray-400" />
                      )}
                      <h4 className="font-medium">{component.displayName}</h4>
                    </div>
                    <p className="text-sm text-gray-500 mt-1">{component.description}</p>
                    <p className="text-xs text-gray-400 mt-2">{attrCount} attributes</p>
                  </div>
                </SelectableCard>
              );
            })}
        </div>
      </div>
    );
  };

  const renderStep3 = () => {
    if (!selectedTemplate) return null;

    const templateDatatypeIds = new Set(selectedTemplate.datatypes.map((d) => d.datatypeId));
    const availableDatatypes = datatypes.filter((d) => templateDatatypeIds.has(d.id));

    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Select Datatypes</h3>
            <p className="text-sm text-gray-500">
              Datatypes define the data format for attributes. Required datatypes cannot be deselected.
            </p>
          </div>
          <div className="text-sm text-gray-500">
            {state.selectedDatatypes.size} of {availableDatatypes.length} selected
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {availableDatatypes.map((dt) => {
            const templateDt = selectedTemplate.datatypes.find((d) => d.datatypeId === dt.id);
            const isRequired = templateDt?.isRequired ?? false;

            return (
              <SelectableCard
                key={dt.id}
                selected={state.selectedDatatypes.has(dt.id)}
                required={isRequired}
                disabled={isRequired}
                onClick={() => toggleSelection('selectedDatatypes', dt.id, isRequired)}
              >
                <div className="pr-8">
                  <div className="flex items-center gap-2">
                    <Database className="h-4 w-4 text-blue-500" />
                    <h4 className="font-medium">{dt.name}</h4>
                  </div>
                  <p className="text-sm text-gray-500 mt-1">{dt.description}</p>
                  <span className="inline-block text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded mt-2">
                    {dt.primitiveType}
                  </span>
                </div>
              </SelectableCard>
            );
          })}
        </div>
      </div>
    );
  };

  const renderStep4 = () => {
    if (!selectedTemplate) return null;

    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Select Enumerations</h3>
            <p className="text-sm text-gray-500">
              Enumerations provide predefined value lists for attributes.
            </p>
          </div>
          <div className="text-sm text-gray-500">
            {state.selectedEnumerations.size} of {selectedTemplate.enumerations.length} selected
          </div>
        </div>

        {selectedTemplate.enumerations.length === 0 ? (
          <Card>
            <CardContent className="py-8 text-center text-gray-500">
              <List className="h-8 w-8 mx-auto mb-2 opacity-50" />
              This template has no enumerations defined.
            </CardContent>
          </Card>
        ) : (
          <div className="grid gap-4 md:grid-cols-2">
            {selectedTemplate.enumerations.map((enumeration) => (
              <SelectableCard
                key={enumeration.id}
                selected={state.selectedEnumerations.has(enumeration.id)}
                onClick={() => toggleSelection('selectedEnumerations', enumeration.id, false)}
              >
                <div className="pr-8">
                  <div className="flex items-center gap-2">
                    <List className="h-4 w-4 text-purple-500" />
                    <h4 className="font-medium">{enumeration.name}</h4>
                  </div>
                  <p className="text-sm text-gray-500 mt-1">{enumeration.description}</p>
                  <div className="flex flex-wrap gap-1 mt-2">
                    {enumeration.values.slice(0, 4).map((v) => (
                      <span key={v} className="text-xs bg-purple-100 text-purple-700 px-2 py-0.5 rounded">
                        {v}
                      </span>
                    ))}
                    {enumeration.values.length > 4 && (
                      <span className="text-xs text-gray-400">+{enumeration.values.length - 4} more</span>
                    )}
                  </div>
                </div>
              </SelectableCard>
            ))}
          </div>
        )}
      </div>
    );
  };

  const renderStep5 = () => {
    if (!selectedTemplate) return null;

    // Filter functionalities based on selected components
    const availableFunctionalities = selectedTemplate.functionalities.filter((f) =>
      f.requiredComponents.every((c) => state.selectedComponents.has(c))
    );

    const unavailableFunctionalities = selectedTemplate.functionalities.filter(
      (f) => !f.requiredComponents.every((c) => state.selectedComponents.has(c))
    );

    return (
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Select Functionalities</h3>
            <p className="text-sm text-gray-500">
              Functionalities are groups of related rules and attributes that provide specific capabilities.
            </p>
          </div>
          <div className="text-sm text-gray-500">
            {state.selectedFunctionalities.size} of {availableFunctionalities.length} available selected
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-2">
          {availableFunctionalities.map((func) => (
            <SelectableCard
              key={func.id}
              selected={state.selectedFunctionalities.has(func.id)}
              required={func.isRequired}
              disabled={func.isRequired}
              onClick={() => toggleSelection('selectedFunctionalities', func.id, func.isRequired)}
            >
              <div className="pr-8">
                <div className="flex items-center gap-2">
                  <Zap className="h-4 w-4 text-yellow-500" />
                  <h4 className="font-medium">{func.displayName}</h4>
                </div>
                <p className="text-sm text-gray-500 mt-1">{func.description}</p>
                <div className="flex flex-wrap gap-1 mt-2">
                  {func.requiredComponents.map((c) => (
                    <span key={c} className="text-xs bg-gray-100 px-2 py-0.5 rounded">
                      {c}
                    </span>
                  ))}
                </div>
                <p className="text-xs text-gray-400 mt-2">
                  {func.requiredAbstractAttributes.length} required attributes
                </p>
              </div>
            </SelectableCard>
          ))}
        </div>

        {unavailableFunctionalities.length > 0 && (
          <div className="border-t pt-4 mt-4">
            <h4 className="text-sm font-medium text-gray-500 mb-2">
              Unavailable (missing required components)
            </h4>
            <div className="grid gap-4 md:grid-cols-2 opacity-50">
              {unavailableFunctionalities.map((func) => (
                <div key={func.id} className="p-4 rounded-lg border border-gray-200 bg-gray-50">
                  <div className="flex items-center gap-2">
                    <Zap className="h-4 w-4 text-gray-400" />
                    <h4 className="font-medium text-gray-500">{func.displayName}</h4>
                  </div>
                  <p className="text-sm text-gray-400 mt-1">
                    Requires: {func.requiredComponents.join(', ')}
                  </p>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    );
  };

  const renderStep6 = () => {
    if (!selectedTemplate) return null;

    // Group attributes by component
    const attrsByComponent = new Map<string, TemplateAbstractAttribute[]>();
    selectedTemplate.abstractAttributes.forEach((attr) => {
      if (!state.selectedComponents.has(attr.componentId)) return;
      const existing = attrsByComponent.get(attr.componentId) || [];
      existing.push(attr);
      attrsByComponent.set(attr.componentId, existing);
    });

    const totalAvailable = Array.from(attrsByComponent.values()).flat().length;
    const requiredCount = Array.from(attrsByComponent.values()).flat().filter(a => a.isRequired).length;
    const optionalCount = totalAvailable - requiredCount;

    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h3 className="text-lg font-semibold">Select Abstract Attributes</h3>
            <p className="text-sm text-gray-500">
              Abstract attributes define the schema for your product. Select which attributes to include.
            </p>
          </div>
          <div className="text-sm text-gray-500">
            {state.selectedAbstractAttributes.size} of {totalAvailable} selected
          </div>
        </div>

        {/* Legend for attribute distinction */}
        <div className="flex flex-wrap gap-4 p-3 bg-gray-50 rounded-lg border">
          <div className="flex items-center gap-2 text-sm">
            <div className="flex items-center gap-1 px-2 py-1 bg-amber-50 border border-amber-300 rounded">
              <Lock className="h-3 w-3 text-amber-600" />
              <span className="text-amber-700 font-medium">Required</span>
            </div>
            <span className="text-gray-600">{requiredCount} must be included</span>
          </div>
          <div className="flex items-center gap-2 text-sm">
            <div className="flex items-center gap-1 px-2 py-1 bg-gray-100 border border-gray-300 rounded">
              <Circle className="h-3 w-3 text-gray-500" />
              <span className="text-gray-700 font-medium">Optional</span>
            </div>
            <span className="text-gray-600">{optionalCount} can be toggled</span>
          </div>
        </div>

        {Array.from(attrsByComponent.entries())
          .sort(([a], [b]) => {
            const compA = selectedTemplate.components.find((c) => c.id === a);
            const compB = selectedTemplate.components.find((c) => c.id === b);
            return (compA?.orderIndex ?? 0) - (compB?.orderIndex ?? 0);
          })
          .map(([componentId, attrs]) => {
            const component = selectedTemplate.components.find((c) => c.id === componentId);
            const inputAttrs = attrs.filter((a) => a.isInput);
            const outputAttrs = attrs.filter((a) => !a.isInput);
            const selectedCount = attrs.filter((a) =>
              state.selectedAbstractAttributes.has(a.name)
            ).length;

            return (
              <Card key={componentId}>
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-base flex items-center gap-2">
                      <Layers className="h-4 w-4" />
                      {component?.displayName || componentId}
                    </CardTitle>
                    <span className="text-sm text-gray-500">
                      {selectedCount} / {attrs.length} selected
                    </span>
                  </div>
                  <CardDescription>{component?.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  {inputAttrs.length > 0 && (
                    <div className="mb-4">
                      <h5 className="text-sm font-medium text-gray-600 mb-2 flex items-center gap-1">
                        <span className="w-2 h-2 bg-blue-500 rounded-full" />
                        Input Attributes
                        <span className="text-xs text-gray-400 ml-1">
                          ({inputAttrs.filter(a => a.isRequired).length} required, {inputAttrs.filter(a => !a.isRequired).length} optional)
                        </span>
                      </h5>
                      <div className="grid gap-2 md:grid-cols-2">
                        {/* Required attributes first, then optional */}
                        {[...inputAttrs.filter(a => a.isRequired), ...inputAttrs.filter(a => !a.isRequired)].map((attr) => (
                          <div
                            key={attr.name}
                            onClick={() =>
                              toggleSelection('selectedAbstractAttributes', attr.name, attr.isRequired)
                            }
                            className={`
                              flex items-start gap-2 p-2 rounded border-2 transition-all
                              ${attr.isRequired
                                ? 'border-amber-300 bg-amber-50/50 cursor-not-allowed'
                                : state.selectedAbstractAttributes.has(attr.name)
                                  ? 'border-primary bg-primary/5 cursor-pointer'
                                  : 'border-gray-200 hover:border-gray-300 cursor-pointer'
                              }
                            `}
                          >
                            {attr.isRequired ? (
                              <Lock className="h-4 w-4 text-amber-600 mt-0.5 shrink-0" />
                            ) : state.selectedAbstractAttributes.has(attr.name) ? (
                              <CheckCircle2 className="h-4 w-4 text-primary mt-0.5 shrink-0" />
                            ) : (
                              <Circle className="h-4 w-4 text-gray-300 mt-0.5 shrink-0" />
                            )}
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-1 flex-wrap">
                                <span className="font-mono text-sm">{attr.name}</span>
                                {attr.isRequired && (
                                  <span className="text-xs bg-amber-100 text-amber-700 px-1.5 py-0.5 rounded font-medium">
                                    Required
                                  </span>
                                )}
                              </div>
                              <p className="text-xs text-gray-500 truncate">{attr.description}</p>
                              <div className="flex gap-1 mt-1 flex-wrap">
                                <span className="text-xs bg-blue-100 text-blue-700 px-1 rounded">
                                  {attr.datatypeId}
                                </span>
                                {attr.enumName && (
                                  <span className="text-xs bg-purple-100 text-purple-700 px-1 rounded">
                                    {attr.enumName}
                                  </span>
                                )}
                              </div>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  {outputAttrs.length > 0 && (
                    <div>
                      <h5 className="text-sm font-medium text-gray-600 mb-2 flex items-center gap-1">
                        <span className="w-2 h-2 bg-green-500 rounded-full" />
                        Computed Attributes
                        <span className="text-xs text-gray-400 ml-1">
                          ({outputAttrs.filter(a => a.isRequired).length} required, {outputAttrs.filter(a => !a.isRequired).length} optional)
                        </span>
                      </h5>
                      <div className="grid gap-2 md:grid-cols-2">
                        {/* Required attributes first, then optional */}
                        {[...outputAttrs.filter(a => a.isRequired), ...outputAttrs.filter(a => !a.isRequired)].map((attr) => (
                          <div
                            key={attr.name}
                            onClick={() =>
                              toggleSelection('selectedAbstractAttributes', attr.name, attr.isRequired)
                            }
                            className={`
                              flex items-start gap-2 p-2 rounded border-2 transition-all
                              ${attr.isRequired
                                ? 'border-amber-300 bg-amber-50/50 cursor-not-allowed'
                                : state.selectedAbstractAttributes.has(attr.name)
                                  ? 'border-primary bg-primary/5 cursor-pointer'
                                  : 'border-gray-200 hover:border-gray-300 cursor-pointer'
                              }
                            `}
                          >
                            {attr.isRequired ? (
                              <Lock className="h-4 w-4 text-amber-600 mt-0.5 shrink-0" />
                            ) : state.selectedAbstractAttributes.has(attr.name) ? (
                              <CheckCircle2 className="h-4 w-4 text-primary mt-0.5 shrink-0" />
                            ) : (
                              <Circle className="h-4 w-4 text-gray-300 mt-0.5 shrink-0" />
                            )}
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-1 flex-wrap">
                                <span className="font-mono text-sm">{attr.name}</span>
                                {attr.isRequired && (
                                  <span className="text-xs bg-amber-100 text-amber-700 px-1.5 py-0.5 rounded font-medium">
                                    Required
                                  </span>
                                )}
                                {attr.immutable && (
                                  <span className="text-xs bg-red-100 text-red-600 px-1 rounded">
                                    immutable
                                  </span>
                                )}
                              </div>
                              <p className="text-xs text-gray-500 truncate">{attr.description}</p>
                              <span className="text-xs bg-green-100 text-green-700 px-1 rounded mt-1 inline-block">
                                {attr.datatypeId}
                              </span>
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })}
      </div>
    );
  };

  const renderStep7 = () => {
    if (!selectedTemplate) return null;

    const selectedCompList = selectedTemplate.components.filter((c) =>
      state.selectedComponents.has(c.id)
    );
    const selectedDtList = datatypes.filter((d) => state.selectedDatatypes.has(d.id));
    const selectedEnumList = selectedTemplate.enumerations.filter((e) =>
      state.selectedEnumerations.has(e.id)
    );
    const selectedFuncList = selectedTemplate.functionalities.filter((f) =>
      state.selectedFunctionalities.has(f.id)
    );
    const selectedAttrList = selectedTemplate.abstractAttributes.filter((a) =>
      state.selectedAbstractAttributes.has(a.name) && state.selectedComponents.has(a.componentId)
    );

    return (
      <div className="space-y-6">
        <div className="text-center pb-4 border-b">
          {isCloneMode ? (
            <>
              <Copy className="h-12 w-12 mx-auto text-indigo-500 mb-2" />
              <h3 className="text-xl font-semibold">Review Your Clone</h3>
              <p className="text-gray-500">Please review your selections before cloning the product.</p>
            </>
          ) : (
            <>
              <CheckCircle2 className="h-12 w-12 mx-auto text-primary mb-2" />
              <h3 className="text-xl font-semibold">Review Your Product</h3>
              <p className="text-gray-500">Please review your selections before creating the product.</p>
            </>
          )}
        </div>

        {/* Clone mode info banner */}
        {isCloneMode && sourceProduct && (
          <div className="p-4 bg-indigo-50 border border-indigo-200 rounded-lg">
            <div className="flex items-start gap-3">
              <Copy className="h-5 w-5 text-indigo-500 mt-0.5" />
              <div>
                <p className="font-medium text-indigo-900">Cloning from: {sourceProduct.name}</p>
                <p className="text-sm text-indigo-700 mt-1">
                  All concrete attribute values and rules will be copied to the new product.
                  The new product will start in <strong>DRAFT</strong> status.
                </p>
              </div>
            </div>
          </div>
        )}

        <div className="grid gap-4 md:grid-cols-2">
          {/* Product Info */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <FileText className="h-4 w-4" />
                Product Information
              </CardTitle>
            </CardHeader>
            <CardContent className="text-sm space-y-2">
              <div className="flex justify-between">
                <span className="text-gray-500">ID:</span>
                <span className="font-mono">{state.productInfo.id}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Name:</span>
                <span>{state.productInfo.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Template:</span>
                <span>{selectedTemplate.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Effective:</span>
                <span>{state.productInfo.effectiveFrom}</span>
              </div>
            </CardContent>
          </Card>

          {/* Summary Stats */}
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <Package className="h-4 w-4" />
                Summary
              </CardTitle>
            </CardHeader>
            <CardContent className="text-sm space-y-2">
              <div className="flex justify-between">
                <span className="text-gray-500">Components:</span>
                <span className="font-medium">{selectedCompList.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Datatypes:</span>
                <span className="font-medium">{selectedDtList.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Enumerations:</span>
                <span className="font-medium">{selectedEnumList.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Functionalities:</span>
                <span className="font-medium">{selectedFuncList.length}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">Attributes:</span>
                <span className="font-medium">{selectedAttrList.length}</span>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Detailed Lists */}
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          <div>
            <h4 className="font-medium mb-2 flex items-center gap-2">
              <Layers className="h-4 w-4" />
              Components ({selectedCompList.length})
            </h4>
            <div className="space-y-1">
              {selectedCompList.map((c) => (
                <div key={c.id} className="text-sm px-2 py-1 bg-gray-50 rounded">
                  {c.displayName}
                </div>
              ))}
            </div>
          </div>

          <div>
            <h4 className="font-medium mb-2 flex items-center gap-2">
              <Zap className="h-4 w-4" />
              Functionalities ({selectedFuncList.length})
            </h4>
            <div className="space-y-1">
              {selectedFuncList.map((f) => (
                <div key={f.id} className="text-sm px-2 py-1 bg-gray-50 rounded">
                  {f.displayName}
                </div>
              ))}
              {selectedFuncList.length === 0 && (
                <div className="text-sm text-gray-400 italic">None selected</div>
              )}
            </div>
          </div>

          <div>
            <h4 className="font-medium mb-2 flex items-center gap-2">
              <List className="h-4 w-4" />
              Enumerations ({selectedEnumList.length})
            </h4>
            <div className="space-y-1">
              {selectedEnumList.map((e) => (
                <div key={e.id} className="text-sm px-2 py-1 bg-gray-50 rounded">
                  {e.name}
                </div>
              ))}
              {selectedEnumList.length === 0 && (
                <div className="text-sm text-gray-400 italic">None selected</div>
              )}
            </div>
          </div>
        </div>

        <div>
          <h4 className="font-medium mb-2 flex items-center gap-2">
            <GitBranch className="h-4 w-4" />
            Attributes ({selectedAttrList.length})
          </h4>
          <div className="grid gap-2 md:grid-cols-3 lg:grid-cols-4">
            {selectedAttrList.map((a) => (
              <div
                key={a.name}
                className="text-sm px-2 py-1 bg-gray-50 rounded flex items-center gap-1"
              >
                <span
                  className={`w-2 h-2 rounded-full ${a.isInput ? 'bg-blue-500' : 'bg-green-500'}`}
                />
                <span className="font-mono truncate">{a.name}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  };

  // =============================================
  // MAIN RENDER
  // =============================================

  if (isLoading) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <Card className="w-full max-w-md">
          <CardContent className="py-12 text-center">
            <div className="animate-spin h-8 w-8 border-4 border-primary border-t-transparent rounded-full mx-auto mb-4" />
            <p className="text-gray-500">Loading templates...</p>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <Card className="w-full max-w-5xl max-h-[90vh] flex flex-col">
        <CardHeader className="border-b shrink-0">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                {isCloneMode ? (
                  <>
                    <Copy className="h-5 w-5" />
                    Clone Product
                  </>
                ) : (
                  <>
                    <Package className="h-5 w-5" />
                    Create Product from Template
                  </>
                )}
              </CardTitle>
              <CardDescription>
                {WIZARD_STEPS[state.step - 1]?.description}
              </CardDescription>
            </div>
            <Button variant="ghost" size="icon" onClick={onCancel}>
              <X className="h-4 w-4" />
            </Button>
          </div>
          <WizardProgress currentStep={state.step} steps={WIZARD_STEPS} />
        </CardHeader>

        <CardContent className="flex-1 overflow-y-auto py-6">
          {error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg flex items-center gap-2 text-red-700">
              <AlertCircle className="h-4 w-4 shrink-0" />
              {error}
            </div>
          )}

          {state.step === 1 && renderStep1()}
          {state.step === 2 && renderStep2()}
          {state.step === 3 && renderStep3()}
          {state.step === 4 && renderStep4()}
          {state.step === 5 && renderStep5()}
          {state.step === 6 && renderStep6()}
          {state.step === 7 && renderStep7()}
        </CardContent>

        <div className="border-t p-4 flex justify-between shrink-0">
          <Button variant="outline" onClick={state.step === 1 ? onCancel : goBack} disabled={isCreating}>
            {state.step === 1 ? (
              'Cancel'
            ) : (
              <>
                <ChevronLeft className="h-4 w-4 mr-1" />
                Back
              </>
            )}
          </Button>

          {state.step < 7 ? (
            <Button onClick={goNext} disabled={!canProceed}>
              Next
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          ) : (
            <Button onClick={handleCreate} disabled={isCreating || !canProceed}>
              {isCreating ? (
                <>
                  <div className="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full mr-2" />
                  {isCloneMode ? 'Cloning...' : 'Creating...'}
                </>
              ) : (
                <>
                  {isCloneMode ? <Copy className="h-4 w-4 mr-1" /> : <Check className="h-4 w-4 mr-1" />}
                  {isCloneMode ? 'Clone Product' : 'Create Product'}
                </>
              )}
            </Button>
          )}
        </div>
      </Card>
    </div>
  );
}

export default ProductCreationWizard;

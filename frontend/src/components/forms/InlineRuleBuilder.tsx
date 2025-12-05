// InlineRuleBuilder - Create rules inline during attribute creation
// Supports Quick Expression mode and Advanced Builder modal

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Wand2,
  Calculator,
  Blocks,
  Plus,
  X,
  ChevronRight,
  ArrowLeft,
  Play,
  CheckCircle2,
  AlertCircle,
} from 'lucide-react';
import type { AbstractAttribute, Rule } from '@/types';

interface InlineRuleBuilderProps {
  abstractAttribute: AbstractAttribute;
  productId: string;
  availableInputs: AbstractAttribute[];
  onRuleCreated: (rule: Rule) => void;
  onOpenAdvancedBuilder?: () => void;
}

type BuilderMode = 'select' | 'quick' | 'advanced';

interface QuickExpressionState {
  name: string;
  expressionType: 'calculation' | 'condition' | 'mapping';
  expression: string;
  inputAttributes: string[];
  testResult: string | null;
  isValid: boolean;
}

// =============================================================================
// QUICK EXPRESSION BUILDER
// =============================================================================

function QuickExpressionBuilder({
  abstractAttribute,
  availableInputs,
  onRuleCreated,
  onBack,
}: {
  abstractAttribute: AbstractAttribute;
  availableInputs: AbstractAttribute[];
  onRuleCreated: (rule: Rule) => void;
  onBack: () => void;
}) {
  const [state, setState] = useState<QuickExpressionState>({
    name: `calc-${abstractAttribute.attributeName}`,
    expressionType: 'calculation',
    expression: '',
    inputAttributes: [],
    testResult: null,
    isValid: false,
  });
  const [isCreating, setIsCreating] = useState(false);

  const expressionTypes = [
    {
      value: 'calculation',
      label: 'Calculation',
      description: 'Math operations (+, -, *, /)',
      example: 'premium * 0.18',
    },
    {
      value: 'condition',
      label: 'Condition',
      description: 'If-then logic',
      example: 'if age >= 18 then "adult" else "minor"',
    },
    {
      value: 'mapping',
      label: 'Mapping',
      description: 'Direct value mapping',
      example: 'category => rate lookup',
    },
  ] as const;

  const updateState = (updates: Partial<QuickExpressionState>) => {
    setState((prev) => ({ ...prev, ...updates }));
  };

  const addInputAttribute = (attrPath: string) => {
    if (!state.inputAttributes.includes(attrPath)) {
      updateState({ inputAttributes: [...state.inputAttributes, attrPath] });
    }
  };

  const removeInputAttribute = (attrPath: string) => {
    updateState({
      inputAttributes: state.inputAttributes.filter((a) => a !== attrPath),
    });
  };

  const insertIntoExpression = (text: string) => {
    updateState({ expression: state.expression + text });
  };

  const testExpression = () => {
    // Simplified test - just validate basic syntax
    try {
      // Very basic validation
      if (state.expression.trim() === '') {
        updateState({ testResult: 'Expression is empty', isValid: false });
        return;
      }

      // Check if all referenced variables are in inputAttributes
      const varPattern = /\$\{([^}]+)\}/g;
      const matches = state.expression.match(varPattern) || [];
      const referencedVars = matches.map((m) => m.slice(2, -1));

      const missingVars = referencedVars.filter(
        (v) => !state.inputAttributes.includes(v)
      );

      if (missingVars.length > 0) {
        updateState({
          testResult: `Missing input attributes: ${missingVars.join(', ')}`,
          isValid: false,
        });
        return;
      }

      updateState({
        testResult: 'Expression is valid!',
        isValid: true,
      });
    } catch (e) {
      updateState({
        testResult: `Error: ${e instanceof Error ? e.message : 'Invalid expression'}`,
        isValid: false,
      });
    }
  };

  const handleCreate = async () => {
    setIsCreating(true);
    try {
      // Convert quick expression to JSON Logic format
      const jsonLogic = convertToJsonLogic(state.expression, state.expressionType);
      const ruleId = `rule-${Date.now()}`;
      const now = Date.now();

      // Create the rule object matching the Rule type
      const newRule: Rule = {
        id: ruleId,
        productId: abstractAttribute.productId,
        ruleType: state.expressionType.toUpperCase(),
        displayExpression: state.expression,
        compiledExpression: JSON.stringify(jsonLogic),
        description: `Auto-generated rule for ${abstractAttribute.attributeName}`,
        enabled: true,
        orderIndex: 0,
        inputAttributes: state.inputAttributes.map((attrPath, idx) => ({
          ruleId,
          attributePath: attrPath,
          orderIndex: idx,
        })),
        outputAttributes: [{
          ruleId,
          attributePath: abstractAttribute.abstractPath,
          orderIndex: 0,
        }],
        createdAt: now,
        updatedAt: now,
      };

      onRuleCreated(newRule);
    } finally {
      setIsCreating(false);
    }
  };

  // Simplified conversion - in production would use proper parser
  const convertToJsonLogic = (
    expr: string,
    _type: 'calculation' | 'condition' | 'mapping'
  ): Record<string, unknown> => {
    // Very basic conversion - just wrap in a var reference for now
    // A real implementation would parse the expression properly
    return {
      var: expr.replace(/\$\{([^}]+)\}/g, '$1'),
    };
  };

  return (
    <div className="space-y-4">
      {/* Header with back button */}
      <div className="flex items-center gap-2 pb-2 border-b">
        <Button variant="ghost" size="sm" onClick={onBack}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div>
          <h4 className="font-medium">Quick Expression Builder</h4>
          <p className="text-xs text-gray-500">
            Create a simple rule for: {abstractAttribute.attributeName}
          </p>
        </div>
      </div>

      {/* Rule Name */}
      <div>
        <label className="text-sm font-medium text-gray-700">Rule Name</label>
        <Input
          value={state.name}
          onChange={(e) => updateState({ name: e.target.value })}
          placeholder="Enter rule name..."
          className="mt-1"
        />
      </div>

      {/* Expression Type */}
      <div>
        <label className="text-sm font-medium text-gray-700">Expression Type</label>
        <div className="grid grid-cols-3 gap-2 mt-1">
          {expressionTypes.map((type) => (
            <button
              key={type.value}
              type="button"
              onClick={() =>
                updateState({ expressionType: type.value as QuickExpressionState['expressionType'] })
              }
              className={`
                p-2 rounded border text-left text-sm transition-all
                ${
                  state.expressionType === type.value
                    ? 'border-primary bg-primary/5'
                    : 'border-gray-200 hover:border-gray-300'
                }
              `}
            >
              <div className="font-medium">{type.label}</div>
              <div className="text-xs text-gray-500">{type.description}</div>
            </button>
          ))}
        </div>
      </div>

      {/* Input Attributes */}
      <div>
        <label className="text-sm font-medium text-gray-700">Input Attributes</label>
        <p className="text-xs text-gray-500 mb-2">
          Select attributes to use in your expression
        </p>

        {/* Selected inputs */}
        {state.inputAttributes.length > 0 && (
          <div className="flex flex-wrap gap-1 mb-2">
            {state.inputAttributes.map((attr) => (
              <span
                key={attr}
                className="inline-flex items-center gap-1 px-2 py-1 bg-blue-100 text-blue-700 rounded text-xs"
              >
                {attr.split(':').pop()}
                <button
                  type="button"
                  onClick={() => removeInputAttribute(attr)}
                  className="hover:text-blue-900"
                >
                  <X className="h-3 w-3" />
                </button>
              </span>
            ))}
          </div>
        )}

        {/* Available inputs */}
        <div className="max-h-32 overflow-y-auto border rounded p-2 space-y-1">
          {availableInputs.length === 0 ? (
            <p className="text-sm text-gray-500 text-center py-2">
              No input attributes available
            </p>
          ) : (
            availableInputs.map((attr) => (
              <button
                key={attr.abstractPath}
                type="button"
                onClick={() => {
                  addInputAttribute(attr.abstractPath);
                  insertIntoExpression(`\${${attr.attributeName}}`);
                }}
                disabled={state.inputAttributes.includes(attr.abstractPath)}
                className={`
                  w-full text-left p-1.5 rounded text-sm
                  ${
                    state.inputAttributes.includes(attr.abstractPath)
                      ? 'bg-gray-100 text-gray-400'
                      : 'hover:bg-gray-100'
                  }
                `}
              >
                <div className="flex items-center justify-between">
                  <span className="font-mono text-xs">{attr.attributeName}</span>
                  <span className="text-xs text-gray-500">{attr.datatypeId}</span>
                </div>
              </button>
            ))
          )}
        </div>
      </div>

      {/* Expression Editor */}
      <div>
        <label className="text-sm font-medium text-gray-700">Expression</label>
        <p className="text-xs text-gray-500 mb-1">
          Use ${'{attributeName}'} to reference attributes
        </p>
        <textarea
          value={state.expression}
          onChange={(e) => updateState({ expression: e.target.value })}
          placeholder={`e.g., \${premium} * 0.18`}
          className="w-full h-20 p-2 border rounded font-mono text-sm resize-none"
        />

        {/* Quick Insert Buttons */}
        <div className="flex gap-1 mt-1">
          {['+', '-', '*', '/', '(', ')'].map((op) => (
            <button
              key={op}
              type="button"
              onClick={() => insertIntoExpression(` ${op} `)}
              className="px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded text-sm font-mono"
            >
              {op}
            </button>
          ))}
        </div>
      </div>

      {/* Test Expression */}
      <div>
        <Button
          variant="outline"
          size="sm"
          onClick={testExpression}
          className="w-full"
        >
          <Play className="h-4 w-4 mr-2" />
          Test Expression
        </Button>

        {state.testResult && (
          <div
            className={`
              mt-2 p-2 rounded text-sm flex items-center gap-2
              ${state.isValid ? 'bg-green-50 text-green-700' : 'bg-red-50 text-red-700'}
            `}
          >
            {state.isValid ? (
              <CheckCircle2 className="h-4 w-4" />
            ) : (
              <AlertCircle className="h-4 w-4" />
            )}
            {state.testResult}
          </div>
        )}
      </div>

      {/* Create Button */}
      <Button
        onClick={handleCreate}
        disabled={!state.isValid || isCreating || !state.name.trim()}
        className="w-full"
      >
        {isCreating ? 'Creating Rule...' : 'Create Rule'}
      </Button>
    </div>
  );
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function InlineRuleBuilder({
  abstractAttribute,
  productId: _productId,
  availableInputs,
  onRuleCreated,
  onOpenAdvancedBuilder,
}: InlineRuleBuilderProps) {
  const [mode, setMode] = useState<BuilderMode>('select');

  if (mode === 'quick') {
    return (
      <QuickExpressionBuilder
        abstractAttribute={abstractAttribute}
        availableInputs={availableInputs}
        onRuleCreated={onRuleCreated}
        onBack={() => setMode('select')}
      />
    );
  }

  return (
    <div className="space-y-4">
      {/* Info Banner */}
      <div className="p-3 bg-blue-50 border border-blue-200 rounded-md">
        <div className="flex items-start gap-2 text-blue-700">
          <Wand2 className="h-5 w-5 mt-0.5 shrink-0" />
          <div>
            <p className="font-medium">Create a Rule for This Attribute</p>
            <p className="text-sm mt-1">
              No existing rules output to <span className="font-mono">{abstractAttribute.attributeName}</span>.
              Create one now or use the Rules page later.
            </p>
          </div>
        </div>
      </div>

      {/* Option Cards */}
      <div className="grid grid-cols-2 gap-3">
        {/* Quick Expression */}
        <button
          type="button"
          onClick={() => setMode('quick')}
          className="p-4 border-2 border-gray-200 rounded-lg hover:border-primary hover:bg-primary/5 transition-all text-left group"
        >
          <div className="flex items-center gap-2 mb-2">
            <div className="p-2 bg-amber-100 rounded-lg">
              <Calculator className="h-5 w-5 text-amber-600" />
            </div>
            <div className="font-medium">Quick Expression</div>
          </div>
          <p className="text-xs text-gray-500 mb-2">
            Simple calculations and mappings
          </p>
          <ul className="text-xs text-gray-600 space-y-1">
            <li className="flex items-center gap-1">
              <Plus className="h-3 w-3" /> Math operations
            </li>
            <li className="flex items-center gap-1">
              <Plus className="h-3 w-3" /> Attribute references
            </li>
          </ul>
          <div className="flex items-center gap-1 mt-3 text-xs text-primary opacity-0 group-hover:opacity-100 transition-opacity">
            <span>Start building</span>
            <ChevronRight className="h-3 w-3" />
          </div>
        </button>

        {/* Advanced Builder */}
        <button
          type="button"
          onClick={onOpenAdvancedBuilder}
          className="p-4 border-2 border-gray-200 rounded-lg hover:border-primary hover:bg-primary/5 transition-all text-left group"
        >
          <div className="flex items-center gap-2 mb-2">
            <div className="p-2 bg-purple-100 rounded-lg">
              <Blocks className="h-5 w-5 text-purple-600" />
            </div>
            <div className="font-medium">Advanced Builder</div>
          </div>
          <p className="text-xs text-gray-500 mb-2">
            Full visual rule editor
          </p>
          <ul className="text-xs text-gray-600 space-y-1">
            <li className="flex items-center gap-1">
              <Plus className="h-3 w-3" /> Complex conditions
            </li>
            <li className="flex items-center gap-1">
              <Plus className="h-3 w-3" /> Multiple cases
            </li>
          </ul>
          <div className="flex items-center gap-1 mt-3 text-xs text-primary opacity-0 group-hover:opacity-100 transition-opacity">
            <span>Open editor</span>
            <ChevronRight className="h-3 w-3" />
          </div>
        </button>
      </div>

      {/* Skip Option */}
      <p className="text-xs text-gray-500 text-center">
        Or create a rule later from the Rules page
      </p>
    </div>
  );
}

export default InlineRuleBuilder;

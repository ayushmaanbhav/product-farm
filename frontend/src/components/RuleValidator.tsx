// RuleValidator - Component for validating rules before saving
// Provides pre-save validation and test evaluation with sample inputs

import { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  Play,
  RefreshCw,
  ChevronDown,
  ChevronRight,
} from 'lucide-react';
import type { Rule, AbstractAttribute } from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface RuleValidatorProps {
  /** Rule to validate */
  rule: Partial<Rule>;
  /** Input attributes for the rule */
  inputAttributes: AbstractAttribute[];
  /** Callback when validation completes */
  onValidationComplete?: (isValid: boolean, errors: ValidationError[]) => void;
}

export interface ValidationError {
  field: string;
  message: string;
  severity: 'error' | 'warning';
}

interface TestInput {
  attributePath: string;
  value: string;
}

interface TestResult {
  success: boolean;
  output?: unknown;
  error?: string;
  executionTimeMs?: number;
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function RuleValidator({
  rule,
  inputAttributes,
  onValidationComplete,
}: RuleValidatorProps) {
  const [isValidating, setIsValidating] = useState(false);
  const [validationErrors, setValidationErrors] = useState<ValidationError[]>([]);
  const [showTestPanel, setShowTestPanel] = useState(false);
  const [testInputs, setTestInputs] = useState<TestInput[]>(() =>
    inputAttributes.map((attr) => ({
      attributePath: attr.abstractPath,
      value: '',
    }))
  );
  const [testResult, setTestResult] = useState<TestResult | null>(null);
  const [isTesting, setIsTesting] = useState(false);

  // Validate rule structure
  const validateRule = useCallback(async () => {
    setIsValidating(true);
    const errors: ValidationError[] = [];

    // Basic validation
    if (!rule.description?.trim()) {
      errors.push({
        field: 'description',
        message: 'Rule description is required',
        severity: 'error',
      });
    }

    if (!rule.compiledExpression) {
      errors.push({
        field: 'compiledExpression',
        message: 'Rule expression is required',
        severity: 'error',
      });
    } else {
      // Validate JSON Logic syntax
      try {
        const parsed = JSON.parse(rule.compiledExpression);
        if (typeof parsed !== 'object' || parsed === null) {
          errors.push({
            field: 'compiledExpression',
            message: 'Expression must be a valid JSON Logic object',
            severity: 'error',
          });
        }
      } catch {
        errors.push({
          field: 'compiledExpression',
          message: 'Expression is not valid JSON',
          severity: 'error',
        });
      }
    }

    if (!rule.ruleType) {
      errors.push({
        field: 'ruleType',
        message: 'Rule type is required',
        severity: 'error',
      });
    }

    // Check for input attributes
    if ((!rule.inputAttributes || rule.inputAttributes.length === 0) && inputAttributes.length > 0) {
      errors.push({
        field: 'inputAttributes',
        message: 'Consider specifying which attributes this rule uses as inputs',
        severity: 'warning',
      });
    }

    // Check for output attributes
    if (!rule.outputAttributes || rule.outputAttributes.length === 0) {
      errors.push({
        field: 'outputAttributes',
        message: 'Rule should specify at least one output attribute',
        severity: 'warning',
      });
    }

    // Simulate API validation delay
    await new Promise((resolve) => setTimeout(resolve, 500));

    setValidationErrors(errors);
    setIsValidating(false);

    const hasErrors = errors.some((e) => e.severity === 'error');
    onValidationComplete?.(!hasErrors, errors);
  }, [rule, inputAttributes, onValidationComplete]);

  // Test rule with sample inputs
  const runTest = useCallback(async () => {
    setIsTesting(true);
    setTestResult(null);

    try {
      // Build input data object
      const inputData: Record<string, unknown> = {};
      testInputs.forEach((input) => {
        const attr = inputAttributes.find((a) => a.abstractPath === input.attributePath);
        if (attr && input.value) {
          // Parse value based on datatype
          let parsedValue: unknown = input.value;
          if (attr.datatypeId.includes('int') || attr.datatypeId.includes('decimal') || attr.datatypeId.includes('float')) {
            parsedValue = parseFloat(input.value);
          } else if (attr.datatypeId.includes('bool')) {
            parsedValue = input.value.toLowerCase() === 'true';
          }
          const attrName = attr.attributeName || attr.abstractPath.split(':').pop() || '';
          inputData[attrName] = parsedValue;
        }
      });

      // Evaluate using the compiled expression
      if (rule.compiledExpression) {
        const startTime = performance.now();
        const expression = JSON.parse(rule.compiledExpression);

        // Simple JSON Logic evaluation (basic ops)
        const result = evaluateJsonLogic(expression, inputData);
        const endTime = performance.now();

        setTestResult({
          success: true,
          output: result,
          executionTimeMs: Math.round(endTime - startTime),
        });
      } else {
        setTestResult({
          success: false,
          error: 'No compiled expression to evaluate',
        });
      }
    } catch (e) {
      setTestResult({
        success: false,
        error: (e as Error).message,
      });
    } finally {
      setIsTesting(false);
    }
  }, [testInputs, inputAttributes, rule.compiledExpression]);

  // Update test input value
  const updateTestInput = (attributePath: string, value: string) => {
    setTestInputs((prev) =>
      prev.map((input) =>
        input.attributePath === attributePath ? { ...input, value } : input
      )
    );
  };

  // Count errors and warnings
  const errorCount = validationErrors.filter((e) => e.severity === 'error').length;
  const warningCount = validationErrors.filter((e) => e.severity === 'warning').length;

  return (
    <div className="border rounded-lg bg-white overflow-hidden">
      {/* Validation Header */}
      <div className="px-4 py-3 border-b bg-gray-50 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">Rule Validation</span>
          {errorCount > 0 && (
            <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-700">
              {errorCount} error{errorCount > 1 ? 's' : ''}
            </span>
          )}
          {warningCount > 0 && (
            <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-700">
              {warningCount} warning{warningCount > 1 ? 's' : ''}
            </span>
          )}
          {validationErrors.length === 0 && errorCount === 0 && warningCount === 0 && (
            <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-700">
              Valid
            </span>
          )}
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={validateRule}
          disabled={isValidating}
          className="gap-1.5"
        >
          {isValidating ? (
            <RefreshCw className="h-3.5 w-3.5 animate-spin" />
          ) : (
            <CheckCircle className="h-3.5 w-3.5" />
          )}
          Validate
        </Button>
      </div>

      {/* Validation Results */}
      {validationErrors.length > 0 && (
        <div className="p-3 space-y-2 border-b">
          {validationErrors.map((error, i) => (
            <div
              key={i}
              className={`flex items-start gap-2 text-sm ${
                error.severity === 'error' ? 'text-red-700' : 'text-amber-700'
              }`}
            >
              {error.severity === 'error' ? (
                <XCircle className="h-4 w-4 mt-0.5 shrink-0" />
              ) : (
                <AlertTriangle className="h-4 w-4 mt-0.5 shrink-0" />
              )}
              <div>
                <span className="font-medium">{error.field}:</span> {error.message}
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Test Panel Toggle */}
      <button
        onClick={() => setShowTestPanel(!showTestPanel)}
        className="w-full px-4 py-2 flex items-center justify-between text-sm font-medium text-gray-700 hover:bg-gray-50"
      >
        <span className="flex items-center gap-2">
          <Play className="h-4 w-4" />
          Test with Sample Inputs
        </span>
        {showTestPanel ? (
          <ChevronDown className="h-4 w-4 text-gray-400" />
        ) : (
          <ChevronRight className="h-4 w-4 text-gray-400" />
        )}
      </button>

      {/* Test Panel */}
      {showTestPanel && (
        <div className="p-4 border-t space-y-4">
          {/* Test Inputs */}
          <div className="space-y-3">
            <label className="text-xs font-medium text-gray-700">Input Values</label>
            {inputAttributes.length === 0 ? (
              <p className="text-sm text-gray-500">No input attributes defined</p>
            ) : (
              <div className="grid gap-2">
                {inputAttributes.map((attr) => {
                  const attrName = attr.attributeName || attr.abstractPath.split(':').pop() || '';
                  const input = testInputs.find((i) => i.attributePath === attr.abstractPath);
                  return (
                    <div key={attr.abstractPath} className="flex items-center gap-2">
                      <label className="text-sm text-gray-600 w-32 truncate" title={attrName}>
                        {attrName}
                      </label>
                      <Input
                        value={input?.value || ''}
                        onChange={(e) => updateTestInput(attr.abstractPath, e.target.value)}
                        placeholder={`Enter ${attr.datatypeId}...`}
                        className="h-8 flex-1"
                      />
                    </div>
                  );
                })}
              </div>
            )}
          </div>

          {/* Run Test Button */}
          <Button
            onClick={runTest}
            disabled={isTesting || !rule.compiledExpression}
            className="w-full gap-2"
          >
            {isTesting ? (
              <>
                <RefreshCw className="h-4 w-4 animate-spin" />
                Evaluating...
              </>
            ) : (
              <>
                <Play className="h-4 w-4" />
                Run Test
              </>
            )}
          </Button>

          {/* Test Result */}
          {testResult && (
            <div
              className={`p-3 rounded-md ${
                testResult.success ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'
              }`}
            >
              <div className="flex items-start gap-2">
                {testResult.success ? (
                  <CheckCircle className="h-5 w-5 text-green-600 shrink-0" />
                ) : (
                  <XCircle className="h-5 w-5 text-red-600 shrink-0" />
                )}
                <div className="flex-1">
                  <p className={`font-medium ${testResult.success ? 'text-green-800' : 'text-red-800'}`}>
                    {testResult.success ? 'Evaluation Successful' : 'Evaluation Failed'}
                  </p>
                  {testResult.success ? (
                    <div className="mt-1">
                      <p className="text-sm text-green-700">
                        Result: <code className="bg-green-100 px-1 rounded">{JSON.stringify(testResult.output)}</code>
                      </p>
                      {testResult.executionTimeMs !== undefined && (
                        <p className="text-xs text-green-600 mt-1">
                          Execution time: {testResult.executionTimeMs}ms
                        </p>
                      )}
                    </div>
                  ) : (
                    <p className="text-sm text-red-700 mt-1">{testResult.error}</p>
                  )}
                </div>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// SIMPLE JSON LOGIC EVALUATOR
// =============================================================================

function evaluateJsonLogic(logic: unknown, data: Record<string, unknown>): unknown {
  if (logic === null || logic === undefined) return null;

  // Primitive values
  if (typeof logic !== 'object') return logic;
  if (Array.isArray(logic)) return logic.map((item) => evaluateJsonLogic(item, data));

  const obj = logic as Record<string, unknown>;
  const operators = Object.keys(obj);
  if (operators.length !== 1) return logic;

  const operator = operators[0];
  const args = obj[operator];

  // Variable reference
  if (operator === 'var') {
    const path = String(args);
    return data[path] ?? null;
  }

  // Evaluate operands
  const operands = Array.isArray(args)
    ? args.map((arg) => evaluateJsonLogic(arg, data))
    : [evaluateJsonLogic(args, data)];

  // Math operations
  switch (operator) {
    case '+':
      return operands.reduce((a, b) => Number(a) + Number(b), 0);
    case '-':
      if (operands.length === 1) return -Number(operands[0]);
      return Number(operands[0]) - Number(operands[1]);
    case '*':
      return operands.reduce((a, b) => Number(a) * Number(b), 1);
    case '/':
      return Number(operands[0]) / Number(operands[1]);
    case '%':
      return Number(operands[0]) % Number(operands[1]);
    case 'max':
      return Math.max(...operands.map(Number));
    case 'min':
      return Math.min(...operands.map(Number));

    // Comparison operations
    case '>':
      return Number(operands[0]) > Number(operands[1]);
    case '<':
      return Number(operands[0]) < Number(operands[1]);
    case '>=':
      return Number(operands[0]) >= Number(operands[1]);
    case '<=':
      return Number(operands[0]) <= Number(operands[1]);
    case '==':
      return operands[0] == operands[1];
    case '===':
      return operands[0] === operands[1];
    case '!=':
      return operands[0] != operands[1];
    case '!==':
      return operands[0] !== operands[1];

    // Logic operations
    case 'and':
      return operands.every(Boolean);
    case 'or':
      return operands.some(Boolean);
    case '!':
      return !operands[0];

    // Control flow
    case 'if':
      for (let i = 0; i < operands.length - 1; i += 2) {
        if (operands[i]) return operands[i + 1];
      }
      return operands.length % 2 === 1 ? operands[operands.length - 1] : null;

    default:
      return logic;
  }
}

export default RuleValidator;

// Real-time Simulation Panel
// Test rules with live feedback, save/load scenarios

import { useState, useEffect, useCallback } from 'react';
import { useProductStore, useSimulationStore } from '@/store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import type { AbstractAttribute, AttributeValue } from '@/types';
import { cn } from '@/lib/utils';
import {
  Play,
  RefreshCw,
  Save,
  Download,
  ChevronDown,
  ChevronRight,
  Zap,
  Clock,
  CheckCircle2,
  XCircle,
  FlaskConical,
  Loader2,
  ToggleLeft,
  ToggleRight,
} from 'lucide-react';

// =============================================================================
// INPUT FIELD COMPONENT
// =============================================================================

interface InputFieldProps {
  attribute: AbstractAttribute;
  value: AttributeValue | undefined;
  onChange: (value: AttributeValue) => void;
}

function InputField({ attribute, value, onChange }: InputFieldProps) {
  const attrName = attribute.attributeName || attribute.abstractPath.split(':').pop() || '';
  const datatype = attribute.datatypeId;

  const handleChange = useCallback(
    (rawValue: string) => {
      let typedValue: AttributeValue;

      switch (datatype) {
        case 'integer':
          typedValue = { type: 'int', value: parseInt(rawValue, 10) || 0 };
          break;
        case 'decimal':
        case 'float':
          typedValue = { type: 'float', value: parseFloat(rawValue) || 0 };
          break;
        case 'boolean':
          typedValue = { type: 'bool', value: rawValue === 'true' };
          break;
        case 'enum':
          typedValue = { type: 'string', value: rawValue };
          break;
        default:
          typedValue = { type: 'string', value: rawValue };
      }

      onChange(typedValue);
    },
    [datatype, onChange]
  );

  const getCurrentValue = (): string => {
    if (!value) return '';
    if ('value' in value) return String(value.value);
    return '';
  };

  return (
    <div className="flex items-center gap-2 py-1.5">
      <label className="min-w-[120px] text-sm font-medium text-gray-700 truncate" title={attrName}>
        {attrName}
      </label>
      <div className="flex-1">
        {datatype === 'boolean' ? (
          <button
            onClick={() => onChange({ type: 'bool', value: !getCurrentValue() })}
            className={cn(
              'flex items-center gap-2 rounded-md px-3 py-1.5 text-sm font-medium transition-colors',
              getCurrentValue() === 'true'
                ? 'bg-emerald-100 text-emerald-700'
                : 'bg-gray-100 text-gray-600'
            )}
          >
            {getCurrentValue() === 'true' ? (
              <ToggleRight className="h-4 w-4" />
            ) : (
              <ToggleLeft className="h-4 w-4" />
            )}
            {getCurrentValue() === 'true' ? 'True' : 'False'}
          </button>
        ) : datatype === 'enum' ? (
          <select
            value={getCurrentValue()}
            onChange={(e) => handleChange(e.target.value)}
            className="w-full rounded-md border bg-white px-3 py-1.5 text-sm"
          >
            <option value="">Select...</option>
            <option value="basic">Basic</option>
            <option value="standard">Standard</option>
            <option value="comprehensive">Comprehensive</option>
          </select>
        ) : (
          <Input
            type={datatype === 'integer' || datatype === 'decimal' || datatype === 'float' ? 'number' : 'text'}
            value={getCurrentValue()}
            onChange={(e) => handleChange(e.target.value)}
            placeholder={`Enter ${attrName}...`}
            className="h-8"
            step={datatype === 'decimal' || datatype === 'float' ? '0.01' : '1'}
          />
        )}
      </div>
      <span className="rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-medium text-gray-500">
        {datatype}
      </span>
    </div>
  );
}

// =============================================================================
// OUTPUT DISPLAY COMPONENT
// =============================================================================

interface OutputDisplayProps {
  name: string;
  value: AttributeValue;
  executionTime?: number;
}

function OutputDisplay({ name, value, executionTime }: OutputDisplayProps) {
  const formatValue = (v: AttributeValue): string => {
    if (!v) return 'null';
    if ('value' in v) {
      if (typeof v.value === 'number') {
        return v.type === 'float' ? v.value.toFixed(2) : String(v.value);
      }
      return String(v.value);
    }
    return JSON.stringify(v);
  };

  return (
    <div className="flex items-center justify-between rounded-lg bg-gradient-to-r from-emerald-50 to-teal-50 px-3 py-2 border border-emerald-100">
      <div className="flex items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-emerald-500" />
        <span className="text-sm font-medium text-gray-700">{name}</span>
      </div>
      <div className="flex items-center gap-3">
        <span className="font-mono text-sm font-semibold text-emerald-700">
          {formatValue(value)}
        </span>
        {executionTime !== undefined && (
          <span className="text-[10px] text-gray-400">
            {(executionTime / 1000).toFixed(1)}µs
          </span>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// MAIN SIMULATION PANEL
// =============================================================================

export function SimulationPanel() {
  const { selectedProduct, abstractAttributes } = useProductStore();
  const {
    inputs,
    results,
    isEvaluating,
    autoEvaluate,
    scenarios,
    setInput,
    setInputs,
    clearInputs,
    evaluate,
    setAutoEvaluate,
    saveScenario,
    loadScenario,
  } = useSimulationStore();

  const [inputsExpanded, setInputsExpanded] = useState(true);
  const [outputsExpanded, setOutputsExpanded] = useState(true);
  const [scenarioName, setScenarioName] = useState('');
  const [showSaveDialog, setShowSaveDialog] = useState(false);

  // Filter input attributes
  const inputAttributes = abstractAttributes.filter((a) =>
    a.tags.some((t) => t.name === 'input')
  );

  // Initialize inputs from attributes
  useEffect(() => {
    if (inputAttributes.length > 0 && inputs.length === 0) {
      const defaultInputs = inputAttributes.map((attr) => {
        const attrName = attr.attributeName || attr.abstractPath.split(':').pop() || '';
        let defaultValue: AttributeValue;

        switch (attr.datatypeId) {
          case 'integer':
            defaultValue = { type: 'int', value: 0 };
            break;
          case 'decimal':
          case 'float':
            defaultValue = { type: 'float', value: 0 };
            break;
          case 'boolean':
            defaultValue = { type: 'bool', value: false };
            break;
          default:
            defaultValue = { type: 'string', value: '' };
        }

        return {
          path: attr.abstractPath,
          value: defaultValue,
          displayName: attrName,
        };
      });
      setInputs(defaultInputs);
    }
  }, [inputAttributes, inputs.length, setInputs]);

  // Auto-evaluate when inputs change
  useEffect(() => {
    if (autoEvaluate && selectedProduct && inputs.length > 0) {
      const timer = setTimeout(() => {
        evaluate(selectedProduct.id);
      }, 500);
      return () => clearTimeout(timer);
    }
  }, [autoEvaluate, selectedProduct, inputs, evaluate]);

  const handleInputChange = useCallback(
    (path: string, value: AttributeValue) => {
      setInput(path, value);
    },
    [setInput]
  );

  const handleRunEvaluation = useCallback(() => {
    if (selectedProduct) {
      evaluate(selectedProduct.id);
    }
  }, [selectedProduct, evaluate]);

  const handleSaveScenario = useCallback(() => {
    if (scenarioName.trim()) {
      saveScenario(scenarioName.trim());
      setScenarioName('');
      setShowSaveDialog(false);
    }
  }, [scenarioName, saveScenario]);

  if (!selectedProduct) {
    return (
      <Card className="h-full">
        <CardContent className="flex h-full items-center justify-center">
          <div className="text-center text-gray-500">
            <FlaskConical className="mx-auto h-12 w-12 text-gray-300 mb-3" />
            <p className="text-sm">Select a product to start simulation</p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2 text-base">
            <FlaskConical className="h-5 w-5 text-indigo-500" />
            Simulation
          </CardTitle>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setAutoEvaluate(!autoEvaluate)}
              className={cn(
                'flex items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium transition-colors',
                autoEvaluate
                  ? 'bg-emerald-100 text-emerald-700'
                  : 'bg-gray-100 text-gray-500'
              )}
            >
              <Zap className="h-3 w-3" />
              Auto
            </button>
            <Button
              variant="default"
              size="sm"
              onClick={handleRunEvaluation}
              disabled={isEvaluating}
              className="gap-1.5"
            >
              {isEvaluating ? (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              ) : (
                <Play className="h-3.5 w-3.5" />
              )}
              Run
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="flex-1 overflow-auto space-y-4">
        {/* Inputs Section */}
        <div className="rounded-lg border bg-gray-50/50">
          <div className="flex w-full items-center justify-between px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-100 transition-colors">
            <button
              onClick={() => setInputsExpanded(!inputsExpanded)}
              className="flex items-center gap-2 flex-1 text-left"
            >
              {inputsExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
              Inputs
              <span className="rounded-full bg-blue-100 px-1.5 py-0.5 text-[10px] font-medium text-blue-600">
                {inputAttributes.length}
              </span>
            </button>
            <div className="flex items-center gap-1.5">
              <Button
                variant="ghost"
                size="sm"
                onClick={clearInputs}
                className="h-6 gap-1 px-2 text-[10px]"
              >
                <RefreshCw className="h-3 w-3" />
                Reset
              </Button>
            </div>
          </div>

          {inputsExpanded && (
            <div className="border-t px-3 py-2 space-y-1">
              {inputAttributes.map((attr) => {
                const inputVal = inputs.find((i) => i.path === attr.abstractPath);
                return (
                  <InputField
                    key={attr.abstractPath}
                    attribute={attr}
                    value={inputVal?.value}
                    onChange={(val) => handleInputChange(attr.abstractPath, val)}
                  />
                );
              })}
            </div>
          )}
        </div>

        {/* Outputs Section */}
        <div className="rounded-lg border bg-white">
          <button
            onClick={() => setOutputsExpanded(!outputsExpanded)}
            className="flex w-full items-center justify-between px-3 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50 transition-colors"
          >
            <span className="flex items-center gap-2">
              {outputsExpanded ? <ChevronDown className="h-4 w-4" /> : <ChevronRight className="h-4 w-4" />}
              Outputs
              {results && (
                <span className="rounded-full bg-emerald-100 px-1.5 py-0.5 text-[10px] font-medium text-emerald-600">
                  {Object.keys(results.outputs).length}
                </span>
              )}
            </span>
            {results?.metrics && (
              <div className="flex items-center gap-1.5 text-[10px] text-gray-400">
                <Clock className="h-3 w-3" />
                {(results.metrics.totalTimeNs / 1000000).toFixed(2)}ms
              </div>
            )}
          </button>

          {outputsExpanded && (
            <div className="border-t px-3 py-2 space-y-2">
              {results ? (
                Object.entries(results.outputs).map(([name, value]) => (
                  <OutputDisplay
                    key={name}
                    name={name}
                    value={value}
                    executionTime={results.ruleResults.find((r) =>
                      r.outputs.some((o) => o.path.includes(name))
                    )?.executionTimeNs}
                  />
                ))
              ) : (
                <div className="py-4 text-center text-sm text-gray-400">
                  Run simulation to see outputs
                </div>
              )}
            </div>
          )}
        </div>

        {/* Execution Metrics */}
        {results?.metrics && (
          <div className="rounded-lg border bg-gradient-to-r from-indigo-50 to-purple-50 p-3">
            <p className="text-xs font-semibold text-gray-700 mb-2">Execution Metrics</p>
            <div className="grid grid-cols-3 gap-2 text-center">
              <div>
                <p className="text-lg font-bold text-indigo-600">{results.metrics.rulesExecuted}</p>
                <p className="text-[10px] text-gray-500">Rules Run</p>
              </div>
              <div>
                <p className="text-lg font-bold text-purple-600">{results.metrics.rulesSkipped}</p>
                <p className="text-[10px] text-gray-500">Skipped</p>
              </div>
              <div>
                <p className="text-lg font-bold text-teal-600">
                  {(results.metrics.totalTimeNs / 1000000).toFixed(2)}
                </p>
                <p className="text-[10px] text-gray-500">ms Total</p>
              </div>
            </div>
          </div>
        )}

        {/* Errors */}
        {results?.errors && results.errors.length > 0 && (
          <div className="rounded-lg border border-red-200 bg-red-50 p-3">
            <p className="text-xs font-semibold text-red-700 mb-2 flex items-center gap-1.5">
              <XCircle className="h-4 w-4" />
              Errors
            </p>
            <ul className="space-y-1">
              {results.errors.map((err, i) => (
                <li key={i} className="text-xs text-red-600">
                  <span className="font-medium">{err.attribute}:</span> {err.message}
                </li>
              ))}
            </ul>
          </div>
        )}

        {/* Scenarios */}
        <div className="rounded-lg border bg-white">
          <div className="flex items-center justify-between px-3 py-2 border-b">
            <span className="text-sm font-semibold text-gray-700 flex items-center gap-2">
              <Save className="h-4 w-4 text-gray-400" />
              Saved Scenarios
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setShowSaveDialog(!showSaveDialog)}
              className="h-6 gap-1 px-2 text-xs"
            >
              <Save className="h-3 w-3" />
              Save
            </Button>
          </div>

          {showSaveDialog && (
            <div className="border-b px-3 py-2 flex gap-2">
              <Input
                value={scenarioName}
                onChange={(e) => setScenarioName(e.target.value)}
                placeholder="Scenario name..."
                className="h-7 text-xs"
              />
              <Button size="sm" onClick={handleSaveScenario} className="h-7 px-2">
                Save
              </Button>
            </div>
          )}

          <div className="px-3 py-2">
            {scenarios.length > 0 ? (
              <div className="space-y-1">
                {scenarios.map((scenario) => (
                  <button
                    key={scenario.id}
                    onClick={() => loadScenario(scenario.id)}
                    className="flex w-full items-center justify-between rounded px-2 py-1.5 text-xs hover:bg-gray-50 transition-colors"
                  >
                    <span className="font-medium text-gray-700">{scenario.name}</span>
                    <Download className="h-3 w-3 text-gray-400" />
                  </button>
                ))}
              </div>
            ) : (
              <p className="text-center text-xs text-gray-400 py-2">
                No saved scenarios
              </p>
            )}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

export default SimulationPanel;

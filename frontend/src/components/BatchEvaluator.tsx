// BatchEvaluator - Component for batch rule evaluation
// Supports CSV/JSON upload, progress tracking, and result export

import { useState, useCallback, useRef } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Upload,
  Play,
  Download,
  X,
  CheckCircle,
  XCircle,
  RefreshCw,
  FileJson,
  FileSpreadsheet,
  Trash2,
} from 'lucide-react';
import { api } from '@/services/api';
import type { AbstractAttribute, EvaluateResponse, BatchMetrics, AttributeValue } from '@/types';
import { cn } from '@/lib/utils';

// =============================================================================
// TYPES
// =============================================================================

export interface BatchEvaluatorProps {
  productId: string;
  inputAttributes: AbstractAttribute[];
  onClose?: () => void;
}

interface BatchInput {
  id: number;
  data: Record<string, unknown>;
  status: 'pending' | 'processing' | 'success' | 'error';
  result?: EvaluateResponse;
  error?: string;
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

function parseCSV(text: string): Record<string, unknown>[] {
  const lines = text.trim().split('\n');
  if (lines.length < 2) return [];

  const headers = lines[0].split(',').map((h) => h.trim().replace(/^"|"$/g, ''));
  const data: Record<string, unknown>[] = [];

  for (let i = 1; i < lines.length; i++) {
    const values = lines[i].split(',').map((v) => v.trim().replace(/^"|"$/g, ''));
    const row: Record<string, unknown> = {};

    headers.forEach((header, idx) => {
      const value = values[idx] ?? '';
      // Try to parse as number or boolean
      if (value === 'true') {
        row[header] = true;
      } else if (value === 'false') {
        row[header] = false;
      } else if (!isNaN(Number(value)) && value !== '') {
        row[header] = Number(value);
      } else {
        row[header] = value;
      }
    });

    data.push(row);
  }

  return data;
}

function exportToCSV(results: BatchInput[], inputAttributes: AbstractAttribute[]): string {
  // Build headers
  const inputHeaders = inputAttributes.map(
    (a) => a.attributeName || a.abstractPath.split(':').pop() || ''
  );
  const headers = ['id', 'status', ...inputHeaders, 'output', 'error'];

  // Build rows
  const rows = results.map((r) => {
    const inputValues = inputHeaders.map((h) => String(r.data[h] ?? ''));
    const output = r.result ? JSON.stringify(r.result.outputs) : '';
    return [r.id, r.status, ...inputValues, output, r.error || ''].join(',');
  });

  return [headers.join(','), ...rows].join('\n');
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function BatchEvaluator({
  productId,
  inputAttributes,
  onClose,
}: BatchEvaluatorProps) {
  const [inputs, setInputs] = useState<BatchInput[]>([]);
  const [isProcessing, setIsProcessing] = useState(false);
  const [progress, setProgress] = useState({ current: 0, total: 0 });
  const [metrics, setMetrics] = useState<BatchMetrics | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Handle file upload
  const handleFileUpload = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const content = e.target?.result as string;
          let data: Record<string, unknown>[];

          if (file.name.endsWith('.csv')) {
            data = parseCSV(content);
          } else {
            const parsed = JSON.parse(content);
            data = Array.isArray(parsed) ? parsed : [parsed];
          }

          setInputs(
            data.map((d, idx) => ({
              id: idx + 1,
              data: d,
              status: 'pending',
            }))
          );
          setMetrics(null);
        } catch (err) {
          alert(`Failed to parse file: ${(err as Error).message}`);
        }
      };
      reader.readAsText(file);
    },
    []
  );

  // Add manual input row
  const handleAddRow = useCallback(() => {
    const newRow: BatchInput = {
      id: inputs.length + 1,
      data: {},
      status: 'pending',
    };
    setInputs((prev) => [...prev, newRow]);
  }, [inputs.length]);

  // Update input value
  const handleUpdateInput = useCallback(
    (rowId: number, field: string, value: string) => {
      setInputs((prev) =>
        prev.map((input) => {
          if (input.id !== rowId) return input;

          // Try to parse value
          let parsedValue: unknown = value;
          if (value === 'true') parsedValue = true;
          else if (value === 'false') parsedValue = false;
          else if (!isNaN(Number(value)) && value !== '') parsedValue = Number(value);

          return {
            ...input,
            data: { ...input.data, [field]: parsedValue },
          };
        })
      );
    },
    []
  );

  // Remove input row
  const handleRemoveRow = useCallback((rowId: number) => {
    setInputs((prev) => prev.filter((i) => i.id !== rowId));
  }, []);

  // Run batch evaluation
  const handleRunBatch = useCallback(async () => {
    if (inputs.length === 0) return;

    setIsProcessing(true);
    setProgress({ current: 0, total: inputs.length });

    const startTime = performance.now();
    let successCount = 0;
    let errorCount = 0;

    for (let i = 0; i < inputs.length; i++) {
      const input = inputs[i];

      setInputs((prev) =>
        prev.map((item) =>
          item.id === input.id ? { ...item, status: 'processing' } : item
        )
      );

      try {
        const result = await api.evaluation.evaluate({
          productId,
          inputData: input.data as Record<string, AttributeValue>,
        });

        setInputs((prev) =>
          prev.map((item) =>
            item.id === input.id
              ? { ...item, status: 'success', result }
              : item
          )
        );
        successCount++;
      } catch (err) {
        setInputs((prev) =>
          prev.map((item) =>
            item.id === input.id
              ? { ...item, status: 'error', error: (err as Error).message }
              : item
          )
        );
        errorCount++;
      }

      setProgress({ current: i + 1, total: inputs.length });
    }

    const endTime = performance.now();

    setMetrics({
      totalRequests: inputs.length,
      successCount,
      failureCount: errorCount,
      totalTimeMs: Math.round(endTime - startTime),
      avgTimePerRequest: Math.round((endTime - startTime) / inputs.length),
    });

    setIsProcessing(false);
  }, [inputs, productId]);

  // Export results
  const handleExport = useCallback(() => {
    const csv = exportToCSV(inputs, inputAttributes);
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `batch-results-${productId}-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [inputs, inputAttributes, productId]);

  // Clear all inputs
  const handleClear = useCallback(() => {
    setInputs([]);
    setMetrics(null);
    setProgress({ current: 0, total: 0 });
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
  }, []);

  // Stats
  const successCount = inputs.filter((i) => i.status === 'success').length;
  const errorCount = inputs.filter((i) => i.status === 'error').length;
  const pendingCount = inputs.filter((i) => i.status === 'pending').length;

  // Get attribute names for table headers
  const attrNames = inputAttributes.map(
    (a) => a.attributeName || a.abstractPath.split(':').pop() || ''
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onClose} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-5xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <FileSpreadsheet className="h-5 w-5 text-primary" />
            </div>
            <div>
              <h3 className="text-lg font-semibold">Batch Evaluation</h3>
              <p className="text-sm text-gray-500">
                Upload CSV/JSON or add inputs manually
              </p>
            </div>
          </div>
          <button onClick={onClose} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        {/* Toolbar */}
        <div className="px-6 py-3 border-b bg-gray-50 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <input
              ref={fileInputRef}
              type="file"
              accept=".csv,.json"
              onChange={handleFileUpload}
              className="hidden"
            />
            <Button
              variant="outline"
              size="sm"
              onClick={() => fileInputRef.current?.click()}
              className="gap-1.5"
            >
              <Upload className="h-3.5 w-3.5" />
              Upload File
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleAddRow}
              className="gap-1.5"
            >
              Add Row
            </Button>
            {inputs.length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleClear}
                className="gap-1.5 text-red-600 hover:text-red-700"
              >
                <Trash2 className="h-3.5 w-3.5" />
                Clear All
              </Button>
            )}
          </div>
          <div className="flex items-center gap-2">
            {inputs.length > 0 && (
              <>
                <span className="text-xs text-gray-500">
                  {inputs.length} input(s)
                </span>
                <Button
                  size="sm"
                  onClick={handleRunBatch}
                  disabled={isProcessing || pendingCount === 0}
                  className="gap-1.5"
                >
                  {isProcessing ? (
                    <>
                      <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                      Processing... ({progress.current}/{progress.total})
                    </>
                  ) : (
                    <>
                      <Play className="h-3.5 w-3.5" />
                      Run Batch
                    </>
                  )}
                </Button>
                {successCount > 0 && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleExport}
                    className="gap-1.5"
                  >
                    <Download className="h-3.5 w-3.5" />
                    Export
                  </Button>
                )}
              </>
            )}
          </div>
        </div>

        {/* Stats */}
        {inputs.length > 0 && (
          <div className="px-6 py-2 border-b flex items-center gap-4 text-sm">
            <span className="flex items-center gap-1 text-gray-500">
              <span className="font-medium">{pendingCount}</span> pending
            </span>
            <span className="flex items-center gap-1 text-green-600">
              <CheckCircle className="h-3.5 w-3.5" />
              <span className="font-medium">{successCount}</span> success
            </span>
            <span className="flex items-center gap-1 text-red-600">
              <XCircle className="h-3.5 w-3.5" />
              <span className="font-medium">{errorCount}</span> failed
            </span>
            {metrics && (
              <>
                <span className="text-gray-300">|</span>
                <span className="text-gray-500">
                  Total: <span className="font-medium">{metrics.totalTimeMs}ms</span>
                </span>
                <span className="text-gray-500">
                  Avg: <span className="font-medium">{metrics.avgTimePerRequest}ms</span>
                </span>
              </>
            )}
          </div>
        )}

        {/* Content */}
        <div className="flex-1 overflow-auto p-6">
          {inputs.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center text-gray-400">
              <div className="flex gap-4 mb-6">
                <div
                  className="p-6 border-2 border-dashed rounded-lg hover:border-primary hover:bg-primary/5 cursor-pointer transition-colors"
                  onClick={() => fileInputRef.current?.click()}
                >
                  <FileJson className="h-10 w-10 mx-auto mb-2 opacity-50" />
                  <p className="text-sm font-medium">Upload JSON</p>
                </div>
                <div
                  className="p-6 border-2 border-dashed rounded-lg hover:border-primary hover:bg-primary/5 cursor-pointer transition-colors"
                  onClick={() => fileInputRef.current?.click()}
                >
                  <FileSpreadsheet className="h-10 w-10 mx-auto mb-2 opacity-50" />
                  <p className="text-sm font-medium">Upload CSV</p>
                </div>
              </div>
              <p className="text-sm">Or click "Add Row" to manually enter inputs</p>
            </div>
          ) : (
            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b bg-gray-50">
                    <th className="text-left p-2 font-medium text-gray-700 w-16">
                      #
                    </th>
                    <th className="text-left p-2 font-medium text-gray-700 w-20">
                      Status
                    </th>
                    {attrNames.map((name) => (
                      <th
                        key={name}
                        className="text-left p-2 font-medium text-gray-700"
                      >
                        {name}
                      </th>
                    ))}
                    <th className="text-left p-2 font-medium text-gray-700">
                      Result
                    </th>
                    <th className="w-10"></th>
                  </tr>
                </thead>
                <tbody>
                  {inputs.map((input) => (
                    <tr
                      key={input.id}
                      className={cn(
                        'border-b',
                        input.status === 'error' && 'bg-red-50',
                        input.status === 'success' && 'bg-green-50',
                        input.status === 'processing' && 'bg-blue-50'
                      )}
                    >
                      <td className="p-2 font-mono text-gray-500">{input.id}</td>
                      <td className="p-2">
                        {input.status === 'pending' && (
                          <span className="inline-flex items-center gap-1 text-gray-500">
                            <span className="w-2 h-2 rounded-full bg-gray-300"></span>
                            Pending
                          </span>
                        )}
                        {input.status === 'processing' && (
                          <span className="inline-flex items-center gap-1 text-blue-600">
                            <RefreshCw className="h-3 w-3 animate-spin" />
                            Running
                          </span>
                        )}
                        {input.status === 'success' && (
                          <span className="inline-flex items-center gap-1 text-green-600">
                            <CheckCircle className="h-3 w-3" />
                            Success
                          </span>
                        )}
                        {input.status === 'error' && (
                          <span className="inline-flex items-center gap-1 text-red-600">
                            <XCircle className="h-3 w-3" />
                            Error
                          </span>
                        )}
                      </td>
                      {attrNames.map((name) => (
                        <td key={name} className="p-2">
                          <Input
                            value={String(input.data[name] ?? '')}
                            onChange={(e) =>
                              handleUpdateInput(input.id, name, e.target.value)
                            }
                            className="h-7 text-xs"
                            disabled={input.status !== 'pending'}
                          />
                        </td>
                      ))}
                      <td className="p-2">
                        {input.result ? (
                          <code className="text-xs bg-gray-100 px-1 py-0.5 rounded">
                            {JSON.stringify(input.result.outputs)}
                          </code>
                        ) : input.error ? (
                          <span
                            className="text-xs text-red-600"
                            title={input.error}
                          >
                            {input.error.substring(0, 50)}
                            {input.error.length > 50 && '...'}
                          </span>
                        ) : (
                          <span className="text-gray-400">-</span>
                        )}
                      </td>
                      <td className="p-2">
                        {input.status === 'pending' && (
                          <button
                            onClick={() => handleRemoveRow(input.id)}
                            className="p-1 hover:bg-gray-200 rounded text-gray-400 hover:text-red-500"
                          >
                            <X className="h-3.5 w-3.5" />
                          </button>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>

        {/* Metrics Panel */}
        {metrics && (
          <div className="px-6 py-4 border-t bg-gray-50">
            <h4 className="text-sm font-medium text-gray-700 mb-2">
              Batch Metrics
            </h4>
            <div className="grid grid-cols-5 gap-4 text-center">
              <div>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics.totalRequests}
                </p>
                <p className="text-xs text-gray-500">Total Inputs</p>
              </div>
              <div>
                <p className="text-2xl font-bold text-green-600">
                  {metrics.successCount}
                </p>
                <p className="text-xs text-gray-500">Successful</p>
              </div>
              <div>
                <p className="text-2xl font-bold text-red-600">
                  {metrics.failureCount}
                </p>
                <p className="text-xs text-gray-500">Failed</p>
              </div>
              <div>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics.totalTimeMs}ms
                </p>
                <p className="text-xs text-gray-500">Total Time</p>
              </div>
              <div>
                <p className="text-2xl font-bold text-gray-900">
                  {metrics.avgTimePerRequest}ms
                </p>
                <p className="text-xs text-gray-500">Avg Time</p>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default BatchEvaluator;

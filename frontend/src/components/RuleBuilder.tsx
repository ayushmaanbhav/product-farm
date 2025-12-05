// Block-based Rule Builder
// Visual drag-and-drop JSON Logic editor

import { useState, useCallback, useMemo, memo } from 'react';
import { useProductStore } from '@/store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { AbstractAttribute, Rule, JsonLogicBlock } from '@/types';
import { cn } from '@/lib/utils';
import {
  Plus,
  Trash2,
  Variable,
  Hash,
  Type,
  ToggleLeft,
  Calculator,
  GitBranch,
  Check,
  Brackets,
  ChevronDown,
  ChevronRight,
  ArrowRight,
  Code,
  Eye,
  ArrowUpDown,
  CheckCircle,
} from 'lucide-react';
import { RuleValidator } from './RuleValidator';

// =============================================================================
// BLOCK PALETTE - Available blocks to drag
// =============================================================================

interface BlockPaletteProps {
  attributes: AbstractAttribute[];
  onAddBlock: (block: JsonLogicBlock) => void;
}

const operators = [
  { op: '+', label: 'Add', category: 'math' },
  { op: '-', label: 'Subtract', category: 'math' },
  { op: '*', label: 'Multiply', category: 'math' },
  { op: '/', label: 'Divide', category: 'math' },
  { op: '%', label: 'Modulo', category: 'math' },
  { op: '>', label: 'Greater Than', category: 'compare' },
  { op: '<', label: 'Less Than', category: 'compare' },
  { op: '>=', label: 'Greater Or Equal', category: 'compare' },
  { op: '<=', label: 'Less Or Equal', category: 'compare' },
  { op: '==', label: 'Equals', category: 'compare' },
  { op: '!=', label: 'Not Equals', category: 'compare' },
  { op: 'and', label: 'And', category: 'logic' },
  { op: 'or', label: 'Or', category: 'logic' },
  { op: '!', label: 'Not', category: 'logic' },
  { op: 'if', label: 'If/Then/Else', category: 'control' },
  { op: 'max', label: 'Maximum', category: 'math' },
  { op: 'min', label: 'Minimum', category: 'math' },
];

function BlockPalette({ attributes, onAddBlock }: BlockPaletteProps) {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set(['variables', 'math', 'compare'])
  );

  const toggleCategory = (category: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  const inputAttrs = attributes.filter((a) => a.tags.some((t) => t.name === 'input'));
  const outputAttrs = attributes.filter((a) => !a.tags.some((t) => t.name === 'input'));

  const categories = [
    {
      id: 'variables',
      label: 'Variables',
      icon: Variable,
      color: 'bg-blue-100 border-blue-300 text-blue-700',
    },
    {
      id: 'literals',
      label: 'Values',
      icon: Hash,
      color: 'bg-purple-100 border-purple-300 text-purple-700',
    },
    {
      id: 'math',
      label: 'Math',
      icon: Calculator,
      color: 'bg-emerald-100 border-emerald-300 text-emerald-700',
    },
    {
      id: 'compare',
      label: 'Compare',
      icon: Brackets,
      color: 'bg-amber-100 border-amber-300 text-amber-700',
    },
    {
      id: 'logic',
      label: 'Logic',
      icon: GitBranch,
      color: 'bg-pink-100 border-pink-300 text-pink-700',
    },
    {
      id: 'control',
      label: 'Control',
      icon: ArrowRight,
      color: 'bg-indigo-100 border-indigo-300 text-indigo-700',
    },
  ];

  return (
    <div className="w-64 border-r bg-gray-50 overflow-y-auto">
      <div className="p-2 border-b bg-white">
        <h3 className="text-xs font-semibold text-gray-700 uppercase tracking-wide">
          Block Palette
        </h3>
      </div>

      <div className="p-2 space-y-2">
        {categories.map((cat) => (
          <div key={cat.id} className="rounded-lg border bg-white overflow-hidden">
            <button
              onClick={() => toggleCategory(cat.id)}
              className="flex items-center justify-between w-full px-2 py-1.5 text-xs font-medium text-gray-700 hover:bg-gray-50"
            >
              <span className="flex items-center gap-1.5">
                <cat.icon className="h-3.5 w-3.5" />
                {cat.label}
              </span>
              {expandedCategories.has(cat.id) ? (
                <ChevronDown className="h-3.5 w-3.5 text-gray-400" />
              ) : (
                <ChevronRight className="h-3.5 w-3.5 text-gray-400" />
              )}
            </button>

            {expandedCategories.has(cat.id) && (
              <div className="border-t p-1.5 space-y-1">
                {cat.id === 'variables' && (
                  <>
                    <p className="text-[10px] text-gray-500 px-1">Input Attributes</p>
                    {inputAttrs.map((attr) => {
                      const name = attr.attributeName || attr.abstractPath.split(':').pop() || '';
                      return (
                        <button
                          key={attr.abstractPath}
                          onClick={() =>
                            onAddBlock({
                              type: 'variable',
                              path: name,
                              datatype: attr.datatypeId,
                            })
                          }
                          className={cn(
                            'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border transition-colors',
                            cat.color,
                            'hover:opacity-80'
                          )}
                        >
                          <Variable className="h-3 w-3" />
                          <span className="truncate">{name}</span>
                        </button>
                      );
                    })}
                    {outputAttrs.length > 0 && (
                      <>
                        <p className="text-[10px] text-gray-500 px-1 mt-2">Computed Attributes</p>
                        {outputAttrs.slice(0, 5).map((attr) => {
                          const name = attr.attributeName || attr.abstractPath.split(':').pop() || '';
                          return (
                            <button
                              key={attr.abstractPath}
                              onClick={() =>
                                onAddBlock({
                                  type: 'variable',
                                  path: name,
                                  datatype: attr.datatypeId,
                                })
                              }
                              className={cn(
                                'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border transition-colors',
                                'bg-amber-50 border-amber-200 text-amber-700',
                                'hover:opacity-80'
                              )}
                            >
                              <Variable className="h-3 w-3" />
                              <span className="truncate">{name}</span>
                            </button>
                          );
                        })}
                      </>
                    )}
                  </>
                )}

                {cat.id === 'literals' && (
                  <>
                    <button
                      onClick={() => onAddBlock({ type: 'literal', value: 0, datatype: 'number' })}
                      className={cn(
                        'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border',
                        cat.color
                      )}
                    >
                      <Hash className="h-3 w-3" />
                      Number
                    </button>
                    <button
                      onClick={() => onAddBlock({ type: 'literal', value: '', datatype: 'string' })}
                      className={cn(
                        'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border',
                        cat.color
                      )}
                    >
                      <Type className="h-3 w-3" />
                      Text
                    </button>
                    <button
                      onClick={() => onAddBlock({ type: 'literal', value: true, datatype: 'boolean' })}
                      className={cn(
                        'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border',
                        cat.color
                      )}
                    >
                      <ToggleLeft className="h-3 w-3" />
                      Boolean
                    </button>
                  </>
                )}

                {(cat.id === 'math' || cat.id === 'compare' || cat.id === 'logic' || cat.id === 'control') &&
                  operators
                    .filter((o) => o.category === cat.id)
                    .map((op) => (
                      <button
                        key={op.op}
                        onClick={() =>
                          onAddBlock({
                            type: 'operator',
                            operator: op.op,
                            operands: [],
                          })
                        }
                        className={cn(
                          'flex items-center gap-1.5 w-full rounded px-2 py-1 text-xs border',
                          cat.color
                        )}
                      >
                        <span className="font-mono font-bold">{op.op}</span>
                        <span className="text-gray-500">{op.label}</span>
                      </button>
                    ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// VISUAL BLOCK COMPONENT
// =============================================================================

interface VisualBlockProps {
  block: JsonLogicBlock;
  path: number[];
  onChange: (path: number[], block: JsonLogicBlock | null) => void;
  attributes: AbstractAttribute[];
  depth?: number;
}

const VisualBlock = memo(function VisualBlock({
  block,
  path,
  onChange,
  attributes,
  depth = 0,
}: VisualBlockProps) {
  const handleDelete = useCallback(() => {
    onChange(path, null);
  }, [path, onChange]);

  const handleAddOperand = useCallback(() => {
    if (block.type === 'operator') {
      const newBlock: JsonLogicBlock = { type: 'literal', value: 0, datatype: 'number' };
      onChange(path, {
        ...block,
        operands: [...block.operands, newBlock],
      });
    }
  }, [block, path, onChange]);

  const handleOperandChange = useCallback(
    (index: number, newBlock: JsonLogicBlock | null) => {
      if (block.type !== 'operator') return;

      if (newBlock === null) {
        onChange(path, {
          ...block,
          operands: block.operands.filter((_, i) => i !== index),
        });
      } else {
        onChange(path, {
          ...block,
          operands: block.operands.map((op, i) => (i === index ? newBlock : op)),
        });
      }
    },
    [block, path, onChange]
  );

  const getBlockColor = () => {
    switch (block.type) {
      case 'variable':
        return 'bg-blue-50 border-blue-300';
      case 'literal':
        return 'bg-purple-50 border-purple-300';
      case 'operator':
        if (['+', '-', '*', '/', '%', 'max', 'min'].includes(block.operator)) {
          return 'bg-emerald-50 border-emerald-300';
        }
        if (['>', '<', '>=', '<=', '==', '!='].includes(block.operator)) {
          return 'bg-amber-50 border-amber-300';
        }
        if (['and', 'or', '!'].includes(block.operator)) {
          return 'bg-pink-50 border-pink-300';
        }
        return 'bg-indigo-50 border-indigo-300';
      case 'condition':
        return 'bg-indigo-50 border-indigo-300';
      default:
        return 'bg-gray-50 border-gray-300';
    }
  };

  // Variable block
  if (block.type === 'variable') {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1.5 rounded-md border-2 px-2 py-1',
          getBlockColor()
        )}
      >
        <Variable className="h-3.5 w-3.5 text-blue-600" />
        <span className="text-sm font-medium text-blue-800">{block.path}</span>
        <button
          onClick={handleDelete}
          className="ml-1 p-0.5 rounded hover:bg-blue-200 text-blue-400 hover:text-blue-600"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
    );
  }

  // Literal block
  if (block.type === 'literal') {
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1.5 rounded-md border-2 px-2 py-1',
          getBlockColor()
        )}
      >
        <Hash className="h-3.5 w-3.5 text-purple-600" />
        <Input
          type={block.datatype === 'number' ? 'number' : 'text'}
          value={String(block.value)}
          onChange={(e) =>
            onChange(path, {
              ...block,
              value: block.datatype === 'number' ? parseFloat(e.target.value) || 0 : e.target.value,
            })
          }
          className="h-6 w-20 text-sm px-1 py-0"
        />
        <button
          onClick={handleDelete}
          className="ml-1 p-0.5 rounded hover:bg-purple-200 text-purple-400 hover:text-purple-600"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
    );
  }

  // Operator block
  if (block.type === 'operator') {
    const isIfOperator = block.operator === 'if';

    if (isIfOperator) {
      return (
        <div
          className={cn(
            'rounded-lg border-2 p-2',
            getBlockColor(),
            depth > 0 && 'ml-4'
          )}
        >
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-bold text-indigo-700 uppercase">IF / THEN / ELSE</span>
            <button
              onClick={handleDelete}
              className="p-0.5 rounded hover:bg-indigo-200 text-indigo-400 hover:text-indigo-600"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </div>

          <div className="space-y-2">
            {/* Condition */}
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-semibold text-indigo-600 w-10">IF</span>
              <div className="flex-1 min-h-[32px] rounded border-2 border-dashed border-indigo-200 bg-indigo-25 p-1">
                {block.operands[0] ? (
                  <VisualBlock
                    block={block.operands[0]}
                    path={[...path, 0]}
                    onChange={(_, newBlock) => handleOperandChange(0, newBlock)}
                    attributes={attributes}
                    depth={depth + 1}
                  />
                ) : (
                  <span className="text-xs text-gray-400">Drop condition here</span>
                )}
              </div>
            </div>

            {/* Then */}
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-semibold text-emerald-600 w-10">THEN</span>
              <div className="flex-1 min-h-[32px] rounded border-2 border-dashed border-emerald-200 bg-emerald-25 p-1">
                {block.operands[1] ? (
                  <VisualBlock
                    block={block.operands[1]}
                    path={[...path, 1]}
                    onChange={(_, newBlock) => handleOperandChange(1, newBlock)}
                    attributes={attributes}
                    depth={depth + 1}
                  />
                ) : (
                  <span className="text-xs text-gray-400">Drop result here</span>
                )}
              </div>
            </div>

            {/* Else */}
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-semibold text-amber-600 w-10">ELSE</span>
              <div className="flex-1 min-h-[32px] rounded border-2 border-dashed border-amber-200 bg-amber-25 p-1">
                {block.operands[2] ? (
                  <VisualBlock
                    block={block.operands[2]}
                    path={[...path, 2]}
                    onChange={(_, newBlock) => handleOperandChange(2, newBlock)}
                    attributes={attributes}
                    depth={depth + 1}
                  />
                ) : (
                  <span className="text-xs text-gray-400">Drop else result here</span>
                )}
              </div>
            </div>
          </div>
        </div>
      );
    }

    // Regular operator
    return (
      <div
        className={cn(
          'inline-flex items-center gap-1 rounded-lg border-2 px-2 py-1',
          getBlockColor()
        )}
      >
        <span className="font-mono font-bold text-sm">{block.operator}</span>
        <span className="text-gray-400">(</span>

        {block.operands.map((operand, i) => (
          <div key={i} className="flex items-center gap-1">
            {i > 0 && <span className="text-gray-400">,</span>}
            <VisualBlock
              block={operand}
              path={[...path, i]}
              onChange={(_, newBlock) => handleOperandChange(i, newBlock)}
              attributes={attributes}
              depth={depth + 1}
            />
          </div>
        ))}

        <button
          onClick={handleAddOperand}
          className="p-0.5 rounded hover:bg-gray-200 text-gray-400"
          title="Add operand"
        >
          <Plus className="h-3 w-3" />
        </button>

        <span className="text-gray-400">)</span>

        <button
          onClick={handleDelete}
          className="ml-1 p-0.5 rounded hover:bg-red-100 text-gray-400 hover:text-red-600"
        >
          <Trash2 className="h-3 w-3" />
        </button>
      </div>
    );
  }

  return null;
});

// =============================================================================
// JSON LOGIC CONVERTER
// =============================================================================

function blockToJsonLogic(block: JsonLogicBlock | null): unknown {
  if (!block) return null;

  switch (block.type) {
    case 'variable':
      return { var: block.path };
    case 'literal':
      return block.value;
    case 'operator':
      return { [block.operator]: block.operands.map(blockToJsonLogic) };
    case 'condition':
      const ifArgs: unknown[] = [];
      block.conditions.forEach((c) => {
        ifArgs.push(blockToJsonLogic(c.if));
        ifArgs.push(blockToJsonLogic(c.then));
      });
      if (block.else) {
        ifArgs.push(blockToJsonLogic(block.else));
      }
      return { if: ifArgs };
    default:
      return null;
  }
}

function jsonLogicToBlock(logic: unknown): JsonLogicBlock | null {
  if (logic === null || logic === undefined) return null;

  // Literal values
  if (typeof logic === 'number') {
    return { type: 'literal', value: logic, datatype: 'number' };
  }
  if (typeof logic === 'string') {
    return { type: 'literal', value: logic, datatype: 'string' };
  }
  if (typeof logic === 'boolean') {
    return { type: 'literal', value: logic, datatype: 'boolean' };
  }

  // Objects (operations)
  if (typeof logic === 'object' && !Array.isArray(logic)) {
    const obj = logic as Record<string, unknown>;
    const keys = Object.keys(obj);
    if (keys.length !== 1) return null;

    const operator = keys[0];
    const operands = obj[operator];

    // Variable reference
    if (operator === 'var') {
      return {
        type: 'variable',
        path: String(operands),
        datatype: 'unknown',
      };
    }

    // Operator
    const operandArray = Array.isArray(operands) ? operands : [operands];
    return {
      type: 'operator',
      operator,
      operands: operandArray.map(jsonLogicToBlock).filter((b): b is JsonLogicBlock => b !== null),
    };
  }

  return null;
}

// =============================================================================
// RULE TYPE OPTIONS
// =============================================================================

const RULE_TYPES = [
  { value: 'calculation', label: 'Calculation', description: 'Computes values from inputs' },
  { value: 'validation', label: 'Validation', description: 'Validates data constraints' },
  { value: 'transformation', label: 'Transformation', description: 'Transforms data format' },
  { value: 'eligibility', label: 'Eligibility', description: 'Checks eligibility criteria' },
  { value: 'pricing', label: 'Pricing', description: 'Calculates pricing/premiums' },
  { value: 'rating', label: 'Rating', description: 'Determines rating factors' },
  { value: 'limit', label: 'Limit', description: 'Sets limits and caps' },
  { value: 'default', label: 'Default', description: 'Provides default values' },
];

// =============================================================================
// ATTRIBUTE MULTI-SELECT
// =============================================================================

interface AttributeMultiSelectProps {
  label: string;
  selected: string[];
  onChange: (selected: string[]) => void;
  attributes: AbstractAttribute[];
  filterTags?: string[];
}

function AttributeMultiSelect({
  label,
  selected,
  onChange,
  attributes,
  filterTags,
}: AttributeMultiSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');

  const filteredAttrs = useMemo(() => {
    let filtered = attributes;
    if (filterTags?.length) {
      filtered = filtered.filter((a) =>
        a.tags.some((t) => filterTags.includes(t.name))
      );
    }
    if (search) {
      const lowerSearch = search.toLowerCase();
      filtered = filtered.filter((a) => {
        const name = a.attributeName || a.abstractPath.split(':').pop() || '';
        return name.toLowerCase().includes(lowerSearch);
      });
    }
    return filtered;
  }, [attributes, filterTags, search]);

  const toggleAttribute = (path: string) => {
    if (selected.includes(path)) {
      onChange(selected.filter((s) => s !== path));
    } else {
      onChange([...selected, path]);
    }
  };

  return (
    <div className="relative">
      <label className="block text-xs font-medium text-gray-700 mb-1">{label}</label>
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className="w-full text-left px-3 py-2 border rounded-md text-sm bg-white hover:bg-gray-50"
      >
        {selected.length === 0 ? (
          <span className="text-gray-400">Select attributes...</span>
        ) : (
          <span>{selected.length} attribute(s) selected</span>
        )}
      </button>

      {isOpen && (
        <div className="absolute z-20 mt-1 w-full bg-white border rounded-md shadow-lg max-h-64 overflow-hidden">
          <div className="p-2 border-b">
            <Input
              placeholder="Search attributes..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-7 text-xs"
            />
          </div>
          <div className="max-h-48 overflow-y-auto p-1">
            {filteredAttrs.length === 0 ? (
              <p className="text-xs text-gray-500 p-2">No attributes found</p>
            ) : (
              filteredAttrs.map((attr) => {
                const name = attr.attributeName || attr.abstractPath.split(':').pop() || '';
                const isSelected = selected.includes(attr.abstractPath);
                return (
                  <button
                    key={attr.abstractPath}
                    type="button"
                    onClick={() => toggleAttribute(attr.abstractPath)}
                    className={cn(
                      'w-full text-left px-2 py-1.5 text-xs rounded flex items-center gap-2',
                      isSelected ? 'bg-primary/10 text-primary' : 'hover:bg-gray-100'
                    )}
                  >
                    <div
                      className={cn(
                        'w-4 h-4 rounded border flex items-center justify-center',
                        isSelected ? 'bg-primary border-primary' : 'border-gray-300'
                      )}
                    >
                      {isSelected && <Check className="h-3 w-3 text-white" />}
                    </div>
                    <span className="truncate">{name}</span>
                    <span className="text-gray-400 text-[10px] truncate">
                      {attr.datatypeId}
                    </span>
                  </button>
                );
              })
            )}
          </div>
          <div className="p-2 border-t flex justify-end">
            <Button size="sm" variant="outline" onClick={() => setIsOpen(false)}>
              Done
            </Button>
          </div>
        </div>
      )}

      {/* Selected tags */}
      {selected.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-1.5">
          {selected.map((path) => {
            const attr = attributes.find((a) => a.abstractPath === path);
            const name = attr?.attributeName || path.split(':').pop() || path;
            return (
              <span
                key={path}
                className="inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] bg-primary/10 text-primary rounded"
              >
                {name}
                <button
                  type="button"
                  onClick={() => toggleAttribute(path)}
                  className="hover:text-primary/70"
                >
                  ×
                </button>
              </span>
            );
          })}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// MAIN RULE BUILDER
// =============================================================================

interface RuleBuilderProps {
  rule?: Rule;
  onSave?: (rule: Partial<Rule>) => void;
  onCancel?: () => void;
}

export function RuleBuilder({ rule, onSave, onCancel }: RuleBuilderProps) {
  const { abstractAttributes, selectedProduct } = useProductStore();

  // Initialize block from rule
  const initialBlock = useMemo(() => {
    if (rule?.compiledExpression) {
      try {
        const parsed = JSON.parse(rule.compiledExpression);
        return jsonLogicToBlock(parsed);
      } catch {
        return null;
      }
    }
    return null;
  }, [rule]);

  const [rootBlock, setRootBlock] = useState<JsonLogicBlock | null>(initialBlock);
  const [description, setDescription] = useState(rule?.description || '');
  const [ruleType, setRuleType] = useState(rule?.ruleType || 'calculation');
  const [showJson, setShowJson] = useState(false);
  const [orderIndex, setOrderIndex] = useState(rule?.orderIndex ?? 0);
  const [inputAttributePaths, setInputAttributePaths] = useState<string[]>(
    rule?.inputAttributes?.map((a) => a.attributePath) || []
  );
  const [outputAttributePaths, setOutputAttributePaths] = useState<string[]>(
    rule?.outputAttributes?.map((a) => a.attributePath) || []
  );
  const [showValidator, setShowValidator] = useState(false);
  const [isValid, setIsValid] = useState(true);

  // Computed JSON Logic
  const jsonLogic = useMemo(() => blockToJsonLogic(rootBlock), [rootBlock]);
  const jsonString = useMemo(() => JSON.stringify(jsonLogic, null, 2), [jsonLogic]);

  const handleAddBlock = useCallback((block: JsonLogicBlock) => {
    setRootBlock((prev) => {
      if (!prev) return block;

      // If root is an operator, add as operand
      if (prev.type === 'operator') {
        return {
          ...prev,
          operands: [...prev.operands, block],
        };
      }

      // Wrap both in an operator
      return {
        type: 'operator',
        operator: '+',
        operands: [prev, block],
      };
    });
  }, []);

  const handleBlockChange = useCallback((path: number[], newBlock: JsonLogicBlock | null) => {
    if (path.length === 0) {
      setRootBlock(newBlock);
      return;
    }

    setRootBlock((prev) => {
      if (!prev) return newBlock;
      // Deep update would be needed for nested paths
      // For simplicity, just set root
      return newBlock;
    });
  }, []);

  // Get selected input attributes as AbstractAttribute objects (for RuleValidator)
  const selectedInputAttrs = useMemo(() => {
    return abstractAttributes.filter((a) => inputAttributePaths.includes(a.abstractPath));
  }, [abstractAttributes, inputAttributePaths]);

  // Convert paths to RuleInputAttribute format
  const inputAttrsForRule = useMemo(() => {
    return inputAttributePaths.map((path, idx) => ({
      ruleId: rule?.id || '',
      attributePath: path,
      orderIndex: idx,
    }));
  }, [inputAttributePaths, rule?.id]);

  const outputAttrsForRule = useMemo(() => {
    return outputAttributePaths.map((path, idx) => ({
      ruleId: rule?.id || '',
      attributePath: path,
      orderIndex: idx,
    }));
  }, [outputAttributePaths, rule?.id]);

  // Build the rule object for validation
  const ruleForValidation: Partial<Rule> = useMemo(() => ({
    productId: selectedProduct?.id,
    ruleType,
    description,
    displayExpression: description || 'Custom rule',
    compiledExpression: jsonString,
    orderIndex,
    inputAttributes: inputAttrsForRule,
    outputAttributes: outputAttrsForRule,
  }), [selectedProduct?.id, ruleType, description, jsonString, orderIndex, inputAttrsForRule, outputAttrsForRule]);

  const handleValidationComplete = useCallback((valid: boolean) => {
    setIsValid(valid);
  }, []);

  const handleSave = useCallback(() => {
    if (!onSave || !rootBlock) return;

    onSave({
      productId: selectedProduct?.id,
      ruleType,
      description,
      displayExpression: description || 'Custom rule',
      compiledExpression: jsonString,
      orderIndex,
      inputAttributes: inputAttrsForRule,
      outputAttributes: outputAttrsForRule,
      enabled: true,
    });
  }, [onSave, rootBlock, ruleType, description, jsonString, orderIndex, inputAttrsForRule, outputAttrsForRule, selectedProduct]);

  return (
    <div className="h-full flex flex-col bg-white rounded-lg border overflow-hidden">
      {/* Header */}
      <div className="flex-shrink-0 border-b p-3 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-semibold text-gray-800">Rule Builder</h3>
          <select
            value={ruleType}
            onChange={(e) => setRuleType(e.target.value)}
            className="h-7 rounded border px-2 text-xs"
            title="Rule Type"
          >
            {RULE_TYPES.map((t) => (
              <option key={t.value} value={t.value}>
                {t.label}
              </option>
            ))}
          </select>
          <div className="flex items-center gap-1" title="Execution Order">
            <ArrowUpDown className="h-3.5 w-3.5 text-gray-400" />
            <Input
              type="number"
              min={0}
              value={orderIndex}
              onChange={(e) => setOrderIndex(parseInt(e.target.value) || 0)}
              className="h-7 w-16 text-xs"
              placeholder="Order"
            />
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowValidator(!showValidator)}
            className={cn('gap-1.5', showValidator && 'bg-primary/10')}
          >
            <CheckCircle className="h-3.5 w-3.5" />
            Validate
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowJson(!showJson)}
            className="gap-1.5"
          >
            {showJson ? <Eye className="h-3.5 w-3.5" /> : <Code className="h-3.5 w-3.5" />}
            {showJson ? 'Visual' : 'JSON'}
          </Button>
          {onCancel && (
            <Button variant="outline" size="sm" onClick={onCancel}>
              Cancel
            </Button>
          )}
          <Button size="sm" onClick={handleSave} disabled={!rootBlock || !isValid} className="gap-1.5">
            <Check className="h-3.5 w-3.5" />
            Save Rule
          </Button>
        </div>
      </div>

      {/* Main Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Palette */}
        <BlockPalette attributes={abstractAttributes} onAddBlock={handleAddBlock} />

        {/* Canvas */}
        <div className="flex-1 p-4 overflow-auto">
          {/* Description */}
          <div className="mb-4">
            <label className="block text-xs font-medium text-gray-700 mb-1">
              Rule Description
            </label>
            <Input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what this rule does..."
              className="h-8"
            />
          </div>

          {/* Input/Output Attributes */}
          <div className="grid grid-cols-2 gap-4 mb-4">
            <AttributeMultiSelect
              label="Input Attributes"
              selected={inputAttributePaths}
              onChange={setInputAttributePaths}
              attributes={abstractAttributes}
              filterTags={['input']}
            />
            <AttributeMultiSelect
              label="Output Attributes"
              selected={outputAttributePaths}
              onChange={setOutputAttributePaths}
              attributes={abstractAttributes}
              filterTags={['output', 'computed']}
            />
          </div>

          {/* Expression Builder */}
          <div className="mb-4">
            <label className="block text-xs font-medium text-gray-700 mb-1">Expression</label>
            {showJson ? (
              <pre className="rounded-lg border bg-gray-900 text-gray-100 p-4 text-sm font-mono overflow-auto max-h-[300px]">
                {jsonString}
              </pre>
            ) : (
              <div className="min-h-[200px] rounded-lg border-2 border-dashed border-gray-200 bg-gray-50 p-4">
                {rootBlock ? (
                  <VisualBlock
                    block={rootBlock}
                    path={[]}
                    onChange={handleBlockChange}
                    attributes={abstractAttributes}
                  />
                ) : (
                  <div className="h-full flex items-center justify-center text-gray-400">
                    <div className="text-center">
                      <Plus className="h-8 w-8 mx-auto mb-2 opacity-50" />
                      <p className="text-sm">Click blocks in the palette to build your expression</p>
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* Preview */}
          {rootBlock && !showJson && (
            <div className="rounded-lg border bg-white p-3 mb-4">
              <label className="block text-xs font-medium text-gray-700 mb-2">JSON Logic Preview</label>
              <pre className="text-xs font-mono text-gray-600 bg-gray-50 rounded p-2 overflow-auto">
                {jsonString}
              </pre>
            </div>
          )}

          {/* Validator Panel */}
          {showValidator && (
            <div className="mt-4">
              <RuleValidator
                rule={ruleForValidation}
                inputAttributes={selectedInputAttrs}
                onValidationComplete={handleValidationComplete}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default RuleBuilder;

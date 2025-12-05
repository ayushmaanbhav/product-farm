// TagMultiSelect - Multi-select component with tag creation ability
// Designed for selecting and creating tags with validation

import { useState, useCallback, useRef, useEffect } from 'react';
import { cn } from '@/lib/utils';
import { validatePattern } from '@/utils/validation';
import {
  X,
  Plus,
  Tag,
  GripVertical,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

export interface TagItem {
  name: string;
  orderIndex: number;
}

export interface TagMultiSelectProps {
  /** Selected tags */
  value: TagItem[];
  /** Available tag options */
  options: string[];
  /** Callback when tags change */
  onChange: (tags: TagItem[]) => void;
  /** Label for the component */
  label?: string;
  /** Whether the field is required */
  required?: boolean;
  /** Allow creating new tags */
  creatable?: boolean;
  /** Callback when a new tag is created */
  onCreate?: (tagName: string) => void;
  /** Maximum number of tags allowed */
  maxTags?: number;
  /** Placeholder text */
  placeholder?: string;
  /** Disabled state */
  disabled?: boolean;
  /** Error message */
  error?: string;
  /** Allow reordering with drag */
  reorderable?: boolean;
  /** Class name */
  className?: string;
}

// =============================================================================
// COMPONENT
// =============================================================================

export function TagMultiSelect({
  value,
  options,
  onChange,
  label,
  required = false,
  creatable = true,
  onCreate,
  maxTags,
  placeholder = 'Add tags...',
  disabled = false,
  error,
  reorderable = false,
  className,
}: TagMultiSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const [createError, setCreateError] = useState<string | null>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Selected tag names for easy lookup
  const selectedNames = new Set(value.map((t) => t.name));

  // Filter available options
  const availableOptions = options.filter(
    (opt) =>
      !selectedNames.has(opt) &&
      opt.toLowerCase().includes(search.toLowerCase())
  );

  // Check if we can add more
  const canAddMore = !maxTags || value.length < maxTags;

  // Validate new tag name
  const validateNewTag = useCallback((name: string): boolean => {
    const result = validatePattern(name, 'tag');
    if (!result.isValid) {
      setCreateError(result.error || 'Invalid tag name');
      return false;
    }
    if (selectedNames.has(name)) {
      setCreateError('Tag already added');
      return false;
    }
    if (options.includes(name) || creatable) {
      setCreateError(null);
      return true;
    }
    return true;
  }, [selectedNames, options, creatable]);

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setSearch('');
        setCreateError(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const addTag = useCallback(
    (name: string) => {
      if (!canAddMore || disabled) return;

      const trimmedName = name.trim().toLowerCase();
      if (!validateNewTag(trimmedName)) return;

      const newTag: TagItem = {
        name: trimmedName,
        orderIndex: value.length,
      };

      onChange([...value, newTag]);
      setSearch('');
      setCreateError(null);

      // If it's a new tag not in options, call onCreate
      if (!options.includes(trimmedName) && onCreate) {
        onCreate(trimmedName);
      }
    },
    [canAddMore, disabled, validateNewTag, value, onChange, options, onCreate]
  );

  const removeTag = useCallback(
    (name: string) => {
      if (disabled) return;
      const newTags = value
        .filter((t) => t.name !== name)
        .map((t, i) => ({ ...t, orderIndex: i }));
      onChange(newTags);
    },
    [disabled, value, onChange]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && search.trim()) {
        e.preventDefault();
        addTag(search);
      } else if (e.key === 'Escape') {
        setIsOpen(false);
        setSearch('');
        setCreateError(null);
      } else if (e.key === 'Backspace' && !search && value.length > 0) {
        removeTag(value[value.length - 1].name);
      }
    },
    [search, addTag, value, removeTag]
  );

  // Check if search matches any available option exactly
  const searchMatchesOption = options.some(
    (opt) => opt.toLowerCase() === search.toLowerCase()
  );

  return (
    <div ref={containerRef} className={cn('relative', className)}>
      {label && (
        <label className="block text-sm font-medium text-gray-700 mb-1.5">
          {label}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}

      {/* Selected Tags & Input */}
      <div
        className={cn(
          'min-h-[42px] flex flex-wrap gap-1.5 p-2 border rounded-md bg-white',
          'focus-within:ring-2 focus-within:ring-primary focus-within:ring-offset-1',
          disabled && 'bg-gray-100 cursor-not-allowed opacity-60',
          error && 'border-red-500',
          !error && !disabled && 'border-gray-300'
        )}
        onClick={() => !disabled && inputRef.current?.focus()}
      >
        {/* Selected Tags */}
        {value.map((tag) => (
          <div
            key={tag.name}
            className={cn(
              'inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-sm',
              'bg-primary/10 text-primary border border-primary/20'
            )}
          >
            {reorderable && <GripVertical className="h-3 w-3 cursor-grab opacity-50" />}
            <Tag className="h-3 w-3" />
            <span>{tag.name}</span>
            {!disabled && (
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  removeTag(tag.name);
                }}
                className="hover:bg-primary/20 rounded p-0.5"
              >
                <X className="h-3 w-3" />
              </button>
            )}
          </div>
        ))}

        {/* Input */}
        {canAddMore && !disabled && (
          <input
            ref={inputRef}
            type="text"
            value={search}
            onChange={(e) => {
              setSearch(e.target.value);
              setCreateError(null);
              if (!isOpen) setIsOpen(true);
            }}
            onFocus={() => setIsOpen(true)}
            onKeyDown={handleKeyDown}
            placeholder={value.length === 0 ? placeholder : ''}
            className="flex-1 min-w-[100px] text-sm outline-none bg-transparent"
          />
        )}
      </div>

      {/* Dropdown */}
      {isOpen && canAddMore && !disabled && (
        <div className="absolute z-50 w-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg max-h-60 overflow-y-auto">
          {/* Available Options */}
          {availableOptions.map((opt) => (
            <button
              key={opt}
              type="button"
              onClick={() => addTag(opt)}
              className="flex items-center w-full px-3 py-2 text-left text-sm hover:bg-gray-50"
            >
              <Tag className="h-3.5 w-3.5 mr-2 text-gray-400" />
              {opt}
            </button>
          ))}

          {/* Create New Tag Option */}
          {creatable && search && !searchMatchesOption && !selectedNames.has(search.toLowerCase()) && (
            <button
              type="button"
              onClick={() => addTag(search)}
              className={cn(
                'flex items-center w-full px-3 py-2 text-left text-sm border-t',
                createError ? 'text-red-600' : 'text-primary hover:bg-primary/5'
              )}
            >
              {createError ? (
                <span className="text-xs">{createError}</span>
              ) : (
                <>
                  <Plus className="h-3.5 w-3.5 mr-2" />
                  Create "{search.toLowerCase()}"
                </>
              )}
            </button>
          )}

          {/* No Options Message */}
          {availableOptions.length === 0 && !search && (
            <div className="px-3 py-4 text-sm text-gray-500 text-center">
              {options.length === 0 ? 'No tags available' : 'All tags selected'}
            </div>
          )}
        </div>
      )}

      {/* Max Tags Indicator */}
      {maxTags && (
        <p className="mt-1 text-xs text-gray-500">
          {value.length} / {maxTags} tags
        </p>
      )}

      {/* Error */}
      {error && <p className="mt-1.5 text-xs text-red-600">{error}</p>}
    </div>
  );
}

export default TagMultiSelect;

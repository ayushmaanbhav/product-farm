// SelectDropdown - Generic searchable dropdown component
// Supports search, custom rendering, and create new option

import { useState, useCallback, useRef, useEffect } from 'react';
import { cn } from '@/lib/utils';
import {
  ChevronDown,
  Search,
  X,
  Plus,
  Check,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

export interface SelectOption<T = string> {
  value: T;
  label: string;
  description?: string;
  icon?: React.ReactNode;
  disabled?: boolean;
  group?: string;
}

export interface SelectDropdownProps<T = string> {
  /** Available options */
  options: SelectOption<T>[];
  /** Selected value */
  value?: T;
  /** Placeholder text when no selection */
  placeholder?: string;
  /** Callback when selection changes */
  onChange: (value: T | undefined) => void;
  /** Label for the dropdown */
  label?: string;
  /** Whether the field is required */
  required?: boolean;
  /** Allow searching/filtering options */
  searchable?: boolean;
  /** Allow clearing the selection */
  clearable?: boolean;
  /** Allow creating new options */
  creatable?: boolean;
  /** Callback when creating new option */
  onCreate?: (inputValue: string) => void;
  /** Custom render function for option */
  renderOption?: (option: SelectOption<T>, isSelected: boolean) => React.ReactNode;
  /** Custom render function for selected value */
  renderValue?: (option: SelectOption<T>) => React.ReactNode;
  /** Disabled state */
  disabled?: boolean;
  /** Error message */
  error?: string;
  /** Class name for the trigger */
  className?: string;
  /** Maximum height of dropdown */
  maxHeight?: number;
  /** Group options by group field */
  grouped?: boolean;
}

// =============================================================================
// COMPONENT
// =============================================================================

export function SelectDropdown<T = string>({
  options,
  value,
  placeholder = 'Select...',
  onChange,
  label,
  required = false,
  searchable = true,
  clearable = false,
  creatable = false,
  onCreate,
  renderOption,
  renderValue,
  disabled = false,
  error,
  className,
  maxHeight = 300,
  grouped = false,
}: SelectDropdownProps<T>) {
  const [isOpen, setIsOpen] = useState(false);
  const [search, setSearch] = useState('');
  const containerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  // Find selected option
  const selectedOption = options.find((opt) => opt.value === value);

  // Filter options based on search
  const filteredOptions = options.filter((opt) =>
    opt.label.toLowerCase().includes(search.toLowerCase()) ||
    opt.description?.toLowerCase().includes(search.toLowerCase())
  );

  // Group options if needed
  const groupedOptions = grouped
    ? filteredOptions.reduce((acc, opt) => {
        const group = opt.group || 'Other';
        if (!acc[group]) acc[group] = [];
        acc[group].push(opt);
        return acc;
      }, {} as Record<string, SelectOption<T>[]>)
    : { '': filteredOptions };

  // Check if search matches any option exactly
  const searchMatchesExisting = filteredOptions.some(
    (opt) => opt.label.toLowerCase() === search.toLowerCase()
  );

  // Close on outside click
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
        setSearch('');
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  // Focus input when opening
  useEffect(() => {
    if (isOpen && searchable) {
      inputRef.current?.focus();
    }
  }, [isOpen, searchable]);

  const handleSelect = useCallback(
    (opt: SelectOption<T>) => {
      if (opt.disabled) return;
      onChange(opt.value);
      setIsOpen(false);
      setSearch('');
    },
    [onChange]
  );

  const handleClear = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onChange(undefined);
    },
    [onChange]
  );

  const handleCreate = useCallback(() => {
    if (onCreate && search.trim()) {
      onCreate(search.trim());
      setSearch('');
      setIsOpen(false);
    }
  }, [onCreate, search]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === 'Escape') {
        setIsOpen(false);
        setSearch('');
      } else if (e.key === 'Enter' && creatable && search && !searchMatchesExisting) {
        e.preventDefault();
        handleCreate();
      }
    },
    [creatable, search, searchMatchesExisting, handleCreate]
  );

  return (
    <div ref={containerRef} className="relative">
      {label && (
        <label className="block text-sm font-medium text-gray-700 mb-1.5">
          {label}
          {required && <span className="text-red-500 ml-1">*</span>}
        </label>
      )}

      {/* Trigger Button */}
      <button
        type="button"
        disabled={disabled}
        onClick={() => !disabled && setIsOpen(!isOpen)}
        className={cn(
          'flex items-center justify-between w-full h-10 px-3 text-left',
          'border rounded-md bg-white text-sm',
          'focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1',
          'transition-colors',
          disabled && 'bg-gray-100 cursor-not-allowed opacity-60',
          error && 'border-red-500',
          !error && !disabled && 'border-gray-300 hover:border-gray-400',
          className
        )}
      >
        <span className={cn('truncate', !selectedOption && !value && 'text-gray-400')}>
          {selectedOption
            ? renderValue
              ? renderValue(selectedOption)
              : selectedOption.label
            : value
              ? String(value)
              : placeholder}
        </span>
        <div className="flex items-center gap-1">
          {clearable && selectedOption && (
            <X
              className="h-4 w-4 text-gray-400 hover:text-gray-600"
              onClick={handleClear}
            />
          )}
          <ChevronDown
            className={cn(
              'h-4 w-4 text-gray-400 transition-transform',
              isOpen && 'transform rotate-180'
            )}
          />
        </div>
      </button>

      {/* Dropdown */}
      {isOpen && (
        <div
          className="absolute z-50 w-full mt-1 bg-white border border-gray-200 rounded-md shadow-lg"
          style={{ maxHeight }}
        >
          {/* Search Input */}
          {searchable && (
            <div className="p-2 border-b">
              <div className="relative">
                <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  ref={inputRef}
                  type="text"
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  onKeyDown={handleKeyDown}
                  placeholder="Search..."
                  className="w-full h-8 pl-8 pr-3 text-sm border border-gray-200 rounded focus:outline-none focus:ring-1 focus:ring-primary"
                />
              </div>
            </div>
          )}

          {/* Options List */}
          <div className="overflow-y-auto" style={{ maxHeight: maxHeight - 60 }}>
            {Object.entries(groupedOptions).map(([group, groupOptions]) => (
              <div key={group}>
                {grouped && group && (
                  <div className="px-3 py-1.5 text-xs font-semibold text-gray-500 bg-gray-50">
                    {group}
                  </div>
                )}
                {groupOptions.map((opt) => {
                  const isSelected = opt.value === value;
                  return (
                    <button
                      key={String(opt.value)}
                      type="button"
                      disabled={opt.disabled}
                      onClick={() => handleSelect(opt)}
                      className={cn(
                        'flex items-center w-full px-3 py-2 text-left text-sm',
                        'transition-colors',
                        isSelected && 'bg-primary/10 text-primary',
                        !isSelected && !opt.disabled && 'hover:bg-gray-50',
                        opt.disabled && 'opacity-50 cursor-not-allowed'
                      )}
                    >
                      {renderOption ? (
                        renderOption(opt, isSelected)
                      ) : (
                        <>
                          {opt.icon && <span className="mr-2">{opt.icon}</span>}
                          <div className="flex-1 min-w-0">
                            <div className="truncate">{opt.label}</div>
                            {opt.description && (
                              <div className="text-xs text-gray-500 truncate">
                                {opt.description}
                              </div>
                            )}
                          </div>
                          {isSelected && <Check className="h-4 w-4 ml-2 shrink-0" />}
                        </>
                      )}
                    </button>
                  );
                })}
              </div>
            ))}

            {/* No Results */}
            {filteredOptions.length === 0 && !creatable && (
              <div className="px-3 py-4 text-sm text-gray-500 text-center">
                No options found
              </div>
            )}

            {/* Create New Option */}
            {creatable && search && !searchMatchesExisting && (
              <button
                type="button"
                onClick={handleCreate}
                className="flex items-center w-full px-3 py-2 text-left text-sm text-primary hover:bg-primary/5 border-t"
              >
                <Plus className="h-4 w-4 mr-2" />
                Create "{search}"
              </button>
            )}
          </div>
        </div>
      )}

      {/* Error */}
      {error && <p className="mt-1.5 text-xs text-red-600">{error}</p>}
    </div>
  );
}

export default SelectDropdown;

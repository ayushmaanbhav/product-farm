// Hierarchical Attribute Explorer
// Tree view of attributes organized by component type

import { useState, useMemo, useCallback } from 'react';
import { useProductStore, useUIStore } from '@/store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { AbstractAttribute } from '@/types';
import { cn } from '@/lib/utils';
import {
  ChevronDown,
  ChevronRight,
  Search,
  Folder,
  FolderOpen,
  FileText,
  Tag,
  Target,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

interface TreeNode {
  id: string;
  label: string;
  type: 'component' | 'attribute';
  children?: TreeNode[];
  data?: AbstractAttribute;
}

// =============================================================================
// TREE NODE COMPONENT
// =============================================================================

interface TreeNodeItemProps {
  node: TreeNode;
  level: number;
  expanded: Set<string>;
  onToggle: (id: string) => void;
  onSelect: (node: TreeNode) => void;
  selectedId: string | null;
  onAnalyzeImpact?: (path: string) => void;
}

function TreeNodeItem({
  node,
  level,
  expanded,
  onToggle,
  onSelect,
  selectedId,
  onAnalyzeImpact,
}: TreeNodeItemProps) {
  const isExpanded = expanded.has(node.id);
  const isSelected = selectedId === node.id;
  const hasChildren = node.children && node.children.length > 0;

  const handleToggle = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onToggle(node.id);
    },
    [node.id, onToggle]
  );

  const handleSelect = useCallback(() => {
    onSelect(node);
  }, [node, onSelect]);

  const handleAnalyzeImpact = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      if (node.data && onAnalyzeImpact) {
        onAnalyzeImpact(node.data.abstractPath);
      }
    },
    [node.data, onAnalyzeImpact]
  );

  return (
    <div>
      <div
        className={cn(
          'group flex items-center gap-1.5 rounded-md px-2 py-1 cursor-pointer transition-colors',
          isSelected
            ? 'bg-primary text-primary-foreground'
            : 'hover:bg-accent hover:text-accent-foreground'
        )}
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        onClick={handleSelect}
      >
        {/* Expand/Collapse Toggle */}
        {hasChildren ? (
          <button onClick={handleToggle} className="p-0.5 -ml-1">
            {isExpanded ? (
              <ChevronDown className="h-3.5 w-3.5" />
            ) : (
              <ChevronRight className="h-3.5 w-3.5" />
            )}
          </button>
        ) : (
          <span className="w-4" />
        )}

        {/* Icon */}
        {node.type === 'component' ? (
          isExpanded ? (
            <FolderOpen className="h-4 w-4 text-amber-500 shrink-0" />
          ) : (
            <Folder className="h-4 w-4 text-amber-500 shrink-0" />
          )
        ) : (
          <FileText className="h-4 w-4 text-blue-500 shrink-0" />
        )}

        {/* Label */}
        <span className="flex-1 truncate text-sm">{node.label}</span>

        {/* Attribute-specific info */}
        {node.type === 'attribute' && node.data && (
          <>
            <span className="rounded bg-gray-200 px-1 py-0.5 text-[10px] font-medium text-gray-600">
              {node.data.datatypeId}
            </span>
            {node.data.tags.some((t) => t.name === 'input') && (
              <span className="rounded bg-blue-100 px-1 py-0.5 text-[10px] font-medium text-blue-600">
                IN
              </span>
            )}
            <button
              onClick={handleAnalyzeImpact}
              className="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-primary/20 transition-opacity"
              title="Analyze impact"
            >
              <Target className="h-3 w-3" />
            </button>
          </>
        )}
      </div>

      {/* Children */}
      {isExpanded && hasChildren && (
        <div>
          {node.children!.map((child) => (
            <TreeNodeItem
              key={child.id}
              node={child}
              level={level + 1}
              expanded={expanded}
              onToggle={onToggle}
              onSelect={onSelect}
              selectedId={selectedId}
              onAnalyzeImpact={onAnalyzeImpact}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// ATTRIBUTE DETAILS PANEL
// =============================================================================

interface AttributeDetailsPanelProps {
  attribute: AbstractAttribute;
  onClose: () => void;
}

function AttributeDetailsPanel({ attribute, onClose }: AttributeDetailsPanelProps) {
  const attrName = attribute.attributeName || attribute.abstractPath.split(':').pop() || '';

  return (
    <div className="border-t bg-gray-50 p-3 space-y-3">
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-semibold text-gray-800">{attrName}</h4>
        <Button variant="ghost" size="sm" onClick={onClose} className="h-6 px-2">
          Close
        </Button>
      </div>

      <div className="space-y-2">
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 w-20">Type:</span>
          <span className="rounded bg-gray-200 px-1.5 py-0.5 text-xs font-medium text-gray-700">
            {attribute.datatypeId}
          </span>
        </div>

        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-500 w-20">Component:</span>
          <span className="text-xs font-medium text-gray-700">{attribute.componentType}</span>
        </div>

        <div className="flex items-start gap-2">
          <span className="text-xs text-gray-500 w-20">Tags:</span>
          <div className="flex flex-wrap gap-1">
            {attribute.tags.map((tag) => (
              <span
                key={tag.name}
                className={cn(
                  'rounded px-1.5 py-0.5 text-[10px] font-medium',
                  tag.name === 'input'
                    ? 'bg-blue-100 text-blue-700'
                    : tag.name === 'output'
                      ? 'bg-amber-100 text-amber-700'
                      : 'bg-gray-100 text-gray-600'
                )}
              >
                <Tag className="inline h-2.5 w-2.5 mr-0.5" />
                {tag.name}
              </span>
            ))}
          </div>
        </div>

        {attribute.description && (
          <div className="flex items-start gap-2">
            <span className="text-xs text-gray-500 w-20">Description:</span>
            <p className="text-xs text-gray-700">{attribute.description}</p>
          </div>
        )}

        <div className="flex items-start gap-2">
          <span className="text-xs text-gray-500 w-20">Path:</span>
          <code className="text-[10px] font-mono text-gray-600 break-all">
            {attribute.abstractPath}
          </code>
        </div>

        {attribute.relatedAttributes.length > 0 && (
          <div className="flex items-start gap-2">
            <span className="text-xs text-gray-500 w-20">Related:</span>
            <div className="space-y-1">
              {attribute.relatedAttributes.map((rel, i) => (
                <div key={i} className="text-[10px] text-gray-600">
                  <span className="font-medium">{rel.relationshipType}:</span>{' '}
                  {rel.relatedPath.split(':').pop()}
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

// =============================================================================
// MAIN ATTRIBUTE EXPLORER
// =============================================================================

// Reserved for future sorting: type SortMode = 'name' | 'type' | 'component';
type FilterMode = 'all' | 'input' | 'computed';

export function AttributeExplorer() {
  const { abstractAttributes, selectedProduct } = useProductStore();
  const { setImpactAnalysisTarget } = useUIStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [expanded, setExpanded] = useState<Set<string>>(new Set(['root']));
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [filterMode, setFilterMode] = useState<FilterMode>('all');
  // sortMode reserved for future use: const [sortMode, setSortMode] = useState<SortMode>('component');

  // Filter attributes
  const filteredAttributes = useMemo(() => {
    let attrs = abstractAttributes;

    // Text search
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      attrs = attrs.filter(
        (a) =>
          a.attributeName?.toLowerCase().includes(query) ||
          a.abstractPath.toLowerCase().includes(query) ||
          a.description?.toLowerCase().includes(query) ||
          a.tags.some((t) => t.name.toLowerCase().includes(query))
      );
    }

    // Filter mode
    if (filterMode === 'input') {
      attrs = attrs.filter((a) => a.tags.some((t) => t.name === 'input'));
    } else if (filterMode === 'computed') {
      attrs = attrs.filter((a) => !a.tags.some((t) => t.name === 'input'));
    }

    return attrs;
  }, [abstractAttributes, searchQuery, filterMode]);

  // Build tree structure
  const treeData = useMemo((): TreeNode => {
    const componentGroups = new Map<string, AbstractAttribute[]>();

    filteredAttributes.forEach((attr) => {
      const component = attr.componentType || 'default';
      if (!componentGroups.has(component)) {
        componentGroups.set(component, []);
      }
      componentGroups.get(component)!.push(attr);
    });

    // Sort components
    const sortedComponents = Array.from(componentGroups.entries()).sort(([a], [b]) =>
      a.localeCompare(b)
    );

    const children: TreeNode[] = sortedComponents.map(([component, attrs]) => {
      // Sort attributes within component
      const sortedAttrs = [...attrs].sort((a, b) => {
        const aName = a.attributeName || a.abstractPath.split(':').pop() || '';
        const bName = b.attributeName || b.abstractPath.split(':').pop() || '';
        return aName.localeCompare(bName);
      });

      return {
        id: `component-${component}`,
        label: component.charAt(0).toUpperCase() + component.slice(1),
        type: 'component' as const,
        children: sortedAttrs.map((attr) => ({
          id: `attr-${attr.abstractPath}`,
          label: attr.attributeName || attr.abstractPath.split(':').pop() || '',
          type: 'attribute' as const,
          data: attr,
        })),
      };
    });

    return {
      id: 'root',
      label: selectedProduct?.name || 'Attributes',
      type: 'component' as const,
      children,
    };
  }, [filteredAttributes, selectedProduct]);

  // Handlers
  const handleToggle = useCallback((id: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const handleSelect = useCallback((node: TreeNode) => {
    setSelectedId(node.id);
  }, []);

  const handleExpandAll = useCallback(() => {
    const allIds = new Set<string>(['root']);
    const addChildren = (node: TreeNode) => {
      allIds.add(node.id);
      node.children?.forEach(addChildren);
    };
    addChildren(treeData);
    setExpanded(allIds);
  }, [treeData]);

  const handleCollapseAll = useCallback(() => {
    setExpanded(new Set(['root']));
  }, []);

  const handleAnalyzeImpact = useCallback(
    (path: string) => {
      setImpactAnalysisTarget(path);
    },
    [setImpactAnalysisTarget]
  );

  // Get selected attribute for details panel
  const selectedAttribute = useMemo(() => {
    if (!selectedId?.startsWith('attr-')) return null;
    const path = selectedId.replace('attr-', '');
    return abstractAttributes.find((a) => a.abstractPath === path);
  }, [selectedId, abstractAttributes]);

  if (!selectedProduct) {
    return (
      <div className="h-full flex flex-col border rounded-lg bg-white">
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-gray-500">
            <Folder className="mx-auto h-12 w-12 text-gray-300 mb-3" />
            <p className="text-sm">Select a product to explore attributes</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col border rounded-lg bg-white overflow-hidden">
      {/* Header */}
      <div className="flex-shrink-0 border-b p-2 space-y-2">
        {/* Search */}
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search attributes..."
            className="h-8 pl-8 text-sm"
          />
        </div>

        {/* Toolbar */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1">
            <Button
              variant={filterMode === 'all' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setFilterMode('all')}
              className="h-6 px-2 text-[10px]"
            >
              All
            </Button>
            <Button
              variant={filterMode === 'input' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setFilterMode('input')}
              className="h-6 px-2 text-[10px]"
            >
              Input
            </Button>
            <Button
              variant={filterMode === 'computed' ? 'default' : 'outline'}
              size="sm"
              onClick={() => setFilterMode('computed')}
              className="h-6 px-2 text-[10px]"
            >
              Computed
            </Button>
          </div>

          <div className="flex items-center gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleExpandAll}
              className="h-6 px-2"
              title="Expand all"
            >
              <ChevronDown className="h-3 w-3" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCollapseAll}
              className="h-6 px-2"
              title="Collapse all"
            >
              <ChevronRight className="h-3 w-3" />
            </Button>
          </div>
        </div>
      </div>

      {/* Tree */}
      <div className="flex-1 overflow-auto p-1">
        {treeData.children && treeData.children.length > 0 ? (
          treeData.children.map((child) => (
            <TreeNodeItem
              key={child.id}
              node={child}
              level={0}
              expanded={expanded}
              onToggle={handleToggle}
              onSelect={handleSelect}
              selectedId={selectedId}
              onAnalyzeImpact={handleAnalyzeImpact}
            />
          ))
        ) : (
          <div className="py-8 text-center text-sm text-gray-400">
            {searchQuery ? 'No matching attributes' : 'No attributes defined'}
          </div>
        )}
      </div>

      {/* Stats Footer */}
      <div className="flex-shrink-0 border-t px-3 py-1.5 text-[10px] text-gray-500 flex items-center justify-between">
        <span>
          {filteredAttributes.length} of {abstractAttributes.length} attributes
        </span>
        <span>
          {abstractAttributes.filter((a) => a.tags.some((t) => t.name === 'input')).length} inputs
        </span>
      </div>

      {/* Details Panel */}
      {selectedAttribute && (
        <AttributeDetailsPanel
          attribute={selectedAttribute}
          onClose={() => setSelectedId(null)}
        />
      )}
    </div>
  );
}

export default AttributeExplorer;

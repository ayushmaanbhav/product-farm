// Functionality Panel Component
// Manages product functionalities with approval workflow support

import { useState, useMemo, useCallback } from 'react';
import { useProductStore, useUIStore } from '@/store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import type { ProductFunctionality, FunctionalityStatus } from '@/types';
import { cn } from '@/lib/utils';
import {
  Boxes,
  Plus,
  Search,
  Lock,
  Unlock,
  Clock,
  CheckCircle,
  Play,
  Eye,
  Edit2,
  Trash2,
  Send,
  Check,
  X,
  FileText,
} from 'lucide-react';

// =============================================================================
// STATUS BADGE
// =============================================================================

function StatusBadge({ status }: { status: FunctionalityStatus }) {
  const config = {
    DRAFT: { icon: Edit2, className: 'bg-gray-100 text-gray-700', label: 'Draft' },
    PENDING_APPROVAL: { icon: Clock, className: 'bg-amber-100 text-amber-700', label: 'Pending' },
    ACTIVE: { icon: CheckCircle, className: 'bg-green-100 text-green-700', label: 'Active' },
  };

  const { icon: Icon, className, label } = config[status];

  return (
    <span className={cn('inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium', className)}>
      <Icon className="h-3 w-3" />
      {label}
    </span>
  );
}

// =============================================================================
// FUNCTIONALITY CARD
// =============================================================================

interface FunctionalityCardProps {
  functionality: ProductFunctionality;
  isSelected: boolean;
  onSelect: () => void;
  onView: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onSubmit: () => void;
  onApprove: () => void;
}

function FunctionalityCard({
  functionality,
  isSelected,
  onSelect,
  onView,
  onEdit,
  onDelete,
  onSubmit,
  onApprove,
}: FunctionalityCardProps) {
  const canEdit = functionality.status === 'DRAFT' && !functionality.immutable;
  const canSubmit = functionality.status === 'DRAFT';
  const canApprove = functionality.status === 'PENDING_APPROVAL';

  return (
    <div
      className={cn(
        'group border rounded-lg p-3 cursor-pointer transition-all hover:shadow-md',
        isSelected ? 'border-primary bg-primary/5 ring-1 ring-primary' : 'hover:border-gray-300'
      )}
      onClick={onSelect}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-2 mb-2">
        <div className="flex items-center gap-2 min-w-0">
          <Boxes className="h-4 w-4 text-indigo-500 shrink-0" />
          <span className="font-medium text-sm truncate">{functionality.displayName}</span>
        </div>
        <div className="flex items-center gap-1 shrink-0" title={functionality.immutable ? 'Immutable' : 'Mutable'}>
          {functionality.immutable ? (
            <Lock className="h-3.5 w-3.5 text-amber-500" />
          ) : (
            <Unlock className="h-3.5 w-3.5 text-gray-400" />
          )}
        </div>
      </div>

      {/* Status and Meta */}
      <div className="flex items-center justify-between gap-2 mb-2">
        <StatusBadge status={functionality.status} />
        <span className="text-[10px] text-gray-400">
          {functionality.requiredAttributes.length} attributes
        </span>
      </div>

      {/* Description */}
      {functionality.description && (
        <p className="text-xs text-gray-600 line-clamp-2 mb-3">{functionality.description}</p>
      )}

      {/* Actions */}
      <div className="flex items-center justify-between gap-1 pt-2 border-t opacity-0 group-hover:opacity-100 transition-opacity">
        <div className="flex items-center gap-1">
          <Button variant="ghost" size="sm" className="h-7 px-2" onClick={(e) => { e.stopPropagation(); onView(); }}>
            <Eye className="h-3.5 w-3.5" />
          </Button>
          {canEdit && (
            <>
              <Button variant="ghost" size="sm" className="h-7 px-2" onClick={(e) => { e.stopPropagation(); onEdit(); }}>
                <Edit2 className="h-3.5 w-3.5" />
              </Button>
              <Button variant="ghost" size="sm" className="h-7 px-2 text-red-600 hover:text-red-700" onClick={(e) => { e.stopPropagation(); onDelete(); }}>
                <Trash2 className="h-3.5 w-3.5" />
              </Button>
            </>
          )}
        </div>
        <div className="flex items-center gap-1">
          {canSubmit && (
            <Button variant="outline" size="sm" className="h-7 px-2 text-xs" onClick={(e) => { e.stopPropagation(); onSubmit(); }}>
              <Send className="h-3 w-3 mr-1" />
              Submit
            </Button>
          )}
          {canApprove && (
            <Button variant="default" size="sm" className="h-7 px-2 text-xs bg-green-600 hover:bg-green-700" onClick={(e) => { e.stopPropagation(); onApprove(); }}>
              <Check className="h-3 w-3 mr-1" />
              Approve
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// FUNCTIONALITY DETAILS PANEL
// =============================================================================

interface FunctionalityDetailsPanelProps {
  functionality: ProductFunctionality;
  onClose: () => void;
  onFilterByThis: () => void;
}

function FunctionalityDetailsPanel({ functionality, onClose, onFilterByThis }: FunctionalityDetailsPanelProps) {
  const { abstractAttributes } = useProductStore();

  // Get attribute details for required attributes
  const requiredAttributeDetails = useMemo(() => {
    return functionality.requiredAttributes.map(ra => {
      const attr = abstractAttributes.find(a => a.abstractPath === ra.abstractPath);
      return {
        ...ra,
        name: attr?.attributeName || ra.abstractPath.split(':').pop() || '',
        datatype: attr?.datatypeId,
        isInput: attr?.tags.some(t => t.name === 'input'),
        isImmutable: attr?.immutable,
      };
    });
  }, [functionality.requiredAttributes, abstractAttributes]);

  return (
    <div className="border-t bg-gray-50 p-4 space-y-4 max-h-96 overflow-auto">
      <div className="flex items-center justify-between">
        <h4 className="font-semibold text-gray-800">{functionality.displayName}</h4>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" className="h-7 text-xs" onClick={onFilterByThis}>
            <Play className="h-3 w-3 mr-1" />
            View Rules
          </Button>
          <Button variant="ghost" size="sm" className="h-7 px-2" onClick={onClose}>
            <X className="h-4 w-4" />
          </Button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4 text-sm">
        <div>
          <span className="text-gray-500">Status:</span>
          <div className="mt-1"><StatusBadge status={functionality.status} /></div>
        </div>
        <div>
          <span className="text-gray-500">Immutable:</span>
          <div className="mt-1 flex items-center gap-1">
            {functionality.immutable ? (
              <><Lock className="h-4 w-4 text-amber-500" /> Yes</>
            ) : (
              <><Unlock className="h-4 w-4 text-gray-400" /> No</>
            )}
          </div>
        </div>
      </div>

      {functionality.description && (
        <div>
          <span className="text-sm text-gray-500">Description:</span>
          <p className="mt-1 text-sm text-gray-700">{functionality.description}</p>
        </div>
      )}

      <div>
        <span className="text-sm text-gray-500 block mb-2">Required Attributes ({requiredAttributeDetails.length}):</span>
        <div className="space-y-2">
          {requiredAttributeDetails.map((attr, idx) => (
            <div key={idx} className="flex items-center justify-between bg-white rounded border px-3 py-2">
              <div className="flex items-center gap-2">
                <FileText className="h-4 w-4 text-blue-500" />
                <span className="text-sm font-medium">{attr.name}</span>
                {attr.isInput && (
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-blue-100 text-blue-700">Input</span>
                )}
                {attr.isImmutable && (
                  <span title="Immutable"><Lock className="h-3 w-3 text-amber-500" /></span>
                )}
              </div>
              <span className="text-xs text-gray-500">{attr.datatype}</span>
            </div>
          ))}
          {requiredAttributeDetails.length === 0 && (
            <p className="text-sm text-gray-400 italic">No required attributes defined</p>
          )}
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// MAIN FUNCTIONALITY PANEL
// =============================================================================

export function FunctionalityPanel() {
  const { selectedProduct, functionalities, submitFunctionality, approveFunctionality, deleteFunctionality } = useProductStore();
  const { setSelectedFunctionality, selectedFunctionalityId } = useUIStore();

  const [searchQuery, setSearchQuery] = useState('');
  const [selectedForDetails, setSelectedForDetails] = useState<string | null>(null);
  const [statusFilter, setStatusFilter] = useState<FunctionalityStatus | 'ALL'>('ALL');

  // Filter functionalities
  const filteredFunctionalities = useMemo(() => {
    let funcs = functionalities;

    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      funcs = funcs.filter(
        f =>
          f.displayName.toLowerCase().includes(query) ||
          f.name.toLowerCase().includes(query) ||
          f.description.toLowerCase().includes(query)
      );
    }

    if (statusFilter !== 'ALL') {
      funcs = funcs.filter(f => f.status === statusFilter);
    }

    return funcs;
  }, [functionalities, searchQuery, statusFilter]);

  // Group by status
  const groupedFunctionalities = useMemo(() => {
    const groups = {
      ACTIVE: [] as ProductFunctionality[],
      PENDING_APPROVAL: [] as ProductFunctionality[],
      DRAFT: [] as ProductFunctionality[],
    };
    filteredFunctionalities.forEach(f => groups[f.status].push(f));
    return groups;
  }, [filteredFunctionalities]);

  // Handlers
  const handleSelect = useCallback((id: string) => {
    setSelectedFunctionality(selectedFunctionalityId === id ? null : id);
  }, [selectedFunctionalityId, setSelectedFunctionality]);

  const handleView = useCallback((id: string) => {
    setSelectedForDetails(id);
  }, []);

  const handleSubmit = useCallback(async (id: string) => {
    try {
      await submitFunctionality(id);
    } catch (error) {
      console.error('Failed to submit functionality:', error);
      alert(`Failed to submit functionality: ${(error as Error).message}`);
    }
  }, [submitFunctionality]);

  const handleApprove = useCallback(async (id: string) => {
    try {
      await approveFunctionality(id);
    } catch (error) {
      console.error('Failed to approve functionality:', error);
      alert(`Failed to approve functionality: ${(error as Error).message}`);
    }
  }, [approveFunctionality]);

  const handleDelete = useCallback(async (id: string) => {
    if (confirm('Are you sure you want to delete this functionality?')) {
      try {
        await deleteFunctionality(id);
        if (selectedForDetails === id) setSelectedForDetails(null);
      } catch (error) {
        console.error('Failed to delete functionality:', error);
        alert(`Failed to delete functionality: ${(error as Error).message}`);
      }
    }
  }, [deleteFunctionality, selectedForDetails]);

  const handleFilterByFunctionality = useCallback((id: string) => {
    setSelectedFunctionality(id);
  }, [setSelectedFunctionality]);

  const selectedFunctionality = functionalities.find(f => f.id === selectedForDetails);

  if (!selectedProduct) {
    return (
      <div className="h-full flex flex-col border rounded-lg bg-white">
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-gray-500">
            <Boxes className="mx-auto h-12 w-12 text-gray-300 mb-3" />
            <p className="text-sm">Select a product to view functionalities</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col border rounded-lg bg-white overflow-hidden">
      {/* Header */}
      <div className="flex-shrink-0 border-b p-3 space-y-3">
        <div className="flex items-center justify-between">
          <h3 className="font-semibold text-gray-800 flex items-center gap-2">
            <Boxes className="h-5 w-5 text-indigo-500" />
            Functionalities
          </h3>
          <Button variant="outline" size="sm" className="h-7 text-xs">
            <Plus className="h-3 w-3 mr-1" />
            Add
          </Button>
        </div>

        {/* Search */}
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-gray-400" />
          <Input
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search functionalities..."
            className="h-8 pl-8 text-sm"
          />
        </div>

        {/* Status Filter */}
        <div className="flex items-center gap-1">
          {(['ALL', 'ACTIVE', 'PENDING_APPROVAL', 'DRAFT'] as const).map(status => (
            <Button
              key={status}
              variant={statusFilter === status ? 'default' : 'outline'}
              size="sm"
              className="h-6 px-2 text-[10px]"
              onClick={() => setStatusFilter(status)}
            >
              {status === 'ALL' ? 'All' : status === 'PENDING_APPROVAL' ? 'Pending' : status.charAt(0) + status.slice(1).toLowerCase()}
            </Button>
          ))}
        </div>
      </div>

      {/* List */}
      <div className="flex-1 overflow-auto p-3 space-y-4">
        {/* Active */}
        {groupedFunctionalities.ACTIVE.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 flex items-center gap-1">
              <CheckCircle className="h-3 w-3 text-green-500" />
              Active ({groupedFunctionalities.ACTIVE.length})
            </h4>
            <div className="space-y-2">
              {groupedFunctionalities.ACTIVE.map(func => (
                <FunctionalityCard
                  key={func.id}
                  functionality={func}
                  isSelected={selectedFunctionalityId === func.id}
                  onSelect={() => handleSelect(func.id)}
                  onView={() => handleView(func.id)}
                  onEdit={() => {}}
                  onDelete={() => handleDelete(func.id)}
                  onSubmit={() => handleSubmit(func.id)}
                  onApprove={() => handleApprove(func.id)}
                />
              ))}
            </div>
          </div>
        )}

        {/* Pending Approval */}
        {groupedFunctionalities.PENDING_APPROVAL.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 flex items-center gap-1">
              <Clock className="h-3 w-3 text-amber-500" />
              Pending Approval ({groupedFunctionalities.PENDING_APPROVAL.length})
            </h4>
            <div className="space-y-2">
              {groupedFunctionalities.PENDING_APPROVAL.map(func => (
                <FunctionalityCard
                  key={func.id}
                  functionality={func}
                  isSelected={selectedFunctionalityId === func.id}
                  onSelect={() => handleSelect(func.id)}
                  onView={() => handleView(func.id)}
                  onEdit={() => {}}
                  onDelete={() => handleDelete(func.id)}
                  onSubmit={() => handleSubmit(func.id)}
                  onApprove={() => handleApprove(func.id)}
                />
              ))}
            </div>
          </div>
        )}

        {/* Draft */}
        {groupedFunctionalities.DRAFT.length > 0 && (
          <div>
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 flex items-center gap-1">
              <Edit2 className="h-3 w-3 text-gray-500" />
              Draft ({groupedFunctionalities.DRAFT.length})
            </h4>
            <div className="space-y-2">
              {groupedFunctionalities.DRAFT.map(func => (
                <FunctionalityCard
                  key={func.id}
                  functionality={func}
                  isSelected={selectedFunctionalityId === func.id}
                  onSelect={() => handleSelect(func.id)}
                  onView={() => handleView(func.id)}
                  onEdit={() => {}}
                  onDelete={() => handleDelete(func.id)}
                  onSubmit={() => handleSubmit(func.id)}
                  onApprove={() => handleApprove(func.id)}
                />
              ))}
            </div>
          </div>
        )}

        {filteredFunctionalities.length === 0 && (
          <div className="py-8 text-center text-sm text-gray-400">
            {searchQuery ? 'No matching functionalities' : 'No functionalities defined'}
          </div>
        )}
      </div>

      {/* Stats Footer */}
      <div className="flex-shrink-0 border-t px-3 py-1.5 text-[10px] text-gray-500 flex items-center justify-between">
        <span>{functionalities.length} total</span>
        <span>
          {functionalities.filter(f => f.status === 'ACTIVE').length} active,{' '}
          {functionalities.filter(f => f.immutable).length} immutable
        </span>
      </div>

      {/* Details Panel */}
      {selectedFunctionality && (
        <FunctionalityDetailsPanel
          functionality={selectedFunctionality}
          onClose={() => setSelectedForDetails(null)}
          onFilterByThis={() => handleFilterByFunctionality(selectedFunctionality.id)}
        />
      )}
    </div>
  );
}

export default FunctionalityPanel;

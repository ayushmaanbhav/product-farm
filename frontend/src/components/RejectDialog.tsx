// RejectDialog - Modal for rejecting products or functionalities with reason
// Used in approval workflows to capture rejection reason

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { X, AlertTriangle } from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

export interface RejectDialogProps {
  /** Title of the dialog */
  title: string;
  /** Name of the entity being rejected */
  entityName: string;
  /** Type of entity (product, functionality) */
  entityType: string;
  /** Callback when rejected */
  onReject: (reason: string) => Promise<void>;
  /** Callback when cancelled */
  onCancel: () => void;
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function RejectDialog({
  title,
  entityName,
  entityType,
  onReject,
  onCancel,
}: RejectDialogProps) {
  const [reason, setReason] = useState('');
  const [isRejecting, setIsRejecting] = useState(false);
  const [error, setError] = useState('');

  const handleReject = async () => {
    if (!reason.trim()) {
      setError('Please provide a reason for rejection');
      return;
    }

    setIsRejecting(true);
    setError('');
    try {
      await onReject(reason);
    } catch (e) {
      setError((e as Error).message);
      setIsRejecting(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-md overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-red-100">
              <AlertTriangle className="h-5 w-5 text-red-600" />
            </div>
            <h3 className="text-lg font-semibold">{title}</h3>
          </div>
          <button onClick={onCancel} className="p-1 hover:bg-gray-100 rounded">
            <X className="h-5 w-5 text-gray-500" />
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4 space-y-4">
          <div className="p-4 bg-red-50 border border-red-100 rounded-md">
            <p className="text-sm text-red-700">
              You are about to reject <strong>{entityName}</strong>.
              This will return the {entityType} to DRAFT status.
            </p>
          </div>

          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Rejection Reason <span className="text-red-500">*</span>
            </label>
            <textarea
              value={reason}
              onChange={(e) => {
                setReason(e.target.value);
                setError('');
              }}
              placeholder="Please explain why this is being rejected..."
              rows={4}
              className="w-full px-3 py-2 border rounded-md text-sm resize-none focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-1"
              autoFocus
            />
            {error && <p className="text-xs text-red-600">{error}</p>}
          </div>
        </div>

        {/* Footer */}
        <div className="flex gap-3 px-6 py-4 border-t bg-gray-50">
          <Button variant="outline" className="flex-1" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            variant="destructive"
            className="flex-1"
            onClick={handleReject}
            disabled={isRejecting}
          >
            {isRejecting ? 'Rejecting...' : 'Reject'}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default RejectDialog;

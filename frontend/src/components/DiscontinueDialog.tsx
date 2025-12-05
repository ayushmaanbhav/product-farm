// DiscontinueDialog - Modal for discontinuing active products
// Shows impact analysis and requires confirmation

import { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { X, AlertTriangle, Ban, Package } from 'lucide-react';
import { api } from '@/services/api';
import type { Product } from '@/types';

// =============================================================================
// TYPES
// =============================================================================

export interface DiscontinueDialogProps {
  /** Product to discontinue */
  product: Product;
  /** Callback when discontinued */
  onDiscontinue: (reason?: string) => Promise<void>;
  /** Callback when cancelled */
  onCancel: () => void;
}

interface ImpactAnalysis {
  childProductsCount: number;
  childProducts: { id: string; name: string }[];
}

// =============================================================================
// MAIN COMPONENT
// =============================================================================

export function DiscontinueDialog({
  product,
  onDiscontinue,
  onCancel,
}: DiscontinueDialogProps) {
  const [reason, setReason] = useState('');
  const [isDiscontinuing, setIsDiscontinuing] = useState(false);
  const [error, setError] = useState('');
  const [impact, setImpact] = useState<ImpactAnalysis | null>(null);
  const [loadingImpact, setLoadingImpact] = useState(true);
  const [confirmText, setConfirmText] = useState('');

  // Fetch impact analysis on mount
  useEffect(() => {
    const fetchImpact = async () => {
      setLoadingImpact(true);
      try {
        // Get all products and find children
        const { items } = await api.products.list(100);
        const childProducts = items.filter((p) => p.parentProductId === product.id);
        setImpact({
          childProductsCount: childProducts.length,
          childProducts: childProducts.map((p) => ({ id: p.id, name: p.name })),
        });
      } catch (e) {
        console.error('Failed to fetch impact analysis:', e);
      } finally {
        setLoadingImpact(false);
      }
    };

    fetchImpact();
  }, [product.id]);

  const handleDiscontinue = async () => {
    if (confirmText !== product.id) {
      setError(`Please type "${product.id}" to confirm`);
      return;
    }

    setIsDiscontinuing(true);
    setError('');
    try {
      await onDiscontinue(reason || undefined);
    } catch (e) {
      setError((e as Error).message);
      setIsDiscontinuing(false);
    }
  };

  const hasImpact = impact && impact.childProductsCount > 0;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      <div className="absolute inset-0 bg-black/50" onClick={onCancel} />
      <div className="relative bg-white rounded-lg shadow-xl w-full max-w-lg overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b bg-red-50">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-red-100">
              <Ban className="h-5 w-5 text-red-600" />
            </div>
            <div>
              <h3 className="text-lg font-semibold text-red-900">Discontinue Product</h3>
              <p className="text-sm text-red-700">This action cannot be undone</p>
            </div>
          </div>
          <button onClick={onCancel} className="p-1 hover:bg-red-100 rounded">
            <X className="h-5 w-5 text-red-500" />
          </button>
        </div>

        {/* Content */}
        <div className="px-6 py-4 space-y-4">
          {/* Warning Banner */}
          <div className="p-4 bg-red-50 border border-red-200 rounded-md">
            <div className="flex items-start gap-3">
              <AlertTriangle className="h-5 w-5 text-red-600 mt-0.5 shrink-0" />
              <div>
                <p className="text-sm text-red-800 font-medium">
                  You are about to discontinue "{product.name}"
                </p>
                <p className="text-sm text-red-700 mt-1">
                  Discontinued products can no longer be used for new business.
                  Existing data will be preserved but the product will be marked as inactive.
                </p>
              </div>
            </div>
          </div>

          {/* Impact Analysis */}
          {loadingImpact ? (
            <div className="p-4 bg-gray-50 border rounded-md animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2"></div>
            </div>
          ) : hasImpact ? (
            <div className="p-4 bg-amber-50 border border-amber-200 rounded-md">
              <div className="flex items-start gap-3">
                <Package className="h-5 w-5 text-amber-600 mt-0.5 shrink-0" />
                <div>
                  <p className="text-sm text-amber-800 font-medium">
                    Impact: {impact.childProductsCount} child product(s) will be affected
                  </p>
                  <div className="mt-2 space-y-1">
                    {impact.childProducts.slice(0, 5).map((child) => (
                      <div key={child.id} className="text-xs text-amber-700 flex items-center gap-1">
                        <span>-</span>
                        <span>{child.name}</span>
                        <span className="text-amber-500">({child.id})</span>
                      </div>
                    ))}
                    {impact.childProductsCount > 5 && (
                      <p className="text-xs text-amber-600">
                        +{impact.childProductsCount - 5} more...
                      </p>
                    )}
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="p-4 bg-green-50 border border-green-200 rounded-md">
              <p className="text-sm text-green-800">
                No child products will be affected by this action.
              </p>
            </div>
          )}

          {/* Reason */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Reason for Discontinuation
            </label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="Optionally explain why this product is being discontinued..."
              rows={3}
              className="w-full px-3 py-2 border rounded-md text-sm resize-none focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-1"
            />
          </div>

          {/* Confirmation */}
          <div className="space-y-1.5">
            <label className="block text-sm font-medium text-gray-700">
              Type <code className="bg-gray-100 px-1 py-0.5 rounded text-red-600">{product.id}</code> to confirm
            </label>
            <input
              type="text"
              value={confirmText}
              onChange={(e) => {
                setConfirmText(e.target.value);
                setError('');
              }}
              placeholder={product.id}
              className="w-full px-3 py-2 border rounded-md text-sm font-mono focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-1"
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
            onClick={handleDiscontinue}
            disabled={isDiscontinuing || confirmText !== product.id}
          >
            {isDiscontinuing ? 'Discontinuing...' : 'Discontinue Product'}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default DiscontinueDialog;

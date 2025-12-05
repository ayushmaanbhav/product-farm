// Clone Product Dialog
// Dialog for cloning a product when modifications affect immutable attributes

import { useState, useCallback } from 'react';
import { useProductStore } from '@/store';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  Copy,
  X,
  AlertTriangle,
  Lock,
  ArrowRight,
  CheckCircle,
  Loader2,
  Package,
  FileText,
  GitBranch,
  Boxes,
} from 'lucide-react';

// =============================================================================
// TYPES
// =============================================================================

interface CloneProductDialogProps {
  open: boolean;
  onClose: () => void;
  reason?: string;
  affectedPaths?: string[];
}

// =============================================================================
// MAIN DIALOG
// =============================================================================

export function CloneProductDialog({
  open,
  onClose,
  reason,
  affectedPaths = [],
}: CloneProductDialogProps) {
  const { selectedProduct, cloneProduct } = useProductStore();

  const [newId, setNewId] = useState('');
  const [newName, setNewName] = useState('');
  const [newDescription, setNewDescription] = useState('');
  const [isCloning, setIsCloning] = useState(false);
  const [cloneResult, setCloneResult] = useState<{
    newProductId: string;
    abstractAttributesCloned: number;
    attributesCloned: number;
    rulesCloned: number;
    functionalitiesCloned: number;
  } | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Initialize name and ID when dialog opens
  useState(() => {
    if (selectedProduct && open) {
      setNewId(`${selectedProduct.id}_v2`);
      setNewName(`${selectedProduct.name} (v2)`);
      setNewDescription(selectedProduct.description);
    }
  });

  const handleClone = useCallback(async () => {
    if (!selectedProduct || !newId.trim() || !newName.trim()) return;

    setIsCloning(true);
    setError(null);

    try {
      const result = await cloneProduct({
        sourceProductId: selectedProduct.id,
        newProductId: newId.trim(),
        newProductName: newName.trim(),
        newProductDescription: newDescription.trim() || undefined,
      });
      setCloneResult(result);
    } catch (e) {
      setError((e as Error).message);
    } finally {
      setIsCloning(false);
    }
  }, [selectedProduct, newId, newName, newDescription, cloneProduct]);

  const handleClose = useCallback(() => {
    setNewId('');
    setNewName('');
    setNewDescription('');
    setCloneResult(null);
    setError(null);
    onClose();
  }, [onClose]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/50 backdrop-blur-sm"
        onClick={handleClose}
      />

      {/* Dialog */}
      <div className="relative bg-white rounded-xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden">
        {/* Header */}
        <div className="px-6 py-4 border-b bg-gradient-to-r from-indigo-500 to-purple-600">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3 text-white">
              <div className="p-2 bg-white/20 rounded-lg">
                <Copy className="h-5 w-5" />
              </div>
              <div>
                <h2 className="font-semibold text-lg">Clone Product</h2>
                <p className="text-sm text-white/80">Create a new version to make changes</p>
              </div>
            </div>
            <button
              onClick={handleClose}
              className="p-1.5 rounded-lg hover:bg-white/20 transition-colors"
            >
              <X className="h-5 w-5 text-white" />
            </button>
          </div>
        </div>

        {cloneResult ? (
          // Success State
          <div className="p-6">
            <div className="text-center mb-6">
              <div className="inline-flex items-center justify-center w-16 h-16 rounded-full bg-green-100 mb-4">
                <CheckCircle className="h-8 w-8 text-green-600" />
              </div>
              <h3 className="text-lg font-semibold text-gray-900">Product Cloned Successfully</h3>
              <p className="text-sm text-gray-500 mt-1">
                Your new product is ready for modifications
              </p>
            </div>

            <div className="bg-gray-50 rounded-lg p-4 space-y-3 mb-6">
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-500 flex items-center gap-2">
                  <FileText className="h-4 w-4" /> Abstract Attributes
                </span>
                <span className="font-medium">{cloneResult.abstractAttributesCloned}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-500 flex items-center gap-2">
                  <FileText className="h-4 w-4" /> Attributes
                </span>
                <span className="font-medium">{cloneResult.attributesCloned}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-500 flex items-center gap-2">
                  <GitBranch className="h-4 w-4" /> Rules
                </span>
                <span className="font-medium">{cloneResult.rulesCloned}</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-500 flex items-center gap-2">
                  <Boxes className="h-4 w-4" /> Functionalities
                </span>
                <span className="font-medium">{cloneResult.functionalitiesCloned}</span>
              </div>
            </div>

            <div className="flex gap-3">
              <Button variant="outline" className="flex-1" onClick={handleClose}>
                Close
              </Button>
              <Button className="flex-1" onClick={handleClose}>
                Open New Product
              </Button>
            </div>
          </div>
        ) : (
          // Form State
          <div className="p-6">
            {/* Warning Message */}
            {reason && (
              <div className="mb-6 p-4 bg-amber-50 border border-amber-200 rounded-lg">
                <div className="flex items-start gap-3">
                  <AlertTriangle className="h-5 w-5 text-amber-500 shrink-0 mt-0.5" />
                  <div>
                    <p className="text-sm font-medium text-amber-800">{reason}</p>
                    {affectedPaths.length > 0 && (
                      <div className="mt-2">
                        <p className="text-xs text-amber-700 mb-1">Affected immutable attributes:</p>
                        <ul className="text-xs text-amber-600 space-y-1">
                          {affectedPaths.slice(0, 3).map((path, i) => (
                            <li key={i} className="flex items-center gap-1">
                              <Lock className="h-3 w-3" />
                              {path.split(':').pop()}
                            </li>
                          ))}
                          {affectedPaths.length > 3 && (
                            <li className="text-amber-500">
                              +{affectedPaths.length - 3} more...
                            </li>
                          )}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            )}

            {/* Source Product Info */}
            <div className="mb-6 p-4 bg-gray-50 rounded-lg">
              <p className="text-xs text-gray-500 mb-2">Source Product</p>
              <div className="flex items-center gap-3">
                <Package className="h-8 w-8 text-gray-400" />
                <div>
                  <p className="font-medium">{selectedProduct?.name}</p>
                  <p className="text-xs text-gray-500">v{selectedProduct?.version}</p>
                </div>
                <ArrowRight className="h-5 w-5 text-gray-400 mx-2" />
                <div className="flex items-center gap-2 px-3 py-2 bg-white border rounded-lg">
                  <Copy className="h-4 w-4 text-indigo-500" />
                  <span className="text-sm text-indigo-600">New Clone</span>
                </div>
              </div>
            </div>

            {/* Form Fields */}
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1.5">
                  New Product ID
                </label>
                <Input
                  value={newId}
                  onChange={(e) => setNewId(e.target.value)}
                  placeholder="e.g., insurance_motor_v2"
                  className="h-10 font-mono"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Must start with a letter, only letters, numbers, and underscores
                </p>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1.5">
                  New Product Name
                </label>
                <Input
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="Enter product name"
                  className="h-10"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1.5">
                  Description (optional)
                </label>
                <textarea
                  value={newDescription}
                  onChange={(e) => setNewDescription(e.target.value)}
                  placeholder="Enter description"
                  rows={3}
                  className="w-full px-3 py-2 border rounded-lg text-sm resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500"
                />
              </div>
            </div>

            {/* Error Message */}
            {error && (
              <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                <p className="text-sm text-red-700">{error}</p>
              </div>
            )}

            {/* Actions */}
            <div className="flex gap-3 mt-6">
              <Button variant="outline" className="flex-1" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                className="flex-1"
                onClick={handleClone}
                disabled={isCloning || !newId.trim() || !newName.trim()}
              >
                {isCloning ? (
                  <>
                    <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                    Cloning...
                  </>
                ) : (
                  <>
                    <Copy className="h-4 w-4 mr-2" />
                    Clone Product
                  </>
                )}
              </Button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

export default CloneProductDialog;

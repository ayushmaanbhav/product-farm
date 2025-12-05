// Products Page - Manage product configurations with full lifecycle support
// Includes create/edit, clone, submit/approve/reject/discontinue workflows

import { useState, useEffect, useMemo } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { useProductStore } from '@/store';
import { Link } from 'react-router-dom';
import {
  Plus,
  Search,
  Package,
  MoreVertical,
  GitBranch,
  Layers,
  Copy,
  Pencil,
  Trash2,
  Send,
  CheckCircle,
  XCircle,
  Ban,
  Clock,
  Calendar,
  LayoutTemplate,
  FilePlus,
} from 'lucide-react';
import { api } from '@/services/api';
import { ProductForm } from '@/components/forms';
import { RejectDialog } from '@/components/RejectDialog';
import { DiscontinueDialog } from '@/components/DiscontinueDialog';
import { ProductCreationWizard } from '@/components/ProductCreationWizard';
import type { Product } from '@/types';

// =============================================================================
// PRODUCT CARD MENU
// =============================================================================

interface ProductMenuProps {
  product: Product;
  onEdit: () => void;
  onClone: () => void;
  onSubmit: () => void;
  onApprove: () => void;
  onReject: () => void;
  onDiscontinue: () => void;
  onDelete: () => void;
}

function ProductMenu({
  product,
  onEdit,
  onClone,
  onSubmit,
  onApprove,
  onReject,
  onDiscontinue,
  onDelete,
}: ProductMenuProps) {
  const [isOpen, setIsOpen] = useState(false);

  const canSubmit = product.status === 'DRAFT';
  const canApproveOrReject = product.status === 'PENDING_APPROVAL';
  const canDiscontinue = product.status === 'ACTIVE';
  const canDelete = product.status === 'DRAFT';

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="icon"
        onClick={(e) => {
          e.stopPropagation();
          setIsOpen(!isOpen);
        }}
      >
        <MoreVertical className="h-4 w-4" />
      </Button>

      {isOpen && (
        <>
          <div className="fixed inset-0 z-10" onClick={() => setIsOpen(false)} />
          <div className="absolute right-0 top-full mt-1 w-48 bg-white border rounded-lg shadow-lg z-20 overflow-hidden">
            <div className="py-1">
              <button
                onClick={() => {
                  onEdit();
                  setIsOpen(false);
                }}
                className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2"
              >
                <Pencil className="h-4 w-4" />
                Edit Details
              </button>
              <button
                onClick={() => {
                  onClone();
                  setIsOpen(false);
                }}
                className="w-full px-4 py-2 text-left text-sm hover:bg-gray-50 flex items-center gap-2"
              >
                <Copy className="h-4 w-4" />
                Clone Product
              </button>

              <div className="border-t my-1" />

              {canSubmit && (
                <button
                  onClick={() => {
                    onSubmit();
                    setIsOpen(false);
                  }}
                  className="w-full px-4 py-2 text-left text-sm hover:bg-blue-50 text-blue-600 flex items-center gap-2"
                >
                  <Send className="h-4 w-4" />
                  Submit for Approval
                </button>
              )}

              {canApproveOrReject && (
                <>
                  <button
                    onClick={() => {
                      onApprove();
                      setIsOpen(false);
                    }}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-green-50 text-green-600 flex items-center gap-2"
                  >
                    <CheckCircle className="h-4 w-4" />
                    Approve
                  </button>
                  <button
                    onClick={() => {
                      onReject();
                      setIsOpen(false);
                    }}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-red-50 text-red-600 flex items-center gap-2"
                  >
                    <XCircle className="h-4 w-4" />
                    Reject
                  </button>
                </>
              )}

              {canDiscontinue && (
                <button
                  onClick={() => {
                    onDiscontinue();
                    setIsOpen(false);
                  }}
                  className="w-full px-4 py-2 text-left text-sm hover:bg-red-50 text-red-600 flex items-center gap-2"
                >
                  <Ban className="h-4 w-4" />
                  Discontinue
                </button>
              )}

              {canDelete && (
                <>
                  <div className="border-t my-1" />
                  <button
                    onClick={() => {
                      onDelete();
                      setIsOpen(false);
                    }}
                    className="w-full px-4 py-2 text-left text-sm hover:bg-red-50 text-red-600 flex items-center gap-2"
                  >
                    <Trash2 className="h-4 w-4" />
                    Delete
                  </button>
                </>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

// =============================================================================
// STATUS BADGE
// =============================================================================

function StatusBadge({ status }: { status: Product['status'] }) {
  const config = {
    ACTIVE: { bg: 'bg-emerald-100', text: 'text-emerald-700', icon: CheckCircle },
    DRAFT: { bg: 'bg-gray-100', text: 'text-gray-700', icon: Pencil },
    PENDING_APPROVAL: { bg: 'bg-amber-100', text: 'text-amber-700', icon: Clock },
    DISCONTINUED: { bg: 'bg-red-100', text: 'text-red-700', icon: Ban },
  }[status] || { bg: 'bg-gray-100', text: 'text-gray-700', icon: Package };

  const Icon = config.icon;

  return (
    <span className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-xs font-medium ${config.bg} ${config.text}`}>
      <Icon className="h-3 w-3" />
      {status.replace(/_/g, ' ')}
    </span>
  );
}

// =============================================================================
// MAIN PAGE COMPONENT
// =============================================================================

export function Products() {
  const { products, selectProduct, selectedProduct, fetchProducts } = useProductStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [statusFilter, setStatusFilter] = useState<Product['status'] | 'ALL'>('ALL');

  // Dialog states
  const [editingProduct, setEditingProduct] = useState<Product | null | 'new'>(null);
  const [showCloneDialog, setShowCloneDialog] = useState(false);
  const [rejectProduct, setRejectProduct] = useState<Product | null>(null);
  const [discontinueProduct, setDiscontinueProduct] = useState<Product | null>(null);
  const [showWizard, setShowWizard] = useState(false);
  const [showCreateMenu, setShowCreateMenu] = useState(false);

  const [isSubmitting, setIsSubmitting] = useState(false);

  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  // Filter products
  const filteredProducts = useMemo(() => {
    return products.filter((p) => {
      const matchesSearch = p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        p.id.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesStatus = statusFilter === 'ALL' || p.status === statusFilter;
      return matchesSearch && matchesStatus;
    });
  }, [products, searchQuery, statusFilter]);

  // Status counts
  const statusCounts = useMemo(() => {
    const counts = { DRAFT: 0, PENDING_APPROVAL: 0, ACTIVE: 0, DISCONTINUED: 0 };
    products.forEach((p) => {
      if (p.status in counts) {
        counts[p.status as keyof typeof counts]++;
      }
    });
    return counts;
  }, [products]);

  // Template types for form
  const templateTypes = useMemo(() => {
    return [...new Set(products.map((p) => p.templateType))];
  }, [products]);

  // Handlers
  const handleSaveProduct = async (data: Partial<Product>) => {
    if (editingProduct === 'new') {
      await api.products.create(data);
    } else if (editingProduct) {
      await api.products.update(editingProduct.id, data);
    }
    setEditingProduct(null);
    fetchProducts();
  };

  const handleSubmit = async (product: Product) => {
    if (!confirm(`Submit "${product.name}" for approval?`)) return;
    setIsSubmitting(true);
    try {
      await api.products.submit(product.id);
      fetchProducts();
    } catch (e) {
      alert((e as Error).message);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleApprove = async (product: Product) => {
    const comments = prompt('Approval comments (optional):');
    setIsSubmitting(true);
    try {
      await api.products.approve(product.id, comments || undefined);
      fetchProducts();
    } catch (e) {
      alert((e as Error).message);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReject = async (reason: string) => {
    if (!rejectProduct) return;
    await api.products.reject(rejectProduct.id, reason);
    setRejectProduct(null);
    fetchProducts();
  };

  const handleDiscontinue = async (reason?: string) => {
    if (!discontinueProduct) return;
    await api.products.discontinue(discontinueProduct.id, reason);
    setDiscontinueProduct(null);
    fetchProducts();
  };

  const handleDelete = async (product: Product) => {
    if (!confirm(`Delete "${product.name}"? This cannot be undone.`)) return;
    try {
      await api.products.delete(product.id);
      fetchProducts();
    } catch (e) {
      alert((e as Error).message);
    }
  };


  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Products</h2>
          <p className="text-muted-foreground">
            Manage your product configurations and rule sets.
          </p>
        </div>
        <div className="relative">
          <Button onClick={() => setShowCreateMenu(!showCreateMenu)} className="gap-2">
            <Plus className="h-4 w-4" />
            New Product
          </Button>
          {showCreateMenu && (
            <>
              <div className="fixed inset-0 z-10" onClick={() => setShowCreateMenu(false)} />
              <div className="absolute right-0 top-full mt-1 w-56 bg-white border rounded-lg shadow-lg z-20 overflow-hidden">
                <div className="py-1">
                  <button
                    onClick={() => {
                      setShowWizard(true);
                      setShowCreateMenu(false);
                    }}
                    className="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-start gap-3"
                  >
                    <LayoutTemplate className="h-5 w-5 text-primary mt-0.5" />
                    <div>
                      <div className="font-medium">From Template</div>
                      <div className="text-xs text-gray-500">
                        Create a product using a pre-defined template with components, attributes, and functionalities
                      </div>
                    </div>
                  </button>
                  <div className="border-t" />
                  <button
                    onClick={() => {
                      setEditingProduct('new');
                      setShowCreateMenu(false);
                    }}
                    className="w-full px-4 py-3 text-left hover:bg-gray-50 flex items-start gap-3"
                  >
                    <FilePlus className="h-5 w-5 text-gray-500 mt-0.5" />
                    <div>
                      <div className="font-medium">From Scratch</div>
                      <div className="text-xs text-gray-500">
                        Create an empty product and manually add components and attributes
                      </div>
                    </div>
                  </button>
                </div>
              </div>
            </>
          )}
        </div>
      </div>

      {/* Stats */}
      <div className="grid gap-4 md:grid-cols-5">
        <Card
          className={`cursor-pointer transition-all ${statusFilter === 'ALL' ? 'ring-2 ring-primary' : 'hover:border-gray-300'}`}
          onClick={() => setStatusFilter('ALL')}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Total</CardTitle>
            <Package className="h-4 w-4 text-purple-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{products.length}</div>
          </CardContent>
        </Card>
        <Card
          className={`cursor-pointer transition-all ${statusFilter === 'DRAFT' ? 'ring-2 ring-gray-400' : 'hover:border-gray-300'}`}
          onClick={() => setStatusFilter('DRAFT')}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Draft</CardTitle>
            <Pencil className="h-4 w-4 text-gray-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts.DRAFT}</div>
          </CardContent>
        </Card>
        <Card
          className={`cursor-pointer transition-all ${statusFilter === 'PENDING_APPROVAL' ? 'ring-2 ring-amber-400' : 'hover:border-gray-300'}`}
          onClick={() => setStatusFilter('PENDING_APPROVAL')}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Pending</CardTitle>
            <Clock className="h-4 w-4 text-amber-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts.PENDING_APPROVAL}</div>
          </CardContent>
        </Card>
        <Card
          className={`cursor-pointer transition-all ${statusFilter === 'ACTIVE' ? 'ring-2 ring-emerald-400' : 'hover:border-gray-300'}`}
          onClick={() => setStatusFilter('ACTIVE')}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Active</CardTitle>
            <CheckCircle className="h-4 w-4 text-emerald-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts.ACTIVE}</div>
          </CardContent>
        </Card>
        <Card
          className={`cursor-pointer transition-all ${statusFilter === 'DISCONTINUED' ? 'ring-2 ring-red-400' : 'hover:border-gray-300'}`}
          onClick={() => setStatusFilter('DISCONTINUED')}
        >
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-gray-600">Discontinued</CardTitle>
            <Ban className="h-4 w-4 text-red-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{statusCounts.DISCONTINUED}</div>
          </CardContent>
        </Card>
      </div>

      {/* Search */}
      <div className="relative max-w-sm">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search products..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="pl-10"
        />
      </div>

      {/* Product Grid */}
      {filteredProducts.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredProducts.map((product) => (
            <Card
              key={product.id}
              className={`cursor-pointer transition-all hover:shadow-md hover:border-primary ${
                selectedProduct?.id === product.id ? 'border-primary ring-1 ring-primary' : ''
              }`}
              onClick={() => selectProduct(product.id)}
            >
              <CardHeader className="flex flex-row items-start justify-between space-y-0">
                <div className="flex items-center gap-3">
                  <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                    <Package className="h-5 w-5 text-primary" />
                  </div>
                  <div>
                    <CardTitle className="text-base">{product.name}</CardTitle>
                    <CardDescription className="text-xs font-mono">{product.id}</CardDescription>
                  </div>
                </div>
                <ProductMenu
                  product={product}
                  onEdit={() => setEditingProduct(product)}
                  onClone={() => { selectProduct(product.id); setShowCloneDialog(true); }}
                  onSubmit={() => handleSubmit(product)}
                  onApprove={() => handleApprove(product)}
                  onReject={() => setRejectProduct(product)}
                  onDiscontinue={() => setDiscontinueProduct(product)}
                  onDelete={() => handleDelete(product)}
                />
              </CardHeader>
              <CardContent>
                {product.description && (
                  <p className="text-sm text-muted-foreground mb-3 line-clamp-2">
                    {product.description}
                  </p>
                )}
                <div className="flex items-center justify-between">
                  <StatusBadge status={product.status} />
                  <div className="flex items-center gap-2 text-xs text-gray-500">
                    <span className="px-1.5 py-0.5 bg-gray-100 rounded">{product.templateType}</span>
                  </div>
                </div>
                {product.effectiveFrom && (
                  <div className="mt-2 flex items-center gap-1 text-xs text-gray-500">
                    <Calendar className="h-3 w-3" />
                    <span>Effective: {new Date(product.effectiveFrom).toLocaleDateString()}</span>
                    {product.expiryAt && (
                      <span className="text-red-500 ml-2">
                        Expires: {new Date(product.expiryAt).toLocaleDateString()}
                      </span>
                    )}
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Package className="h-12 w-12 text-muted-foreground/50" />
            <h3 className="mt-4 text-lg font-semibold">
              {searchQuery || statusFilter !== 'ALL' ? 'No products found' : 'No products yet'}
            </h3>
            <p className="mt-2 text-sm text-muted-foreground">
              {searchQuery || statusFilter !== 'ALL'
                ? 'Try adjusting your filters.'
                : 'Create your first product to get started.'}
            </p>
            {!searchQuery && statusFilter === 'ALL' && (
              <div className="flex gap-2 mt-4">
                <Button onClick={() => setShowWizard(true)} className="gap-2">
                  <LayoutTemplate className="h-4 w-4" />
                  From Template
                </Button>
                <Button variant="outline" onClick={() => setEditingProduct('new')} className="gap-2">
                  <FilePlus className="h-4 w-4" />
                  From Scratch
                </Button>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Selected Product Actions */}
      {selectedProduct && (
        <Card className="border-primary">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <Package className="h-5 w-5" />
                  {selectedProduct.name}
                </CardTitle>
                <CardDescription>
                  {selectedProduct.description || 'No description'}
                </CardDescription>
              </div>
              <StatusBadge status={selectedProduct.status} />
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex flex-wrap gap-2">
              <Button asChild variant="default">
                <Link to="/rules" className="gap-2">
                  <GitBranch className="h-4 w-4" />
                  Rule Canvas
                </Link>
              </Button>
              <Button asChild variant="outline">
                <Link to="/attributes" className="gap-2">
                  <Layers className="h-4 w-4" />
                  Attributes
                </Link>
              </Button>
              <Button variant="outline" onClick={() => setShowCloneDialog(true)} className="gap-2">
                <Copy className="h-4 w-4" />
                Clone
              </Button>

              {/* Lifecycle actions based on status */}
              {selectedProduct.status === 'DRAFT' && (
                <Button
                  variant="default"
                  onClick={() => handleSubmit(selectedProduct)}
                  disabled={isSubmitting}
                  className="gap-2 bg-blue-600 hover:bg-blue-700"
                >
                  <Send className="h-4 w-4" />
                  Submit for Approval
                </Button>
              )}
              {selectedProduct.status === 'PENDING_APPROVAL' && (
                <>
                  <Button
                    variant="default"
                    onClick={() => handleApprove(selectedProduct)}
                    disabled={isSubmitting}
                    className="gap-2 bg-emerald-600 hover:bg-emerald-700"
                  >
                    <CheckCircle className="h-4 w-4" />
                    Approve
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setRejectProduct(selectedProduct)}
                    className="gap-2 text-red-600 border-red-200 hover:bg-red-50"
                  >
                    <XCircle className="h-4 w-4" />
                    Reject
                  </Button>
                </>
              )}
              {selectedProduct.status === 'ACTIVE' && (
                <Button
                  variant="outline"
                  onClick={() => setDiscontinueProduct(selectedProduct)}
                  className="gap-2 text-red-600 border-red-200 hover:bg-red-50"
                >
                  <Ban className="h-4 w-4" />
                  Discontinue
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Dialogs */}
      {editingProduct && (
        <ProductForm
          product={editingProduct === 'new' ? null : editingProduct}
          existingTemplateTypes={templateTypes}
          onSave={handleSaveProduct}
          onCancel={() => setEditingProduct(null)}
        />
      )}

      {/* Clone wizard - uses same wizard as template creation but in clone mode */}
      {showCloneDialog && selectedProduct && (
        <ProductCreationWizard
          mode="clone"
          sourceProduct={selectedProduct}
          onComplete={(productId) => {
            setShowCloneDialog(false);
            fetchProducts();
            selectProduct(productId);
          }}
          onCancel={() => setShowCloneDialog(false)}
        />
      )}

      {rejectProduct && (
        <RejectDialog
          title="Reject Product"
          entityName={rejectProduct.name}
          entityType="product"
          onReject={handleReject}
          onCancel={() => setRejectProduct(null)}
        />
      )}

      {discontinueProduct && (
        <DiscontinueDialog
          product={discontinueProduct}
          onDiscontinue={handleDiscontinue}
          onCancel={() => setDiscontinueProduct(null)}
        />
      )}

      {showWizard && (
        <ProductCreationWizard
          onComplete={(productId) => {
            setShowWizard(false);
            fetchProducts();
            selectProduct(productId);
          }}
          onCancel={() => setShowWizard(false)}
        />
      )}
    </div>
  );
}

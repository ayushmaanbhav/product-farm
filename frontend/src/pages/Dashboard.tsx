// Dashboard - Overview and Quick Access
// Shows stats, recent activity, and quick actions

import { useEffect } from 'react';
import { Link } from 'react-router-dom';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { useProductStore, useUIStore } from '@/store';
import {
  Package,
  Layers,
  GitBranch,
  TrendingUp,
  ArrowRight,
  Plus,
  Sparkles,
  FlaskConical,
  Target,
  Zap,
  MessageSquare,
} from 'lucide-react';

export function Dashboard() {
  const { products, abstractAttributes, rules, fetchProducts } = useProductStore();
  const { toggleChat } = useUIStore();

  useEffect(() => {
    fetchProducts();
  }, [fetchProducts]);

  const inputCount = abstractAttributes.filter((a) => a.tags.some((t) => t.name === 'input')).length;
  const computedCount = abstractAttributes.length - inputCount;

  const stats = [
    {
      name: 'Products',
      value: products.length,
      icon: Package,
      description: 'Total products configured',
      color: 'text-blue-600 bg-blue-100',
    },
    {
      name: 'Attributes',
      value: abstractAttributes.length,
      icon: Layers,
      description: `${inputCount} inputs, ${computedCount} computed`,
      color: 'text-purple-600 bg-purple-100',
    },
    {
      name: 'Rules',
      value: rules.length,
      icon: GitBranch,
      description: `${rules.filter((r) => r.enabled).length} active`,
      color: 'text-emerald-600 bg-emerald-100',
    },
    {
      name: 'Execution Levels',
      value: rules.length > 0 ? Math.ceil(rules.length / 2) : 0,
      icon: TrendingUp,
      description: 'DAG depth',
      color: 'text-amber-600 bg-amber-100',
    },
  ];

  const quickActions = [
    {
      title: 'Create New Product',
      description: 'Set up a new product with attributes and rules',
      icon: Plus,
      href: '/products',
      color: 'bg-blue-500',
    },
    {
      title: 'Build Rules Visually',
      description: 'Use the interactive DAG editor',
      icon: GitBranch,
      href: '/rules',
      color: 'bg-emerald-500',
    },
    {
      title: 'Test with Simulation',
      description: 'Run scenarios and see live results',
      icon: FlaskConical,
      href: '/rules',
      color: 'bg-purple-500',
    },
    {
      title: 'Ask AI Assistant',
      description: 'Get help creating complex rules',
      icon: Sparkles,
      onClick: toggleChat,
      color: 'bg-indigo-500',
    },
  ];

  const features = [
    {
      icon: GitBranch,
      title: 'Visual Rule Canvas',
      description: 'Interactive DAG visualization with execution levels, impact analysis, and minimap navigation',
    },
    {
      icon: Zap,
      title: 'Block-based Editor',
      description: 'Build JSON Logic expressions visually with drag-and-drop blocks',
    },
    {
      icon: FlaskConical,
      title: 'Real-time Simulation',
      description: 'Test rules instantly with live feedback and save scenarios',
    },
    {
      icon: Target,
      title: 'Impact Analysis',
      description: 'See which attributes and rules are affected by changes',
    },
    {
      icon: MessageSquare,
      title: 'AI Assistant',
      description: 'Create rules from natural language descriptions',
    },
    {
      icon: Layers,
      title: 'Attribute Explorer',
      description: 'Hierarchical view of all attributes organized by component',
    },
  ];

  return (
    <div className="space-y-8">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-3xl font-bold tracking-tight text-gray-900">
            Welcome to Product-FARM
          </h2>
          <p className="mt-1 text-lg text-gray-500">
            Build, visualize, and test complex rule-based products with AI assistance.
          </p>
        </div>
        <Button asChild size="lg" className="gap-2">
          <Link to="/rules">
            <GitBranch className="h-5 w-5" />
            Open Rule Canvas
            <ArrowRight className="h-4 w-4" />
          </Link>
        </Button>
      </div>

      {/* Stats Grid */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <Card key={stat.name} className="relative overflow-hidden">
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-gray-600">{stat.name}</CardTitle>
              <div className={`rounded-lg p-2 ${stat.color}`}>
                <stat.icon className="h-4 w-4" />
              </div>
            </CardHeader>
            <CardContent>
              <div className="text-3xl font-bold text-gray-900">{stat.value}</div>
              <p className="text-xs text-gray-500 mt-1">{stat.description}</p>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Quick Actions */}
      <div>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Quick Actions</h3>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {quickActions.map((action) =>
            action.href ? (
              <Link key={action.title} to={action.href}>
                <Card className="h-full transition-all hover:shadow-md hover:border-gray-300 cursor-pointer">
                  <CardContent className="pt-6">
                    <div className={`w-10 h-10 rounded-lg ${action.color} flex items-center justify-center mb-3`}>
                      <action.icon className="h-5 w-5 text-white" />
                    </div>
                    <h4 className="font-semibold text-gray-900">{action.title}</h4>
                    <p className="text-sm text-gray-500 mt-1">{action.description}</p>
                  </CardContent>
                </Card>
              </Link>
            ) : (
              <Card
                key={action.title}
                className="h-full transition-all hover:shadow-md hover:border-gray-300 cursor-pointer"
                onClick={action.onClick}
              >
                <CardContent className="pt-6">
                  <div className={`w-10 h-10 rounded-lg ${action.color} flex items-center justify-center mb-3`}>
                    <action.icon className="h-5 w-5 text-white" />
                  </div>
                  <h4 className="font-semibold text-gray-900">{action.title}</h4>
                  <p className="text-sm text-gray-500 mt-1">{action.description}</p>
                </CardContent>
              </Card>
            )
          )}
        </div>
      </div>

      {/* Features Overview */}
      <div className="grid gap-6 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sparkles className="h-5 w-5 text-indigo-500" />
              Key Features
            </CardTitle>
            <CardDescription>
              Everything you need to manage complex rule-based products
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 sm:grid-cols-2">
              {features.map((feature) => (
                <div key={feature.title} className="flex gap-3">
                  <div className="flex-shrink-0 w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center">
                    <feature.icon className="h-4 w-4 text-gray-600" />
                  </div>
                  <div>
                    <h4 className="text-sm font-medium text-gray-900">{feature.title}</h4>
                    <p className="text-xs text-gray-500 mt-0.5">{feature.description}</p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <MessageSquare className="h-5 w-5 text-purple-500" />
              AI Assistant Tips
            </CardTitle>
            <CardDescription>
              Try these prompts to get started with the AI assistant
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            {[
              'Create a rule: if customer_age > 60, add 20% to premium',
              'Explain how the premium calculation works',
              'What would be affected if I change vehicle_value?',
              'Suggest optimizations for my rules',
              'Create a validation rule for age between 18-100',
            ].map((prompt) => (
              <button
                key={prompt}
                onClick={toggleChat}
                className="flex items-start gap-2 w-full text-left rounded-lg border p-3 text-sm hover:bg-gray-50 transition-colors"
              >
                <Sparkles className="h-4 w-4 text-purple-500 mt-0.5 shrink-0" />
                <span className="text-gray-700">{prompt}</span>
              </button>
            ))}
          </CardContent>
        </Card>
      </div>

      {/* Recent Products (if any) */}
      {products.length > 0 && (
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <div>
              <CardTitle>Recent Products</CardTitle>
              <CardDescription>Your configured products</CardDescription>
            </div>
            <Button asChild variant="outline" size="sm">
              <Link to="/products">View All</Link>
            </Button>
          </CardHeader>
          <CardContent>
            <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
              {products.slice(0, 3).map((product) => (
                <Link
                  key={product.id}
                  to="/rules"
                  className="flex items-center gap-3 rounded-lg border p-3 hover:bg-gray-50 transition-colors"
                >
                  <div className="w-10 h-10 rounded-lg bg-blue-100 flex items-center justify-center">
                    <Package className="h-5 w-5 text-blue-600" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="font-medium text-gray-900 truncate">{product.name}</p>
                    <p className="text-xs text-gray-500">{product.templateType}</p>
                  </div>
                  <span
                    className={`px-2 py-0.5 rounded text-[10px] font-medium ${
                      product.status === 'ACTIVE'
                        ? 'bg-emerald-100 text-emerald-700'
                        : product.status === 'DRAFT'
                          ? 'bg-gray-100 text-gray-700'
                          : 'bg-amber-100 text-amber-700'
                    }`}
                  >
                    {product.status}
                  </span>
                </Link>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}

export default Dashboard;

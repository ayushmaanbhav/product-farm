import { Link, Outlet, useLocation } from 'react-router-dom';
import { cn } from '@/lib/utils';
import { useUIStore } from '@/store';
import { Button } from '@/components/ui/button';
import { AIChat } from '@/components/AIChat';
import {
  Package,
  Layers,
  GitBranch,
  MessageSquare,
  Menu,
  Settings,
  Home,
  Boxes,
  Database,
  List,
} from 'lucide-react';

const navigation = [
  { name: 'Dashboard', href: '/', icon: Home },
  { name: 'Products', href: '/products', icon: Package },
  { name: 'Datatypes', href: '/datatypes', icon: Database },
  { name: 'Enumerations', href: '/enumerations', icon: List },
  { name: 'Attributes', href: '/attributes', icon: Layers },
  { name: 'Functions', href: '/functionalities', icon: Boxes },
  { name: 'Rules', href: '/rules', icon: GitBranch },
];

export function Layout() {
  const location = useLocation();
  const { sidebarOpen, toggleSidebar, chatOpen, toggleChat } = useUIStore();

  return (
    <div className="flex h-screen bg-background">
      {/* Sidebar */}
      <aside
        className={cn(
          'flex flex-col border-r bg-card transition-all duration-300',
          sidebarOpen ? 'w-64' : 'w-16'
        )}
      >
        {/* Logo */}
        <div className="flex h-16 items-center justify-between border-b px-4">
          {sidebarOpen && (
            <span className="text-lg font-bold text-primary">Product-FARM</span>
          )}
          <Button variant="ghost" size="icon" onClick={toggleSidebar}>
            <Menu className="h-5 w-5" />
          </Button>
        </div>

        {/* Navigation */}
        <nav className="flex-1 space-y-1 p-2">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href;
            return (
              <Link
                key={item.name}
                to={item.href}
                className={cn(
                  'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
                  isActive
                    ? 'bg-primary text-primary-foreground'
                    : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
                )}
              >
                <item.icon className="h-5 w-5 shrink-0" />
                {sidebarOpen && <span>{item.name}</span>}
              </Link>
            );
          })}
        </nav>

        {/* Settings */}
        <div className="border-t p-2">
          <Link
            to="/settings"
            className="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
          >
            <Settings className="h-5 w-5 shrink-0" />
            {sidebarOpen && <span>Settings</span>}
          </Link>
        </div>
      </aside>

      {/* Main Content */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Header */}
        <header className="flex h-16 items-center justify-between border-b bg-card px-6">
          <h1 className="text-xl font-semibold">
            {navigation.find((n) => n.href === location.pathname)?.name || 'Settings'}
          </h1>
          <Button
            variant={chatOpen ? 'default' : 'outline'}
            size="sm"
            onClick={toggleChat}
            className="gap-2"
          >
            <MessageSquare className="h-4 w-4" />
            AI Assistant
          </Button>
        </header>

        {/* Content Area */}
        <div className="flex flex-1 overflow-hidden">
          {/* Main Content */}
          <main className="flex-1 overflow-auto p-6">
            <Outlet />
          </main>

          {/* AI Chat Panel */}
          {chatOpen && (
            <aside className="w-96 border-l bg-card">
              <AIChat />
            </aside>
          )}
        </div>
      </div>
    </div>
  );
}

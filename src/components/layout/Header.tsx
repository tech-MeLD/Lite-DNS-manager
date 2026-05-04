import { useLocation } from 'react-router-dom';
import { Moon, Sun, ChevronRight, Home } from 'lucide-react';
import { Link } from 'react-router-dom';
import { useTheme } from '../../context/ThemeContext';

const routeLabels: Record<string, string> = {
  '/': 'Dashboard',
  '/credentials': 'Credentials',
  '/domains': 'Domains',
  '/search': 'Search',
  '/settings': 'Settings',
};

export default function Header() {
  const location = useLocation();
  const { resolvedTheme, toggleTheme } = useTheme();

  const pathSegments = location.pathname.split('/').filter(Boolean);
  const breadcrumbs = [{ path: '/', label: 'Home' }];

  if (pathSegments.length > 0) {
    breadcrumbs.push({
      path: `/${pathSegments[0]}`,
      label: routeLabels[`/${pathSegments[0]}`] || pathSegments[0],
    });
  }

  if (pathSegments.length > 1) {
    breadcrumbs.push({
      path: location.pathname,
      label: decodeURIComponent(pathSegments.slice(1).join('/')),
    });
  }

  return (
    <header className="flex h-14 items-center justify-between border-b border-border bg-card px-6">
      <nav className="flex items-center gap-1 text-sm text-muted-foreground">
        {breadcrumbs.map((crumb, i) => (
          <span key={crumb.path} className="flex items-center gap-1">
            {i > 0 && <ChevronRight className="h-3 w-3" />}
            {i === 0 ? (
              <Link to={crumb.path} className="hover:text-foreground transition-colors">
                <Home className="h-4 w-4" />
              </Link>
            ) : i < breadcrumbs.length - 1 ? (
              <Link to={crumb.path} className="hover:text-foreground transition-colors">
                {crumb.label}
              </Link>
            ) : (
              <span className="text-foreground font-medium">{crumb.label}</span>
            )}
          </span>
        ))}
      </nav>
      <button
        onClick={toggleTheme}
        className="rounded-md p-2 text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        title={`Switch to ${resolvedTheme === 'dark' ? 'light' : 'dark'} mode`}
      >
        {resolvedTheme === 'dark' ? (
          <Sun className="h-4 w-4" />
        ) : (
          <Moon className="h-4 w-4" />
        )}
      </button>
    </header>
  );
}

import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { Globe, Key, Search, Plus } from 'lucide-react';
import { getDomainSummary } from '../lib/tauri';
import { getCredentials } from '../lib/tauri';
import { useApp } from '../context/AppContext';
import LoadingSpinner from '../components/common/LoadingSpinner';
import ErrorAlert from '../components/common/ErrorAlert';
import type { DomainSummary } from '../types';

export default function Dashboard() {
  const { state, dispatch } = useApp();
  const [summary, setSummary] = useState<DomainSummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchData = async () => {
    setLoading(true);
    setError(null);
    try {
      const [creds, domSummary] = await Promise.all([
        getCredentials(),
        getDomainSummary(),
      ]);
      dispatch({ type: 'SET_CREDENTIALS', payload: creds });
      setSummary(domSummary);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  if (loading) return <LoadingSpinner size="lg" />;
  if (error) return <ErrorAlert message={error} onRetry={fetchData} />;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Dashboard</h1>
        <p className="text-sm text-muted-foreground mt-1">
          Manage your DNS across all providers
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          title="Total Domains"
          value={summary?.total_domains ?? 0}
          icon={<Globe className="h-5 w-5" />}
          color="bg-primary/10 text-primary"
        />
        <StatCard
          title="DNSPod"
          value={summary?.dnspod_count ?? 0}
          icon={<Globe className="h-5 w-5" />}
          color="bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-200"
        />
        <StatCard
          title="Cloudflare"
          value={summary?.cloudflare_count ?? 0}
          icon={<Globe className="h-5 w-5" />}
          color="bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-200"
        />
        <StatCard
          title="AliDNS"
          value={summary?.alidns_count ?? 0}
          icon={<Globe className="h-5 w-5" />}
          color="bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-200"
        />
      </div>

      {/* Quick Actions */}
      <div>
        <h2 className="text-lg font-semibold text-foreground mb-3">Quick Actions</h2>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3">
          <Link
            to="/credentials"
            className="flex items-center gap-3 rounded-lg border border-border p-4 hover:bg-accent transition-colors"
          >
            <Key className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium text-foreground">Manage Credentials</p>
              <p className="text-xs text-muted-foreground">
                {state.credentials.length} provider(s) configured
              </p>
            </div>
          </Link>
          <Link
            to="/domains"
            className="flex items-center gap-3 rounded-lg border border-border p-4 hover:bg-accent transition-colors"
          >
            <Globe className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium text-foreground">View Domains</p>
              <p className="text-xs text-muted-foreground">
                {summary?.total_domains ?? 0} total domains
              </p>
            </div>
          </Link>
          <Link
            to="/search"
            className="flex items-center gap-3 rounded-lg border border-border p-4 hover:bg-accent transition-colors"
          >
            <Search className="h-5 w-5 text-muted-foreground" />
            <div>
              <p className="text-sm font-medium text-foreground">Search Records</p>
              <p className="text-xs text-muted-foreground">
                Cross-provider record search
              </p>
            </div>
          </Link>
        </div>
      </div>

      {/* Provider Overview */}
      <div>
        <h2 className="text-lg font-semibold text-foreground mb-3">Provider Overview</h2>
        {state.credentials.length === 0 ? (
          <div className="rounded-lg border border-dashed border-border p-8 text-center">
            <Key className="mx-auto h-8 w-8 text-muted-foreground/50" />
            <p className="mt-2 text-sm text-muted-foreground">
              No credentials configured yet
            </p>
            <Link
              to="/credentials"
              className="mt-3 inline-flex items-center gap-1 text-sm font-medium text-primary hover:underline"
            >
              <Plus className="h-3 w-3" />
              Add your first credential
            </Link>
          </div>
        ) : (
          <div className="space-y-2">
            {state.credentials.map((cred) => (
              <div
                key={cred.id}
                className="flex items-center justify-between rounded-lg border border-border px-4 py-3"
              >
                <div>
                  <p className="text-sm font-medium text-foreground">{cred.label}</p>
                  <p className="text-xs text-muted-foreground">
                    {cred.provider_type.toUpperCase()}
                  </p>
                </div>
                <span className="flex h-2 w-2 rounded-full bg-green-500" title="Connected" />
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function StatCard({
  title,
  value,
  icon,
  color,
}: {
  title: string;
  value: number;
  icon: React.ReactNode;
  color: string;
}) {
  return (
    <div className="rounded-lg border border-border bg-card p-4">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">{title}</p>
        <div className={`rounded-md p-1.5 ${color}`}>{icon}</div>
      </div>
      <p className="mt-2 text-2xl font-bold text-foreground">{value}</p>
    </div>
  );
}

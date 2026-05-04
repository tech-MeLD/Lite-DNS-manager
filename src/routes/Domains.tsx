import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { Globe, RefreshCw } from 'lucide-react';
import { listDomains } from '../lib/tauri';
import ProviderBadge from '../components/common/ProviderBadge';
import LoadingSpinner from '../components/common/LoadingSpinner';
import ErrorAlert from '../components/common/ErrorAlert';
import type { Domain } from '../types';
import { formatDate } from '../lib/utils';

export default function Domains() {
  const [domains, setDomains] = useState<Domain[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<string>('all');

  const fetchDomains = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listDomains();
      setDomains(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchDomains();
  }, []);

  const filtered = filter === 'all' ? domains : domains.filter((d) => d.provider === filter);

  if (loading) return <LoadingSpinner size="lg" />;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Domains</h1>
          <p className="text-sm text-muted-foreground mt-1">
            {domains.length} domains across all providers
          </p>
        </div>
        <button
          onClick={fetchDomains}
          className="inline-flex items-center gap-1 rounded-md border border-border px-3 py-2 text-sm text-muted-foreground hover:bg-accent transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          Refresh
        </button>
      </div>

      {/* Filter Tabs */}
      <div className="flex gap-2 border-b border-border pb-2">
        {['all', 'cloudflare', 'dnspod', 'alidns'].map((f) => (
          <button
            key={f}
            onClick={() => setFilter(f)}
            className={`rounded-md px-3 py-1 text-sm font-medium transition-colors ${
              filter === f
                ? 'bg-primary/10 text-primary'
                : 'text-muted-foreground hover:bg-accent'
            }`}
          >
            {f === 'all' ? 'All' : f === 'dnspod' ? 'DNSPod' : f === 'alidns' ? 'AliDNS' : 'Cloudflare'}
          </button>
        ))}
      </div>

      {error && <ErrorAlert message={error} onRetry={fetchDomains} />}

      {filtered.length === 0 && !error ? (
        <div className="rounded-lg border border-dashed border-border p-12 text-center">
          <Globe className="mx-auto h-8 w-8 text-muted-foreground/50" />
          <p className="mt-2 text-sm text-muted-foreground">No domains found</p>
          <p className="text-xs text-muted-foreground/70 mt-1">
            Make sure your credentials are configured and valid
          </p>
        </div>
      ) : (
        <div className="rounded-lg border border-border">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border bg-muted/50">
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Domain</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Provider</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Status</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Records</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Created</th>
              </tr>
            </thead>
            <tbody>
              {filtered.map((domain) => (
                <tr
                  key={`${domain.provider}-${domain.id}`}
                  className="border-b border-border hover:bg-accent/50 transition-colors"
                >
                  <td className="px-4 py-3">
                    <Link
                      to={`/domains/${domain.provider}/${domain.id}`}
                      className="text-sm font-medium text-primary hover:underline"
                    >
                      {domain.name}
                    </Link>
                  </td>
                  <td className="px-4 py-3">
                    <ProviderBadge provider={domain.provider} />
                  </td>
                  <td className="px-4 py-3">
                    <span
                      className={`inline-flex items-center gap-1 text-xs ${
                        domain.status === 'active'
                          ? 'text-green-600'
                          : 'text-yellow-600'
                      }`}
                    >
                      <span className={`h-1.5 w-1.5 rounded-full ${
                        domain.status === 'active' ? 'bg-green-500' : 'bg-yellow-500'
                      }`} />
                      {domain.status}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">
                    {domain.record_count}
                  </td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">
                    {formatDate(domain.created_on)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

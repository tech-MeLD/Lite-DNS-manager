import { useState } from 'react';
import { Search as SearchIcon } from 'lucide-react';
import { searchRecords } from '../lib/tauri';
import ProviderBadge from '../components/common/ProviderBadge';
import RecordTypeBadge from '../components/common/RecordTypeBadge';
import LoadingSpinner from '../components/common/LoadingSpinner';
import ErrorAlert from '../components/common/ErrorAlert';
import type { ProviderType, RecordType, SearchQuery, SearchResult } from '../types';

export default function Search() {
  const [keyword, setKeyword] = useState('');
  const [providers, setProviders] = useState<ProviderType[]>([]);
  const [recordType, setRecordType] = useState<RecordType | ''>('');
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const toggleProvider = (p: ProviderType) => {
    setProviders((prev) =>
      prev.includes(p) ? prev.filter((x) => x !== p) : [...prev, p]
    );
  };

  const handleSearch = async () => {
    if (!keyword.trim()) return;
    setLoading(true);
    setSearched(true);
    try {
      const query: SearchQuery = {
        keyword: keyword.trim(),
        providers: providers.length > 0 ? providers : undefined,
        record_type: recordType || undefined,
      };
      const res = await searchRecords(query);
      setResults(res);
    } catch (e) {
      setError(String(e));
      setResults([]);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-foreground">Search</h1>
        <p className="text-sm text-muted-foreground mt-1">
          Search DNS records across all providers
        </p>
      </div>

      {/* Search Bar */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <SearchIcon className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <input
            type="text"
            value={keyword}
            onChange={(e) => setKeyword(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Search by name, content, or record type..."
            className="w-full rounded-md border border-input bg-background pl-10 pr-4 py-2.5 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          />
        </div>
        <button
          onClick={handleSearch}
          disabled={!keyword.trim()}
          className="rounded-md bg-primary px-6 py-2.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
        >
          Search
        </button>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap gap-4">
        <div>
          <span className="text-xs text-muted-foreground uppercase font-medium">Providers</span>
          <div className="mt-1 flex gap-2">
            {(['cloudflare', 'dnspod', 'alidns'] as ProviderType[]).map((p) => (
              <button
                key={p}
                onClick={() => toggleProvider(p)}
                className={`rounded-md px-3 py-1 text-xs font-medium border transition-colors ${
                  providers.includes(p)
                    ? 'border-primary bg-primary/10 text-primary'
                    : 'border-border text-muted-foreground hover:bg-accent'
                }`}
              >
                {p === 'dnspod' ? 'DNSPod' : p === 'alidns' ? 'AliDNS' : 'Cloudflare'}
              </button>
            ))}
          </div>
        </div>
        <div>
          <span className="text-xs text-muted-foreground uppercase font-medium">Record Type</span>
          <div className="mt-1">
            <select
              value={recordType}
              onChange={(e) => setRecordType(e.target.value as RecordType | '')}
              className="rounded-md border border-border bg-background px-3 py-1.5 text-sm text-foreground"
            >
              <option value="">All types</option>
              {['A', 'AAAA', 'CNAME', 'MX', 'TXT', 'NS', 'SRV', 'CAA', 'SOA', 'PTR'].map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>
        </div>
      </div>

      {/* Results */}
      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}
      {loading ? (
        <LoadingSpinner />
      ) : searched ? (
        results.length === 0 ? (
          <div className="rounded-lg border border-dashed border-border p-12 text-center">
            <SearchIcon className="mx-auto h-8 w-8 text-muted-foreground/50" />
            <p className="mt-2 text-sm text-muted-foreground">No records found</p>
          </div>
        ) : (
          <div>
            <p className="mb-3 text-sm text-muted-foreground">{results.length} results found</p>
            <div className="rounded-lg border border-border">
              <table className="w-full">
                <thead>
                  <tr className="border-b border-border bg-muted/50">
                    <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Type</th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Name</th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Content</th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Domain</th>
                    <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Provider</th>
                  </tr>
                </thead>
                <tbody>
                  {results.map((r, i) => (
                    <tr key={i} className="border-b border-border hover:bg-accent/50">
                      <td className="px-4 py-3">
                        <RecordTypeBadge type={r.record.record_type} />
                      </td>
                      <td className="px-4 py-3 text-sm text-foreground font-mono">
                        {r.record.name}
                      </td>
                      <td className="px-4 py-3 text-sm text-foreground font-mono max-w-xs truncate">
                        {r.record.content}
                      </td>
                      <td className="px-4 py-3 text-sm text-muted-foreground">
                        {r.domain_name}
                      </td>
                      <td className="px-4 py-3">
                        <ProviderBadge provider={r.provider} />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )
      ) : (
        <div className="rounded-lg border border-dashed border-border p-12 text-center">
          <SearchIcon className="mx-auto h-8 w-8 text-muted-foreground/50" />
          <p className="mt-2 text-sm text-muted-foreground">
            Enter a search term to find DNS records
          </p>
        </div>
      )}
    </div>
  );
}

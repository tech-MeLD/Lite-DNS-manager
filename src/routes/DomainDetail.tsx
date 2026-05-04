import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { Plus, Trash2, RefreshCw, Download } from 'lucide-react';
import {
  listRecords,
  createRecord,
  updateRecord,
  deleteRecord,
  exportZone,
} from '../lib/tauri';
import ProviderBadge from '../components/common/ProviderBadge';
import RecordTypeBadge from '../components/common/RecordTypeBadge';
import ConfirmDialog from '../components/common/ConfirmDialog';
import LoadingSpinner from '../components/common/LoadingSpinner';
import ErrorAlert from '../components/common/ErrorAlert';
import type { DnsRecord, CreateRecordRequest, UpdateRecordRequest, ProviderType, RecordType } from '../types';
const RECORD_TYPES: RecordType[] = ['A', 'AAAA', 'CNAME', 'MX', 'TXT', 'NS', 'SRV', 'CAA', 'SOA', 'PTR',
  'CERT', 'DNSKEY', 'DS', 'LOC', 'NAPTR', 'SMIMEA', 'SSHFP', 'TLSA', 'URI'];

export default function DomainDetail() {
  const { provider, domainId } = useParams<{ provider: string; domainId: string }>();
  const [records, setRecords] = useState<DnsRecord[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingRecord, setEditingRecord] = useState<DnsRecord | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<DnsRecord | null>(null);

  const [form, setForm] = useState<CreateRecordRequest>({
    record_type: 'A',
    name: '@',
    content: '',
    ttl: 3600,
    priority: undefined,
    proxied: false,
  });

  const fetchRecords = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await listRecords(provider as ProviderType, domainId!);
      setRecords(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    let ignore = false;
    const load = async () => {
      setLoading(true);
      setError(null);
      try {
        const result = await listRecords(provider as ProviderType, domainId!);
        if (!ignore) setRecords(result);
      } catch (e) {
        if (!ignore) setError(String(e));
      } finally {
        if (!ignore) setLoading(false);
      }
    };
    load();
    return () => { ignore = true; };
  }, [provider, domainId]);

  const handleOpenCreate = () => {
    setEditingRecord(null);
    setForm({ record_type: 'A', name: '@', content: '', ttl: 3600, proxied: false });
    setShowForm(true);
  };

  const handleOpenEdit = (record: DnsRecord) => {
    setEditingRecord(record);
    setForm({
      record_type: record.record_type,
      name: record.name,
      content: record.content,
      ttl: record.ttl,
      priority: record.priority ?? undefined,
      proxied: record.proxied ?? undefined,
    });
    setShowForm(true);
  };

  const handleSubmit = async () => {
    try {
      if (editingRecord) {
        const updateReq: UpdateRecordRequest = {
          record_type: form.record_type,
          name: form.name,
          content: form.content,
          ttl: form.ttl,
          priority: form.priority,
          proxied: form.proxied,
        };
        await updateRecord(provider as ProviderType, domainId!, editingRecord.id, updateReq);
      } else {
        await createRecord(provider as ProviderType, domainId!, form);
      }
      setShowForm(false);
      fetchRecords();
    } catch (e) {
      alert(`Failed to save record: ${e}`);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteRecord(provider as ProviderType, domainId!, deleteTarget.id);
      setDeleteTarget(null);
      fetchRecords();
    } catch (e) {
      alert(`Failed to delete record: ${e}`);
    }
  };

  const handleExport = async () => {
    try {
      const zone = await exportZone(provider as ProviderType, domainId!);
      const blob = new Blob([zone], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `zone-${domainId}.txt`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      alert(`Export failed: ${e}`);
    }
  };

  if (loading) return <LoadingSpinner size="lg" />;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">
            {records[0]?.domain_name || domainId}
          </h1>
          <div className="mt-1 flex items-center gap-2">
            <ProviderBadge provider={provider as ProviderType} />
            <span className="text-sm text-muted-foreground">{records.length} records</span>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleExport}
            className="inline-flex items-center gap-1 rounded-md border border-border px-3 py-2 text-sm text-muted-foreground hover:bg-accent transition-colors"
          >
            <Download className="h-4 w-4" />
            Export Zone
          </button>
          <button
            onClick={fetchRecords}
            className="inline-flex items-center gap-1 rounded-md border border-border px-3 py-2 text-sm text-muted-foreground hover:bg-accent transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </button>
          <button
            onClick={handleOpenCreate}
            className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
          >
            <Plus className="h-4 w-4" />
            Add Record
          </button>
        </div>
      </div>

      {error && <ErrorAlert message={error} onRetry={fetchRecords} />}

      {records.length === 0 && !error ? (
        <div className="rounded-lg border border-dashed border-border p-12 text-center">
          <p className="text-sm text-muted-foreground">No DNS records found</p>
          <button
            onClick={handleOpenCreate}
            className="mt-2 text-sm font-medium text-primary hover:underline"
          >
            Add your first record
          </button>
        </div>
      ) : (
        <div className="rounded-lg border border-border">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border bg-muted/50">
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Type</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Name</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Content</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">TTL</th>
                <th className="px-4 py-3 text-left text-xs font-medium text-muted-foreground uppercase">Status</th>
                <th className="px-4 py-3 text-right text-xs font-medium text-muted-foreground uppercase">Actions</th>
              </tr>
            </thead>
            <tbody>
              {records.map((record) => (
                <tr
                  key={record.id}
                  className="border-b border-border hover:bg-accent/50 transition-colors"
                >
                  <td className="px-4 py-3">
                    <RecordTypeBadge type={record.record_type} />
                  </td>
                  <td className="px-4 py-3 text-sm text-foreground font-mono">
                    {record.name}
                  </td>
                  <td className="px-4 py-3 text-sm text-foreground font-mono max-w-xs truncate">
                    {record.content}
                  </td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">
                    {record.ttl}
                  </td>
                  <td className="px-4 py-3">
                    {record.proxied ? (
                      <span className="inline-flex items-center rounded-full bg-orange-100 px-2 py-0.5 text-xs font-medium text-orange-700 dark:bg-orange-900 dark:text-orange-200">
                        Proxied
                      </span>
                    ) : (
                      <span className="text-xs text-muted-foreground">DNS only</span>
                    )}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <div className="flex items-center justify-end gap-1">
                      <button
                        onClick={() => handleOpenEdit(record)}
                        className="rounded-md px-2 py-1 text-xs text-muted-foreground hover:bg-accent transition-colors"
                      >
                        Edit
                      </button>
                      <button
                        onClick={() => setDeleteTarget(record)}
                        className="rounded-md p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
                      >
                        <Trash2 className="h-3 w-3" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {/* Record Form Modal */}
      {showForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-xl">
            <h2 className="text-lg font-semibold text-foreground">
              {editingRecord ? 'Edit Record' : 'Add Record'}
            </h2>

            <div className="mt-4 space-y-3">
              <div>
                <label className="block text-sm font-medium text-foreground mb-1">Type</label>
                <select
                  value={form.record_type}
                  onChange={(e) => setForm({ ...form, record_type: e.target.value as RecordType })}
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                >
                  {RECORD_TYPES.map((t) => (
                    <option key={t} value={t}>{t}</option>
                  ))}
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium text-foreground mb-1">Name</label>
                <input
                  type="text"
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  placeholder="@ or subdomain"
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-foreground mb-1">Content</label>
                <input
                  type="text"
                  value={form.content}
                  onChange={(e) => setForm({ ...form, content: e.target.value })}
                  placeholder="IP address or hostname"
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                />
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1">TTL</label>
                  <input
                    type="number"
                    value={form.ttl}
                    onChange={(e) => setForm({ ...form, ttl: parseInt(e.target.value) || 3600 })}
                    className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                  />
                </div>
                {form.record_type === 'MX' && (
                  <div>
                    <label className="block text-sm font-medium text-foreground mb-1">Priority</label>
                    <input
                      type="number"
                      value={form.priority || ''}
                      onChange={(e) =>
                        setForm({ ...form, priority: parseInt(e.target.value) || undefined })
                      }
                      className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm"
                    />
                  </div>
                )}
              </div>
              {provider === 'cloudflare' && (
                <label className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={form.proxied || false}
                    onChange={(e) => setForm({ ...form, proxied: e.target.checked })}
                    className="rounded border-input"
                  />
                  <span className="text-sm text-foreground">Proxied (Cloudflare CDN)</span>
                </label>
              )}
            </div>

            <div className="mt-6 flex justify-end gap-2">
              <button
                onClick={() => setShowForm(false)}
                className="rounded-md border border-border px-4 py-2 text-sm font-medium text-foreground hover:bg-accent transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleSubmit}
                disabled={!form.content}
                className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {editingRecord ? 'Update' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}

      <ConfirmDialog
        open={!!deleteTarget}
        title="Delete Record"
        message={`Delete ${deleteTarget?.record_type} record "${deleteTarget?.name}" → "${deleteTarget?.content}"? This cannot be undone.`}
        confirmLabel="Delete"
        destructive
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  );
}

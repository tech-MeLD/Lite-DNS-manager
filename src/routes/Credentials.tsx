import { useEffect, useState } from 'react';
import { Plus, Trash2, CheckCircle, XCircle } from 'lucide-react';
import { getCredentials, saveCredential, deleteCredential, testCredential } from '../lib/tauri';
import { useApp } from '../context/AppContext';
import ProviderBadge from '../components/common/ProviderBadge';
import ConfirmDialog from '../components/common/ConfirmDialog';
import LoadingSpinner from '../components/common/LoadingSpinner';
import ErrorAlert from '../components/common/ErrorAlert';
import type { CredentialInput, ProviderCredential, ProviderType } from '../types';

export default function Credentials() {
  const { state, dispatch } = useApp();
  const [loading, setLoading] = useState(!state.loaded);
  const [error, setError] = useState<string | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ProviderCredential | null>(null);
  const [testing, setTesting] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, boolean>>({});

  const [form, setForm] = useState<CredentialInput>({
    provider_type: 'cloudflare',
    label: '',
    api_token: '',
  });

  useEffect(() => {
    if (!state.loaded) {
      loadCredentials();
    }
  }, []);

  const loadCredentials = async () => {
    setLoading(true);
    setError(null);
    try {
      const creds = await getCredentials();
      dispatch({ type: 'SET_CREDENTIALS', payload: creds });
    } catch (e) {
      setError(`Failed to load credentials: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  const handleOpenForm = (cred?: ProviderCredential) => {
    if (cred) {
      setEditingId(cred.id);
      setForm({
        provider_type: cred.provider_type,
        label: cred.label,
      });
    } else {
      setEditingId(null);
      setForm({
        provider_type: 'cloudflare',
        label: '',
        api_token: '',
      });
    }
    setShowForm(true);
  };

  const handleSubmit = async () => {
    try {
      const saved = await saveCredential(form);
      dispatch({ type: 'ADD_CREDENTIAL', payload: saved });
      setShowForm(false);
      setError(null);
    } catch (e) {
      setError(`Failed to save: ${e}`);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await deleteCredential(deleteTarget.id);
      dispatch({ type: 'REMOVE_CREDENTIAL', payload: deleteTarget.id });
      setDeleteTarget(null);
      setError(null);
    } catch (e) {
      setError(`Failed to delete: ${e}`);
    }
  };

  const handleTest = async (id: string) => {
    setTesting(id);
    try {
      const result = await testCredential(id);
      setTestResults((prev) => ({ ...prev, [id]: result }));
    } catch {
      setTestResults((prev) => ({ ...prev, [id]: false }));
    } finally {
      setTesting(null);
    }
  };

  const updateField = (field: string, value: string) => {
    setForm((prev) => {
      // Clear other provider fields when switching provider
      if (field === 'provider_type') {
        return {
          provider_type: value as ProviderType,
          label: prev.label,
          [value === 'cloudflare' ? 'api_token' : value === 'dnspod' ? 'secret_id' : 'access_key_id']: '',
        };
      }
      return { ...prev, [field]: value };
    });
  };

  if (loading) return <LoadingSpinner size="lg" />;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-foreground">Credentials</h1>
          <p className="text-sm text-muted-foreground mt-1">
            Manage API keys for your DNS providers
          </p>
        </div>
        <button
          onClick={() => handleOpenForm()}
          className="inline-flex items-center gap-2 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          <Plus className="h-4 w-4" />
          Add Credential
        </button>
      </div>

      {error && <ErrorAlert message={error} onDismiss={() => setError(null)} />}

      {state.credentials.length === 0 ? (
        <div className="rounded-lg border border-dashed border-border p-12 text-center">
          <p className="text-muted-foreground">No credentials configured</p>
          <button
            onClick={() => handleOpenForm()}
            className="mt-2 text-sm font-medium text-primary hover:underline"
          >
            Add your first credential
          </button>
        </div>
      ) : (
        <div className="space-y-3">
          {state.credentials.map((cred) => (
            <div
              key={cred.id}
              className="flex items-center justify-between rounded-lg border border-border bg-card px-4 py-3"
            >
              <div className="flex items-center gap-3">
                <div>
                  <p className="text-sm font-medium text-foreground">{cred.label}</p>
                  <div className="mt-0.5 flex items-center gap-2">
                    <ProviderBadge provider={cred.provider_type} />
                    <span className="text-xs text-muted-foreground">
                      Added {new Date(cred.created_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {testResults[cred.id] !== undefined && (
                  testResults[cred.id] ? (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  ) : (
                    <XCircle className="h-4 w-4 text-destructive" />
                  )
                )}
                <button
                  onClick={() => handleTest(cred.id)}
                  disabled={testing === cred.id}
                  className="rounded-md px-3 py-1 text-xs font-medium text-muted-foreground hover:bg-accent transition-colors disabled:opacity-50"
                >
                  {testing === cred.id ? 'Testing...' : 'Test'}
                </button>
                <button
                  onClick={() => setDeleteTarget(cred)}
                  className="rounded-md p-1.5 text-muted-foreground hover:bg-destructive/10 hover:text-destructive transition-colors"
                  title="Delete credential"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Add/Edit Form Modal */}
      {showForm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="w-full max-w-md rounded-lg border border-border bg-card p-6 shadow-xl">
            <h2 className="text-lg font-semibold text-foreground">
              {editingId ? 'Edit Credential' : 'Add Credential'}
            </h2>

            <div className="mt-4 space-y-3">
              <div>
                <label className="block text-sm font-medium text-foreground mb-1">Label</label>
                <input
                  type="text"
                  value={form.label}
                  onChange={(e) => updateField('label', e.target.value)}
                  placeholder="My Production Account"
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-foreground mb-1">Provider</label>
                <select
                  value={form.provider_type}
                  onChange={(e) => updateField('provider_type', e.target.value)}
                  className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                >
                  <option value="cloudflare">Cloudflare</option>
                  <option value="dnspod">DNSPod</option>
                  <option value="alidns">AliDNS</option>
                </select>
              </div>

              {form.provider_type === 'cloudflare' && (
                <div>
                  <label className="block text-sm font-medium text-foreground mb-1">API Token</label>
                  <input
                    type="password"
                    value={form.api_token || ''}
                    onChange={(e) => updateField('api_token', e.target.value)}
                    placeholder="Your Cloudflare API token"
                    className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                  />
                </div>
              )}

              {form.provider_type === 'dnspod' && (
                <>
                  <div>
                    <label className="block text-sm font-medium text-foreground mb-1">Secret ID</label>
                    <input
                      type="password"
                      value={form.secret_id || ''}
                      onChange={(e) => updateField('secret_id', e.target.value)}
                      placeholder="Your Tencent Cloud SecretId"
                      className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-foreground mb-1">Secret Key</label>
                    <input
                      type="password"
                      value={form.secret_key || ''}
                      onChange={(e) => updateField('secret_key', e.target.value)}
                      placeholder="Your Tencent Cloud SecretKey"
                      className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                </>
              )}

              {form.provider_type === 'alidns' && (
                <>
                  <div>
                    <label className="block text-sm font-medium text-foreground mb-1">AccessKey ID</label>
                    <input
                      type="password"
                      value={form.access_key_id || ''}
                      onChange={(e) => updateField('access_key_id', e.target.value)}
                      placeholder="Your Alibaba Cloud AccessKey ID"
                      className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-foreground mb-1">AccessKey Secret</label>
                    <input
                      type="password"
                      value={form.access_key_secret || ''}
                      onChange={(e) => updateField('access_key_secret', e.target.value)}
                      placeholder="Your Alibaba Cloud AccessKey Secret"
                      className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring"
                    />
                  </div>
                </>
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
                disabled={!form.label}
                className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50"
              >
                {editingId ? 'Update' : 'Save'}
              </button>
            </div>
          </div>
        </div>
      )}

      <ConfirmDialog
        open={!!deleteTarget}
        title="Delete Credential"
        message={`Are you sure you want to delete "${deleteTarget?.label}"? This action cannot be undone.`}
        confirmLabel="Delete"
        destructive
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  );
}

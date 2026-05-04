import { invoke } from '@tauri-apps/api/core';
import type {
  CredentialInput,
  ProviderCredential,
  ProviderType,
  Domain,
  DomainSummary,
  DnsRecord,
  CreateRecordRequest,
  UpdateRecordRequest,
  SearchQuery,
  SearchResult,
} from '../types';

// ──── Credentials ────

export async function getCredentials(): Promise<ProviderCredential[]> {
  return invoke('get_credentials');
}

export async function saveCredential(input: CredentialInput): Promise<ProviderCredential> {
  return invoke('save_credential', { input });
}

export async function deleteCredential(id: string): Promise<void> {
  return invoke('delete_credential', { id });
}

export async function testCredential(id: string): Promise<boolean> {
  return invoke('test_credential', { id });
}

// ──── Domains ────

export async function listDomains(
  providerFilter?: ProviderType[]
): Promise<Domain[]> {
  return invoke('list_domains', { providerFilter: providerFilter ?? null });
}

export async function getDomain(
  provider: ProviderType,
  domainId: string
): Promise<Domain> {
  return invoke('get_domain', { provider, domainId });
}

export async function getDomainSummary(): Promise<DomainSummary> {
  return invoke('get_domain_summary');
}

// ──── Records ────

export async function listRecords(
  provider: ProviderType,
  domainId: string
): Promise<DnsRecord[]> {
  return invoke('list_records', { provider, domainId });
}

export async function createRecord(
  provider: ProviderType,
  domainId: string,
  record: CreateRecordRequest
): Promise<DnsRecord> {
  return invoke('create_record', { provider, domainId, record });
}

export async function updateRecord(
  provider: ProviderType,
  domainId: string,
  recordId: string,
  record: UpdateRecordRequest
): Promise<DnsRecord> {
  return invoke('update_record', { provider, domainId, recordId, record });
}

export async function deleteRecord(
  provider: ProviderType,
  domainId: string,
  recordId: string
): Promise<void> {
  return invoke('delete_record', { provider, domainId, recordId });
}

export async function searchRecords(
  query: SearchQuery
): Promise<SearchResult[]> {
  return invoke('search_records', { query });
}

// ──── Export ────

export async function exportZone(
  provider: ProviderType,
  domainId: string
): Promise<string> {
  return invoke('export_zone', { provider, domainId });
}

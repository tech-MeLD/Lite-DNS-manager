export type ProviderType = 'dnspod' | 'cloudflare' | 'alidns';

export interface ProviderCredential {
  id: string;
  provider_type: ProviderType;
  label: string;
  created_at: string;
  updated_at: string;
}

export interface CredentialInput {
  provider_type: ProviderType;
  label: string;
  secret_id?: string;
  secret_key?: string;
  api_token?: string;
  access_key_id?: string;
  access_key_secret?: string;
}

export type RecordType = 'A' | 'AAAA' | 'CNAME' | 'MX' | 'TXT' | 'NS' | 'SRV' | 'CAA' | 'SOA' | 'PTR'
  | 'CERT' | 'DNSKEY' | 'DS' | 'LOC' | 'NAPTR' | 'SMIMEA' | 'SSHFP' | 'TLSA' | 'URI';

export interface Domain {
  id: string;
  provider: ProviderType;
  name: string;
  status: string;
  record_count: number;
  created_on: string | null;
  name_servers: string[];
}

export interface DomainSummary {
  total_domains: number;
  dnspod_count: number;
  cloudflare_count: number;
  alidns_count: number;
}

export interface DnsRecord {
  id: string;
  provider: ProviderType;
  domain_id: string;
  domain_name: string;
  record_type: RecordType;
  name: string;
  content: string;
  ttl: number;
  priority: number | null;
  proxied: boolean | null;
  locked: boolean | null;
  created_on: string | null;
  updated_on: string | null;
}

export interface CreateRecordRequest {
  record_type: RecordType;
  name: string;
  content: string;
  ttl: number;
  priority?: number;
  proxied?: boolean;
}

export interface UpdateRecordRequest {
  record_type?: RecordType;
  name?: string;
  content?: string;
  ttl?: number;
  priority?: number;
  proxied?: boolean;
}

export interface SearchQuery {
  keyword: string;
  record_type?: RecordType;
  providers?: ProviderType[];
  domain_ids?: string[];
}

export interface SearchResult {
  record: DnsRecord;
  provider: ProviderType;
  domain_name: string;
}

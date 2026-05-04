use serde::{Deserialize, Serialize};

use super::{DnsRecord, ProviderType, RecordType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: String,
    pub record_type: Option<RecordType>,
    pub providers: Option<Vec<ProviderType>>,
    pub domain_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub record: DnsRecord,
    pub provider: ProviderType,
    pub domain_name: String,
}

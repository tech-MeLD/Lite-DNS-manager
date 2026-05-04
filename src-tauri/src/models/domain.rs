use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ProviderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub provider: ProviderType,
    pub name: String,
    pub status: String,
    pub record_count: u32,
    pub created_on: Option<DateTime<Utc>>,
    pub name_servers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSummary {
    pub total_domains: u32,
    pub dnspod_count: u32,
    pub cloudflare_count: u32,
    pub alidns_count: u32,
}

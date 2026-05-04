use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::ProviderType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum RecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
    CAA,
    SOA,
    PTR,
    #[serde(rename = "CERT")]
    Cert,
    #[serde(rename = "DNSKEY")]
    DnsKey,
    #[serde(rename = "DS")]
    Ds,
    #[serde(rename = "LOC")]
    Loc,
    #[serde(rename = "NAPTR")]
    Naptr,
    #[serde(rename = "SMIMEA")]
    Smimea,
    #[serde(rename = "SSHFP")]
    Sshfp,
    #[serde(rename = "TLSA")]
    Tlsa,
    #[serde(rename = "URI")]
    Uri,
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
            RecordType::CNAME => "CNAME",
            RecordType::MX => "MX",
            RecordType::TXT => "TXT",
            RecordType::NS => "NS",
            RecordType::SRV => "SRV",
            RecordType::CAA => "CAA",
            RecordType::SOA => "SOA",
            RecordType::PTR => "PTR",
            RecordType::Cert => "CERT",
            RecordType::DnsKey => "DNSKEY",
            RecordType::Ds => "DS",
            RecordType::Loc => "LOC",
            RecordType::Naptr => "NAPTR",
            RecordType::Smimea => "SMIMEA",
            RecordType::Sshfp => "SSHFP",
            RecordType::Tlsa => "TLSA",
            RecordType::Uri => "URI",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub provider: ProviderType,
    pub domain_id: String,
    pub domain_name: String,
    pub record_type: RecordType,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub priority: Option<u32>,
    pub proxied: Option<bool>,
    pub locked: Option<bool>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecordRequest {
    pub record_type: RecordType,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub priority: Option<u32>,
    pub proxied: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecordRequest {
    pub record_type: Option<RecordType>,
    pub name: Option<String>,
    pub content: Option<String>,
    pub ttl: Option<u32>,
    pub priority: Option<u32>,
    pub proxied: Option<bool>,
}

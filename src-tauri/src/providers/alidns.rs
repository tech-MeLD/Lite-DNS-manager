use async_trait::async_trait;
use base64::Engine;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha1::Sha1;
use std::collections::BTreeMap;

use super::DnsProvider;
use crate::error::ProviderError;
use crate::models::{
    CreateRecordRequest, DnsRecord, Domain, ProviderType, RecordType, UpdateRecordRequest,
};

type HmacSha1 = Hmac<Sha1>;

pub struct AliDnsProvider {
    client: reqwest::Client,
    access_key_id: String,
    access_key_secret: String,
}

#[derive(Debug, Deserialize)]
struct AliDnsDomain {
    #[serde(rename = "DomainId")]
    domain_id: String,
    #[serde(rename = "DomainName")]
    domain_name: String,
    #[serde(rename = "RecordCount")]
    record_count: Option<u32>,
    #[serde(rename = "CreateTime")]
    create_time: Option<String>,
    #[serde(rename = "DnsServers")]
    dns_servers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AliDnsDomainListResponse {
    #[serde(rename = "Domains")]
    domains: Option<AliDnsDomainsContainer>,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "TotalCount")]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AliDnsDomainsContainer {
    #[serde(rename = "Domain")]
    domain: Vec<AliDnsDomain>,
}

#[derive(Debug, Deserialize)]
struct AliDnsRecordListResponse {
    #[serde(rename = "DomainRecords")]
    domain_records: Option<AliDnsRecordsContainer>,
    #[serde(rename = "RequestId")]
    request_id: String,
    #[serde(rename = "TotalCount")]
    total_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct AliDnsRecordsContainer {
    #[serde(rename = "Record")]
    record: Vec<AliDnsRecord>,
}

#[derive(Debug, Deserialize)]
struct AliDnsRecord {
    #[serde(rename = "RecordId")]
    record_id: String,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "RR")]
    rr: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "TTL")]
    ttl: u32,
    #[serde(rename = "Priority")]
    priority: Option<u32>,
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Debug, Deserialize)]
struct AliDnsCreateResponse {
    #[serde(rename = "RecordId")]
    record_id: String,
    #[serde(rename = "RequestId")]
    request_id: String,
}

#[derive(Debug, Deserialize)]
struct AliDnsErrorResponse {
    #[serde(rename = "Code")]
    code: Option<String>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "RequestId")]
    request_id: Option<String>,
}

impl AliDnsProvider {
    pub fn new(access_key_id: String, access_key_secret: String) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            access_key_id,
            access_key_secret,
        })
    }

    fn sign(&self, params: &BTreeMap<String, String>) -> Result<String, ProviderError> {
        let canonicalized = params
            .iter()
            .map(|(k, v)| {
                format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
            })
            .collect::<Vec<_>>()
            .join("&");

        let string_to_sign = format!(
            "GET&{}&{}",
            urlencoding::encode("/"),
            urlencoding::encode(&canonicalized)
        );

        let key = format!("{}&", self.access_key_secret);
        let mut mac = HmacSha1::new_from_slice(key.as_bytes())
            .map_err(|e| ProviderError::Other(format!("HMAC key error: {}", e)))?;
        mac.update(string_to_sign.as_bytes());
        Ok(base64::prelude::BASE64_STANDARD.encode(mac.finalize().into_bytes()))
    }

    fn common_params(&self) -> BTreeMap<String, String> {
        let mut params = BTreeMap::new();
        params.insert("Format".to_string(), "JSON".to_string());
        params.insert("Version".to_string(), "2015-01-09".to_string());
        params.insert("AccessKeyId".to_string(), self.access_key_id.clone());
        params.insert("SignatureMethod".to_string(), "HMAC-SHA1".to_string());
        params.insert("Timestamp".to_string(), Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string());
        params.insert("SignatureVersion".to_string(), "1.0".to_string());
        params.insert("SignatureNonce".to_string(), uuid::Uuid::new_v4().to_string());
        params
    }

    fn build_url(&self, action: &str, extra_params: BTreeMap<String, String>) -> Result<String, ProviderError> {
        let mut params = self.common_params();
        params.insert("Action".to_string(), action.to_string());
        for (k, v) in extra_params {
            params.insert(k, v);
        }

        let signature = self.sign(&params)?;
        params.insert("Signature".to_string(), signature);

        let query = params
            .iter()
            .map(|(k, v)| {
                format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
            })
            .collect::<Vec<_>>()
            .join("&");

        Ok(format!("https://alidns.aliyuncs.com/?{}", query))
    }

    fn parse_record_type(t: &str) -> RecordType {
        match t.to_uppercase().as_str() {
            "A" => RecordType::A,
            "AAAA" => RecordType::AAAA,
            "CNAME" => RecordType::CNAME,
            "MX" => RecordType::MX,
            "TXT" => RecordType::TXT,
            "NS" => RecordType::NS,
            "SRV" => RecordType::SRV,
            "CAA" => RecordType::CAA,
            "SOA" => RecordType::SOA,
            "PTR" => RecordType::PTR,
            _ => RecordType::A,
        }
    }
}

#[async_trait]
impl DnsProvider for AliDnsProvider {
    async fn list_domains(&self) -> Result<Vec<Domain>, ProviderError> {
        let url = self.build_url("DescribeDomains", BTreeMap::new())?;

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let err_body: AliDnsErrorResponse = response.json().await.map_err(|e| {
                ProviderError::Deserialization(format!("AliDNS error parse: {}", e))
            })?;
            return Err(ProviderError::AliDns {
                code: err_body.code.unwrap_or_default(),
                message: err_body.message.unwrap_or_default(),
            });
        }

        let list_response: AliDnsDomainListResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("AliDNS response parse: {}", e))
        })?;

        let domains = list_response
            .domains
            .unwrap_or(AliDnsDomainsContainer { domain: vec![] })
            .domain
            .into_iter()
            .map(|d| Domain {
                id: d.domain_id,
                provider: ProviderType::AliDns,
                name: d.domain_name,
                status: "active".to_string(),
                record_count: d.record_count.unwrap_or(0),
                created_on: d.create_time.and_then(|t| {
                    chrono::DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&chrono::Utc))
                }),
                name_servers: d.dns_servers.unwrap_or_default(),
            })
            .collect();

        Ok(domains)
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain, ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("DomainId".to_string(), domain_id.to_string());
        let url = self.build_url("DescribeDomain", params)?;

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::NotFound("Domain not found".to_string()));
        }

        let list_response: AliDnsDomainListResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("AliDNS response parse: {}", e))
        })?;

        let domain = list_response
            .domains
            .unwrap_or(AliDnsDomainsContainer { domain: vec![] })
            .domain
            .into_iter()
            .next()
            .ok_or_else(|| ProviderError::NotFound("Domain not found".to_string()))?;

        Ok(Domain {
            id: domain.domain_id,
            provider: ProviderType::AliDns,
            name: domain.domain_name,
            status: "active".to_string(),
            record_count: domain.record_count.unwrap_or(0),
            created_on: domain.create_time.and_then(|t| {
                chrono::DateTime::parse_from_rfc3339(&t).ok().map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            name_servers: domain.dns_servers.unwrap_or_default(),
        })
    }

    async fn list_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("DomainId".to_string(), domain_id.to_string());
        let url = self.build_url("DescribeDomainRecords", params)?;

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::NotFound("Domain not found".to_string()));
        }

        let list_response: AliDnsRecordListResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("AliDNS response parse: {}", e))
        })?;

        let records = list_response
            .domain_records
            .unwrap_or(AliDnsRecordsContainer { record: vec![] })
            .record
            .into_iter()
            .map(|r| DnsRecord {
                id: r.record_id,
                provider: ProviderType::AliDns,
                domain_id: domain_id.to_string(),
                domain_name: String::new(),
                record_type: Self::parse_record_type(&r.record_type),
                name: r.rr,
                content: r.value,
                ttl: r.ttl,
                priority: r.priority,
                proxied: None,
                locked: Some(r.status == "ENABLE"),
                created_on: None,
                updated_on: None,
            })
            .collect();

        Ok(records)
    }

    async fn create_record(
        &self,
        domain_id: &str,
        record: &CreateRecordRequest,
    ) -> Result<DnsRecord, ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("DomainId".to_string(), domain_id.to_string());
        params.insert("RR".to_string(), record.name.clone());
        params.insert("Type".to_string(), record.record_type.to_string());
        params.insert("Value".to_string(), record.content.clone());
        params.insert("TTL".to_string(), record.ttl.to_string());
        if let Some(priority) = record.priority {
            params.insert("Priority".to_string(), priority.to_string());
        }

        let url = self.build_url("AddDomainRecord", params)?;

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::BadRequest("Failed to create record".to_string()));
        }

        let create_resp: AliDnsCreateResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("AliDNS response parse: {}", e))
        })?;

        Ok(DnsRecord {
            id: create_resp.record_id,
            provider: ProviderType::AliDns,
            domain_id: domain_id.to_string(),
            domain_name: String::new(),
            record_type: record.record_type.clone(),
            name: record.name.clone(),
            content: record.content.clone(),
            ttl: record.ttl,
            priority: record.priority,
            proxied: None,
            locked: None,
            created_on: Some(chrono::Utc::now()),
            updated_on: Some(chrono::Utc::now()),
        })
    }

    async fn update_record(
        &self,
        domain_id: &str,
        record_id: &str,
        record: &UpdateRecordRequest,
    ) -> Result<DnsRecord, ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("RecordId".to_string(), record_id.to_string());
        params.insert("DomainId".to_string(), domain_id.to_string());
        if let Some(ref t) = record.record_type {
            params.insert("Type".to_string(), t.to_string());
        }
        if let Some(ref n) = record.name {
            params.insert("RR".to_string(), n.clone());
        }
        if let Some(ref c) = record.content {
            params.insert("Value".to_string(), c.clone());
        }
        if let Some(ttl) = record.ttl {
            params.insert("TTL".to_string(), ttl.to_string());
        }
        if let Some(priority) = record.priority {
            params.insert("Priority".to_string(), priority.to_string());
        }

        let url = self.build_url("UpdateDomainRecord", params)?;
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::NotFound("Record not found".to_string()));
        }

        Ok(DnsRecord {
            id: record_id.to_string(),
            provider: ProviderType::AliDns,
            domain_id: domain_id.to_string(),
            domain_name: String::new(),
            record_type: record.record_type.clone().unwrap_or(RecordType::A),
            name: record.name.clone().unwrap_or_default(),
            content: record.content.clone().unwrap_or_default(),
            ttl: record.ttl.unwrap_or(600),
            priority: record.priority,
            proxied: None,
            locked: None,
            created_on: None,
            updated_on: Some(chrono::Utc::now()),
        })
    }

    async fn delete_record(
        &self,
        domain_id: &str,
        record_id: &str,
    ) -> Result<(), ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("RecordId".to_string(), record_id.to_string());
        params.insert("DomainId".to_string(), domain_id.to_string());

        let url = self.build_url("DeleteDomainRecord", params)?;
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(ProviderError::NotFound("Record not found".to_string()));
        }

        Ok(())
    }

    async fn search_records(
        &self,
        domain_id: &str,
        query: &str,
    ) -> Result<Vec<DnsRecord>, ProviderError> {
        let mut params = BTreeMap::new();
        params.insert("DomainId".to_string(), domain_id.to_string());
        params.insert("KeyWord".to_string(), query.to_string());
        params.insert("SearchMode".to_string(), "LIKE".to_string());

        let url = self.build_url("DescribeDomainRecords", params)?;
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let list_response: AliDnsRecordListResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("AliDNS response parse: {}", e))
        })?;

        let records = list_response
            .domain_records
            .unwrap_or(AliDnsRecordsContainer { record: vec![] })
            .record
            .into_iter()
            .map(|r| DnsRecord {
                id: r.record_id,
                provider: ProviderType::AliDns,
                domain_id: domain_id.to_string(),
                domain_name: String::new(),
                record_type: Self::parse_record_type(&r.record_type),
                name: r.rr,
                content: r.value,
                ttl: r.ttl,
                priority: r.priority,
                proxied: None,
                locked: Some(r.status == "ENABLE"),
                created_on: None,
                updated_on: None,
            })
            .collect();

        Ok(records)
    }

    async fn export_zone(&self, domain_id: &str) -> Result<String, ProviderError> {
        let records = self.list_records(domain_id).await?;
        let domain = self.get_domain(domain_id).await?;

        let mut zone = String::new();
        zone.push_str(&format!("$ORIGIN {}.\n", domain.name));
        zone.push_str("$TTL 600\n\n");

        for record in &records {
            zone.push_str(&format!(
                "{:<30} IN {:<10} {:<50}\n",
                record.name, record.record_type.to_string(), record.content
            ));
        }

        Ok(zone)
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::AliDns
    }
}

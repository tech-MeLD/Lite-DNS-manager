use async_trait::async_trait;
use chrono::DateTime;
use serde::{Deserialize, Serialize};

use super::DnsProvider;
use crate::error::ProviderError;
use crate::models::{
    CreateRecordRequest, DnsRecord, Domain, ProviderType, RecordType, UpdateRecordRequest,
};

pub struct CloudflareProvider {
    client: reqwest::Client,
    api_token: String,
}

// Cloudflare API response envelope
#[derive(Debug, Deserialize)]
struct CfResponse<T> {
    result: Option<T>,
    success: bool,
    errors: Vec<CfError>,
    messages: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CfError {
    code: u32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct CfDomain {
    id: String,
    name: String,
    status: String,
    created_on: Option<String>,
    name_servers: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct CfDnsRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    priority: Option<u32>,
    proxied: Option<bool>,
    locked: Option<bool>,
    created_on: Option<String>,
    modified_on: Option<String>,
}

#[derive(Debug, Serialize)]
struct CfCreateRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    priority: Option<u32>,
    proxied: Option<bool>,
}

impl CloudflareProvider {
    pub fn new(api_token: String) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { client, api_token })
    }

    fn base_url(&self) -> &str {
        "https://api.cloudflare.com/client/v4"
    }

    async fn handle_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: reqwest::Response,
    ) -> Result<CfResponse<T>, ProviderError> {
        let status = response.status();

        if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::Unauthorized(body));
        }

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(ProviderError::RateLimited {
                retry_after: response
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok()),
            });
        }

        let cf_response: CfResponse<T> = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("Cloudflare response parse error: {}", e))
        })?;

        if !cf_response.success {
            let error_msg = cf_response
                .errors
                .first()
                .map(|e| e.message.clone())
                .unwrap_or_else(|| "Unknown Cloudflare error".to_string());

            if status.as_u16() == 404 {
                return Err(ProviderError::NotFound(error_msg));
            }

            return Err(ProviderError::Cloudflare {
                code: if cf_response.errors.is_empty() {
                    status.as_u16()
                } else {
                    cf_response.errors[0].code as u16
                },
                message: error_msg,
            });
        }

        Ok(cf_response)
    }

    fn parse_cf_domain(&self, cf: CfDomain) -> Domain {
        Domain {
            id: cf.id,
            provider: ProviderType::Cloudflare,
            name: cf.name,
            status: cf.status,
            record_count: 0, // Cloudflare doesn't include count in list
            created_on: cf.created_on.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            name_servers: cf.name_servers.unwrap_or_default(),
        }
    }

    fn parse_cf_record(&self, domain_id: &str, domain_name: &str, cf: CfDnsRecord) -> DnsRecord {
        DnsRecord {
            id: cf.id,
            provider: ProviderType::Cloudflare,
            domain_id: domain_id.to_string(),
            domain_name: domain_name.to_string(),
            record_type: parse_record_type(&cf.record_type),
            name: cf.name,
            content: cf.content,
            ttl: cf.ttl,
            priority: cf.priority,
            proxied: cf.proxied,
            locked: cf.locked,
            created_on: cf.created_on.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            updated_on: cf.modified_on.and_then(|d| {
                DateTime::parse_from_rfc3339(&d)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
        }
    }
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
        other => {
            log::warn!("Unknown record type from Cloudflare: {}", other);
            RecordType::A // fallback
        }
    }
}

#[async_trait]
impl DnsProvider for CloudflareProvider {
    async fn list_domains(&self) -> Result<Vec<Domain>, ProviderError> {
        let url = format!("{}/zones", self.base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let cf_response: CfResponse<Vec<CfDomain>> = self.handle_response(response).await?;
        let domains: Vec<Domain> = cf_response
            .result
            .unwrap_or_default()
            .into_iter()
            .map(|d| self.parse_cf_domain(d))
            .collect();

        Ok(domains)
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain, ProviderError> {
        let url = format!("{}/zones/{}", self.base_url(), domain_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let cf_response: CfResponse<CfDomain> = self.handle_response(response).await?;
        let domain = cf_response
            .result
            .ok_or_else(|| ProviderError::NotFound("Domain not found".to_string()))?;

        Ok(self.parse_cf_domain(domain))
    }

    async fn list_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ProviderError> {
        let url = format!(
            "{}/zones/{}/dns_records?per_page=500",
            self.base_url(),
            domain_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let cf_response: CfResponse<Vec<CfDnsRecord>> = self.handle_response(response).await?;

        let domain = self.get_domain(domain_id).await?;
        let records: Vec<DnsRecord> = cf_response
            .result
            .unwrap_or_default()
            .into_iter()
            .map(|r| self.parse_cf_record(domain_id, &domain.name, r))
            .collect();

        Ok(records)
    }

    async fn create_record(
        &self,
        domain_id: &str,
        record: &CreateRecordRequest,
    ) -> Result<DnsRecord, ProviderError> {
        let url = format!(
            "{}/zones/{}/dns_records",
            self.base_url(),
            domain_id
        );

        let body = CfCreateRecord {
            record_type: record.record_type.to_string(),
            name: record.name.clone(),
            content: record.content.clone(),
            ttl: record.ttl,
            priority: record.priority,
            proxied: record.proxied,
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let cf_response: CfResponse<CfDnsRecord> = self.handle_response(response).await?;
        let cf_record = cf_response
            .result
            .ok_or_else(|| ProviderError::BadRequest("Failed to create record".to_string()))?;

        let domain = self.get_domain(domain_id).await?;
        Ok(self.parse_cf_record(domain_id, &domain.name, cf_record))
    }

    async fn update_record(
        &self,
        domain_id: &str,
        record_id: &str,
        record: &UpdateRecordRequest,
    ) -> Result<DnsRecord, ProviderError> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.base_url(),
            domain_id,
            record_id
        );

        // Use serde_json::Value for flexible partial update
        let mut body = serde_json::Map::new();
        if let Some(ref t) = record.record_type {
            body.insert("type".to_string(), serde_json::Value::String(t.to_string()));
        }
        if let Some(ref n) = record.name {
            body.insert("name".to_string(), serde_json::Value::String(n.clone()));
        }
        if let Some(ref c) = record.content {
            body.insert(
                "content".to_string(),
                serde_json::Value::String(c.clone()),
            );
        }
        if let Some(ttl) = record.ttl {
            body.insert("ttl".to_string(), serde_json::json!(ttl));
        }
        if let Some(priority) = record.priority {
            body.insert("priority".to_string(), serde_json::json!(priority));
        }
        if let Some(proxied) = record.proxied {
            body.insert("proxied".to_string(), serde_json::json!(proxied));
        }

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let cf_response: CfResponse<CfDnsRecord> = self.handle_response(response).await?;
        let cf_record = cf_response
            .result
            .ok_or_else(|| ProviderError::NotFound("Record not found".to_string()))?;

        let domain = self.get_domain(domain_id).await?;
        Ok(self.parse_cf_record(domain_id, &domain.name, cf_record))
    }

    async fn delete_record(
        &self,
        domain_id: &str,
        record_id: &str,
    ) -> Result<(), ProviderError> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            self.base_url(),
            domain_id,
            record_id
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await?;

        let _: CfResponse<serde_json::Value> = self.handle_response(response).await?;
        Ok(())
    }

    async fn search_records(
        &self,
        domain_id: &str,
        query: &str,
    ) -> Result<Vec<DnsRecord>, ProviderError> {
        // Cloudflare doesn't have a native search API, fetch all and filter
        let all_records = self.list_records(domain_id).await?;
        let query_lower = query.to_lowercase();

        let filtered: Vec<DnsRecord> = all_records
            .into_iter()
            .filter(|r| {
                r.name.to_lowercase().contains(&query_lower)
                    || r.content.to_lowercase().contains(&query_lower)
                    || r.record_type.to_string().to_lowercase().contains(&query_lower)
            })
            .collect();

        Ok(filtered)
    }

    async fn export_zone(
        &self,
        domain_id: &str,
    ) -> Result<String, ProviderError> {
        let records = self.list_records(domain_id).await?;
        let domain_name = records
            .first()
            .map(|r| r.domain_name.clone())
            .unwrap_or_else(|| "unknown".to_string());

        let mut zone = String::new();
        zone.push_str(&format!("$ORIGIN {}.\n", domain_name));
        zone.push_str("$TTL 3600\n\n");

        for record in &records {
            let name = if record.name == domain_name || record.name == "@" || record.name.is_empty() {
                "@".to_string()
            } else if record.name.ends_with(&format!(".{}", domain_name)) {
                record.name.trim_end_matches(&format!(".{}", domain_name)).to_string()
            } else {
                record.name.clone()
            };

            zone.push_str(&format!(
                "{:<30} IN {:<10} {:<50}",
                name,
                record.record_type.to_string(),
                record.content
            ));

            if let Some(priority) = record.priority {
                zone.push_str(&format!(" ; priority={}", priority));
            }
            zone.push('\n');
        }

        Ok(zone)
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::Cloudflare
    }
}

use async_trait::async_trait;
use chrono::Utc;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Sha256, Digest};

use super::DnsProvider;
use crate::error::ProviderError;
use crate::models::{
    CreateRecordRequest, DnsRecord, Domain, ProviderType, RecordType, UpdateRecordRequest,
};

type HmacSha256 = Hmac<Sha256>;

pub struct DnsPodProvider {
    client: reqwest::Client,
    secret_id: String,
    secret_key: String,
}

// DNSPod API response types
#[derive(Debug, Deserialize)]
struct DnsPodResponse {
    #[serde(rename = "Response")]
    response: DnsPodResponseInner,
}

#[derive(Debug, Deserialize)]
struct DnsPodResponseInner {
    #[serde(rename = "DomainList")]
    domain_list: Option<Vec<DnsPodDomain>>,
    #[serde(rename = "RecordList")]
    record_list: Option<Vec<DnsPodRecord>>,
    #[serde(rename = "RecordId")]
    record_id: Option<u64>,
    #[serde(rename = "DomainInfo")]
    domain_info: Option<DnsPodDomain>,
    #[serde(rename = "Error")]
    error: Option<DnsPodError>,
    #[serde(rename = "RequestId")]
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DnsPodError {
    #[serde(rename = "Code")]
    code: String,
    #[serde(rename = "Message")]
    message: String,
}

#[derive(Debug, Deserialize)]
struct DnsPodDomain {
    #[serde(rename = "DomainId")]
    domain_id: u64,
    #[serde(rename = "Domain")]
    name: String,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "RecordCount")]
    record_count: Option<u32>,
    #[serde(rename = "CreatedOn")]
    created_on: Option<String>,
    #[serde(rename = "DnsStatus")]
    dns_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DnsPodRecord {
    #[serde(rename = "RecordId")]
    record_id: u64,
    #[serde(rename = "Type")]
    record_type: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
    #[serde(rename = "TTL")]
    ttl: u32,
    #[serde(rename = "MX")]
    mx: Option<u32>,
    #[serde(rename = "Status")]
    status: String,
    #[serde(rename = "UpdatedOn")]
    updated_on: Option<String>,
}

impl DnsPodProvider {
    pub fn new(secret_id: String, secret_key: String) -> Result<Self, ProviderError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            secret_id,
            secret_key,
        })
    }

    fn sign(&self, action: &str, body: &str) -> Result<(String, String), ProviderError> {
        let timestamp = Utc::now().timestamp();
        let date = Utc::now().format("%Y-%m-%d").to_string();
        let service = "dnspod";
        let host = "dnspod.tencentcloudapi.com";
        let algorithm = "TC3-HMAC-SHA256";

        // Step 1: Canonical Request
        let canonical_uri = "/";
        let canonical_querystring = "";
        let content_type = "application/json";

        let mut hasher = Sha256::new();
        hasher.update(body);
        let payload_hash = hex::encode(hasher.finalize());

        let canonical_headers = format!(
            "content-type:{}\nhost:{}\nx-tc-action:{}\n",
            content_type, host, action.to_lowercase()
        );
        let signed_headers = "content-type;host;x-tc-action";

        let mut hasher = Sha256::new();
        hasher.update(format!(
            "POST\n{}\n{}\n{}\n{}\n{}",
            canonical_uri, canonical_querystring, canonical_headers, signed_headers, payload_hash
        ));
        let hashed_canonical_request = hex::encode(hasher.finalize());

        // Step 2: String to Sign
        let credential_scope = format!("{}/{}/tc3_request", date, service);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm, timestamp, credential_scope, hashed_canonical_request
        );

        // Step 3: Signature
        fn hmac_sha256(key: &[u8], msg: &str) -> Result<Vec<u8>, ProviderError> {
            let mut mac = HmacSha256::new_from_slice(key)
                .map_err(|e| ProviderError::Other(format!("HMAC key error: {}", e)))?;
            mac.update(msg.as_bytes());
            Ok(mac.finalize().into_bytes().to_vec())
        }

        let secret_date = hmac_sha256(
            format!("TC3{}", self.secret_key).as_bytes(),
            &date,
        )?;
        let secret_service = hmac_sha256(&secret_date, service)?;
        let secret_signing = hmac_sha256(&secret_service, "tc3_request")?;
        let signature = hex::encode(hmac_sha256(&secret_signing, &string_to_sign)?);

        // Step 4: Authorization header
        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm, self.secret_id, credential_scope, signed_headers, signature
        );

        Ok((authorization, timestamp.to_string()))
    }

    async fn request(
        &self,
        action: &str,
        body: serde_json::Value,
    ) -> Result<DnsPodResponse, ProviderError> {
        let body_str = serde_json::to_string(&body)
            .map_err(|e| ProviderError::Other(format!("Failed to serialize request body: {}", e)))?;
        let (authorization, timestamp) = self.sign(action, &body_str)?;

        let response = self
            .client
            .post("https://dnspod.tencentcloudapi.com")
            .header("Authorization", &authorization)
            .header("Content-Type", "application/json")
            .header("Host", "dnspod.tencentcloudapi.com")
            .header("X-TC-Action", action)
            .header("X-TC-Timestamp", timestamp)
            .header("X-TC-Version", "2021-03-23")
            .body(body_str)
            .send()
            .await?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(ProviderError::Unauthorized(
                format!("DNSPod HTTP {}", status.as_u16())
            ));
        }

        let dp_response: DnsPodResponse = response.json().await.map_err(|e| {
            ProviderError::Deserialization(format!("DNSPod response parse error: {}", e))
        })?;

        if let Some(error) = dp_response.response.error {
            match error.code.as_str() {
                "AuthFailure.SignatureFailure"
                | "AuthFailure.SecretIdNotFound"
                | "AuthFailure.TokenFailure" => {
                    return Err(ProviderError::Unauthorized(error.message));
                }
                "ResourceNotFound" => {
                    return Err(ProviderError::NotFound(error.message));
                }
                "LimitExceeded" => {
                    return Err(ProviderError::RateLimited { retry_after: None });
                }
                _ => {
                    return Err(ProviderError::DnsPod {
                        message: error.message,
                        request_id: dp_response
                            .response
                            .request_id
                            .unwrap_or_default(),
                    });
                }
            }
        }

        Ok(dp_response)
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
impl DnsProvider for DnsPodProvider {
    async fn list_domains(&self) -> Result<Vec<Domain>, ProviderError> {
        let body = serde_json::json!({});
        let response = self.request("DescribeDomainList", body).await?;

        let domains = response
            .response
            .domain_list
            .unwrap_or_default()
            .into_iter()
            .map(|d| Domain {
                id: d.domain_id.to_string(),
                provider: ProviderType::DnsPod,
                name: d.name,
                status: d.status,
                record_count: d.record_count.unwrap_or(0),
                created_on: d.created_on.and_then(|dt| {
                    chrono::NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|naive| {
                            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                naive,
                                chrono::Utc,
                            )
                        })
                }),
                name_servers: vec![],
            })
            .collect();

        Ok(domains)
    }

    async fn get_domain(&self, domain_id: &str) -> Result<Domain, ProviderError> {
        let body = serde_json::json!({
            "DomainId": domain_id.parse::<u64>().unwrap_or(0)
        });
        let response = self.request("DescribeDomain", body).await?;

        let domain = response
            .response
            .domain_info
            .ok_or_else(|| ProviderError::NotFound("Domain not found".to_string()))?;

        Ok(Domain {
            id: domain.domain_id.to_string(),
            provider: ProviderType::DnsPod,
            name: domain.name,
            status: domain.status,
            record_count: domain.record_count.unwrap_or(0),
            created_on: domain.created_on.and_then(|dt| {
                chrono::NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|naive| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            naive,
                            chrono::Utc,
                        )
                    })
            }),
            name_servers: vec![],
        })
    }

    async fn list_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ProviderError> {
        let body = serde_json::json!({
            "DomainId": domain_id.parse::<u64>().unwrap_or(0)
        });
        let response = self.request("DescribeRecordList", body).await?;

        let records = response
            .response
            .record_list
            .unwrap_or_default()
            .into_iter()
            .map(|r| DnsRecord {
                id: r.record_id.to_string(),
                provider: ProviderType::DnsPod,
                domain_id: domain_id.to_string(),
                domain_name: String::new(),
                record_type: Self::parse_record_type(&r.record_type),
                name: r.name,
                content: r.value,
                ttl: r.ttl,
                priority: r.mx,
                proxied: None,
                locked: Some(r.status == "ENABLE"),
                created_on: None,
                updated_on: r.updated_on.and_then(|dt| {
                    chrono::NaiveDateTime::parse_from_str(&dt, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|naive| {
                            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                                naive,
                                chrono::Utc,
                            )
                        })
                }),
            })
            .collect();

        Ok(records)
    }

    async fn create_record(
        &self,
        domain_id: &str,
        record: &CreateRecordRequest,
    ) -> Result<DnsRecord, ProviderError> {
        let body = serde_json::json!({
            "DomainId": domain_id.parse::<u64>().unwrap_or(0),
            "RecordType": record.record_type.to_string(),
            "RecordLine": "默认",
            "Value": record.content,
            "SubDomain": record.name,
            "TTL": record.ttl,
            "MX": record.priority.unwrap_or(0),
        });
        let response = self.request("CreateRecord", body).await?;

        let record_id = response
            .response
            .record_id
            .unwrap_or(0);

        Ok(DnsRecord {
            id: record_id.to_string(),
            provider: ProviderType::DnsPod,
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
        let mut body = serde_json::json!({
            "RecordId": record_id.parse::<u64>().unwrap_or(0),
            "DomainId": domain_id.parse::<u64>().unwrap_or(0),
        });

        if let Some(ref t) = record.record_type {
            body["RecordType"] = serde_json::Value::String(t.to_string());
        }
        if let Some(ref n) = record.name {
            body["SubDomain"] = serde_json::Value::String(n.clone());
        }
        if let Some(ref c) = record.content {
            body["Value"] = serde_json::Value::String(c.clone());
        }
        if let Some(ttl) = record.ttl {
            body["TTL"] = serde_json::json!(ttl);
        }
        if let Some(mx) = record.priority {
            body["MX"] = serde_json::json!(mx);
        }

        let _response = self.request("ModifyRecord", body).await?;

        Ok(DnsRecord {
            id: record_id.to_string(),
            provider: ProviderType::DnsPod,
            domain_id: String::new(),
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
        let body = serde_json::json!({
            "RecordId": record_id.parse::<u64>().unwrap_or(0),
            "DomainId": domain_id.parse::<u64>().unwrap_or(0)
        });
        let _response = self.request("DeleteRecord", body).await?;
        Ok(())
    }

    async fn search_records(
        &self,
        domain_id: &str,
        query: &str,
    ) -> Result<Vec<DnsRecord>, ProviderError> {
        let body = serde_json::json!({
            "Keyword": query,
            "DomainId": domain_id.parse::<u64>().unwrap_or(0)
        });
        let response = self.request("DescribeRecordList", body).await?;

        let records = response
            .response
            .record_list
            .unwrap_or_default()
            .into_iter()
            .map(|r| DnsRecord {
                id: r.record_id.to_string(),
                provider: ProviderType::DnsPod,
                domain_id: String::new(),
                domain_name: String::new(),
                record_type: Self::parse_record_type(&r.record_type),
                name: r.name,
                content: r.value,
                ttl: r.ttl,
                priority: r.mx,
                proxied: None,
                locked: Some(r.status == "ENABLE"),
                created_on: None,
                updated_on: None,
            })
            .collect();

        Ok(records)
    }

    async fn export_zone(
        &self,
        domain_id: &str,
    ) -> Result<String, ProviderError> {
        let records = self.list_records(domain_id).await?;
        let domain = self.get_domain(domain_id).await?;

        let mut zone = String::new();
        zone.push_str(&format!("$ORIGIN {}.\n", domain.name));
        zone.push_str("$TTL 600\n\n");

        for record in &records {
            zone.push_str(&format!(
                "{:<30} IN {:<10} {:<50}\n",
                record.name,
                record.record_type.to_string(),
                record.content
            ));
        }

        Ok(zone)
    }

    fn provider_type(&self) -> ProviderType {
        ProviderType::DnsPod
    }
}

pub mod cloudflare;
pub mod dnspod;
pub mod alidns;

use async_trait::async_trait;
use crate::error::ProviderError;
use crate::models::{
    CreateRecordRequest, CredentialSecretData, DnsRecord, Domain, ProviderType,
    UpdateRecordRequest,
};

/// Core abstraction for DNS provider implementations
#[async_trait]
pub trait DnsProvider: Send + Sync {
    /// List all domains for this provider
    async fn list_domains(&self) -> Result<Vec<Domain>, ProviderError>;

    /// Get a single domain's details
    async fn get_domain(&self, domain_id: &str) -> Result<Domain, ProviderError>;

    /// List all DNS records for a domain (paginated internally)
    async fn list_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ProviderError>;

    /// Create a new DNS record
    async fn create_record(
        &self,
        domain_id: &str,
        record: &CreateRecordRequest,
    ) -> Result<DnsRecord, ProviderError>;

    /// Update an existing DNS record (all fields optional)
    async fn update_record(
        &self,
        domain_id: &str,
        record_id: &str,
        record: &UpdateRecordRequest,
    ) -> Result<DnsRecord, ProviderError>;

    /// Delete a DNS record
    async fn delete_record(
        &self,
        domain_id: &str,
        record_id: &str,
    ) -> Result<(), ProviderError>;

    /// Search records within a domain
    async fn search_records(
        &self,
        domain_id: &str,
        query: &str,
    ) -> Result<Vec<DnsRecord>, ProviderError>;

    /// Export zone file content for a domain
    async fn export_zone(
        &self,
        domain_id: &str,
    ) -> Result<String, ProviderError>;

    /// Return the provider type discriminant
    fn provider_type(&self) -> ProviderType;

    /// Get the domain name by ID (used for search results)
    async fn get_domain_name(&self, domain_id: &str) -> Result<String, ProviderError> {
        self.get_domain(domain_id)
            .await
            .map(|d| d.name)
    }
}

/// Factory function to create the appropriate provider implementation
pub fn create_provider(
    credential: &CredentialSecretData,
) -> Result<Box<dyn DnsProvider + Send + Sync>, ProviderError> {
    match credential {
        CredentialSecretData::DnsPod {
            secret_id,
            secret_key,
        } => {
            let provider = dnspod::DnsPodProvider::new(secret_id.clone(), secret_key.clone())?;
            Ok(Box::new(provider))
        }
        CredentialSecretData::Cloudflare { api_token } => {
            let provider = cloudflare::CloudflareProvider::new(api_token.clone())?;
            Ok(Box::new(provider))
        }
        CredentialSecretData::AliDns {
            access_key_id,
            access_key_secret,
        } => {
            let provider =
                alidns::AliDnsProvider::new(access_key_id.clone(), access_key_secret.clone())?;
            Ok(Box::new(provider))
        }
    }
}

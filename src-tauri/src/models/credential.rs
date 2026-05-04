use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    DnsPod,
    Cloudflare,
    AliDns,
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderType::DnsPod => write!(f, "DNSPod"),
            ProviderType::Cloudflare => write!(f, "Cloudflare"),
            ProviderType::AliDns => write!(f, "AliDNS"),
        }
    }
}

/// Credential metadata (returned to frontend — NO secrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCredential {
    pub id: Uuid,
    pub provider_type: ProviderType,
    pub label: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input from frontend when saving a credential
#[derive(Clone, Serialize, Deserialize)]
pub struct CredentialInput {
    pub provider_type: ProviderType,
    pub label: String,
    pub secret_id: Option<String>,
    pub secret_key: Option<String>,
    pub api_token: Option<String>,
    pub access_key_id: Option<String>,
    pub access_key_secret: Option<String>,
}

impl std::fmt::Debug for CredentialInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialInput")
            .field("provider_type", &self.provider_type)
            .field("label", &self.label)
            .finish_non_exhaustive()
    }
}

/// Secret data stored in OS keychain — NEVER sent to frontend
#[derive(Clone, Serialize, Deserialize)]
pub enum CredentialSecretData {
    DnsPod {
        secret_id: String,
        secret_key: String,
    },
    Cloudflare {
        api_token: String,
    },
    AliDns {
        access_key_id: String,
        access_key_secret: String,
    },
}

impl std::fmt::Debug for CredentialSecretData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DnsPod { .. } => f.debug_struct("DnsPod").finish_non_exhaustive(),
            Self::Cloudflare { .. } => f.debug_struct("Cloudflare").finish_non_exhaustive(),
            Self::AliDns { .. } => f.debug_struct("AliDns").finish_non_exhaustive(),
        }
    }
}

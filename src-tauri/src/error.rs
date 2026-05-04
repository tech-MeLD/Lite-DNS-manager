use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Credential error: {0}")]
    Credential(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize)]
pub struct AppErrorResponse {
    pub code: String,
    pub message: String,
}

impl From<ProviderError> for AppError {
    fn from(e: ProviderError) -> Self {
        AppError::Provider(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let (code, message) = match self {
            AppError::Provider(m) => ("PROVIDER_ERROR", m.as_str()),
            AppError::Credential(m) => ("CREDENTIAL_ERROR", m.as_str()),
            AppError::Serialization(m) => ("SERIALIZATION_ERROR", m.as_str()),
            AppError::NotFound(m) => ("NOT_FOUND", m.as_str()),
            AppError::Validation(m) => ("VALIDATION_ERROR", m.as_str()),
            AppError::Internal(m) => ("INTERNAL_ERROR", m.as_str()),
        };

        AppErrorResponse {
            code: code.to_string(),
            message: message.to_string(),
        }
        .serialize(serializer)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("DNSPod error: {message} (request_id: {request_id})")]
    DnsPod { message: String, request_id: String },

    #[error("Cloudflare error (code {code}): {message}")]
    Cloudflare { code: u16, message: String },

    #[error("AliDNS error (code {code}): {message}")]
    AliDns { code: String, message: String },

    #[error("Rate limited (retry after {retry_after:?}s)")]
    RateLimited { retry_after: Option<u64> },

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("{0}")]
    Other(String),
}

impl From<reqwest::Error> for ProviderError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            ProviderError::Network("Request timed out".to_string())
        } else if e.is_connect() {
            ProviderError::Network(format!("Connection failed: {}", e))
        } else {
            ProviderError::Network(e.to_string())
        }
    }
}

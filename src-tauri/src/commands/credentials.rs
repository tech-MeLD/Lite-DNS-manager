use crate::commands::shared;
use crate::error::AppError;
use crate::models::{CredentialInput, ProviderCredential, ProviderType, CredentialSecretData};
use crate::security::credential_manager;
use chrono::Utc;
use uuid::Uuid;

#[tauri::command]
pub fn get_credentials() -> Result<Vec<ProviderCredential>, AppError> {
    Ok(shared::load_credentials_metadata())
}

#[tauri::command]
pub fn save_credential(input: CredentialInput) -> Result<ProviderCredential, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let secret = match input.provider_type {
        ProviderType::DnsPod => {
            let secret_id = input
                .secret_id
                .ok_or_else(|| AppError::Validation("SecretId is required for DNSPod".into()))?;
            let secret_key = input
                .secret_key
                .ok_or_else(|| AppError::Validation("SecretKey is required for DNSPod".into()))?;
            CredentialSecretData::DnsPod { secret_id, secret_key }
        }
        ProviderType::Cloudflare => {
            let api_token = input
                .api_token
                .ok_or_else(|| AppError::Validation("API Token is required for Cloudflare".into()))?;
            CredentialSecretData::Cloudflare { api_token }
        }
        ProviderType::AliDns => {
            let access_key_id = input
                .access_key_id
                .ok_or_else(|| AppError::Validation("AccessKey ID is required for AliDNS".into()))?;
            let access_key_secret = input
                .access_key_secret
                .ok_or_else(|| AppError::Validation("AccessKey Secret is required for AliDNS".into()))?;
            CredentialSecretData::AliDns { access_key_id, access_key_secret }
        }
    };

    credential_manager::store_secret(&id, &input.label, &secret)?;

    let credential = ProviderCredential {
        id,
        provider_type: input.provider_type,
        label: input.label,
        created_at: now,
        updated_at: now,
    };

    let mut credentials = shared::load_credentials_metadata();
    credentials.push(credential.clone());
    shared::save_credentials_metadata(&credentials);

    Ok(credential)
}

#[tauri::command]
pub fn delete_credential(id: Uuid) -> Result<(), AppError> {
    credential_manager::delete_secret(&id)?;

    let mut credentials = shared::load_credentials_metadata();
    credentials.retain(|c| c.id != id);
    shared::save_credentials_metadata(&credentials);

    Ok(())
}

#[tauri::command]
pub async fn test_credential(id: Uuid) -> Result<bool, AppError> {
    let credentials = shared::load_credentials_metadata();
    let _credential = credentials
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Credential not found: {}", id)))?;

    let secret = credential_manager::retrieve_secret(&id)?;
    let provider = crate::providers::create_provider(&secret)?;

    match provider.list_domains().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

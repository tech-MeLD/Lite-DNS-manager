use crate::commands::shared;
use crate::error::AppError;
use crate::models::ProviderType;
use crate::security::credential_manager;

#[tauri::command]
pub async fn export_zone(
    provider: ProviderType,
    domain_id: String,
) -> Result<String, AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.export_zone(&domain_id).await.map_err(AppError::from)
}

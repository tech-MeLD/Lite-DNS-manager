use crate::commands::shared;
use crate::error::AppError;
use crate::models::{Domain, DomainSummary, ProviderType};
use crate::security::credential_manager;

#[tauri::command]
pub async fn list_domains(
    provider_filter: Option<Vec<ProviderType>>,
) -> Result<Vec<Domain>, AppError> {
    let credentials = shared::load_credentials_metadata();
    let mut tasks = Vec::new();

    for cred in credentials {
        if let Some(ref filter) = provider_filter {
            if !filter.contains(&cred.provider_type) {
                continue;
            }
        }

        let provider_type = cred.provider_type;
        if let Ok(secret) = credential_manager::retrieve_secret(&cred.id) {
            if let Ok(provider) = crate::providers::create_provider(&secret) {
                tasks.push(tokio::spawn(async move {
                    match provider.list_domains().await {
                        Ok(domains) => domains,
                        Err(e) => {
                            log::warn!("Failed to list domains for {}: {}", provider_type, e);
                            vec![]
                        }
                    }
                }));
            }
        }
    }

    let mut all_domains = Vec::new();
    for task in tasks {
        if let Ok(domains) = task.await {
            all_domains.extend(domains);
        }
    }

    all_domains.sort_by(|a, b| {
        a.provider.to_string().cmp(&b.provider.to_string())
            .then(a.name.cmp(&b.name))
    });

    Ok(all_domains)
}

#[tauri::command]
pub async fn get_domain(
    provider: ProviderType,
    domain_id: String,
) -> Result<Domain, AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.get_domain(&domain_id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn get_domain_summary() -> Result<DomainSummary, AppError> {
    let domains = list_domains(None).await?;

    let mut summary = DomainSummary {
        total_domains: domains.len() as u32,
        dnspod_count: 0,
        cloudflare_count: 0,
        alidns_count: 0,
    };

    for domain in &domains {
        match domain.provider {
            ProviderType::DnsPod => summary.dnspod_count += 1,
            ProviderType::Cloudflare => summary.cloudflare_count += 1,
            ProviderType::AliDns => summary.alidns_count += 1,
        }
    }

    Ok(summary)
}

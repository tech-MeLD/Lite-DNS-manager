use crate::commands::shared;
use crate::error::AppError;
use crate::models::{
    CreateRecordRequest, DnsRecord, ProviderType, SearchQuery, SearchResult, UpdateRecordRequest,
};
use crate::security::credential_manager;

#[tauri::command]
pub async fn list_records(
    provider: ProviderType,
    domain_id: String,
) -> Result<Vec<DnsRecord>, AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.list_records(&domain_id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn create_record(
    provider: ProviderType,
    domain_id: String,
    record: CreateRecordRequest,
) -> Result<DnsRecord, AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.create_record(&domain_id, &record).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn update_record(
    provider: ProviderType,
    domain_id: String,
    record_id: String,
    record: UpdateRecordRequest,
) -> Result<DnsRecord, AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.update_record(&domain_id, &record_id, &record).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn delete_record(
    provider: ProviderType,
    domain_id: String,
    record_id: String,
) -> Result<(), AppError> {
    let credentials = shared::load_credentials_metadata();
    let cred = credentials
        .iter()
        .find(|c| c.provider_type == provider)
        .ok_or_else(|| AppError::NotFound(format!("No credentials for {}", provider)))?;

    let secret = credential_manager::retrieve_secret(&cred.id)?;
    let provider_impl = crate::providers::create_provider(&secret)?;
    provider_impl.delete_record(&domain_id, &record_id).await.map_err(AppError::from)
}

#[tauri::command]
pub async fn search_records(query: SearchQuery) -> Result<Vec<SearchResult>, AppError> {
    let credentials = shared::load_credentials_metadata();
    let mut tasks = Vec::new();

    for cred in &credentials {
        if let Some(ref providers) = query.providers {
            if !providers.contains(&cred.provider_type) {
                continue;
            }
        }

        let domain_ids = query.domain_ids.clone();
        let keyword = query.keyword.clone();
        let record_type = query.record_type.clone();

        if let Ok(secret) = credential_manager::retrieve_secret(&cred.id) {
            if let Ok(provider_impl) = crate::providers::create_provider(&secret) {
                let provider_type = cred.provider_type.clone();
                tasks.push(tokio::spawn(async move {
                    let mut results = Vec::new();

                    let domains = match (domain_ids.as_ref(), provider_impl.list_domains().await) {
                        (Some(ids), _) => {
                            let mut list = Vec::new();
                            for id in ids {
                                if let Ok(d) = provider_impl.get_domain(id).await {
                                    list.push(d);
                                }
                            }
                            list
                        }
                        (None, Ok(list)) => list,
                        _ => return results,
                    };

                    for domain in domains {
                        let records = match provider_impl.search_records(&domain.id, &keyword).await {
                            Ok(recs) => recs,
                            Err(_) => continue,
                        };

                        for record in records {
                            if let Some(ref rt) = record_type {
                                if record.record_type != *rt {
                                    continue;
                                }
                            }
                            results.push(SearchResult {
                                domain_name: domain.name.clone(),
                                provider: provider_type.clone(),
                                record,
                            });
                        }
                    }

                    results
                }));
            }
        }
    }

    let mut all_results = Vec::new();
    for task in tasks {
        if let Ok(results) = task.await {
            all_results.extend(results);
        }
    }

    Ok(all_results)
}

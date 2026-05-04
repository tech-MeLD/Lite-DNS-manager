use crate::models::ProviderCredential;
use std::fs;
use std::path::PathBuf;

pub(crate) fn get_credentials_file_path() -> PathBuf {
    let mut path = app_data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".dns-manager");
    if let Err(e) = fs::create_dir_all(&path) {
        log::error!("Failed to create app data directory: {}", e);
    }
    path.push("credentials.json");
    path
}

pub(crate) fn load_credentials_metadata() -> Vec<ProviderCredential> {
    let path = get_credentials_file_path();
    match fs::read_to_string(&path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(creds) => creds,
            Err(e) => {
                log::error!("Failed to parse credentials file: {}", e);
                vec![]
            }
        },
        Err(e) => {
            if e.kind() != std::io::ErrorKind::NotFound {
                log::error!("Failed to read credentials file: {}", e);
            }
            vec![]
        }
    }
}

pub(crate) fn save_credentials_metadata(credentials: &[ProviderCredential]) {
    let path = get_credentials_file_path();
    match serde_json::to_string_pretty(credentials) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Failed to write credentials file: {}", e);
            }
        }
        Err(e) => {
            log::error!("Failed to serialize credentials: {}", e);
        }
    }
}

pub(crate) fn app_data_dir() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            if let Some(home) = std::env::var("USERPROFILE").ok() {
                Some(PathBuf::from(home).join("AppData").join("Roaming"))
            } else {
                None
            }
        })
}

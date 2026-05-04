use crate::models::CredentialSecretData;
use anyhow::{Context, Result};
use uuid::Uuid;

const SERVICE_NAME: &str = "dns-manager";

/// Stores a credential secret in the OS keychain (Windows Credential Manager)
pub fn store_secret(id: &Uuid, label: &str, secret: &CredentialSecretData) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, &id.to_string())?;

    let secret_json = serde_json::to_string(secret)
        .context("Failed to serialize credential secret")?;

    entry.set_password(&secret_json)
        .context("Failed to store credential in keychain")?;

    log::info!("Credential stored successfully: {} ({})", label, id);
    Ok(())
}

/// Retrieves a credential secret from the OS keychain
pub fn retrieve_secret(id: &Uuid) -> Result<CredentialSecretData> {
    let entry = keyring::Entry::new(SERVICE_NAME, &id.to_string())?;

    let secret_json = entry
        .get_password()
        .context(format!("Credential not found in keychain: {}", id))?;

    serde_json::from_str(&secret_json)
        .context("Failed to deserialize credential secret")
}

/// Deletes a credential secret from the OS keychain
pub fn delete_secret(id: &Uuid) -> Result<()> {
    let entry = keyring::Entry::new(SERVICE_NAME, &id.to_string())?;

    entry
        .delete_credential()
        .context(format!("Failed to delete credential from keychain: {}", id))?;

    log::info!("Credential deleted from keychain: {}", id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_and_retrieve_secret() {
        let id = Uuid::new_v4();
        let secret = CredentialSecretData::Cloudflare {
            api_token: "test-token-123".to_string(),
        };

        store_secret(&id, "Test CF", &secret).unwrap();
        let retrieved = retrieve_secret(&id).unwrap();

        match (secret, retrieved) {
            (CredentialSecretData::Cloudflare { api_token: a }, CredentialSecretData::Cloudflare { api_token: b }) => {
                assert_eq!(a, b);
            }
            _ => panic!("Secret type mismatch"),
        }

        delete_secret(&id).unwrap();
    }
}

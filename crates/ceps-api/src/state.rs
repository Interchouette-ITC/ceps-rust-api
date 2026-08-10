//! Shared application state.

use crate::config::Config;
use crate::sign::LocalKeyring;
use std::sync::Arc;

#[cfg(feature = "sign-kms")]
use crate::kms::KmsClient;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub keyring: LocalKeyring,
    #[cfg(feature = "sign-kms")]
    pub kms: Option<KmsClient>,
}

impl AppState {
    #[must_use]
    pub fn new(config: Config) -> Self {
        #[cfg(feature = "sign-kms")]
        let kms = if config.kms_url_configured() {
            Some(KmsClient::new(config.kms_url.clone()))
        } else {
            None
        };

        Self {
            keyring: config.local_keys.clone(),
            config: Arc::new(config),
            #[cfg(feature = "sign-kms")]
            kms,
        }
    }
}

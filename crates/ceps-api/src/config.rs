//! Environment configuration. No PEM, faucet, or bootstrap key vars.

use crate::constants::{
    DEFAULT_APP_ADDR, DEFAULT_APP_PORT, DEFAULT_CHAIN_NAME, DEFAULT_RPC_URL, DEFAULT_SSE_URL,
};
use std::env;
use std::path::PathBuf;

/// Runtime signing backend. Unset or empty `SIGN_BACKEND` means [`SignBackend::None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignBackend {
    #[default]
    None,
    Local,
    Kms,
}

impl SignBackend {
    /// Parse `SIGN_BACKEND`. Unset, empty, or `none` → [`SignBackend::None`].
    /// Operators do not need to set the variable for the no-signer socle.
    #[must_use]
    pub fn from_env() -> Self {
        match env::var("SIGN_BACKEND") {
            Err(_) => Self::None,
            Ok(raw) => {
                let v = raw.trim();
                if v.is_empty() || v.eq_ignore_ascii_case("none") {
                    Self::None
                } else if v.eq_ignore_ascii_case("local") {
                    Self::Local
                } else if v.eq_ignore_ascii_case("kms") {
                    Self::Kms
                } else {
                    tracing::warn!(
                        value = %raw,
                        "invalid SIGN_BACKEND (expected kms|local|none or unset); using none"
                    );
                    Self::None
                }
            }
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Local => "local",
            Self::Kms => "kms",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub addr: String,
    pub port: u16,
    pub rpc_url: String,
    pub sse_url: String,
    pub chain_name: String,
    /// Empty when KMS is not configured (normal for `SIGN_BACKEND` none).
    pub kms_url: String,
    pub sign_backend: SignBackend,
    pub wasm_root: PathBuf,
}

impl Config {
    /// Load from process environment (and dotenv if loaded by caller).
    #[must_use]
    pub fn from_env() -> Self {
        let wasm_root =
            env::var("CEPS_WASM_ROOT").map_or_else(|_| default_wasm_root(), PathBuf::from);
        let kms_url = env::var("KMS_URL")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        Self {
            addr: env::var("APP_ADDR").unwrap_or_else(|_| DEFAULT_APP_ADDR.to_string()),
            port: env::var("APP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_APP_PORT),
            rpc_url: env::var("CEPS_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_string()),
            sse_url: env::var("CEPS_SSE_URL").unwrap_or_else(|_| DEFAULT_SSE_URL.to_string()),
            chain_name: env::var("CEPS_CHAIN_NAME")
                .unwrap_or_else(|_| DEFAULT_CHAIN_NAME.to_string()),
            kms_url,
            sign_backend: SignBackend::from_env(),
            wasm_root,
        }
    }

    #[must_use]
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }

    #[must_use]
    pub fn kms_url_configured(&self) -> bool {
        !self.kms_url.is_empty()
    }

    #[must_use]
    pub const fn enabled_ceps(&self) -> &'static [&'static str] {
        enabled_cep_features()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            addr: DEFAULT_APP_ADDR.to_string(),
            port: DEFAULT_APP_PORT,
            rpc_url: DEFAULT_RPC_URL.to_string(),
            sse_url: DEFAULT_SSE_URL.to_string(),
            chain_name: DEFAULT_CHAIN_NAME.to_string(),
            kms_url: String::new(),
            sign_backend: SignBackend::None,
            wasm_root: default_wasm_root(),
        }
    }
}

fn default_wasm_root() -> PathBuf {
    PathBuf::from("../ceps-rust-ts-client/tests/wasm")
}

#[must_use]
pub const fn enabled_cep_features() -> &'static [&'static str] {
    &[
        #[cfg(feature = "cep18")]
        "18",
        #[cfg(feature = "cep78")]
        "78",
        #[cfg(feature = "cep85")]
        "85",
        #[cfg(feature = "cep95")]
        "95",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_socle_none() {
        let cfg = Config::default();
        assert_eq!(cfg.port, DEFAULT_APP_PORT);
        assert_eq!(cfg.sign_backend, SignBackend::None);
        assert!(!cfg.kms_url_configured());
        assert!(!cfg.rpc_url.is_empty());
    }

    #[test]
    fn sign_backend_as_str() {
        assert_eq!(SignBackend::None.as_str(), "none");
        assert_eq!(SignBackend::Local.as_str(), "local");
        assert_eq!(SignBackend::Kms.as_str(), "kms");
    }

    #[test]
    fn enabled_ceps_lists_compiled_features() {
        let list = enabled_cep_features();
        #[cfg(feature = "all")]
        {
            assert!(list.contains(&"18"));
            assert!(list.contains(&"78"));
            assert!(list.contains(&"85"));
            assert!(list.contains(&"95"));
        }
    }
}

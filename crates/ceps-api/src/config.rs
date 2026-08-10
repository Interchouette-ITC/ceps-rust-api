//! Environment configuration. No PEM, faucet, or bootstrap key vars in HTTP.

use crate::constants::{
    DEFAULT_APP_ADDR, DEFAULT_APP_PORT, DEFAULT_CHAIN_NAME, DEFAULT_RPC_URL, DEFAULT_SSE_URL,
};
use crate::sign::LocalKeyring;
use std::env;
use std::path::PathBuf;

/// Runtime signing backend. Unset or empty `SIGN_BACKEND` means [`SignBackend::None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignBackend {
    /// No in-process secrets. `submit=return` + optional `chain-put` only.
    #[default]
    None,
    /// Lab / tests: PEMs from `LOCAL_KEYS_JSON`.
    Local,
    /// Private-network production without KMS: PEMs from `LOCAL_KEYS_JSON_PRODUCTION`.
    LocalProduction,
    /// Private-network production with KMS peer.
    Kms,
}

impl SignBackend {
    /// Parse `SIGN_BACKEND`. Unset, empty, or `none` → [`SignBackend::None`].
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
                } else if v.eq_ignore_ascii_case("local-production")
                    || v.eq_ignore_ascii_case("local_production")
                {
                    Self::LocalProduction
                } else if v.eq_ignore_ascii_case("kms") {
                    Self::Kms
                } else {
                    tracing::warn!(
                        value = %raw,
                        "invalid SIGN_BACKEND (expected none|local|local-production|kms); using none"
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
            Self::LocalProduction => "local-production",
            Self::Kms => "kms",
        }
    }

    /// In-process PEM keyring (`local` or `local-production`).
    #[must_use]
    pub const fn uses_local_keyring(self) -> bool {
        matches!(self, Self::Local | Self::LocalProduction)
    }

    /// Modes that sign for any HTTP caller who names a known public key.
    #[must_use]
    pub const fn signs_for_callers(self) -> bool {
        matches!(self, Self::Local | Self::LocalProduction | Self::Kms)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub addr: String,
    pub port: u16,
    pub rpc_url: String,
    pub sse_url: String,
    pub chain_name: String,
    pub kms_url: String,
    pub sign_backend: SignBackend,
    pub wasm_root: PathBuf,
    pub local_keys: LocalKeyring,
}

impl Config {
    /// Load from environment and validate. Fatal signing misconfig returns `Err`.
    pub fn from_env() -> Result<Self, String> {
        let wasm_root =
            env::var("CEPS_WASM_ROOT").map_or_else(|_| default_wasm_root(), PathBuf::from);
        let kms_url = env::var("KMS_URL")
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let sign_backend = SignBackend::from_env();
        let local_keys = load_local_keys(sign_backend)?;

        let cfg = Self {
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
            sign_backend,
            wasm_root,
            local_keys,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    /// Fail-fast checks for signing backends (call after building or mutating config).
    pub fn validate(&self) -> Result<(), String> {
        if self.sign_backend == SignBackend::Kms && self.kms_url.is_empty() {
            return Err("SIGN_BACKEND=kms requires KMS_URL".into());
        }
        if self.sign_backend == SignBackend::Local && self.local_keys.is_empty() {
            return Err("SIGN_BACKEND=local requires a non-empty LOCAL_KEYS_JSON".into());
        }
        if self.sign_backend == SignBackend::LocalProduction && self.local_keys.is_empty() {
            return Err(
                "SIGN_BACKEND=local-production requires a non-empty LOCAL_KEYS_JSON_PRODUCTION"
                    .into(),
            );
        }
        if self.sign_backend.uses_local_keyring() && !cfg!(feature = "sign-local") {
            return Err(format!(
                "SIGN_BACKEND={} requires feature sign-local",
                self.sign_backend.as_str()
            ));
        }
        if self.sign_backend == SignBackend::Kms && !cfg!(feature = "sign-kms") {
            return Err("SIGN_BACKEND=kms requires feature sign-kms".into());
        }
        Ok(())
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
            local_keys: LocalKeyring::new(),
        }
    }
}

/// Load `.env` unless `DOTENV_DISABLE` is set to a truthy value (`1`, `true`, `yes`).
pub fn load_dotenv() {
    if dotenv_disabled() {
        return;
    }
    let _ = dotenvy::dotenv();
}

fn dotenv_disabled() -> bool {
    match env::var("DOTENV_DISABLE") {
        Ok(v) => {
            let v = v.trim();
            v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

fn load_local_keys(backend: SignBackend) -> Result<LocalKeyring, String> {
    let (var, label) = match backend {
        SignBackend::Local => ("LOCAL_KEYS_JSON", "LOCAL_KEYS_JSON"),
        SignBackend::LocalProduction => {
            ("LOCAL_KEYS_JSON_PRODUCTION", "LOCAL_KEYS_JSON_PRODUCTION")
        }
        SignBackend::None | SignBackend::Kms => return Ok(LocalKeyring::new()),
    };
    match env::var(var) {
        Ok(raw) => LocalKeyring::from_json(&raw, label).map_err(|e| e.to_string()),
        Err(_) => Ok(LocalKeyring::new()),
    }
}

fn default_wasm_root() -> PathBuf {
    // In-tree tip contracts (`tests/wasm/`), same role as the client pack.
    let from_crate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/wasm");
    if from_crate.is_dir() {
        return from_crate;
    }
    PathBuf::from("tests/wasm")
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
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_kms_requires_url() {
        let cfg = Config {
            sign_backend: SignBackend::Kms,
            kms_url: String::new(),
            ..Default::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(err.contains("KMS_URL"));
    }

    #[test]
    fn validate_local_requires_keys() {
        let cfg = Config {
            sign_backend: SignBackend::Local,
            local_keys: LocalKeyring::new(),
            ..Default::default()
        };
        assert!(cfg.validate().unwrap_err().contains("LOCAL_KEYS_JSON"));
    }

    #[test]
    fn validate_local_production_requires_keys() {
        let cfg = Config {
            sign_backend: SignBackend::LocalProduction,
            local_keys: LocalKeyring::new(),
            ..Default::default()
        };
        assert!(cfg
            .validate()
            .unwrap_err()
            .contains("LOCAL_KEYS_JSON_PRODUCTION"));
    }

    #[test]
    fn sign_backend_as_str() {
        assert_eq!(SignBackend::None.as_str(), "none");
        assert_eq!(SignBackend::Local.as_str(), "local");
        assert_eq!(SignBackend::LocalProduction.as_str(), "local-production");
        assert_eq!(SignBackend::Kms.as_str(), "kms");
    }

    #[test]
    fn enabled_ceps_lists_compiled_features() {
        let list = enabled_cep_features();
        #[cfg(feature = "ceps-all")]
        {
            assert!(list.contains(&"18"));
            assert!(list.contains(&"78"));
            assert!(list.contains(&"85"));
            assert!(list.contains(&"95"));
        }
        #[cfg(not(feature = "ceps-all"))]
        {
            let _ = list;
        }
    }
}

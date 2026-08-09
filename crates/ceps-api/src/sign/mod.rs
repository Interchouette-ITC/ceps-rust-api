//! Local PEM keyring (never accepted from HTTP bodies).

use crate::error::ApiError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct LocalKeyring {
    inner: Arc<RwLock<HashMap<String, String>>>,
}

impl LocalKeyring {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn from_json(raw: &str) -> Result<Self, ApiError> {
        let ring = Self::new();
        if raw.trim().is_empty() {
            return Ok(ring);
        }
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| ApiError::BadRequest(format!("LOCAL_KEYS_JSON: {e}")))?;
        match value {
            serde_json::Value::Object(map) if map.contains_key("keys") => {
                let keys = map.get("keys").and_then(|v| v.as_array()).ok_or_else(|| {
                    ApiError::BadRequest("LOCAL_KEYS_JSON.keys must be array".into())
                })?;
                for item in keys {
                    let pk = item
                        .get("public_key")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ApiError::BadRequest("key missing public_key".into()))?;
                    let pem = item
                        .get("secret_key_pem")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ApiError::BadRequest("key missing secret_key_pem".into()))?;
                    ring.insert(pk.to_string(), pem.to_string());
                }
            }
            serde_json::Value::Object(map) => {
                for (pk, pem_v) in map {
                    let pem = pem_v.as_str().ok_or_else(|| {
                        ApiError::BadRequest("LOCAL_KEYS_JSON values must be PEM strings".into())
                    })?;
                    ring.insert(pk, pem.to_string());
                }
            }
            _ => {
                return Err(ApiError::BadRequest(
                    "LOCAL_KEYS_JSON must be object map or {keys:[...]}".into(),
                ));
            }
        }
        Ok(ring)
    }

    pub fn insert(&self, public_key: String, secret_key_pem: String) {
        self.inner.write().insert(public_key, secret_key_pem);
    }

    #[must_use]
    pub fn get(&self, public_key: &str) -> Option<String> {
        self.inner.read().get(public_key).cloned()
    }

    pub fn require(&self, public_key: &str) -> Result<String, ApiError> {
        self.get(public_key).ok_or_else(|| {
            ApiError::NoSigner(format!(
                "public_key {public_key} not found in local keyring"
            ))
        })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn public_keys(&self) -> Vec<String> {
        self.inner.read().keys().cloned().collect()
    }
}

impl std::fmt::Debug for LocalKeyring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalKeyring")
            .field("len", &self.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_map_and_keys_array() {
        let map = r#"{"01aa":"pem-a"}"#;
        let ring = LocalKeyring::from_json(map).unwrap();
        assert_eq!(ring.get("01aa").as_deref(), Some("pem-a"));

        let arr = r#"{"keys":[{"public_key":"01bb","secret_key_pem":"pem-b"}]}"#;
        let ring = LocalKeyring::from_json(arr).unwrap();
        assert_eq!(ring.require("01bb").unwrap(), "pem-b");
    }
}

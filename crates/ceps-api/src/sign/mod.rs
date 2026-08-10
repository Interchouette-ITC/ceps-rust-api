//! Local PEM keyring (never accepted from HTTP bodies).
//!
//! Values may be inline PEM text or a filesystem path to a `.pem` file.

use crate::error::ApiError;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
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

    /// Parse keyring JSON. `env_name` is used in error messages (`LOCAL_KEYS_JSON` / production).
    pub fn from_json(raw: &str, env_name: &str) -> Result<Self, ApiError> {
        let ring = Self::new();
        if raw.trim().is_empty() {
            return Ok(ring);
        }
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|e| ApiError::BadRequest(format!("{env_name}: {e}")))?;
        match value {
            serde_json::Value::Object(map) if map.contains_key("keys") => {
                let keys = map.get("keys").and_then(|v| v.as_array()).ok_or_else(|| {
                    ApiError::BadRequest(format!("{env_name}.keys must be array"))
                })?;
                for item in keys {
                    let pk = item
                        .get("public_key")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ApiError::BadRequest("key missing public_key".into()))?;
                    let material = item
                        .get("secret_key_pem")
                        .or_else(|| item.get("secret_key_path"))
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            ApiError::BadRequest(
                                "key needs secret_key_pem or secret_key_path".into(),
                            )
                        })?;
                    let pem = resolve_pem_material(material, env_name)?;
                    ring.insert(pk.to_string(), pem);
                }
            }
            serde_json::Value::Object(map) => {
                for (pk, pem_v) in map {
                    let material = pem_v.as_str().ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "{env_name} values must be PEM strings or paths"
                        ))
                    })?;
                    let pem = resolve_pem_material(material, env_name)?;
                    ring.insert(pk, pem);
                }
            }
            _ => {
                return Err(ApiError::BadRequest(format!(
                    "{env_name} must be object map or {{keys:[...]}}"
                )));
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

/// Inline PEM text, or a path to a PEM file on disk.
fn resolve_pem_material(value: &str, env_name: &str) -> Result<String, ApiError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ApiError::BadRequest(format!(
            "{env_name}: empty secret material"
        )));
    }
    if looks_like_pem(trimmed) {
        return Ok(trimmed.to_string());
    }
    let path = Path::new(trimmed);
    if path.is_file() {
        return std::fs::read_to_string(path).map_err(|e| {
            ApiError::BadRequest(format!("{env_name}: read {}: {e}", path.display()))
        });
    }
    Err(ApiError::BadRequest(format!(
        "{env_name}: value is neither PEM text nor an existing file path"
    )))
}

fn looks_like_pem(s: &str) -> bool {
    s.contains("-----BEGIN") && s.contains("PRIVATE KEY")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_map_and_keys_array() {
        let map = r#"{"01aa":"-----BEGIN PRIVATE KEY-----\npem-a\n-----END PRIVATE KEY-----"}"#;
        let ring = LocalKeyring::from_json(map, "LOCAL_KEYS_JSON").unwrap();
        assert!(ring.get("01aa").unwrap().contains("pem-a"));

        let arr = r#"{"keys":[{"public_key":"01bb","secret_key_pem":"-----BEGIN PRIVATE KEY-----\npem-b\n-----END PRIVATE KEY-----"}]}"#;
        let ring = LocalKeyring::from_json(arr, "LOCAL_KEYS_JSON").unwrap();
        assert!(ring.require("01bb").unwrap().contains("pem-b"));
    }

    #[test]
    fn load_pem_from_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("secret.pem");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            "-----BEGIN PRIVATE KEY-----\nfrom-file\n-----END PRIVATE KEY-----"
        )
        .unwrap();
        drop(f);

        let json = format!(
            r#"{{"01cc":"{}"}}"#,
            path.to_str().unwrap().replace('\\', "\\\\")
        );
        let ring = LocalKeyring::from_json(&json, "LOCAL_KEYS_JSON").unwrap();
        assert!(ring.get("01cc").unwrap().contains("from-file"));

        let json_arr = format!(
            r#"{{"keys":[{{"public_key":"01dd","secret_key_path":"{}"}}]}}"#,
            path.to_str().unwrap().replace('\\', "\\\\")
        );
        let ring = LocalKeyring::from_json(&json_arr, "LOCAL_KEYS_JSON_PRODUCTION").unwrap();
        assert!(ring.require("01dd").unwrap().contains("from-file"));
    }
}

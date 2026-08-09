//! Custody helpers: create local keys into the process keyring.

use crate::error::ApiError;
use crate::sign::LocalKeyring;
use casper_rust_wasm_sdk::helpers::{
    public_key_from_secret_key, secret_key_generate, secret_key_secp256k1_generate,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyAlgo {
    #[default]
    Ed25519,
    Secp256k1,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateKeyRequest {
    #[serde(default)]
    pub algo: KeyAlgo,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateKeyResult {
    pub public_key: String,
    pub algo: KeyAlgo,
}

pub fn create_local_key(ring: &LocalKeyring, algo: KeyAlgo) -> Result<CreateKeyResult, ApiError> {
    let sk = match algo {
        KeyAlgo::Ed25519 => secret_key_generate()
            .map_err(|e| ApiError::Internal(format!("ed25519 generate: {e}")))?,
        KeyAlgo::Secp256k1 => secret_key_secp256k1_generate()
            .map_err(|e| ApiError::Internal(format!("secp256k1 generate: {e}")))?,
    };
    let pem = sk
        .to_pem()
        .map_err(|e| ApiError::Internal(format!("to_pem: {e}")))?;
    let public_key = public_key_from_secret_key(&pem)
        .map_err(|e| ApiError::Internal(format!("public key: {e}")))?;
    ring.insert(public_key.clone(), pem);
    Ok(CreateKeyResult { public_key, algo })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_ed25519_lands_in_keyring() {
        let ring = LocalKeyring::new();
        let created = create_local_key(&ring, KeyAlgo::Ed25519).unwrap();
        assert!(created.public_key.starts_with("01") || created.public_key.len() > 10);
        assert!(ring.get(&created.public_key).is_some());
    }
}

//! Compiled Cargo feature flags exposed on hello / OpenAPI.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct CompiledFeatures {
    pub cep18: bool,
    pub cep78: bool,
    pub cep85: bool,
    pub cep95: bool,
    pub tx_return: bool,
    pub sign_local: bool,
    pub sign_kms: bool,
    pub chain_put: bool,
    pub swagger_ui: bool,
}

impl CompiledFeatures {
    #[must_use]
    pub const fn current() -> Self {
        Self {
            cep18: cfg!(feature = "cep18"),
            cep78: cfg!(feature = "cep78"),
            cep85: cfg!(feature = "cep85"),
            cep95: cfg!(feature = "cep95"),
            tx_return: cfg!(feature = "tx-return"),
            sign_local: cfg!(feature = "sign-local"),
            sign_kms: cfg!(feature = "sign-kms"),
            chain_put: cfg!(feature = "chain-put"),
            swagger_ui: cfg!(feature = "swagger-ui"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_build_has_socle_addons() {
        let f = CompiledFeatures::current();
        assert_eq!(f.cep18, cfg!(feature = "cep18"));
        assert_eq!(f.cep78, cfg!(feature = "cep78"));
        assert_eq!(f.cep85, cfg!(feature = "cep85"));
        assert_eq!(f.cep95, cfg!(feature = "cep95"));
        assert_eq!(f.tx_return, cfg!(feature = "tx-return"));
        assert_eq!(f.sign_local, cfg!(feature = "sign-local"));
        assert_eq!(f.sign_kms, cfg!(feature = "sign-kms"));
        assert_eq!(f.chain_put, cfg!(feature = "chain-put"));
        assert_eq!(f.swagger_ui, cfg!(feature = "swagger-ui"));
    }
}

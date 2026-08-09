//! In-memory CEP contract instance registry.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct InstanceRecord {
    pub id: String,
    pub cep: String,
    pub contract_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Default)]
pub struct InstanceRegistry {
    inner: Arc<RwLock<HashMap<String, InstanceRecord>>>,
}

impl InstanceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        cep: impl Into<String>,
        contract_hash: impl Into<String>,
        package_hash: Option<String>,
        label: Option<String>,
    ) -> InstanceRecord {
        let id = Uuid::new_v4().to_string();
        let rec = InstanceRecord {
            id: id.clone(),
            cep: cep.into(),
            contract_hash: contract_hash.into(),
            package_hash,
            label,
        };
        self.inner.write().insert(id, rec.clone());
        rec
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<InstanceRecord> {
        self.inner.read().get(id).cloned()
    }

    #[must_use]
    pub fn list(&self) -> Vec<InstanceRecord> {
        self.inner.read().values().cloned().collect()
    }

    pub fn remove(&self, id: &str) -> bool {
        self.inner.write().remove(id).is_some()
    }
}

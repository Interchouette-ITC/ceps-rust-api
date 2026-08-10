//! HTTP mutate envelope (submit / wait / signer). No PEM fields.

use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum SubmitMode {
    #[default]
    Put,
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum WaitMode {
    Accepted,
    #[default]
    Processed,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignerRef {
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MutateEnvelope {
    #[serde(default)]
    pub submit: SubmitMode,
    #[serde(default)]
    pub wait: WaitMode,
    pub signer: SignerRef,
    #[serde(default = "default_payment")]
    pub payment_amount: String,
}

fn default_payment() -> String {
    crate::constants::DEFAULT_CALL_PAYMENT.to_string()
}

impl MutateEnvelope {
    pub fn validate_features(&self) -> Result<(), ApiError> {
        if matches!(self.submit, SubmitMode::Return) && !cfg!(feature = "tx-return") {
            return Err(ApiError::FeatureDisabled(
                "submit=return requires feature tx-return".into(),
            ));
        }
        if self.signer.public_key.trim().is_empty() {
            return Err(ApiError::BadRequest("signer.public_key is required".into()));
        }
        Ok(())
    }

    #[must_use]
    pub fn wait_on_put(&self) -> bool {
        matches!(self.wait, WaitMode::Processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn return_requires_tx_return_feature() {
        let env = MutateEnvelope {
            submit: SubmitMode::Return,
            wait: WaitMode::Processed,
            signer: SignerRef {
                public_key: "01ab".into(),
            },
            payment_amount: "1".into(),
        };
        #[cfg(feature = "tx-return")]
        assert!(env.validate_features().is_ok());
        #[cfg(not(feature = "tx-return"))]
        assert!(matches!(
            env.validate_features(),
            Err(ApiError::FeatureDisabled(_))
        ));
    }

    #[test]
    fn deserialize_defaults_to_put_processed() {
        let v: MutateEnvelope =
            serde_json::from_str(r#"{"signer":{"public_key":"01aa"},"payment_amount":"5"}"#)
                .unwrap();
        assert_eq!(v.submit, SubmitMode::Put);
        assert_eq!(v.wait, WaitMode::Processed);
    }
}

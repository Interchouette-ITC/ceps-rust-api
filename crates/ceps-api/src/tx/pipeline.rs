//! Build ceps-client TransactionParams and finalize CallResult for HTTP.

use crate::config::SignBackend;
use crate::error::ApiError;
use crate::sign::LocalKeyring;
use crate::state::AppState;
use crate::tx::{MutateEnvelope, SubmitMode};
use ceps_client::{CallResult, CepCore, TransactionParams};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PipelineOutcome {
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub put_result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_result: Option<Value>,
}

impl PipelineOutcome {
    #[must_use]
    pub fn from_call(result: CallResult, submit: SubmitMode) -> Self {
        let put_result = if result.put_result.is_null() {
            None
        } else {
            Some(result.put_result)
        };
        let transaction = match submit {
            SubmitMode::Return => result.transaction,
            SubmitMode::Put => None,
        };
        Self {
            transaction_hash: result.transaction_hash,
            transaction,
            put_result,
            execution_result: result.execution_result,
        }
    }
}

/// Build client params for local PEM put/return or unsigned make-only.
pub fn build_transaction_params(
    state: &AppState,
    envelope: &MutateEnvelope,
) -> Result<TransactionParams, ApiError> {
    envelope.validate_features()?;
    let pk = envelope.signer.public_key.trim();

    match envelope.submit {
        SubmitMode::Return => {
            if state.config.sign_backend == SignBackend::Local {
                if let Some(pem) = state.keyring.get(pk) {
                    let mut tx = TransactionParams::new(pem, &envelope.payment_amount).make_only();
                    tx = tx.with_chain_name(state.config.chain_name.clone());
                    return Ok(tx);
                }
            }
            let tx = TransactionParams::for_make(&envelope.payment_amount)
                .with_initiator_addr(pk)
                .with_chain_name(state.config.chain_name.clone());
            Ok(tx)
        }
        SubmitMode::Put => match state.config.sign_backend {
            SignBackend::None => Err(ApiError::NoSigner(
                "submit=put requires SIGN_BACKEND local or kms".into(),
            )),
            SignBackend::Local => {
                let pem = state.keyring.require(pk)?;
                let mut tx = TransactionParams::new(pem, &envelope.payment_amount);
                if !envelope.wait_on_put() {
                    tx = tx.without_wait();
                }
                Ok(tx.with_chain_name(state.config.chain_name.clone()))
            }
            SignBackend::Kms => Ok(TransactionParams::for_make(&envelope.payment_amount)
                .with_initiator_addr(pk)
                .with_chain_name(state.config.chain_name.clone())),
        },
    }
}

/// After a CEP client call: for KMS put, sign+put+wait; otherwise map CallResult.
pub async fn finalize_call(
    state: &AppState,
    core: &CepCore,
    envelope: &MutateEnvelope,
    result: CallResult,
) -> Result<PipelineOutcome, ApiError> {
    envelope.validate_features()?;

    if matches!(envelope.submit, SubmitMode::Put) && state.config.sign_backend == SignBackend::Kms {
        #[cfg(feature = "sign-kms")]
        {
            let mut result = result;
            let tx_json = result.transaction.take().ok_or_else(|| {
                ApiError::Internal("make-only result missing transaction JSON".into())
            })?;
            let kms = state
                .kms
                .as_ref()
                .ok_or_else(|| ApiError::Kms("KMS client not configured (set KMS_URL)".into()))?;
            let signed = kms
                .sign_transaction(&envelope.signer.public_key, &tx_json)
                .await?;
            result = crate::tx::put_signed_transaction(core, &signed).await?;
            if envelope.wait_on_put() {
                let event = core
                    .wait_transaction(&result.transaction_hash, None)
                    .await
                    .map_err(|e| ApiError::Chain(e.to_string()))?;
                result = result.with_execution(event);
            }
            return Ok(PipelineOutcome::from_call(result, SubmitMode::Put));
        }
        #[cfg(not(feature = "sign-kms"))]
        {
            let _ = core;
            return Err(ApiError::FeatureDisabled(
                "SIGN_BACKEND=kms requires feature sign-kms".into(),
            ));
        }
    }

    #[cfg(not(feature = "sign-kms"))]
    let _ = core;

    Ok(PipelineOutcome::from_call(result, envelope.submit))
}

/// Resolve local PEM helper for tests / local sign backend.
#[allow(dead_code)]
#[must_use]
pub fn keyring_len(ring: &LocalKeyring) -> usize {
    ring.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tx::{SignerRef, WaitMode};

    #[test]
    fn put_with_none_is_no_signer() {
        let state = AppState::new(Config::default());
        let env = MutateEnvelope {
            submit: SubmitMode::Put,
            wait: WaitMode::Processed,
            signer: SignerRef {
                public_key: "01aa".into(),
            },
            payment_amount: "1".into(),
        };
        let err = build_transaction_params(&state, &env).unwrap_err();
        assert!(matches!(err, ApiError::NoSigner(_)));
    }

    #[test]
    fn return_unsigned_uses_initiator() {
        let state = AppState::new(Config::default());
        let env = MutateEnvelope {
            submit: SubmitMode::Return,
            wait: WaitMode::Accepted,
            signer: SignerRef {
                public_key: "01aabb".into(),
            },
            payment_amount: "100".into(),
        };
        #[cfg(feature = "tx-return")]
        {
            let tx = build_transaction_params(&state, &env).unwrap();
            assert!(!tx.put);
            assert_eq!(tx.initiator_addr.as_deref(), Some("01aabb"));
        }
    }
}

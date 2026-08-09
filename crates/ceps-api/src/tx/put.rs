//! Put a pre-signed Transaction JSON via the SDK.

use crate::error::ApiError;
use casper_rust_wasm_sdk::types::transaction::Transaction;
use ceps_client::{CallResult, CepCore};
use serde_json::Value;

pub async fn put_signed_transaction(
    core: &CepCore,
    signed: &Value,
) -> Result<CallResult, ApiError> {
    let json_str = serde_json::to_string(signed)
        .map_err(|e| ApiError::BadRequest(format!("transaction json: {e}")))?;
    let transaction = Transaction::from_json_string(&json_str)
        .map_err(|e| ApiError::BadRequest(format!("invalid transaction JSON: {e}")))?;
    let put = core
        .sdk()
        .put_transaction(
            transaction,
            Some(core.verbosity()),
            Some(core.rpc_url().to_string()),
        )
        .await
        .map_err(|e| ApiError::Chain(e.to_string()))?;
    let hash = casper_rust_wasm_sdk::types::hash::transaction_hash::TransactionHash::from(
        put.result.transaction_hash,
    )
    .to_string();
    let put_json = serde_json::to_value(&put.result)
        .map_err(|e| ApiError::Internal(format!("serialize put: {e}")))?;
    Ok(CallResult::new(hash, put_json))
}

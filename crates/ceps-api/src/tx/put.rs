//! Put already-signed Transaction JSON via [`CEPClient::put_transaction`].

use crate::error::ApiError;
use ceps_client::{CEPClient, CallResult};
use serde_json::Value;

pub async fn put_signed_transaction(
    client: &CEPClient,
    signed: &Value,
    wait: bool,
) -> Result<CallResult, ApiError> {
    client
        .put_transaction(signed, wait, None)
        .await
        .map_err(ApiError::from_cep)
}

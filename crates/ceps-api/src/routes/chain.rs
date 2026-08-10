//! Chain put-transaction route (feature `chain-put`).
//!
//! Account/balance/transaction *queries* are not part of this CEP API.
//! Use the node RPC or CEP wait/`CallResult` for transaction outcome.

#[cfg(feature = "chain-put")]
use crate::error::ApiError;
#[cfg(feature = "chain-put")]
use crate::routes::common::cep_core;
#[cfg(feature = "chain-put")]
use crate::state::AppState;
#[cfg(feature = "chain-put")]
use crate::tx::PipelineOutcome;
#[cfg(feature = "chain-put")]
use actix_web::{post, web, HttpResponse};
#[cfg(feature = "chain-put")]
use serde::Deserialize;
#[cfg(feature = "chain-put")]
use serde_json::Value;
#[cfg(feature = "chain-put")]
use utoipa::ToSchema;

#[cfg(feature = "chain-put")]
#[derive(Deserialize, ToSchema)]
pub struct PutTransactionBody {
    pub transaction: Value,
    #[serde(default)]
    pub wait: crate::tx::WaitMode,
}

#[cfg(feature = "chain-put")]
#[utoipa::path(
    post,
    path = "/v1/chain/put-transaction",
    request_body = PutTransactionBody,
    responses((status = 200, description = "Put result", body = PipelineOutcome)),
    tag = "Chain"
)]
#[cfg(feature = "chain-put")]
#[post("/v1/chain/put-transaction")]
pub async fn chain_put_transaction(
    state: web::Data<AppState>,
    body: web::Json<PutTransactionBody>,
) -> Result<HttpResponse, ApiError> {
    let core = cep_core(&state)?;
    let mut result = crate::tx::put_signed_transaction(&core, &body.transaction).await?;
    if matches!(body.wait, crate::tx::WaitMode::Processed) {
        let event = core
            .wait_transaction(&result.transaction_hash, None)
            .await
            .map_err(|e| ApiError::Chain(e.to_string()))?;
        result = result.with_execution(event);
    }
    Ok(HttpResponse::Ok().json(PipelineOutcome::from_call(
        result,
        crate::tx::SubmitMode::Put,
    )))
}

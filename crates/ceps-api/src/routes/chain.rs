//! Chain put-transaction route (feature `chain-put`).
//!
//! Account/balance/transaction *queries* are not part of this CEP API.
//! Transaction outcome comes from CEP `wait` / `CallResult` (see `CEPClient::wait_transaction`
//! when you put with wait disabled).

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
    let client = cep_core(&state)?;
    let wait = matches!(body.wait, crate::tx::WaitMode::Processed);
    let result = crate::tx::put_signed_transaction(&client, &body.transaction, wait).await?;
    Ok(HttpResponse::Ok().json(PipelineOutcome::from_call(
        result,
        crate::tx::SubmitMode::Put,
    )))
}

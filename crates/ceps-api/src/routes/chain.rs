//! Chain query and put-transaction routes.

use crate::error::ApiError;
use crate::routes::common::cep_core;
use crate::state::AppState;
#[cfg(feature = "chain-put")]
use crate::tx::PipelineOutcome;
#[cfg(feature = "chain-put")]
use actix_web::post;
use actix_web::{get, web, HttpResponse};
#[cfg(feature = "chain-put")]
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct BalanceResult {
    pub raw: Value,
}

#[utoipa::path(
    get,
    path = "/v1/chain/balance/{public_key}",
    params(("public_key" = String, Path, description = "Account public key hex")),
    responses((status = 200, description = "Account JSON (includes main purse)", body = BalanceResult)),
    tag = "Chain"
)]
#[get("/v1/chain/balance/{public_key}")]
pub async fn chain_balance(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let core = cep_core(&state)?;
    let account = {
        #[allow(deprecated)]
        core.sdk()
            .get_account(
                None,
                Some(path.into_inner()),
                None,
                Some(core.verbosity()),
                Some(core.rpc_url().to_string()),
            )
            .await
            .map_err(|e| ApiError::Chain(e.to_string()))?
    };
    let raw =
        serde_json::to_value(&account.result).map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(raw))
}

#[utoipa::path(
    get,
    path = "/v1/chain/account/{public_key}",
    params(("public_key" = String, Path, description = "Account public key hex")),
    responses((status = 200, description = "Account JSON")),
    tag = "Chain"
)]
#[get("/v1/chain/account/{public_key}")]
pub async fn chain_account(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let core = cep_core(&state)?;
    let account = {
        #[allow(deprecated)]
        core.sdk()
            .get_account(
                None,
                Some(path.into_inner()),
                None,
                Some(core.verbosity()),
                Some(core.rpc_url().to_string()),
            )
            .await
            .map_err(|e| ApiError::Chain(e.to_string()))?
    };
    let raw =
        serde_json::to_value(&account.result).map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(raw))
}

#[utoipa::path(
    get,
    path = "/v1/chain/transaction/{hash}",
    params(("hash" = String, Path, description = "Transaction hash hex")),
    responses((status = 200, description = "Transaction JSON")),
    tag = "Chain"
)]
#[get("/v1/chain/transaction/{hash}")]
pub async fn chain_transaction(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let core = cep_core(&state)?;
    let tx_hash = casper_rust_wasm_sdk::types::hash::transaction_hash::TransactionHash::new(&path)
        .map_err(|e| ApiError::BadRequest(e.to_string()))?;
    let tx = core
        .sdk()
        .get_transaction(
            tx_hash,
            Some(false),
            Some(core.verbosity()),
            Some(core.rpc_url().to_string()),
        )
        .await
        .map_err(|e| ApiError::Chain(e.to_string()))?;
    let raw = serde_json::to_value(&tx.result).map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(HttpResponse::Ok().json(raw))
}

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

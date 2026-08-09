//! Chain query, fund, and put-transaction routes.

use crate::error::ApiError;
use crate::routes::common::cep_core;
use crate::state::AppState;
use crate::tx::PipelineOutcome;
use actix_web::{get, post, web, HttpResponse};
use casper_rust_wasm_sdk::types::transaction_params::transaction_str_params::TransactionStrParams;
use serde::{Deserialize, Serialize};
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

#[cfg(feature = "custody")]
#[derive(Deserialize, ToSchema)]
pub struct FundBody {
    #[serde(flatten)]
    pub envelope: crate::tx::MutateEnvelope,
    pub target: String,
    pub amount: String,
}

#[cfg(feature = "custody")]
#[utoipa::path(
    post,
    path = "/v1/chain/fund",
    request_body = FundBody,
    responses((status = 200, description = "Native transfer result", body = PipelineOutcome)),
    tag = "Chain"
)]
#[cfg(feature = "custody")]
#[post("/v1/chain/fund")]
pub async fn chain_fund(
    state: web::Data<AppState>,
    body: web::Json<FundBody>,
) -> Result<HttpResponse, ApiError> {
    use crate::config::SignBackend;
    use crate::tx::{finalize_call, SubmitMode};

    body.envelope.validate_features()?;
    let core = cep_core(&state)?;
    let payment = body.envelope.payment_amount.clone();

    let str_params = TransactionStrParams::default();
    str_params.set_chain_name(&state.config.chain_name);
    str_params.set_payment_amount(&payment);

    match (body.envelope.submit, state.config.sign_backend) {
        (SubmitMode::Put, SignBackend::None) => Err(ApiError::NoSigner(
            "submit=put requires SIGN_BACKEND local or kms".into(),
        )),
        (SubmitMode::Put, SignBackend::Local) => {
            let pem = state.keyring.require(&body.envelope.signer.public_key)?;
            str_params.set_secret_key(&pem);
            let put = core
                .sdk()
                .transfer_transaction(
                    None,
                    &body.target,
                    &body.amount,
                    str_params,
                    None,
                    Some(core.verbosity()),
                    Some(core.rpc_url().to_string()),
                )
                .await
                .map_err(|e| ApiError::Chain(e.to_string()))?;
            let hash = casper_rust_wasm_sdk::types::hash::transaction_hash::TransactionHash::from(
                put.result.transaction_hash,
            )
            .to_string();
            let put_json =
                serde_json::to_value(&put.result).map_err(|e| ApiError::Internal(e.to_string()))?;
            let mut call = ceps_client::CallResult::new(hash, put_json);
            if body.envelope.wait_on_put() {
                if let Ok(event) = core.wait_transaction(&call.transaction_hash, None).await {
                    call = call.with_execution(event);
                }
            }
            Ok(HttpResponse::Ok().json(PipelineOutcome::from_call(call, SubmitMode::Put)))
        }
        _ => {
            str_params.set_initiator_addr(&body.envelope.signer.public_key);
            let made = core
                .sdk()
                .make_transfer_transaction(None, &body.target, &body.amount, str_params, None)
                .map_err(|e| ApiError::Chain(e.to_string()))?;
            let json_str = made
                .to_json_string()
                .map_err(|e| ApiError::Internal(e.to_string()))?;
            let value: Value =
                serde_json::from_str(&json_str).map_err(|e| ApiError::Internal(e.to_string()))?;
            let call = ceps_client::CallResult::from_make(made.hash().to_string(), value);
            let out = finalize_call(&state, &core, &body.envelope, call).await?;
            Ok(HttpResponse::Ok().json(out))
        }
    }
}

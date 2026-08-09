//! CEP-95 HTTP routes (core mutates + queries).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep95::InstallArgs;
use ceps_client::Cep95Client;
use serde::Deserialize;

fn client(state: &AppState) -> Result<Cep95Client, ApiError> {
    Cep95Client::new(
        state.config.rpc_url.clone(),
        Some(state.config.sse_url.clone()),
        Some(state.config.chain_name.clone()),
        None,
    )
    .map_err(ApiError::from_cep)
}

macro_rules! mutate {
    ($state:expr, $envelope:expr, $call:expr) => {{
        let tx = build_transaction_params(&$state, &$envelope)?;
        let result = $call(&tx).await.map_err(ApiError::from_cep)?;
        let core = cep_core(&$state)?;
        let out = finalize_call(&$state, &core, &$envelope, result).await?;
        Ok::<_, ApiError>(HttpResponse::Ok().json(out))
    }};
}

#[derive(Deserialize)]
pub struct ContractRef {
    pub contract_hash: String,
    pub package_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct InstallBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    pub wasm: String,
    pub name: String,
    pub symbol: String,
    pub package_hash_key_name: String,
}

#[post("/v1/cep95/install")]
pub async fn cep95_install(
    state: web::Data<AppState>,
    body: web::Json<InstallBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let args = InstallArgs::new(&body.name, &body.symbol, &body.package_hash_key_name);
    mutate!(state, body.envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize)]
pub struct TransferBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub from: String,
    pub to: String,
    pub token_id: String,
}

#[post("/v1/cep95/transfer-from")]
pub async fn cep95_transfer_from(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.transfer_from(
        &body.from,
        &body.to,
        &body.token_id,
        tx
    ))
}

#[get("/v1/cep95/{contract_hash}/name")]
pub async fn cep95_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": client.name().await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep95/{contract_hash}/owner-of/{token_id}")]
pub async fn cep95_owner_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, token_id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "owner": client.owner_of(&token_id).await.map_err(ApiError::from_cep)?
    })))
}

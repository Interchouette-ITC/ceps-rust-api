//! CEP-85 HTTP routes (core mutates + queries).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep85::InstallArgs;
use ceps_client::Cep85Client;
use serde::Deserialize;

fn client(state: &AppState) -> Result<Cep85Client, ApiError> {
    Cep85Client::new(
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
    pub uri: String,
}

#[post("/v1/cep85/install")]
pub async fn cep85_install(
    state: web::Data<AppState>,
    body: web::Json<InstallBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let args = InstallArgs::new(&body.name, &body.uri);
    mutate!(state, body.envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize)]
pub struct MintBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub recipient: String,
    pub id: String,
    pub amount: String,
}

#[post("/v1/cep85/mint")]
pub async fn cep85_mint(
    state: web::Data<AppState>,
    body: web::Json<MintBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.mint(
        &body.recipient,
        &body.id,
        &body.amount,
        None,
        tx
    ))
}

#[derive(Deserialize)]
pub struct TransferBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub from: String,
    pub to: String,
    pub id: String,
    pub amount: String,
}

#[post("/v1/cep85/transfer")]
pub async fn cep85_transfer(
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
    mutate!(state, body.envelope, |tx| client.transfer(
        &body.from,
        &body.to,
        &body.id,
        &body.amount,
        tx
    ))
}

#[derive(Deserialize)]
pub struct BurnBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub owner: String,
    pub id: String,
    pub amount: String,
}

#[post("/v1/cep85/burn")]
pub async fn cep85_burn(
    state: web::Data<AppState>,
    body: web::Json<BurnBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.burn(
        &body.owner,
        &body.id,
        &body.amount,
        tx
    ))
}

#[get("/v1/cep85/{contract_hash}/collection-name")]
pub async fn cep85_collection_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "collection_name": client.collection_name().await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep85/{contract_hash}/balance-of/{owner}/{id}")]
pub async fn cep85_balance_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balance": client.balance_of(&owner, &id).await.map_err(ApiError::from_cep)?
    })))
}

//! CEP-78 HTTP routes (core mutates + queries).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep78::OwnershipMode;
use ceps_client::cep78::{InstallArgs, TokenIdentifier};
use ceps_client::Cep78Client;
use serde::Deserialize;

fn client(state: &AppState) -> Result<Cep78Client, ApiError> {
    Cep78Client::new(
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
    pub collection_name: String,
    pub collection_symbol: String,
    pub total_token_supply: u64,
    #[serde(default = "default_ownership")]
    pub ownership_mode: u8,
}

fn default_ownership() -> u8 {
    2
}

#[post("/v1/cep78/install")]
pub async fn cep78_install(
    state: web::Data<AppState>,
    body: web::Json<InstallBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let args = InstallArgs::new(
        &body.collection_name,
        &body.collection_symbol,
        body.total_token_supply,
    )
    .with_ownership_mode(match body.ownership_mode {
        0 => OwnershipMode::Minter,
        1 => OwnershipMode::Assigned,
        _ => OwnershipMode::Transferable,
    });
    mutate!(state, body.envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize)]
pub struct MintBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub owner: String,
    pub token_meta_data: String,
    pub token_hash: Option<String>,
}

#[post("/v1/cep78/mint")]
pub async fn cep78_mint(
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
        &body.owner,
        &body.token_meta_data,
        body.token_hash.as_deref(),
        tx
    ))
}

#[derive(Deserialize)]
pub struct TransferBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub source: String,
    pub target: String,
    pub token_id: Option<String>,
    pub token_hash: Option<String>,
}

fn token_from(id: &Option<String>, hash: &Option<String>) -> Result<TokenIdentifier, ApiError> {
    if let Some(id) = id {
        Ok(TokenIdentifier::Id(id.parse().map_err(|e| {
            ApiError::BadRequest(format!("token_id: {e}"))
        })?))
    } else if let Some(h) = hash {
        Ok(TokenIdentifier::Hash(h.clone()))
    } else {
        Err(ApiError::BadRequest(
            "token_id or token_hash required".into(),
        ))
    }
}

#[post("/v1/cep78/transfer")]
pub async fn cep78_transfer(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    let token = token_from(&body.token_id, &body.token_hash)?;
    mutate!(state, body.envelope, |tx| client.transfer(
        &body.source,
        &body.target,
        &token,
        tx
    ))
}

#[derive(Deserialize)]
pub struct BurnBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub token_id: Option<String>,
    pub token_hash: Option<String>,
}

#[post("/v1/cep78/burn")]
pub async fn cep78_burn(
    state: web::Data<AppState>,
    body: web::Json<BurnBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    let token = token_from(&body.token_id, &body.token_hash)?;
    mutate!(state, body.envelope, |tx| client.burn(&token, tx))
}

#[get("/v1/cep78/{contract_hash}/collection-name")]
pub async fn cep78_collection_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "collection_name": client.collection_name().await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep78/{contract_hash}/total-token-supply")]
pub async fn cep78_total_token_supply(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_token_supply": client.total_token_supply().await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep78/{contract_hash}/owner-of/{token}")]
pub async fn cep78_owner_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, token) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    let ident = if token.chars().all(|c| c.is_ascii_digit()) {
        TokenIdentifier::Id(
            token
                .parse()
                .map_err(|e| ApiError::BadRequest(format!("{e}")))?,
        )
    } else {
        TokenIdentifier::Hash(token)
    };
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "owner": client.owner_of(&ident).await.map_err(ApiError::from_cep)?
    })))
}

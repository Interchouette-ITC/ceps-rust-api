//! CEP-95 HTTP routes (Cep95Client surface).

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

fn bound(state: &AppState, contract: &ContractRef) -> Result<Cep95Client, ApiError> {
    let mut c = client(state)?;
    bind_contract(
        c.core_mut(),
        &contract.contract_hash,
        contract.package_hash.as_deref(),
    )?;
    Ok(c)
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
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.transfer_from(
        &body.from,
        &body.to,
        &body.token_id,
        tx
    ))
}

#[post("/v1/cep95/safe-transfer-from")]
pub async fn cep95_safe_transfer_from(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.safe_transfer_from(
        &body.from,
        &body.to,
        &body.token_id,
        None,
        tx
    ))
}

#[derive(Deserialize)]
pub struct ApproveBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub spender: String,
    pub token_id: String,
}

#[post("/v1/cep95/approve")]
pub async fn cep95_approve(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.approve(
        &body.spender,
        &body.token_id,
        tx
    ))
}

#[post("/v1/cep95/revoke-approval")]
pub async fn cep95_revoke_approval(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client
        .revoke_approval(&body.token_id, tx))
}

#[derive(Deserialize)]
pub struct ApproveForAllBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub operator: String,
}

#[post("/v1/cep95/approve-for-all")]
pub async fn cep95_approve_for_all(
    state: web::Data<AppState>,
    body: web::Json<ApproveForAllBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client
        .approve_for_all(&body.operator, tx))
}

#[post("/v1/cep95/revoke-approval-for-all")]
pub async fn cep95_revoke_approval_for_all(
    state: web::Data<AppState>,
    body: web::Json<ApproveForAllBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client
        .revoke_approval_for_all(&body.operator, tx))
}

#[derive(Deserialize)]
pub struct MintBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub to: String,
    pub token_id: String,
    pub token_meta_data: Option<Vec<(String, String)>>,
}

#[post("/v1/cep95/mint")]
pub async fn cep95_mint(
    state: web::Data<AppState>,
    body: web::Json<MintBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let meta = body.token_meta_data.clone();
    let meta_ref = meta.as_deref();
    mutate!(state, body.envelope, |tx| client.mint(
        &body.to,
        &body.token_id,
        meta_ref,
        tx
    ))
}

#[derive(Deserialize)]
pub struct BurnBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub token_id: String,
}

#[post("/v1/cep95/burn")]
pub async fn cep95_burn(
    state: web::Data<AppState>,
    body: web::Json<BurnBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.burn(&body.token_id, tx))
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

#[get("/v1/cep95/{contract_hash}/symbol")]
pub async fn cep95_symbol(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "symbol": client.symbol().await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep95/{contract_hash}/total-supply")]
pub async fn cep95_total_supply(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_supply": client.total_supply().await.map_err(ApiError::from_cep)?
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

#[get("/v1/cep95/{contract_hash}/balance-of/{owner}")]
pub async fn cep95_balance_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balance": client.balance_of(&owner).await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep95/{contract_hash}/get-approved/{token_id}")]
pub async fn cep95_get_approved(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, token_id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "approved": client.get_approved(&token_id).await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep95/{contract_hash}/is-approved-for-all/{owner}/{operator}")]
pub async fn cep95_is_approved_for_all(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner, operator) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "approved": client.is_approved_for_all(&owner, &operator).await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep95/{contract_hash}/token-metadata/{token_id}")]
pub async fn cep95_token_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, token_id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "metadata": client.token_metadata(&token_id).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize)]
pub struct BindOdraInstallBody {
    pub installer_public_key: String,
    pub package_hash_key_name: String,
    /// When set, register the resolved hashes in the in-memory instance registry.
    pub label: Option<String>,
}

#[post("/v1/cep95/bind-odra-install")]
pub async fn cep95_bind_odra_install(
    state: web::Data<AppState>,
    body: web::Json<BindOdraInstallBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let (contract_hash, package_hash) = client
        .bind_odra_install(&body.installer_public_key, &body.package_hash_key_name)
        .await
        .map_err(ApiError::from_cep)?;
    let mut out = serde_json::json!({
        "contract_hash": contract_hash,
        "package_hash": package_hash,
    });
    if body.label.is_some() {
        let rec = state.registry.register(
            "95",
            &contract_hash,
            Some(package_hash.clone()),
            body.label.clone(),
        );
        out["instance"] = serde_json::to_value(rec).unwrap_or_default();
    }
    Ok(HttpResponse::Ok().json(out))
}

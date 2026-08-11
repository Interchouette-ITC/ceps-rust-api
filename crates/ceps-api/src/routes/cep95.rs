//! CEP-95 HTTP routes (CEP95Client surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, optional_hex_bytes, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep95::InstallArgs;
use ceps_client::CEP95Client;
use serde::Deserialize;
use utoipa::ToSchema;

fn client(state: &AppState) -> Result<CEP95Client, ApiError> {
    CEP95Client::new(
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

#[derive(Deserialize, ToSchema)]
pub struct ContractRef {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

fn bound(state: &AppState, contract: &ContractRef) -> Result<CEP95Client, ApiError> {
    let mut c = client(state)?;
    bind_contract(
        c.core_mut(),
        &contract.contract_hash,
        contract.package_hash.as_deref(),
    )?;
    Ok(c)
}

#[derive(Deserialize, ToSchema)]
pub struct InstallBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep95")]
    pub wasm: String,
    /// Collection name used for install named-key lookup.
    #[schema(example = "MyOdraNft")]
    pub name: String,
    /// Token ticker symbol.
    #[schema(example = "MON")]
    pub symbol: String,
    /// Named key under which the package hash is stored on the installer account.
    #[schema(example = "cep95_package_hash_MyOdraNft")]
    pub package_hash_key_name: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/install",
    request_body(
        content = InstallBody,
        example = json!({
    "submit": "put",
    "wait": "processed",
    "signer": {"public_key": "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
    "payment_amount": "2500000000",
    "wasm": "cep95",
    "name": "MyOdraNft",
    "symbol": "MON",
    "package_hash_key_name": "cep95_package_hash_MyOdraNft"
}),
    ),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct TransferBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Sender key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub from: String,
    /// Recipient key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub to: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: String,
    /// Optional receiver data as hex (safe_transfer_from only).
    #[schema(example = "0x")]
    pub data: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/transfer-from",
    request_body = TransferBody,
    responses((status = 200, description = "Transfer pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    post,
    path = "/v1/cep95/safe-transfer-from",
    request_body = TransferBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/safe-transfer-from")]
pub async fn cep95_safe_transfer_from(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let data = optional_hex_bytes(body.data.as_deref())?;
    mutate!(state, body.envelope, |tx| client.safe_transfer_from(
        &body.from,
        &body.to,
        &body.token_id,
        data.as_deref(),
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ApproveBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Spender account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub spender: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/approve",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    post,
    path = "/v1/cep95/revoke-approval",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct ApproveForAllBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/approve-for-all",
    request_body = ApproveForAllBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    post,
    path = "/v1/cep95/revoke-approval-for-all",
    request_body = ApproveForAllBody,
    responses((status = 200, description = "Pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct MintBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Recipient key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub to: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: String,
    /// Token metadata JSON string.
    #[schema(example = "{}")]
    pub token_meta_data: Option<Vec<(String, String)>>,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/mint",
    request_body = MintBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct BurnBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/burn",
    request_body = BurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/burn")]
pub async fn cep95_burn(
    state: web::Data<AppState>,
    body: web::Json<BurnBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.burn(&body.token_id, tx))
}

#[derive(Deserialize, ToSchema)]
pub struct TransferOwnershipBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// New Ownable owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub new_owner: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/transfer-ownership",
    request_body = TransferOwnershipBody,
    responses((status = 200, description = "Transfer ownership outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/transfer-ownership")]
pub async fn cep95_transfer_ownership(
    state: web::Data<AppState>,
    body: web::Json<TransferOwnershipBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client
        .transfer_ownership(&body.new_owner, tx))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/get-owner",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Ownable contract owner")),
    tag = "CEP-95"
)]
#[get("/v1/cep95/{contract_hash}/get-owner")]
pub async fn cep95_get_owner(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "owner": client.get_owner().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/name",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/symbol",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/total-supply",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/owner-of/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id")
    ),
    responses((status = 200, description = "Owner key")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/balance-of/{owner}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/get-approved/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/is-approved-for-all/{owner}/{operator}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("operator" = String, Path, description = "Operator account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/token-metadata/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-95"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct BindOdraInstallBody {
    pub installer_public_key: String,
    pub package_hash_key_name: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/bind-odra-install",
    request_body = BindOdraInstallBody,
    responses((status = 200, description = "Resolved contract and package hashes")),
    tag = "CEP-95"
)]
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
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "contract_hash": contract_hash,
        "package_hash": package_hash,
    })))
}

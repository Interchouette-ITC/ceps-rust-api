//! CEP-95 HTTP routes (query-param mutates).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, optional_hex_bytes, resolve_wasm};
use crate::routes::extractors::{
    BalanceResponse, BoolResponse, ContractQuery, MutateQuery, NameResponse,
    OptionalStringResponse, SymbolResponse, TotalSupplyResponse,
};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep95::InstallArgs;
use ceps_client::CEP95Client;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

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

fn bound(state: &AppState, contract: &ContractQuery) -> Result<CEP95Client, ApiError> {
    let mut c = client(state)?;
    bind_contract(
        c.core_mut(),
        &contract.contract_hash,
        contract.package_hash.as_deref(),
    )?;
    Ok(c)
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct InstallQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[param(example = "cep95")]
    pub wasm: String,
    #[param(example = "MyOdraNft")]
    pub name: String,
    #[param(example = "MON")]
    pub symbol: String,
    #[param(example = "cep95_package_hash_MyOdraNft")]
    pub package_hash_key_name: String,
    #[serde(default)]
    #[param(example = false)]
    pub allow_key_override: Option<bool>,
    #[serde(default)]
    #[param(example = true)]
    pub is_upgradable: Option<bool>,
    #[serde(default)]
    #[param(example = false)]
    pub is_upgrade: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/install",
    params(InstallQuery),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/install")]
pub async fn cep95_install(
    state: web::Data<AppState>,
    query: web::Query<InstallQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = InstallArgs::new(&q.name, &q.symbol, &q.package_hash_key_name);
    if let Some(v) = q.allow_key_override {
        args = args.with_allow_key_override(v);
    }
    if let Some(v) = q.is_upgradable {
        args = args.with_upgradable(v);
    }
    if let Some(v) = q.is_upgrade {
        args = args.with_upgrade(v);
    }
    mutate!(state, envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct TransferQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub from: String,
    #[param(example = "01bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")]
    pub to: String,
    #[param(example = "1")]
    pub token_id: String,
    /// Optional receiver data as hex (safe_transfer_from).
    #[serde(default)]
    pub data: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/transfer-from",
    params(TransferQuery),
    responses((status = 200, description = "Transfer pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/transfer-from")]
pub async fn cep95_transfer_from(
    state: web::Data<AppState>,
    query: web::Query<TransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.transfer_from(
        &q.from,
        &q.to,
        &q.token_id,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep95/safe-transfer-from",
    params(TransferQuery),
    responses((status = 200, description = "Safe-transfer pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/safe-transfer-from")]
pub async fn cep95_safe_transfer_from(
    state: web::Data<AppState>,
    query: web::Query<TransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let data = optional_hex_bytes(q.data.as_deref())?;
    mutate!(state, envelope, |tx| client.safe_transfer_from(
        &q.from,
        &q.to,
        &q.token_id,
        data.as_deref(),
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ApproveQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub spender: String,
    #[param(example = "1")]
    pub token_id: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/approve",
    params(ApproveQuery),
    responses((status = 200, description = "Approve pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/approve")]
pub async fn cep95_approve(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.approve(
        &q.spender,
        &q.token_id,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep95/revoke-approval",
    params(ApproveQuery),
    responses((status = 200, description = "Revoke-approval pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/revoke-approval")]
pub async fn cep95_revoke_approval(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .revoke_approval(&q.token_id, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ApproveForAllQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/approve-for-all",
    params(ApproveForAllQuery),
    responses((status = 200, description = "Approve-for-all pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/approve-for-all")]
pub async fn cep95_approve_for_all(
    state: web::Data<AppState>,
    query: web::Query<ApproveForAllQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .approve_for_all(&q.operator, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep95/revoke-approval-for-all",
    params(ApproveForAllQuery),
    responses((status = 200, description = "Revoke-approval-for-all pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/revoke-approval-for-all")]
pub async fn cep95_revoke_approval_for_all(
    state: web::Data<AppState>,
    query: web::Query<ApproveForAllQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .revoke_approval_for_all(&q.operator, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MintQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub to: String,
    #[param(example = "1")]
    pub token_id: String,
}

/// Selected body: Odra metadata key/value pairs (how CEP-95 mint works).
#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct MintMetadataBody {
    #[serde(default)]
    pub metadata: Vec<MetaPair>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MetaPair {
    pub key: String,
    pub value: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/mint",
    params(MintQuery),
    request_body(content = MintMetadataBody, description = "Optional metadata pairs"),
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/mint")]
pub async fn cep95_mint(
    state: web::Data<AppState>,
    query: web::Query<MintQuery>,
    body: Option<web::Json<MintMetadataBody>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let pairs: Vec<(String, String)> = body
        .map(|b| b.into_inner())
        .unwrap_or_default()
        .metadata
        .into_iter()
        .map(|p| (p.key, p.value))
        .collect();
    let meta_ref = if pairs.is_empty() {
        None
    } else {
        Some(pairs.as_slice())
    };
    mutate!(state, envelope, |tx| client.mint(
        &q.to,
        &q.token_id,
        meta_ref,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BurnQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "1")]
    pub token_id: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/burn",
    params(BurnQuery),
    responses((status = 200, description = "Burn pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/burn")]
pub async fn cep95_burn(
    state: web::Data<AppState>,
    query: web::Query<BurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.burn(&q.token_id, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct TransferOwnershipQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub new_owner: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/transfer-ownership",
    params(TransferOwnershipQuery),
    responses((status = 200, description = "Transfer ownership outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/transfer-ownership")]
pub async fn cep95_transfer_ownership(
    state: web::Data<AppState>,
    query: web::Query<TransferOwnershipQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .transfer_ownership(&q.new_owner, tx))
}

#[derive(Serialize, ToSchema)]
pub struct OwnerResponse {
    pub owner: String,
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/get-owner",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Ownable contract owner", body = OwnerResponse)),
    tag = "CEP-95"
)]
#[get("/v1/cep95/{contract_hash}/get-owner")]
pub async fn cep95_get_owner(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(OwnerResponse {
        owner: client.get_owner().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/name",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Collection name", body = NameResponse)),
    tag = "CEP-95"
)]
#[get("/v1/cep95/{contract_hash}/name")]
pub async fn cep95_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(NameResponse {
        name: client.name().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/symbol",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Collection symbol", body = SymbolResponse)),
    tag = "CEP-95"
)]
#[get("/v1/cep95/{contract_hash}/symbol")]
pub async fn cep95_symbol(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(SymbolResponse {
        symbol: client.symbol().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/total-supply",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Total supply", body = TotalSupplyResponse)),
    tag = "CEP-95"
)]
#[get("/v1/cep95/{contract_hash}/total-supply")]
pub async fn cep95_total_supply(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(TotalSupplyResponse {
        total_supply: client.total_supply().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/owner-of/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id")
    ),
    responses((status = 200, description = "Token owner", body = OwnerResponse)),
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
    Ok(HttpResponse::Ok().json(OwnerResponse {
        owner: client
            .owner_of(&token_id)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/balance-of/{owner}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
    ),
    responses((status = 200, description = "Balance", body = BalanceResponse)),
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
    Ok(HttpResponse::Ok().json(BalanceResponse {
        balance: client
            .balance_of(&owner)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/get-approved/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Approved spender", body = OptionalStringResponse)),
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
    Ok(HttpResponse::Ok().json(OptionalStringResponse {
        value: client
            .get_approved(&token_id)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/is-approved-for-all/{owner}/{operator}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("operator" = String, Path, description = "Operator account or key"),
    ),
    responses((status = 200, description = "Approved for all", body = BoolResponse)),
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
    Ok(HttpResponse::Ok().json(BoolResponse {
        value: client
            .is_approved_for_all(&owner, &operator)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[derive(Serialize, ToSchema)]
pub struct TokenMetadataResponse {
    pub metadata: serde_json::Value,
}

#[utoipa::path(
    get,
    path = "/v1/cep95/{contract_hash}/token-metadata/{token_id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token_id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Token metadata", body = TokenMetadataResponse)),
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
    let metadata = client
        .token_metadata(&token_id)
        .await
        .map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(TokenMetadataResponse {
        metadata: serde_json::to_value(metadata).unwrap_or(serde_json::Value::Null),
    }))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BindOdraInstallQuery {
    pub installer_public_key: String,
    pub package_hash_key_name: String,
}

#[derive(Serialize, ToSchema)]
pub struct BindOdraInstallResponse {
    pub contract_hash: String,
    pub package_hash: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep95/bind-odra-install",
    params(BindOdraInstallQuery),
    responses((status = 200, description = "Resolved contract and package hashes", body = BindOdraInstallResponse)),
    tag = "CEP-95"
)]
#[post("/v1/cep95/bind-odra-install")]
pub async fn cep95_bind_odra_install(
    state: web::Data<AppState>,
    query: web::Query<BindOdraInstallQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let mut client = client(&state)?;
    let (contract_hash, package_hash) = client
        .bind_odra_install(&q.installer_public_key, &q.package_hash_key_name)
        .await
        .map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(BindOdraInstallResponse {
        contract_hash,
        package_hash,
    }))
}

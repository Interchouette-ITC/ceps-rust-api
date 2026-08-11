//! CEP-18 HTTP routes (query-param mutates).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::routes::extractors::{
    opt_list, AllowanceResponse, BadgeResponse, BalanceResponse, ContractQuery, DecimalsResponse,
    EnabledResponse, EventsModeResponse, MutateQuery, NameResponse, SymbolResponse,
    TotalSupplyResponse,
};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep18::{ChangeSecurityArgs, InstallArgs, UpgradeArgs};
use ceps_client::CEP18Client;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

fn client(state: &AppState) -> Result<CEP18Client, ApiError> {
    CEP18Client::new(
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

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct InstallOp {
    /// Install/upgrade named-key name (becomes cep18_contract_hash_{name}).
    #[param(example = "MyToken")]
    pub name: String,
    /// Token ticker symbol.
    #[param(example = "MTK")]
    pub symbol: String,
    /// Token decimals.
    #[param(example = 9)]
    pub decimals: u8,
    /// Initial total supply as decimal string (base units).
    #[param(example = "1000000000000")]
    pub total_supply: String,
    /// Canonical wasm id or path under configured wasm roots.
    #[param(example = "cep18")]
    pub wasm: String,
    /// Events mode (`NoEvents`|`CES`|`Native`|`NativeBytes`).
    #[serde(default)]
    #[param(inline)]
    pub events_mode: Option<crate::routes::extractors::EventsModeParam>,
    /// Enable mint and burn entrypoints.
    #[serde(default)]
    #[param(example = true)]
    pub enable_mint_and_burn: Option<bool>,
    /// Admin public keys or account-hashes (repeat query key).
    #[serde(default)]
    pub admin_list: Vec<String>,
    /// Minter public keys or account-hashes (repeat query key).
    #[serde(default)]
    pub minter_list: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct InstallQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    /// Install/upgrade named-key name (becomes cep18_contract_hash_{name}).
    pub name: String,
    /// Token ticker symbol.
    pub symbol: String,
    /// Token decimals.
    pub decimals: u8,
    /// Initial total supply as decimal string (base units).
    pub total_supply: String,
    /// Canonical wasm id or path under configured wasm roots.
    pub wasm: String,
    /// Events mode (`NoEvents`|`CES`|`Native`|`NativeBytes`).
    #[serde(default)]
    pub events_mode: Option<crate::routes::extractors::EventsModeParam>,
    /// Enable mint and burn entrypoints.
    #[serde(default)]
    pub enable_mint_and_burn: Option<bool>,
    /// Admin public keys or account-hashes (repeat query key).
    #[serde(default)]
    pub admin_list: Vec<String>,
    /// Minter public keys or account-hashes (repeat query key).
    #[serde(default)]
    pub minter_list: Vec<String>,
}

crate::impl_flat_query_params!(InstallQuery, mutate, InstallOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/install",
    params(InstallQuery),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/install")]
pub async fn cep18_install(
    state: web::Data<AppState>,
    query: web::Query<InstallQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = InstallArgs::new(&q.name, &q.symbol, q.decimals, &q.total_supply);
    if let Some(m) = q.events_mode {
        args.events_mode = Some(m.to_client());
    }
    if let Some(v) = q.enable_mint_and_burn {
        args = args.with_mint_and_burn(v);
    }
    if let Some(a) = opt_list(q.admin_list) {
        args.admin_list = Some(a);
    }
    if let Some(m) = opt_list(q.minter_list) {
        args.minter_list = Some(m);
    }
    mutate!(state, envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct UpgradeOp {
    #[param(example = "MyToken")]
    pub name: String,
    #[param(example = "cep18")]
    pub wasm: String,
    #[serde(default)]
    #[param(example = "CES")]
    #[param(inline)]
    pub events_mode: Option<crate::routes::extractors::EventsModeParam>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpgradeQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    pub name: String,
    pub wasm: String,
    #[serde(default)]
    pub events_mode: Option<crate::routes::extractors::EventsModeParam>,
}

crate::impl_flat_query_params!(UpgradeQuery, mutate, UpgradeOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/upgrade",
    params(UpgradeQuery),
    responses((status = 200, description = "Upgrade pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/upgrade")]
pub async fn cep18_upgrade(
    state: web::Data<AppState>,
    query: web::Query<UpgradeQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = UpgradeArgs::new(&q.name);
    if let Some(m) = q.events_mode {
        args.events_mode = Some(m.to_client());
    }
    mutate!(state, envelope, |tx| client.upgrade(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct TransferOp {
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub recipient: String,
    #[param(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub recipient: String,
    pub amount: String,
}

crate::impl_flat_query_params!(TransferQuery, mutate_contract, TransferOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/transfer",
    params(TransferQuery),
    responses((status = 200, description = "Transfer pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/transfer")]
pub async fn cep18_transfer(
    state: web::Data<AppState>,
    query: web::Query<TransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client.transfer(
        &q.recipient,
        &q.amount,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct TransferFromOp {
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    #[param(example = "01bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")]
    pub recipient: String,
    #[param(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferFromQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub owner: String,
    pub recipient: String,
    pub amount: String,
}

crate::impl_flat_query_params!(TransferFromQuery, mutate_contract, TransferFromOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/transfer-from",
    params(TransferFromQuery),
    responses((status = 200, description = "Transfer-from pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/transfer-from")]
pub async fn cep18_transfer_from(
    state: web::Data<AppState>,
    query: web::Query<TransferFromQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client.transfer_from(
        &q.owner,
        &q.recipient,
        &q.amount,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct ApproveOp {
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub spender: String,
    #[param(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ApproveQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub spender: String,
    pub amount: String,
}

crate::impl_flat_query_params!(ApproveQuery, mutate_contract, ApproveOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/approve",
    params(ApproveQuery),
    responses((status = 200, description = "Approve pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/approve")]
pub async fn cep18_approve(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client
        .approve(&q.spender, &q.amount, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/increase-allowance",
    params(ApproveQuery),
    responses((status = 200, description = "Increase-allowance pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/increase-allowance")]
pub async fn cep18_increase_allowance(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client
        .increase_allowance(&q.spender, &q.amount, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/decrease-allowance",
    params(ApproveQuery),
    responses((status = 200, description = "Decrease-allowance pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/decrease-allowance")]
pub async fn cep18_decrease_allowance(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client
        .decrease_allowance(&q.spender, &q.amount, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct MintBurnOp {
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    #[param(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct MintBurnQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub owner: String,
    pub amount: String,
}

crate::impl_flat_query_params!(MintBurnQuery, mutate_contract, MintBurnOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/mint",
    params(MintBurnQuery),
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/mint")]
pub async fn cep18_mint(
    state: web::Data<AppState>,
    query: web::Query<MintBurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client.mint(&q.owner, &q.amount, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/burn",
    params(MintBurnQuery),
    responses((status = 200, description = "Burn pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/burn")]
pub async fn cep18_burn(
    state: web::Data<AppState>,
    query: web::Query<MintBurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    mutate!(state, envelope, |tx| client.burn(&q.owner, &q.amount, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct ChangeEventsOp {
    #[param(example = "CES")]
    #[param(inline)]
    pub events_mode: crate::routes::extractors::EventsModeParam,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeEventsQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub events_mode: crate::routes::extractors::EventsModeParam,
}

crate::impl_flat_query_params!(ChangeEventsQuery, mutate_contract, ChangeEventsOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/change-events-mode",
    params(ChangeEventsQuery),
    responses((status = 200, description = "Change-events-mode pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/change-events-mode")]
pub async fn cep18_change_events_mode(
    state: web::Data<AppState>,
    query: web::Query<ChangeEventsQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    let mode = q.events_mode.to_client();
    mutate!(state, envelope, |tx| client.change_events_mode(mode, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct ChangeSecurityOp {
    #[serde(default)]
    pub admin_list: Vec<String>,
    #[serde(default)]
    pub minter_list: Vec<String>,
    #[serde(default)]
    pub none_list: Vec<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeSecurityQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    #[serde(default)]
    pub admin_list: Vec<String>,
    #[serde(default)]
    pub minter_list: Vec<String>,
    #[serde(default)]
    pub none_list: Vec<String>,
}

crate::impl_flat_query_params!(ChangeSecurityQuery, mutate_contract, ChangeSecurityOp);

#[utoipa::path(
    post,
    path = "/v1/cep18/change-security",
    params(ChangeSecurityQuery),
    responses((status = 200, description = "Change-security pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/change-security")]
pub async fn cep18_change_security(
    state: web::Data<AppState>,
    query: web::Query<ChangeSecurityQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &q.contract.contract_hash,
        q.contract.package_hash.as_deref(),
    )?;
    let args = ChangeSecurityArgs {
        admin_list: opt_list(q.admin_list),
        minter_list: opt_list(q.minter_list),
        none_list: opt_list(q.none_list),
    };
    mutate!(state, envelope, |tx| client.change_security(&args, tx))
}

fn bound_client(
    state: &AppState,
    contract_hash: &str,
    package_hash: Option<&str>,
) -> Result<CEP18Client, ApiError> {
    let mut client = client(state)?;
    bind_contract(client.core_mut(), contract_hash, package_hash)?;
    Ok(client)
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/name",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Token name", body = NameResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/name")]
pub async fn cep18_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(NameResponse {
        name: client.name().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/symbol",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Token symbol", body = SymbolResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/symbol")]
pub async fn cep18_symbol(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(SymbolResponse {
        symbol: client.symbol().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/decimals",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Decimals", body = DecimalsResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/decimals")]
pub async fn cep18_decimals(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(DecimalsResponse {
        decimals: client.decimals().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/total-supply",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Total supply", body = TotalSupplyResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/total-supply")]
pub async fn cep18_total_supply(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(TotalSupplyResponse {
        total_supply: client.total_supply().await.map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/events-mode",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Events mode", body = EventsModeResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/events-mode")]
pub async fn cep18_events_mode(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    let mode = client.events_mode().await.map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(EventsModeResponse {
        events_mode: mode.as_str().to_string(),
        events_mode_u8: u8::from(mode),
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/is-mint-and-burn-enabled",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Mint/burn enabled", body = EnabledResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/is-mint-and-burn-enabled")]
pub async fn cep18_is_mint_and_burn_enabled(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(EnabledResponse {
        enabled: client
            .is_mint_and_burn_enabled()
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/balance-of/{owner}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner key or account-hash")
    ),
    responses((status = 200, description = "Balance", body = BalanceResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/balance-of/{owner}")]
pub async fn cep18_balance_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner) = path.into_inner();
    let client = bound_client(&state, &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(BalanceResponse {
        balance: client
            .balance_of(&owner)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/allowances/{owner}/{spender}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("spender" = String, Path, description = "Spender account or key"),
    ),
    responses((status = 200, description = "Allowance", body = AllowanceResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/allowances/{owner}/{spender}")]
pub async fn cep18_allowances(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner, spender) = path.into_inner();
    let client = bound_client(&state, &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(AllowanceResponse {
        allowance: client
            .allowances(&owner, &spender)
            .await
            .map_err(ApiError::from_cep)?,
    }))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/security-badge/{account}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("account" = String, Path, description = "Account or entity key")
    ),
    responses((status = 200, description = "Security badge", body = BadgeResponse)),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/security-badge/{account}")]
pub async fn cep18_security_badge(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, account) = path.into_inner();
    let client = bound_client(&state, &contract_hash, None)?;
    let badge = client
        .security_badge(&account)
        .await
        .map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(BadgeResponse {
        badge: badge.map(|b| b.as_str().to_string()),
        badge_u8: badge.map(|b| b as u8),
    }))
}

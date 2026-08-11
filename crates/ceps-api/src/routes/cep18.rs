//! CEP-18 HTTP routes.

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep18::{ChangeSecurityArgs, InstallArgs, UpgradeArgs};
use ceps_client::{CEP18Client, EventsMode};
use serde::Deserialize;
use utoipa::ToSchema;

fn client(state: &AppState) -> Result<CEP18Client, ApiError> {
    CEP18Client::new(
        state.config.rpc_url.clone(),
        Some(state.config.sse_url.clone()),
        Some(state.config.chain_name.clone()),
        None,
    )
    .map_err(ApiError::from_cep)
}

#[derive(Deserialize, ToSchema)]
pub struct ContractQuery {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct InstallBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    /// Install/upgrade named-key name (becomes cep18_contract_hash_{name}).
    #[schema(example = "MyToken")]
    pub name: String,
    /// Token ticker symbol.
    #[schema(example = "MTK")]
    pub symbol: String,
    /// Token decimals.
    #[schema(example = 9)]
    pub decimals: u8,
    /// Initial total supply as decimal string (base units).
    #[schema(example = "1000000000000")]
    pub total_supply: String,
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep18")]
    pub wasm: String,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: Option<u8>,
    /// Enable mint and burn entrypoints.
    #[schema(example = true)]
    pub enable_mint_and_burn: Option<bool>,
    /// Admin public keys or account-hashes.
    pub admin_list: Option<Vec<String>>,
    /// Minter public keys or account-hashes.
    pub minter_list: Option<Vec<String>>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpgradeBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    /// Install/upgrade named-key name (becomes cep18_contract_hash_{name}).
    #[schema(example = "MyToken")]
    pub name: String,
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep18")]
    pub wasm: String,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: Option<u8>,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Recipient account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub recipient: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferFromBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Recipient account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub recipient: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ApproveBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Spender account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub spender: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct MintBurnBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeEventsBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: u8,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeSecurityBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractQuery,
    /// Admin public keys or account-hashes.
    pub admin_list: Option<Vec<String>>,
    /// Minter public keys or account-hashes.
    pub minter_list: Option<Vec<String>>,
    /// Keys to clear from security lists.
    pub none_list: Option<Vec<String>>,
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

#[utoipa::path(
    post,
    path = "/v1/cep18/install",
    request_body(
        content = InstallBody,
        example = json!({
    "submit": "put",
    "wait": "processed",
    "signer": {"public_key": "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
    "payment_amount": "2500000000",
    "name": "MyToken",
    "symbol": "MTK",
    "decimals": 9,
    "total_supply": "1000000000000",
    "wasm": "cep18",
    "events_mode": 1,
    "enable_mint_and_burn": true
}),
    ),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/install")]
pub async fn cep18_install(
    state: web::Data<AppState>,
    body: web::Json<InstallBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let mut args = InstallArgs::new(&body.name, &body.symbol, body.decimals, &body.total_supply);
    if let Some(m) = body.events_mode {
        args.events_mode = Some(events_mode_from_u8(m)?);
    }
    if let Some(v) = body.enable_mint_and_burn {
        args = args.with_mint_and_burn(v);
    }
    if let Some(a) = body.admin_list.clone() {
        args.admin_list = Some(a);
    }
    if let Some(m) = body.minter_list.clone() {
        args.minter_list = Some(m);
    }
    mutate!(state, body.envelope, |tx| client.install(&args, &wasm, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/upgrade",
    request_body = UpgradeBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/upgrade")]
pub async fn cep18_upgrade(
    state: web::Data<AppState>,
    body: web::Json<UpgradeBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let mut args = UpgradeArgs::new(&body.name);
    if let Some(m) = body.events_mode {
        args.events_mode = Some(events_mode_from_u8(m)?);
    }
    mutate!(state, body.envelope, |tx| client.upgrade(&args, &wasm, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/transfer",
    request_body(
        content = TransferBody,
        example = json!({
    "submit": "put",
    "wait": "processed",
    "signer": {"public_key": "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
    "payment_amount": "2500000000",
    "contract_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "recipient": "01bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "amount": "1000000000"
}),
    ),
    responses((status = 200, description = "Transfer pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/transfer")]
pub async fn cep18_transfer(
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
        &body.recipient,
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/transfer-from",
    request_body = TransferFromBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/transfer-from")]
pub async fn cep18_transfer_from(
    state: web::Data<AppState>,
    body: web::Json<TransferFromBody>,
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
        &body.owner,
        &body.recipient,
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/approve",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/approve")]
pub async fn cep18_approve(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.approve(
        &body.spender,
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/increase-allowance",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/increase-allowance")]
pub async fn cep18_increase_allowance(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.increase_allowance(
        &body.spender,
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/decrease-allowance",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/decrease-allowance")]
pub async fn cep18_decrease_allowance(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    let _tx_check = build_transaction_params(&state, &body.envelope)?;
    let _ = &_tx_check;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    mutate!(state, body.envelope, |tx| client.decrease_allowance(
        &body.spender,
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/mint",
    request_body = MintBurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/mint")]
pub async fn cep18_mint(
    state: web::Data<AppState>,
    body: web::Json<MintBurnBody>,
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
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/burn",
    request_body = MintBurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/burn")]
pub async fn cep18_burn(
    state: web::Data<AppState>,
    body: web::Json<MintBurnBody>,
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
        &body.amount,
        tx
    ))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/change-events-mode",
    request_body = ChangeEventsBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/change-events-mode")]
pub async fn cep18_change_events_mode(
    state: web::Data<AppState>,
    body: web::Json<ChangeEventsBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    let mode = events_mode_from_u8(body.events_mode)?;
    mutate!(state, body.envelope, |tx| client
        .change_events_mode(mode, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep18/change-security",
    request_body = ChangeSecurityBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-18"
)]
#[post("/v1/cep18/change-security")]
pub async fn cep18_change_security(
    state: web::Data<AppState>,
    body: web::Json<ChangeSecurityBody>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(
        client.core_mut(),
        &body.contract.contract_hash,
        body.contract.package_hash.as_deref(),
    )?;
    let args = ChangeSecurityArgs {
        admin_list: body.admin_list.clone(),
        minter_list: body.minter_list.clone(),
        none_list: body.none_list.clone(),
    };
    mutate!(state, body.envelope, |tx| client.change_security(&args, tx))
}

fn events_mode_from_u8(v: u8) -> Result<EventsMode, ApiError> {
    EventsMode::from_u8(v).ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {v}")))
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
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/name")]
pub async fn cep18_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "name": client.name().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/symbol",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Token symbol")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/symbol")]
pub async fn cep18_symbol(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "symbol": client.symbol().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/decimals",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/decimals")]
pub async fn cep18_decimals(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "decimals": client.decimals().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/total-supply",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Total supply")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/total-supply")]
pub async fn cep18_total_supply(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_supply": client.total_supply().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/events-mode",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/events-mode")]
pub async fn cep18_events_mode(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "events_mode": client.events_mode().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/is-mint-and-burn-enabled",
    params(("contract_hash" = String, Path, description = "Contract hash hex")),
    responses((status = 200, description = "Mint/burn enabled flag")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/is-mint-and-burn-enabled")]
pub async fn cep18_is_mint_and_burn_enabled(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let client = bound_client(&state, &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "enabled": client.is_mint_and_burn_enabled().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/balance-of/{owner}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner key or account-hash")
    ),
    responses((status = 200, description = "Balance string")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/balance-of/{owner}")]
pub async fn cep18_balance_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner) = path.into_inner();
    let client = bound_client(&state, &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balance": client.balance_of(&owner).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/allowances/{owner}/{spender}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("spender" = String, Path, description = "Spender account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-18"
)]
#[get("/v1/cep18/{contract_hash}/allowances/{owner}/{spender}")]
pub async fn cep18_allowances(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner, spender) = path.into_inner();
    let client = bound_client(&state, &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "allowance": client.allowances(&owner, &spender).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep18/{contract_hash}/security-badge/{account}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("account" = String, Path, description = "Account or entity key")
    ),
    responses((status = 200, description = "Security badge name if present")),
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
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "badge": badge.map(|b| b.as_str()),
        "badge_u8": badge.map(|b| b as u8),
    })))
}

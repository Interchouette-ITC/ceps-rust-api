//! CEP-78 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep78::{
    InstallArgs, NftMetadataKind, OwnershipMode, SetVariablesArgs, TokenIdentifier, UpgradeArgs,
};
use ceps_client::{Cep78Client, EventsMode78};
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

fn token_from(id: Option<&str>, hash: Option<&str>) -> Result<TokenIdentifier, ApiError> {
    if let Some(id) = id {
        Ok(TokenIdentifier::Id(id.parse().map_err(|e| {
            ApiError::BadRequest(format!("token_id: {e}"))
        })?))
    } else if let Some(h) = hash {
        Ok(TokenIdentifier::Hash(h.to_string()))
    } else {
        Err(ApiError::BadRequest(
            "token_id or token_hash required".into(),
        ))
    }
}

#[derive(Deserialize)]
pub struct ContractRef {
    pub contract_hash: String,
    pub package_hash: Option<String>,
}

fn bound(state: &AppState, contract: &ContractRef) -> Result<Cep78Client, ApiError> {
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
pub struct UpgradeBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    pub wasm: String,
    pub collection_name: String,
    pub total_token_supply: Option<u64>,
    pub events_mode: Option<u8>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[post("/v1/cep78/upgrade")]
pub async fn cep78_upgrade(
    state: web::Data<AppState>,
    body: web::Json<UpgradeBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let mut args = UpgradeArgs::new(&body.collection_name);
    args.total_token_supply = body.total_token_supply;
    if let Some(m) = body.events_mode {
        args.events_mode = Some(events_mode78_from_u8(m)?);
    }
    args.acl_package_mode = body.acl_package_mode;
    args.package_operator_mode = body.package_operator_mode;
    args.operator_burn_mode = body.operator_burn_mode;
    mutate!(state, body.envelope, |tx| client.upgrade(&args, &wasm, tx))
}

fn events_mode78_from_u8(v: u8) -> Result<EventsMode78, ApiError> {
    EventsMode78::from_u8(v).ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {v}")))
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
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
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

#[post("/v1/cep78/transfer")]
pub async fn cep78_transfer(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
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
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    mutate!(state, body.envelope, |tx| client.burn(&token, tx))
}

#[derive(Deserialize)]
pub struct RegisterOwnerBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub token_owner: String,
}

#[post("/v1/cep78/register-owner")]
pub async fn cep78_register_owner(
    state: web::Data<AppState>,
    body: web::Json<RegisterOwnerBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client
        .register_owner(&body.token_owner, tx))
}

#[derive(Deserialize)]
pub struct ApproveBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub spender: String,
    pub token_id: Option<String>,
    pub token_hash: Option<String>,
}

#[post("/v1/cep78/approve")]
pub async fn cep78_approve(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    mutate!(state, body.envelope, |tx| client.approve(
        &body.spender,
        &token,
        tx
    ))
}

#[post("/v1/cep78/revoke")]
pub async fn cep78_revoke(
    state: web::Data<AppState>,
    body: web::Json<ApproveBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    mutate!(state, body.envelope, |tx| client.revoke(
        &body.spender,
        &token,
        tx
    ))
}

#[derive(Deserialize)]
pub struct ApprovalForAllBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub operator: String,
    pub approve_all: bool,
}

#[post("/v1/cep78/set-approval-for-all")]
pub async fn cep78_set_approval_for_all(
    state: web::Data<AppState>,
    body: web::Json<ApprovalForAllBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.set_approval_for_all(
        &body.operator,
        body.approve_all,
        tx
    ))
}

#[derive(Deserialize)]
pub struct SetMetaBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub token_meta_data: String,
    pub token_id: Option<String>,
    pub token_hash: Option<String>,
}

#[post("/v1/cep78/set-token-metadata")]
pub async fn cep78_set_token_metadata(
    state: web::Data<AppState>,
    body: web::Json<SetMetaBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    mutate!(state, body.envelope, |tx| client.set_token_metadata(
        &body.token_meta_data,
        &token,
        tx
    ))
}

#[derive(Deserialize)]
pub struct SetVariablesBody {
    #[serde(flatten)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    pub contract: ContractRef,
    pub allow_minting: Option<bool>,
    pub acl_whitelist: Option<Vec<String>>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[post("/v1/cep78/set-variables")]
pub async fn cep78_set_variables(
    state: web::Data<AppState>,
    body: web::Json<SetVariablesBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let args = SetVariablesArgs {
        allow_minting: body.allow_minting,
        acl_whitelist: body.acl_whitelist.clone(),
        acl_package_mode: body.acl_package_mode,
        package_operator_mode: body.package_operator_mode,
        operator_burn_mode: body.operator_burn_mode,
    };
    mutate!(state, body.envelope, |tx| client.set_variables(&args, tx))
}

macro_rules! qstr {
    ($name:ident, $path:literal, $method:ident) => {
        #[get($path)]
        pub async fn $name(
            state: web::Data<AppState>,
            path: web::Path<String>,
        ) -> Result<HttpResponse, ApiError> {
            let mut client = client(&state)?;
            bind_contract(client.core_mut(), &path, None)?;
            let v = client.$method().await.map_err(ApiError::from_cep)?;
            Ok(HttpResponse::Ok().json(serde_json::json!({ stringify!($method): v })))
        }
    };
}

qstr!(
    cep78_collection_name,
    "/v1/cep78/{contract_hash}/collection-name",
    collection_name
);
qstr!(
    cep78_collection_symbol,
    "/v1/cep78/{contract_hash}/collection-symbol",
    collection_symbol
);
qstr!(
    cep78_total_token_supply,
    "/v1/cep78/{contract_hash}/total-token-supply",
    total_token_supply
);
qstr!(
    cep78_number_of_minted_tokens,
    "/v1/cep78/{contract_hash}/number-of-minted-tokens",
    number_of_minted_tokens
);

#[get("/v1/cep78/{contract_hash}/events-mode")]
pub async fn cep78_events_mode(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    let mode = client.events_mode().await.map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "events_mode": mode.as_str(),
        "events_mode_u8": mode as u8,
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

#[get("/v1/cep78/{contract_hash}/balance-of/{owner}")]
pub async fn cep78_balance_of(
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

#[get("/v1/cep78/{contract_hash}/is-approved-for-all/{owner}/{operator}")]
pub async fn cep78_is_approved_for_all(
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

#[get("/v1/cep78/{contract_hash}/get-approved/{token}")]
pub async fn cep78_get_approved(
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
        "approved": client.get_approved(&ident).await.map_err(ApiError::from_cep)?
    })))
}

#[get("/v1/cep78/{contract_hash}/metadata/{token}")]
pub async fn cep78_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
    query: web::Query<MetadataQuery>,
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
    let kind = match query.kind.unwrap_or(0) {
        0 => NftMetadataKind::Cep78,
        1 => NftMetadataKind::Nft721,
        2 => NftMetadataKind::Raw,
        3 => NftMetadataKind::CustomValidated,
        other => {
            return Err(ApiError::BadRequest(format!(
                "invalid metadata kind {other}"
            )))
        }
    };
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "metadata": client.metadata(&ident, kind).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize)]
pub struct MetadataQuery {
    pub kind: Option<u8>,
}

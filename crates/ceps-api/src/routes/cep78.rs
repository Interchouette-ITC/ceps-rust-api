//! CEP-78 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::routes::extractors::{
    json_value_to_string, opt_list, parse_events_mode78, ContractQuery, MutateQuery,
};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep78::{
    BurnMode, HolderMode, IdentifierMode, InstallArgs, MetadataMutability, MintingMode,
    NamedKeyConventionMode, NftKind, NftMetadataKind, OwnerReverseLookupMode, OwnershipMode,
    SetVariablesArgs, TokenIdentifier, UpgradeArgs, WhitelistMode,
};
use ceps_client::CEP78Client;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

fn client(state: &AppState) -> Result<CEP78Client, ApiError> {
    CEP78Client::new(
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

fn parse_ownership(s: &str) -> Result<OwnershipMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "minter" => Ok(OwnershipMode::Minter),
        "1" | "assigned" => Ok(OwnershipMode::Assigned),
        "2" | "transferable" => Ok(OwnershipMode::Transferable),
        _ => Err(ApiError::BadRequest(format!("invalid ownership_mode {s}"))),
    }
}
fn parse_nft_metadata_kind(s: &str) -> Result<NftMetadataKind, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "cep78" => Ok(NftMetadataKind::CEP78),
        "1" | "nft721" => Ok(NftMetadataKind::Nft721),
        "2" | "raw" => Ok(NftMetadataKind::Raw),
        "3" | "customvalidated" | "custom_validated" => Ok(NftMetadataKind::CustomValidated),
        _ => Err(ApiError::BadRequest(format!(
            "invalid nft_metadata_kind {s}"
        ))),
    }
}
fn parse_identifier_mode(s: &str) -> Result<IdentifierMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "ordinal" => Ok(IdentifierMode::Ordinal),
        "1" | "hash" => Ok(IdentifierMode::Hash),
        _ => Err(ApiError::BadRequest(format!("invalid identifier_mode {s}"))),
    }
}
fn parse_metadata_mutability(s: &str) -> Result<MetadataMutability, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "immutable" => Ok(MetadataMutability::Immutable),
        "1" | "mutable" => Ok(MetadataMutability::Mutable),
        _ => Err(ApiError::BadRequest(format!(
            "invalid metadata_mutability {s}"
        ))),
    }
}
fn parse_nft_kind(s: &str) -> Result<NftKind, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "physical" => Ok(NftKind::Physical),
        "1" | "digital" => Ok(NftKind::Digital),
        "2" | "virtual" => Ok(NftKind::Virtual),
        _ => Err(ApiError::BadRequest(format!("invalid nft_kind {s}"))),
    }
}
fn parse_minting_mode(s: &str) -> Result<MintingMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "installer" => Ok(MintingMode::Installer),
        "1" | "public" => Ok(MintingMode::Public),
        "2" | "acl" => Ok(MintingMode::Acl),
        _ => Err(ApiError::BadRequest(format!("invalid minting_mode {s}"))),
    }
}
fn parse_burn_mode(s: &str) -> Result<BurnMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "burnable" => Ok(BurnMode::Burnable),
        "1" | "nonburnable" | "non_burnable" => Ok(BurnMode::NonBurnable),
        _ => Err(ApiError::BadRequest(format!("invalid burn_mode {s}"))),
    }
}
fn parse_whitelist_mode(s: &str) -> Result<WhitelistMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "unlocked" => Ok(WhitelistMode::Unlocked),
        "1" | "locked" => Ok(WhitelistMode::Locked),
        _ => Err(ApiError::BadRequest(format!("invalid whitelist_mode {s}"))),
    }
}
fn parse_holder_mode(s: &str) -> Result<HolderMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "accounts" => Ok(HolderMode::Accounts),
        "1" | "contracts" => Ok(HolderMode::Contracts),
        "2" | "mixed" => Ok(HolderMode::Mixed),
        _ => Err(ApiError::BadRequest(format!("invalid holder_mode {s}"))),
    }
}
fn parse_owner_reverse_lookup(s: &str) -> Result<OwnerReverseLookupMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "nolookup" | "no_lookup" => Ok(OwnerReverseLookupMode::NoLookup),
        "1" | "complete" => Ok(OwnerReverseLookupMode::Complete),
        "2" | "transfersonly" | "transfers_only" => Ok(OwnerReverseLookupMode::TransfersOnly),
        _ => Err(ApiError::BadRequest(format!(
            "invalid owner_reverse_lookup_mode {s}"
        ))),
    }
}
fn parse_named_key_convention(s: &str) -> Result<NamedKeyConventionMode, ApiError> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "derivedfromcollectionname" | "derived_from_collection_name" => {
            Ok(NamedKeyConventionMode::DerivedFromCollectionName)
        }
        "1" | "v1_0standard" | "v1_0_standard" => Ok(NamedKeyConventionMode::V1_0Standard),
        "2" | "v1_0custom" | "v1_0_custom" => Ok(NamedKeyConventionMode::V1_0Custom),
        _ => Err(ApiError::BadRequest(format!(
            "invalid named_key_convention {s}"
        ))),
    }
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

fn bound(state: &AppState, contract: &ContractQuery) -> Result<CEP78Client, ApiError> {
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
    #[param(example = "cep78")]
    pub wasm: String,
    #[param(example = "MyNFTs")]
    pub collection_name: String,
    #[param(example = "MNFT")]
    pub collection_symbol: String,
    #[param(example = 1000)]
    pub total_token_supply: u64,
    #[serde(default = "default_ownership")]
    #[param(example = "Transferable")]
    pub ownership_mode: String,
    #[serde(default)]
    pub nft_metadata_kind: Option<String>,
    #[serde(default)]
    pub identifier_mode: Option<String>,
    #[serde(default)]
    pub metadata_mutability: Option<String>,
    #[serde(default)]
    pub nft_kind: Option<String>,
    #[serde(default)]
    pub minting_mode: Option<String>,
    #[serde(default)]
    pub allow_minting: Option<bool>,
    #[serde(default)]
    pub operator_burn_mode: Option<bool>,
    #[serde(default)]
    pub package_operator_mode: Option<bool>,
    #[serde(default)]
    pub whitelist_mode: Option<String>,
    #[serde(default)]
    pub holder_mode: Option<String>,
    #[serde(default)]
    pub acl_package_mode: Option<bool>,
    #[serde(default)]
    pub acl_whitelist: Vec<String>,
    #[serde(default)]
    pub burn_mode: Option<String>,
    #[serde(default)]
    pub owner_reverse_lookup_mode: Option<String>,
    #[serde(default)]
    pub named_key_convention: Option<String>,
    #[serde(default)]
    pub access_key_name: Option<String>,
    #[serde(default)]
    pub hash_key_name: Option<String>,
    #[serde(default)]
    pub events_mode: Option<String>,
    #[serde(default)]
    pub transfer_filter_contract: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct InstallSchemaBody {
    #[serde(default)]
    pub json_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct TokenMetaBody {
    pub token_meta_data: serde_json::Value,
}

fn default_ownership() -> String {
    "Transferable".into()
}

#[utoipa::path(
    post,
    path = "/v1/cep78/install",
    params(InstallQuery),
    request_body(content = InstallSchemaBody, description = "Optional json_schema object"),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/install")]
pub async fn cep78_install(
    state: web::Data<AppState>,
    query: web::Query<InstallQuery>,
    body: Option<web::Json<InstallSchemaBody>>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = InstallArgs::new(
        &q.collection_name,
        &q.collection_symbol,
        q.total_token_supply,
    )
    .with_ownership_mode(parse_ownership(&q.ownership_mode)?);
    if let Some(ref v) = q.nft_metadata_kind {
        args = args.with_nft_metadata_kind(parse_nft_metadata_kind(v)?);
    }
    if let Some(ref v) = q.identifier_mode {
        args = args.with_identifier_mode(parse_identifier_mode(v)?);
    }
    if let Some(ref v) = q.metadata_mutability {
        args = args.with_metadata_mutability(parse_metadata_mutability(v)?);
    }
    if let Some(ref v) = q.nft_kind {
        args.nft_kind = Some(parse_nft_kind(v)?);
    }
    if let Some(ref v) = q.minting_mode {
        args = args.with_minting_mode(parse_minting_mode(v)?);
    }
    if let Some(v) = q.allow_minting {
        args.allow_minting = Some(v);
    }
    if let Some(v) = q.operator_burn_mode {
        args.operator_burn_mode = Some(v);
    }
    if let Some(v) = q.package_operator_mode {
        args.package_operator_mode = Some(v);
    }
    if let Some(ref v) = q.whitelist_mode {
        args.whitelist_mode = Some(parse_whitelist_mode(v)?);
    }
    if let Some(ref v) = q.holder_mode {
        args = args.with_holder_mode(parse_holder_mode(v)?);
    }
    if let Some(v) = q.acl_package_mode {
        args.acl_package_mode = Some(v);
    }
    if let Some(v) = opt_list(q.acl_whitelist) {
        args.acl_whitelist = Some(v);
    }
    if let Some(ref v) = q.burn_mode {
        args = args.with_burn_mode(parse_burn_mode(v)?);
    }
    if let Some(ref v) = q.owner_reverse_lookup_mode {
        args = args.with_owner_reverse_lookup_mode(parse_owner_reverse_lookup(v)?);
    }
    if let Some(ref v) = q.named_key_convention {
        args.named_key_convention = Some(parse_named_key_convention(v)?);
    }
    args.access_key_name = q.access_key_name.clone();
    args.hash_key_name = q.hash_key_name.clone();
    if let Some(ref m) = q.events_mode {
        args.events_mode = Some(parse_events_mode78(m)?);
    }
    args.transfer_filter_contract = q.transfer_filter_contract.clone();
    if let Some(schema) = body.and_then(|b| b.into_inner().json_schema) {
        args.json_schema = Some(json_value_to_string(&schema)?);
    }
    mutate!(state, envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct UpgradeQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep78")]
    pub wasm: String,
    /// NFT collection name.
    #[schema(example = "MyNFTs")]
    pub collection_name: String,
    /// Max collection supply.
    #[schema(example = 1000)]
    pub total_token_supply: Option<u64>,
    #[serde(default)]
    #[param(example = "CES")]
    pub events_mode: Option<String>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/upgrade",
    params(UpgradeQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/upgrade")]
pub async fn cep78_upgrade(
    state: web::Data<AppState>,
    query: web::Query<UpgradeQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = UpgradeArgs::new(&q.collection_name);
    args.total_token_supply = q.total_token_supply;
    if let Some(ref m) = q.events_mode {
        args.events_mode = Some(parse_events_mode78(m)?);
    }
    args.acl_package_mode = q.acl_package_mode;
    args.package_operator_mode = q.package_operator_mode;
    args.operator_burn_mode = q.operator_burn_mode;
    mutate!(state, envelope, |tx| client.upgrade(&args, &wasm, tx))
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
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/mint",
    params(MintQuery),
    request_body = TokenMetaBody,
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/mint")]
pub async fn cep78_mint(
    state: web::Data<AppState>,
    query: web::Query<MintQuery>,
    body: web::Json<TokenMetaBody>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let meta = json_value_to_string(&body.token_meta_data)?;
    mutate!(state, envelope, |tx| client.mint(
        &q.owner,
        &meta,
        q.token_hash.as_deref(),
        tx
    ))
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
    pub source: String,
    pub target: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/transfer",
    params(TransferQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/transfer")]
pub async fn cep78_transfer(
    state: web::Data<AppState>,
    query: web::Query<TransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    mutate!(state, envelope, |tx| client
        .transfer(&q.source, &q.target, &token, tx))
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
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/burn",
    params(BurnQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/burn")]
pub async fn cep78_burn(
    state: web::Data<AppState>,
    query: web::Query<BurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    mutate!(state, envelope, |tx| client.burn(&token, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct RegisterOwnerQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/register-owner",
    params(RegisterOwnerQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/register-owner")]
pub async fn cep78_register_owner(
    state: web::Data<AppState>,
    query: web::Query<RegisterOwnerQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .register_owner(&q.token_owner, tx))
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
    /// Spender account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub spender: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/approve",
    params(ApproveQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/approve")]
pub async fn cep78_approve(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    mutate!(state, envelope, |tx| client.approve(&q.spender, &token, tx))
}

#[utoipa::path(
    post,
    path = "/v1/cep78/revoke",
    params(ApproveQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/revoke")]
pub async fn cep78_revoke(
    state: web::Data<AppState>,
    query: web::Query<ApproveQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    mutate!(state, envelope, |tx| client.revoke(&q.spender, &token, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ApprovalForAllQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    /// Approve operator for all tokens.
    #[schema(example = true)]
    pub approve_all: bool,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/set-approval-for-all",
    params(ApprovalForAllQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/set-approval-for-all")]
pub async fn cep78_set_approval_for_all(
    state: web::Data<AppState>,
    query: web::Query<ApprovalForAllQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.set_approval_for_all(
        &q.operator,
        q.approve_all,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetMetaQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/set-token-metadata",
    params(SetMetaQuery),
    request_body = TokenMetaBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/set-token-metadata")]
pub async fn cep78_set_token_metadata(
    state: web::Data<AppState>,
    query: web::Query<SetMetaQuery>,
    body: web::Json<TokenMetaBody>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    let meta = json_value_to_string(&body.token_meta_data)?;
    mutate!(state, envelope, |tx| client
        .set_token_metadata(&meta, &token, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetVariablesQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Whether minting is allowed.
    #[schema(example = true)]
    pub allow_minting: Option<bool>,
    #[serde(default)]
    pub acl_whitelist: Vec<String>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/set-variables",
    params(SetVariablesQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/set-variables")]
pub async fn cep78_set_variables(
    state: web::Data<AppState>,
    query: web::Query<SetVariablesQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let args = SetVariablesArgs {
        allow_minting: q.allow_minting,
        acl_whitelist: opt_list(q.acl_whitelist),
        acl_package_mode: q.acl_package_mode,
        package_operator_mode: q.package_operator_mode,
        operator_burn_mode: q.operator_burn_mode,
    };
    mutate!(state, envelope, |tx| client.set_variables(&args, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MintSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/mint-session",
    params(MintSessionQuery),
    request_body = TokenMetaBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/mint-session")]
pub async fn cep78_mint_session(
    state: web::Data<AppState>,
    query: web::Query<MintSessionQuery>,
    body: web::Json<TokenMetaBody>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    let meta = json_value_to_string(&body.token_meta_data)?;
    mutate!(state, envelope, |tx| client.mint_session(
        &q.owner,
        &meta,
        q.token_hash.as_deref(),
        &wasm,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct TransferSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    pub source: String,
    pub target: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/transfer-session",
    params(TransferSessionQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/transfer-session")]
pub async fn cep78_transfer_session(
    state: web::Data<AppState>,
    query: web::Query<TransferSessionQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client
        .transfer_session(&q.source, &q.target, &token, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct UpdatedReceiptsQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/updated-receipts",
    params(UpdatedReceiptsQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/updated-receipts")]
pub async fn cep78_updated_receipts(
    state: web::Data<AppState>,
    query: web::Query<UpdatedReceiptsQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client.updated_receipts(&wasm, tx))
}

macro_rules! qstr {
    ($name:ident, $path:literal, $method:ident) => {
        #[utoipa::path(
            get,
            path = $path,
            params(("contract_hash" = String, Path, description = "Contract hash hex")),
            responses((status = 200, description = "Named-key / storage query")),
            tag = "CEP-78"
        )]
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
qstr!(
    cep78_allow_minting,
    "/v1/cep78/{contract_hash}/allow-minting",
    allow_minting
);
qstr!(
    cep78_operator_burn_mode,
    "/v1/cep78/{contract_hash}/operator-burn-mode",
    operator_burn_mode
);
qstr!(
    cep78_package_operator_mode,
    "/v1/cep78/{contract_hash}/package-operator-mode",
    package_operator_mode
);
qstr!(
    cep78_acl_package_mode,
    "/v1/cep78/{contract_hash}/acl-package-mode",
    acl_package_mode
);
qstr!(
    cep78_json_schema,
    "/v1/cep78/{contract_hash}/json-schema",
    json_schema
);

macro_rules! qmode {
    ($name:ident, $path:literal, $method:ident, $key:literal) => {
        #[utoipa::path(
            get,
            path = $path,
            params(("contract_hash" = String, Path, description = "Contract hash hex")),
            responses((status = 200, description = "Mode / config query (name + u8)")),
            tag = "CEP-78"
        )]
        #[get($path)]
        pub async fn $name(
            state: web::Data<AppState>,
            path: web::Path<String>,
        ) -> Result<HttpResponse, ApiError> {
            let mut client = client(&state)?;
            bind_contract(client.core_mut(), &path, None)?;
            let v = client.$method().await.map_err(ApiError::from_cep)?;
            Ok(HttpResponse::Ok().json(serde_json::json!({
                $key: format!("{:?}", v),
                concat!($key, "_u8"): v as u8,
            })))
        }
    };
}

qmode!(
    cep78_minting_mode,
    "/v1/cep78/{contract_hash}/minting-mode",
    minting_mode,
    "minting_mode"
);
qmode!(
    cep78_whitelist_mode,
    "/v1/cep78/{contract_hash}/whitelist-mode",
    whitelist_mode,
    "whitelist_mode"
);
qmode!(
    cep78_reporting_mode,
    "/v1/cep78/{contract_hash}/reporting-mode",
    reporting_mode,
    "reporting_mode"
);
qmode!(
    cep78_burn_mode,
    "/v1/cep78/{contract_hash}/burn-mode",
    burn_mode,
    "burn_mode"
);
qmode!(
    cep78_holder_mode,
    "/v1/cep78/{contract_hash}/holder-mode",
    holder_mode,
    "holder_mode"
);
qmode!(
    cep78_identifier_mode,
    "/v1/cep78/{contract_hash}/identifier-mode",
    identifier_mode,
    "identifier_mode"
);
qmode!(
    cep78_metadata_mutability,
    "/v1/cep78/{contract_hash}/metadata-mutability",
    metadata_mutability,
    "metadata_mutability"
);
qmode!(
    cep78_nft_kind,
    "/v1/cep78/{contract_hash}/nft-kind",
    nft_kind,
    "nft_kind"
);
qmode!(
    cep78_nft_metadata_kind,
    "/v1/cep78/{contract_hash}/nft-metadata-kind",
    nft_metadata_kind,
    "nft_metadata_kind"
);
qmode!(
    cep78_ownership_mode,
    "/v1/cep78/{contract_hash}/ownership-mode",
    ownership_mode,
    "ownership_mode"
);

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/events-mode",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/owner-of/{token}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token" = String, Path, description = "Token id or hash")
    ),
    responses((status = 200, description = "Owner key")),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/balance-of/{owner}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/is-approved-for-all/{owner}/{operator}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("operator" = String, Path, description = "Operator account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/get-approved/{token}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token" = String, Path, description = "Token id or hash"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/metadata/{token}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("token" = String, Path, description = "Token id or hash"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
#[get("/v1/cep78/{contract_hash}/metadata/{token}")]
pub async fn cep78_metadata(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
    query: web::Query<MetadataQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
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
    let kind = match q.kind.unwrap_or(0) {
        0 => NftMetadataKind::CEP78,
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

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MetadataQuery {
    pub kind: Option<u8>,
}

#[utoipa::path(
    get,
    path = "/v1/cep78/{contract_hash}/is-acl-whitelisted/{entity}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("entity" = String, Path, description = "Entity key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-78"
)]
#[get("/v1/cep78/{contract_hash}/is-acl-whitelisted/{entity}")]
pub async fn cep78_is_acl_whitelisted(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, entity) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "whitelisted": client
            .is_acl_whitelisted(&entity)
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct OwnerOfSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub key_name: String,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/owner-of-session",
    params(OwnerOfSessionQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/owner-of-session")]
pub async fn cep78_owner_of_session(
    state: web::Data<AppState>,
    query: web::Query<OwnerOfSessionQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client.owner_of_session(
        &token,
        &q.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BalanceOfSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    pub key_name: String,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/balance-of-session",
    params(BalanceOfSessionQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/balance-of-session")]
pub async fn cep78_balance_of_session(
    state: web::Data<AppState>,
    query: web::Query<BalanceOfSessionQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client.balance_of_session(
        &q.token_owner,
        &q.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct GetApprovedSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub key_name: String,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/get-approved-session",
    params(GetApprovedSessionQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/get-approved-session")]
pub async fn cep78_get_approved_session(
    state: web::Data<AppState>,
    query: web::Query<GetApprovedSessionQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let token = token_from(q.token_id.as_deref(), q.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client.get_approved_session(
        &token,
        &q.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct IsApprovedForAllSessionQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    pub key_name: String,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/is-approved-for-all-session",
    params(IsApprovedForAllSessionQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/is-approved-for-all-session")]
pub async fn cep78_is_approved_for_all_session(
    state: web::Data<AppState>,
    query: web::Query<IsApprovedForAllSessionQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();

    let client = bound(&state, &q.contract)?;
    let wasm = resolve_wasm(&state, &q.session_wasm)?;
    mutate!(state, envelope, |tx| client.is_approved_for_all_session(
        &q.token_owner,
        &q.operator,
        &q.key_name,
        &wasm,
        tx
    ))
}

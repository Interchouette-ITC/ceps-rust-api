//! CEP-78 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::routes::extractors::{json_value_to_string, opt_list, ContractQuery, MutateQuery};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep78::{InstallArgs, SetVariablesArgs, TokenIdentifier, UpgradeArgs};
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
#[into_params(parameter_in = Query, style = Form)]
pub struct InstallOp {
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
    #[param(inline)]
    pub ownership_mode: crate::routes::extractors::OwnershipModeParam,
    #[serde(default)]
    #[param(inline)]
    pub nft_metadata_kind: Option<crate::routes::extractors::NftMetadataKindParam>,
    #[serde(default)]
    #[param(inline)]
    pub identifier_mode: Option<crate::routes::extractors::IdentifierModeParam>,
    #[serde(default)]
    #[param(inline)]
    pub metadata_mutability: Option<crate::routes::extractors::MetadataMutabilityParam>,
    #[serde(default)]
    #[param(inline)]
    pub nft_kind: Option<crate::routes::extractors::NftKindParam>,
    #[serde(default)]
    #[param(inline)]
    pub minting_mode: Option<crate::routes::extractors::MintingModeParam>,
    #[serde(default)]
    pub allow_minting: Option<bool>,
    #[serde(default)]
    pub operator_burn_mode: Option<bool>,
    #[serde(default)]
    pub package_operator_mode: Option<bool>,
    #[serde(default)]
    #[param(inline)]
    pub whitelist_mode: Option<crate::routes::extractors::WhitelistModeParam>,
    #[serde(default)]
    #[param(inline)]
    pub holder_mode: Option<crate::routes::extractors::HolderModeParam>,
    #[serde(default)]
    pub acl_package_mode: Option<bool>,
    #[serde(default)]
    pub acl_whitelist: Vec<String>,
    #[serde(default)]
    #[param(inline)]
    pub burn_mode: Option<crate::routes::extractors::BurnModeParam>,
    #[serde(default)]
    #[param(inline)]
    pub owner_reverse_lookup_mode: Option<crate::routes::extractors::OwnerReverseLookupModeParam>,
    #[serde(default)]
    #[param(inline)]
    pub named_key_convention: Option<crate::routes::extractors::NamedKeyConventionModeParam>,
    #[serde(default)]
    pub access_key_name: Option<String>,
    #[serde(default)]
    pub hash_key_name: Option<String>,
    #[serde(default)]
    #[param(inline)]
    pub events_mode: Option<crate::routes::extractors::EventsMode78Param>,
    #[serde(default)]
    pub transfer_filter_contract: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct InstallQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    pub wasm: String,
    pub collection_name: String,
    pub collection_symbol: String,
    pub total_token_supply: u64,
    #[serde(default = "default_ownership")]
    pub ownership_mode: crate::routes::extractors::OwnershipModeParam,
    #[serde(default)]
    pub nft_metadata_kind: Option<crate::routes::extractors::NftMetadataKindParam>,
    #[serde(default)]
    pub identifier_mode: Option<crate::routes::extractors::IdentifierModeParam>,
    #[serde(default)]
    pub metadata_mutability: Option<crate::routes::extractors::MetadataMutabilityParam>,
    #[serde(default)]
    pub nft_kind: Option<crate::routes::extractors::NftKindParam>,
    #[serde(default)]
    pub minting_mode: Option<crate::routes::extractors::MintingModeParam>,
    #[serde(default)]
    pub allow_minting: Option<bool>,
    #[serde(default)]
    pub operator_burn_mode: Option<bool>,
    #[serde(default)]
    pub package_operator_mode: Option<bool>,
    #[serde(default)]
    pub whitelist_mode: Option<crate::routes::extractors::WhitelistModeParam>,
    #[serde(default)]
    pub holder_mode: Option<crate::routes::extractors::HolderModeParam>,
    #[serde(default)]
    pub acl_package_mode: Option<bool>,
    #[serde(default)]
    pub acl_whitelist: Vec<String>,
    #[serde(default)]
    pub burn_mode: Option<crate::routes::extractors::BurnModeParam>,
    #[serde(default)]
    pub owner_reverse_lookup_mode: Option<crate::routes::extractors::OwnerReverseLookupModeParam>,
    #[serde(default)]
    pub named_key_convention: Option<crate::routes::extractors::NamedKeyConventionModeParam>,
    #[serde(default)]
    pub access_key_name: Option<String>,
    #[serde(default)]
    pub hash_key_name: Option<String>,
    #[serde(default)]
    pub events_mode: Option<crate::routes::extractors::EventsMode78Param>,
    #[serde(default)]
    pub transfer_filter_contract: Option<String>,
}

crate::impl_flat_query_params!(InstallQuery, mutate, InstallOp);

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct InstallSchemaBody {
    #[serde(default)]
    pub json_schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, ToSchema, Default)]
pub struct TokenMetaBody {
    pub token_meta_data: serde_json::Value,
}

fn default_ownership() -> crate::routes::extractors::OwnershipModeParam {
    crate::routes::extractors::OwnershipModeParam::Transferable
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
    .with_ownership_mode(q.ownership_mode.to_client());
    if let Some(v) = q.nft_metadata_kind {
        args = args.with_nft_metadata_kind(v.to_client());
    }
    if let Some(v) = q.identifier_mode {
        args = args.with_identifier_mode(v.to_client());
    }
    if let Some(v) = q.metadata_mutability {
        args = args.with_metadata_mutability(v.to_client());
    }
    if let Some(v) = q.nft_kind {
        args.nft_kind = Some(v.to_client());
    }
    if let Some(v) = q.minting_mode {
        args = args.with_minting_mode(v.to_client());
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
    if let Some(v) = q.whitelist_mode {
        args.whitelist_mode = Some(v.to_client());
    }
    if let Some(v) = q.holder_mode {
        args = args.with_holder_mode(v.to_client());
    }
    if let Some(v) = q.acl_package_mode {
        args.acl_package_mode = Some(v);
    }
    if let Some(v) = opt_list(q.acl_whitelist) {
        args.acl_whitelist = Some(v);
    }
    if let Some(v) = q.burn_mode {
        args = args.with_burn_mode(v.to_client());
    }
    if let Some(v) = q.owner_reverse_lookup_mode {
        args = args.with_owner_reverse_lookup_mode(v.to_client());
    }
    if let Some(v) = q.named_key_convention {
        args.named_key_convention = Some(v.to_client());
    }
    args.access_key_name = q.access_key_name.clone();
    args.hash_key_name = q.hash_key_name.clone();
    if let Some(m) = q.events_mode {
        args.events_mode = Some(m.to_client());
    }
    args.transfer_filter_contract = q.transfer_filter_contract.clone();
    if let Some(schema) = body.and_then(|b| b.into_inner().json_schema) {
        args.json_schema = Some(json_value_to_string(&schema)?);
    }
    mutate!(state, envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct UpgradeOp {
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
    #[param(inline, example = "CES")]
    pub events_mode: Option<crate::routes::extractors::EventsMode78Param>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpgradeQuery {
    #[serde(flatten)]
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
    pub events_mode: Option<crate::routes::extractors::EventsMode78Param>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

crate::impl_flat_query_params!(UpgradeQuery, mutate, UpgradeOp);

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
    if let Some(m) = q.events_mode {
        args.events_mode = Some(m.to_client());
    }
    args.acl_package_mode = q.acl_package_mode;
    args.package_operator_mode = q.package_operator_mode;
    args.operator_burn_mode = q.operator_burn_mode;
    mutate!(state, envelope, |tx| client.upgrade(&args, &wasm, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct MintOp {
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct MintQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

crate::impl_flat_query_params!(MintQuery, mutate_contract, MintOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct TransferOp {
    pub source: String,
    pub target: String,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct TransferQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(TransferQuery, mutate_contract, TransferOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct BurnOp {
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct BurnQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

crate::impl_flat_query_params!(BurnQuery, mutate_contract, BurnOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct RegisterOwnerOp {
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterOwnerQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
}

crate::impl_flat_query_params!(RegisterOwnerQuery, mutate_contract, RegisterOwnerOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct ApproveOp {
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

#[derive(Deserialize, ToSchema)]
pub struct ApproveQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(ApproveQuery, mutate_contract, ApproveOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct ApprovalForAllOp {
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    /// Approve operator for all tokens.
    #[schema(example = true)]
    pub approve_all: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct ApprovalForAllQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    /// Approve operator for all tokens.
    #[schema(example = true)]
    pub approve_all: bool,
}

crate::impl_flat_query_params!(ApprovalForAllQuery, mutate_contract, ApprovalForAllOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct SetMetaOp {
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetMetaQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

crate::impl_flat_query_params!(SetMetaQuery, mutate_contract, SetMetaOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct SetVariablesOp {
    /// Whether minting is allowed.
    #[schema(example = true)]
    pub allow_minting: Option<bool>,
    #[serde(default)]
    pub acl_whitelist: Vec<String>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct SetVariablesQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(SetVariablesQuery, mutate_contract, SetVariablesOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct MintSessionOp {
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct MintSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub session_wasm: String,
}

crate::impl_flat_query_params!(MintSessionQuery, mutate_contract, MintSessionOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct TransferSessionOp {
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

#[derive(Deserialize, ToSchema)]
pub struct TransferSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(TransferSessionQuery, mutate_contract, TransferSessionOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct UpdatedReceiptsOp {
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatedReceiptsQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    pub session_wasm: String,
}

crate::impl_flat_query_params!(UpdatedReceiptsQuery, mutate_contract, UpdatedReceiptsOp);

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
        MetadataQuery,
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
    let kind = q
        .kind
        .unwrap_or(crate::routes::extractors::NftMetadataKindParam::CEP78)
        .to_client();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "metadata": client.metadata(&ident, kind).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct MetadataQuery {
    /// NFT metadata encoding kind for this collection.
    #[serde(default)]
    #[param(inline, example = "CEP78")]
    pub kind: Option<crate::routes::extractors::NftMetadataKindParam>,
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
#[into_params(parameter_in = Query, style = Form)]
pub struct OwnerOfSessionOp {
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub key_name: String,
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct OwnerOfSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(OwnerOfSessionQuery, mutate_contract, OwnerOfSessionOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct BalanceOfSessionOp {
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    pub key_name: String,
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct BalanceOfSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    pub contract: ContractQuery,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    pub key_name: String,
    pub session_wasm: String,
}

crate::impl_flat_query_params!(BalanceOfSessionQuery, mutate_contract, BalanceOfSessionOp);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct GetApprovedSessionOp {
    /// Token id.
    #[schema(example = "1")]
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub key_name: String,
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct GetApprovedSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(
    GetApprovedSessionQuery,
    mutate_contract,
    GetApprovedSessionOp
);

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
#[into_params(parameter_in = Query, style = Form)]
pub struct IsApprovedForAllSessionOp {
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    pub key_name: String,
    pub session_wasm: String,
}

#[derive(Deserialize, ToSchema)]
pub struct IsApprovedForAllSessionQuery {
    #[serde(flatten)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
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

crate::impl_flat_query_params!(
    IsApprovedForAllSessionQuery,
    mutate_contract,
    IsApprovedForAllSessionOp
);

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

//! CEP-78 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep78::{
    InstallArgs, NftMetadataKind, OwnershipMode, SetVariablesArgs, TokenIdentifier, UpgradeArgs,
};
use ceps_client::{CEP78Client, EventsMode78};
use serde::Deserialize;
use utoipa::ToSchema;

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

#[derive(Deserialize, ToSchema)]
pub struct ContractRef {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

fn bound(state: &AppState, contract: &ContractRef) -> Result<CEP78Client, ApiError> {
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
    #[schema(example = "cep78")]
    pub wasm: String,
    /// NFT collection name.
    #[schema(example = "MyNFTs")]
    pub collection_name: String,
    /// NFT collection symbol.
    #[schema(example = "MNFT")]
    pub collection_symbol: String,
    /// Max collection supply.
    #[schema(example = 1000)]
    pub total_token_supply: u64,
    /// Ownership mode u8 (see CEP-78).
    #[serde(default = "default_ownership")]
    #[schema(example = 2)]
    pub ownership_mode: u8,
}

fn default_ownership() -> u8 {
    2
}

#[utoipa::path(
    post,
    path = "/v1/cep78/install",
    request_body(
        content = InstallBody,
        example = json!({
    "submit": "put",
    "wait": "processed",
    "signer": {"public_key": "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
    "payment_amount": "2500000000",
    "wasm": "cep78",
    "collection_name": "MyNFTs",
    "collection_symbol": "MNFT",
    "total_token_supply": 1000,
    "ownership_mode": 2
}),
    ),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct UpgradeBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep78")]
    pub wasm: String,
    /// NFT collection name.
    #[schema(example = "MyNFTs")]
    pub collection_name: String,
    /// Max collection supply.
    #[schema(example = 1000)]
    pub total_token_supply: Option<u64>,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: Option<u8>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/upgrade",
    request_body = UpgradeBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct MintBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token metadata JSON string.
    #[schema(example = "{}")]
    pub token_meta_data: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/mint",
    request_body = MintBody,
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct TransferBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = TransferBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/burn",
    request_body = BurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct RegisterOwnerBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/register-owner",
    request_body = RegisterOwnerBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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
    pub token_id: Option<String>,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/approve",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[utoipa::path(
    post,
    path = "/v1/cep78/revoke",
    request_body = ApproveBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct ApprovalForAllBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = ApprovalForAllBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct SetMetaBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token metadata JSON string.
    #[schema(example = "{}")]
    pub token_meta_data: String,
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
    request_body = SetMetaBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct SetVariablesBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Whether minting is allowed.
    #[schema(example = true)]
    pub allow_minting: Option<bool>,
    pub acl_whitelist: Option<Vec<String>>,
    pub acl_package_mode: Option<bool>,
    pub package_operator_mode: Option<bool>,
    pub operator_burn_mode: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/set-variables",
    request_body = SetVariablesBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
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

#[derive(Deserialize, ToSchema)]
pub struct MintSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token metadata JSON string.
    #[schema(example = "{}")]
    pub token_meta_data: String,
    /// Token hash hex when identifier mode is Hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_hash: Option<String>,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/mint-session",
    request_body = MintSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/mint-session")]
pub async fn cep78_mint_session(
    state: web::Data<AppState>,
    body: web::Json<MintSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client.mint_session(
        &body.owner,
        &body.token_meta_data,
        body.token_hash.as_deref(),
        &wasm,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct TransferSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = TransferSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/transfer-session")]
pub async fn cep78_transfer_session(
    state: web::Data<AppState>,
    body: web::Json<TransferSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client.transfer_session(
        &body.source,
        &body.target,
        &token,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct UpdatedReceiptsBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/updated-receipts",
    request_body = UpdatedReceiptsBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/updated-receipts")]
pub async fn cep78_updated_receipts(
    state: web::Data<AppState>,
    body: web::Json<UpdatedReceiptsBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client
        .updated_receipts(&wasm, tx))
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

#[derive(Deserialize, ToSchema)]
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

#[derive(Deserialize, ToSchema)]
pub struct OwnerOfSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = OwnerOfSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/owner-of-session")]
pub async fn cep78_owner_of_session(
    state: web::Data<AppState>,
    body: web::Json<OwnerOfSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client.owner_of_session(
        &token,
        &body.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct BalanceOfSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token owner key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub token_owner: String,
    pub key_name: String,
    pub session_wasm: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep78/balance-of-session",
    request_body = BalanceOfSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/balance-of-session")]
pub async fn cep78_balance_of_session(
    state: web::Data<AppState>,
    body: web::Json<BalanceOfSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client.balance_of_session(
        &body.token_owner,
        &body.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct GetApprovedSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = GetApprovedSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/get-approved-session")]
pub async fn cep78_get_approved_session(
    state: web::Data<AppState>,
    body: web::Json<GetApprovedSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let token = token_from(body.token_id.as_deref(), body.token_hash.as_deref())?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client.get_approved_session(
        &token,
        &body.key_name,
        &wasm,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct IsApprovedForAllSessionBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = IsApprovedForAllSessionBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-78"
)]
#[post("/v1/cep78/is-approved-for-all-session")]
pub async fn cep78_is_approved_for_all_session(
    state: web::Data<AppState>,
    body: web::Json<IsApprovedForAllSessionBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let wasm = resolve_wasm(&state, &body.session_wasm)?;
    mutate!(state, body.envelope, |tx| client
        .is_approved_for_all_session(
            &body.token_owner,
            &body.operator,
            &body.key_name,
            &wasm,
            tx
        ))
}

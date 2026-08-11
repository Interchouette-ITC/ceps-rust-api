//! CEP-85 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, optional_hex_bytes, resolve_wasm};
use crate::routes::extractors::{opt_list, parse_events_mode, ContractQuery, MutateQuery};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep85::{ChangeSecurityArgs, InstallArgs, UpgradeArgs};
use ceps_client::CEP85Client;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

fn client(state: &AppState) -> Result<CEP85Client, ApiError> {
    CEP85Client::new(
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

fn bound(state: &AppState, contract: &ContractQuery) -> Result<CEP85Client, ApiError> {
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
    /// Canonical wasm id or path under configured wasm roots.
    #[schema(example = "cep85")]
    pub wasm: String,
    /// Collection name used for install named-key lookup.
    #[schema(example = "MyMulti")]
    pub name: String,
    /// Collection or token URI template.
    #[schema(example = "https://example.com/meta/{id}.json")]
    pub uri: String,
    /// Events mode name or u8.
    #[serde(default)]
    #[param(example = "CES")]
    pub events_mode: Option<String>,
    /// Enable burn entrypoints.
    #[schema(example = true)]
    pub enable_burn: Option<bool>,
    /// Admin public keys or account-hashes.
    #[serde(default)]
    pub admin_list: Vec<String>,
    /// Minter public keys or account-hashes.
    #[serde(default)]
    pub minter_list: Vec<String>,
    /// Burner keys.
    #[serde(default)]
    pub burner_list: Vec<String>,
    /// Meta-admin keys.
    #[serde(default)]
    pub meta_list: Vec<String>,
    /// Optional transfer-filter contract hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub transfer_filter_contract: Option<String>,
    /// Optional transfer-filter entrypoint name.
    #[schema(example = "can_transfer")]
    pub transfer_filter_method: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/install",
    params(InstallQuery),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/install")]
pub async fn cep85_install(
    state: web::Data<AppState>,
    query: web::Query<InstallQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = InstallArgs::new(&q.name, &q.uri);
    if let Some(ref m) = q.events_mode {
        args = args.with_events_mode(parse_events_mode(m)?);
    }
    if let Some(v) = q.enable_burn {
        args = args.with_enable_burn(v);
    }
    if let Some(v) = opt_list(q.admin_list.clone()) {
        args = args.with_admin_list(v);
    }
    if let Some(v) = opt_list(q.minter_list.clone()) {
        args = args.with_minter_list(v);
    }
    if let Some(v) = opt_list(q.burner_list.clone()) {
        args = args.with_burner_list(v);
    }
    if let Some(v) = opt_list(q.meta_list.clone()) {
        args = args.with_meta_list(v);
    }
    match (
        q.transfer_filter_contract.as_deref(),
        q.transfer_filter_method.as_deref(),
    ) {
        (Some(c), Some(m)) => args = args.with_transfer_filter(c, m),
        (None, None) => {}
        _ => {
            return Err(ApiError::BadRequest(
                "transfer_filter_contract and transfer_filter_method must both be set".into(),
            ));
        }
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
    #[schema(example = "cep85")]
    pub wasm: String,
    /// Collection name used for install named-key lookup.
    #[schema(example = "MyToken")]
    pub name: String,
    /// Optional transfer-filter contract hash.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub transfer_filter_contract: Option<String>,
    /// Optional transfer-filter entrypoint name.
    #[schema(example = "can_transfer")]
    pub transfer_filter_method: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/upgrade",
    params(UpgradeQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/upgrade")]
pub async fn cep85_upgrade(
    state: web::Data<AppState>,
    query: web::Query<UpgradeQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &q.wasm)?;
    let mut args = UpgradeArgs::new(&q.name);
    match (
        q.transfer_filter_contract.as_deref(),
        q.transfer_filter_method.as_deref(),
    ) {
        (Some(c), Some(m)) => args = args.with_transfer_filter(c, m),
        (None, None) => {}
        _ => {
            return Err(ApiError::BadRequest(
                "transfer_filter_contract and transfer_filter_method must both be set".into(),
            ));
        }
    }
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
    /// Recipient account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub recipient: String,
    /// Token id.
    #[schema(example = "1")]
    pub id: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
    /// Collection or token URI template.
    #[schema(example = "https://example.com/meta/{id}.json")]
    pub uri: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/mint",
    params(MintQuery),
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/mint")]
pub async fn cep85_mint(
    state: web::Data<AppState>,
    query: web::Query<MintQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.mint(
        &q.recipient,
        &q.id,
        &q.amount,
        q.uri.as_deref(),
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BatchMintQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Recipient account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub recipient: String,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
    /// Batch amounts.
    pub amounts: Vec<String>,
    /// Collection or token URI template.
    #[schema(example = "https://example.com/meta/{id}.json")]
    pub uri: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/batch-mint",
    params(BatchMintQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-mint")]
pub async fn cep85_batch_mint(
    state: web::Data<AppState>,
    query: web::Query<BatchMintQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = q.amounts.iter().map(String::as_str).collect();
    mutate!(state, envelope, |tx| client.batch_mint(
        &q.recipient,
        &ids,
        &amounts,
        q.uri.as_deref(),
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
    /// Sender key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub from: String,
    /// Recipient key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub to: String,
    /// Token id.
    #[schema(example = "1")]
    pub id: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
    /// Optional calldata as hex string.
    #[schema(example = "0x")]
    pub data: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/transfer",
    params(TransferQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/transfer")]
pub async fn cep85_transfer(
    state: web::Data<AppState>,
    query: web::Query<TransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let data = optional_hex_bytes(q.data.as_deref())?;
    mutate!(state, envelope, |tx| client.transfer(
        &q.from,
        &q.to,
        &q.id,
        &q.amount,
        data.as_deref(),
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BatchTransferQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Sender key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub from: String,
    /// Recipient key.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub to: String,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
    /// Batch amounts.
    pub amounts: Vec<String>,
    /// Optional calldata as hex string.
    #[schema(example = "0x")]
    pub data: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/batch-transfer",
    params(BatchTransferQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-transfer")]
pub async fn cep85_batch_transfer(
    state: web::Data<AppState>,
    query: web::Query<BatchTransferQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = q.amounts.iter().map(String::as_str).collect();
    let data = optional_hex_bytes(q.data.as_deref())?;
    mutate!(state, envelope, |tx| client.batch_transfer(
        &q.from,
        &q.to,
        &ids,
        &amounts,
        data.as_deref(),
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
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token id.
    #[schema(example = "1")]
    pub id: String,
    /// Amount as decimal string.
    #[schema(example = "1000000000")]
    pub amount: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/burn",
    params(BurnQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/burn")]
pub async fn cep85_burn(
    state: web::Data<AppState>,
    query: web::Query<BurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client
        .burn(&q.owner, &q.id, &q.amount, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BatchBurnQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Owner account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub owner: String,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
    /// Batch amounts.
    pub amounts: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/batch-burn",
    params(BatchBurnQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-burn")]
pub async fn cep85_batch_burn(
    state: web::Data<AppState>,
    query: web::Query<BatchBurnQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = q.amounts.iter().map(String::as_str).collect();
    mutate!(state, envelope, |tx| client
        .batch_burn(&q.owner, &ids, &amounts, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ApprovalQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    pub approved: bool,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-approval-for-all",
    params(ApprovalQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-approval-for-all")]
pub async fn cep85_set_approval_for_all(
    state: web::Data<AppState>,
    query: web::Query<ApprovalQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.set_approval_for_all(
        &q.operator,
        q.approved,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetUriQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Collection or token URI template.
    #[schema(example = "https://example.com/meta/{id}.json")]
    pub uri: String,
    /// Token id.
    #[schema(example = "1")]
    pub id: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-uri",
    params(SetUriQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-uri")]
pub async fn cep85_set_uri(
    state: web::Data<AppState>,
    query: web::Query<SetUriQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.set_uri(
        &q.uri,
        q.id.as_deref(),
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetTotalSupplyQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token id.
    #[schema(example = "1")]
    pub id: String,
    /// Initial total supply as decimal string (base units).
    #[schema(example = "1000000000000")]
    pub total_supply: String,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-total-supply-of",
    params(SetTotalSupplyQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-total-supply-of")]
pub async fn cep85_set_total_supply_of(
    state: web::Data<AppState>,
    query: web::Query<SetTotalSupplyQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    mutate!(state, envelope, |tx| client.set_total_supply_of(
        &q.id,
        &q.total_supply,
        tx
    ))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetTotalSupplyBatchQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
    pub total_supplies: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-total-supply-of-batch",
    params(SetTotalSupplyBatchQuery),
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-total-supply-of-batch")]
pub async fn cep85_set_total_supply_of_batch(
    state: web::Data<AppState>,
    query: web::Query<SetTotalSupplyBatchQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    let supplies: Vec<&str> = q.total_supplies.iter().map(String::as_str).collect();
    mutate!(state, envelope, |tx| client
        .set_total_supply_of_batch(&ids, &supplies, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ChangeSecurityQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Admin public keys or account-hashes.
    #[serde(default)]
    pub admin_list: Vec<String>,
    /// Minter public keys or account-hashes.
    #[serde(default)]
    pub minter_list: Vec<String>,
    /// Burner keys.
    #[serde(default)]
    pub burner_list: Vec<String>,
    /// Meta-admin keys.
    #[serde(default)]
    pub meta_list: Vec<String>,
    /// Keys to clear from security lists.
    #[serde(default)]
    pub none_list: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/change-security",
    params(ChangeSecurityQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/change-security")]
pub async fn cep85_change_security(
    state: web::Data<AppState>,
    query: web::Query<ChangeSecurityQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let args = ChangeSecurityArgs {
        admin_list: opt_list(q.admin_list),
        minter_list: opt_list(q.minter_list),
        burner_list: opt_list(q.burner_list),
        meta_list: opt_list(q.meta_list),
        none_list: opt_list(q.none_list),
    };
    mutate!(state, envelope, |tx| client.change_security(&args, tx))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct SetModalitiesQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub mutate: MutateQuery,
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Enable burn entrypoints.
    #[schema(example = true)]
    pub enable_burn: Option<bool>,
    /// Events mode name or u8.
    #[serde(default)]
    #[param(example = "CES")]
    pub events_mode: Option<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-modalities",
    params(SetModalitiesQuery),
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-modalities")]
pub async fn cep85_set_modalities(
    state: web::Data<AppState>,
    query: web::Query<SetModalitiesQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let envelope = q.mutate.envelope();
    let client = bound(&state, &q.contract)?;
    let mode = match q.events_mode.as_deref() {
        Some(v) => Some(parse_events_mode(v)?),
        None => None,
    };
    mutate!(state, envelope, |tx| client.set_modalities(
        q.enable_burn,
        mode,
        tx
    ))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/collection-name",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/collection-name")]
pub async fn cep85_collection_name(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "collection_name": client.collection_name().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/collection-uri",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/collection-uri")]
pub async fn cep85_collection_uri(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "collection_uri": client.collection_uri().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/balance-of/{owner}/{id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner key"),
        ("id" = String, Path, description = "Token id")
    ),
    responses((status = 200, description = "Balance string")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/balance-of/{owner}/{id}")]
pub async fn cep85_balance_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, owner, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balance": client.balance_of(&owner, &id).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/supply-of/{id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/supply-of/{id}")]
pub async fn cep85_supply_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "supply": client.supply_of(&id).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/total-supply-of/{id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/total-supply-of/{id}")]
pub async fn cep85_total_supply_of(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_supply": client.total_supply_of(&id).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/uri",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/uri")]
pub async fn cep85_uri(
    state: web::Data<AppState>,
    path: web::Path<String>,
    query: web::Query<UriQuery>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "uri": client.uri(query.id.as_deref()).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct UriQuery {
    /// Token id.
    #[schema(example = "1")]
    pub id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/is-non-fungible/{id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/is-non-fungible/{id}")]
pub async fn cep85_is_non_fungible(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "is_non_fungible": client.is_non_fungible(&id).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/is-approved-for-all/{owner}/{operator}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("owner" = String, Path, description = "Owner account or key"),
        ("operator" = String, Path, description = "Operator account or key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/is-approved-for-all/{owner}/{operator}")]
pub async fn cep85_is_approved_for_all(
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

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BatchAccountsIdsQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Account keys for batch queries.
    pub accounts: Vec<String>,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/balance-of-batch",
    params(BatchAccountsIdsQuery),
    responses((status = 200, description = "Batch balances")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/balance-of-batch")]
pub async fn cep85_balance_of_batch(
    state: web::Data<AppState>,
    query: web::Query<BatchAccountsIdsQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let client = bound(&state, &q.contract)?;
    let accounts: Vec<&str> = q.accounts.iter().map(String::as_str).collect();
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balances": client.balance_of_batch(&accounts, &ids).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct BatchIdsQuery {
    #[serde(flatten)]
    #[param(inline)]
    pub contract: ContractQuery,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/supply-of-batch",
    params(BatchIdsQuery),
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/supply-of-batch")]
pub async fn cep85_supply_of_batch(
    state: web::Data<AppState>,
    query: web::Query<BatchIdsQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "supplies": client.supply_of_batch(&ids).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    post,
    path = "/v1/cep85/total-supply-of-batch",
    params(BatchIdsQuery),
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/total-supply-of-batch")]
pub async fn cep85_total_supply_of_batch(
    state: web::Data<AppState>,
    query: web::Query<BatchIdsQuery>,
) -> Result<HttpResponse, ApiError> {
    let q = query.into_inner();
    let client = bound(&state, &q.contract)?;
    let ids: Vec<&str> = q.ids.iter().map(String::as_str).collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_supplies": client
            .total_supply_of_batch(&ids)
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/total-fungible-supply/{id}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("id" = String, Path, description = "Token id"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/total-fungible-supply/{id}")]
pub async fn cep85_total_fungible_supply(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, id) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_fungible_supply": client
            .total_fungible_supply(&id)
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/enable-burn",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/enable-burn")]
pub async fn cep85_enable_burn(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "enable_burn": client.enable_burn().await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/events-mode",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/events-mode")]
pub async fn cep85_events_mode(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    let mode = client.events_mode().await.map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "events_mode": mode.as_str(),
        "events_mode_u8": u8::from(mode),
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/number-of-minted-tokens",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/number-of-minted-tokens")]
pub async fn cep85_number_of_minted_tokens(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "number_of_minted_tokens": client
            .number_of_minted_tokens()
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/transfer-filter-contract",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/transfer-filter-contract")]
pub async fn cep85_transfer_filter_contract(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transfer_filter_contract": client
            .transfer_filter_contract()
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/transfer-filter-method",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/transfer-filter-method")]
pub async fn cep85_transfer_filter_method(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &path, None)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "transfer_filter_method": client
            .transfer_filter_method()
            .await
            .map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    get,
    path = "/v1/cep85/{contract_hash}/security-badge/{entity}",
    params(
        ("contract_hash" = String, Path, description = "Contract hash hex"),
        ("entity" = String, Path, description = "Entity key"),
    ),
    responses((status = 200, description = "Query result")),
    tag = "CEP-85"
)]
#[get("/v1/cep85/{contract_hash}/security-badge/{entity}")]
pub async fn cep85_security_badge(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, ApiError> {
    let (contract_hash, entity) = path.into_inner();
    let mut client = client(&state)?;
    bind_contract(client.core_mut(), &contract_hash, None)?;
    let badge = client
        .security_badge(&entity)
        .await
        .map_err(ApiError::from_cep)?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "badge": badge.map(|b| b.as_str()),
        "badge_u8": badge.map(|b| b as u8),
    })))
}

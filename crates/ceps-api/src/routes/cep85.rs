//! CEP-85 HTTP routes (MCP-aligned surface).

use crate::error::ApiError;
use crate::routes::common::{bind_contract, cep_core, optional_hex_bytes, resolve_wasm};
use crate::state::AppState;
use crate::tx::{build_transaction_params, finalize_call, MutateEnvelope};
use actix_web::{get, post, web, HttpResponse};
use ceps_client::cep85::{ChangeSecurityArgs, InstallArgs, UpgradeArgs};
use ceps_client::{CEP85Client, EventsMode};
use serde::Deserialize;
use utoipa::ToSchema;

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

#[derive(Deserialize, ToSchema)]
pub struct ContractRef {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[schema(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

fn bound(state: &AppState, contract: &ContractRef) -> Result<CEP85Client, ApiError> {
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
    #[schema(example = "cep85")]
    pub wasm: String,
    /// Collection name used for install named-key lookup.
    #[schema(example = "MyMulti")]
    pub name: String,
    /// Collection or token URI template.
    #[schema(example = "https://example.com/meta/{id}.json")]
    pub uri: String,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: Option<u8>,
    /// Enable burn entrypoints.
    #[schema(example = true)]
    pub enable_burn: Option<bool>,
    /// Admin public keys or account-hashes.
    pub admin_list: Option<Vec<String>>,
    /// Minter public keys or account-hashes.
    pub minter_list: Option<Vec<String>>,
    /// Burner keys.
    pub burner_list: Option<Vec<String>>,
    /// Meta-admin keys.
    pub meta_list: Option<Vec<String>>,
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
    request_body(
        content = InstallBody,
        example = json!({
    "submit": "put",
    "wait": "processed",
    "signer": {"public_key": "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"},
    "payment_amount": "2500000000",
    "wasm": "cep85",
    "name": "MyMulti",
    "uri": "https://example.com/meta/{id}.json",
    "events_mode": 1,
    "enable_burn": true
}),
    ),
    responses((status = 200, description = "Install pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/install")]
pub async fn cep85_install(
    state: web::Data<AppState>,
    body: web::Json<InstallBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let mut args = InstallArgs::new(&body.name, &body.uri);
    if let Some(m) = body.events_mode {
        args = args.with_events_mode(
            EventsMode::from_u8(m)
                .ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {m}")))?,
        );
    }
    if let Some(v) = body.enable_burn {
        args = args.with_enable_burn(v);
    }
    if let Some(a) = body.admin_list.clone() {
        args = args.with_admin_list(a);
    }
    if let Some(m) = body.minter_list.clone() {
        args = args.with_minter_list(m);
    }
    if let Some(b) = body.burner_list.clone() {
        args = args.with_burner_list(b);
    }
    if let Some(m) = body.meta_list.clone() {
        args = args.with_meta_list(m);
    }
    match (
        body.transfer_filter_contract.as_deref(),
        body.transfer_filter_method.as_deref(),
    ) {
        (Some(c), Some(m)) => args = args.with_transfer_filter(c, m),
        (None, None) => {}
        _ => {
            return Err(ApiError::BadRequest(
                "transfer_filter_contract and transfer_filter_method must both be set".into(),
            ));
        }
    }
    mutate!(state, body.envelope, |tx| client.install(&args, &wasm, tx))
}

#[derive(Deserialize, ToSchema)]
pub struct UpgradeBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
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
    request_body = UpgradeBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/upgrade")]
pub async fn cep85_upgrade(
    state: web::Data<AppState>,
    body: web::Json<UpgradeBody>,
) -> Result<HttpResponse, ApiError> {
    let client = client(&state)?;
    let wasm = resolve_wasm(&state, &body.wasm)?;
    let mut args = UpgradeArgs::new(&body.name);
    match (
        body.transfer_filter_contract.as_deref(),
        body.transfer_filter_method.as_deref(),
    ) {
        (Some(c), Some(m)) => args = args.with_transfer_filter(c, m),
        (None, None) => {}
        _ => {
            return Err(ApiError::BadRequest(
                "transfer_filter_contract and transfer_filter_method must both be set".into(),
            ));
        }
    }
    mutate!(state, body.envelope, |tx| client.upgrade(&args, &wasm, tx))
}

#[derive(Deserialize, ToSchema)]
pub struct MintBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = MintBody,
    responses((status = 200, description = "Mint pipeline outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/mint")]
pub async fn cep85_mint(
    state: web::Data<AppState>,
    body: web::Json<MintBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.mint(
        &body.recipient,
        &body.id,
        &body.amount,
        body.uri.as_deref(),
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct BatchMintBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = BatchMintBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-mint")]
pub async fn cep85_batch_mint(
    state: web::Data<AppState>,
    body: web::Json<BatchMintBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = body.amounts.iter().map(String::as_str).collect();
    mutate!(state, body.envelope, |tx| client.batch_mint(
        &body.recipient,
        &ids,
        &amounts,
        body.uri.as_deref(),
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
    request_body = TransferBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/transfer")]
pub async fn cep85_transfer(
    state: web::Data<AppState>,
    body: web::Json<TransferBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let data = optional_hex_bytes(body.data.as_deref())?;
    mutate!(state, body.envelope, |tx| client.transfer(
        &body.from,
        &body.to,
        &body.id,
        &body.amount,
        data.as_deref(),
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct BatchTransferBody {
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
    request_body = BatchTransferBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-transfer")]
pub async fn cep85_batch_transfer(
    state: web::Data<AppState>,
    body: web::Json<BatchTransferBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = body.amounts.iter().map(String::as_str).collect();
    let data = optional_hex_bytes(body.data.as_deref())?;
    mutate!(state, body.envelope, |tx| client.batch_transfer(
        &body.from,
        &body.to,
        &ids,
        &amounts,
        data.as_deref(),
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
    request_body = BurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/burn")]
pub async fn cep85_burn(
    state: web::Data<AppState>,
    body: web::Json<BurnBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.burn(
        &body.owner,
        &body.id,
        &body.amount,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct BatchBurnBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = BatchBurnBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/batch-burn")]
pub async fn cep85_batch_burn(
    state: web::Data<AppState>,
    body: web::Json<BatchBurnBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    let amounts: Vec<&str> = body.amounts.iter().map(String::as_str).collect();
    mutate!(state, body.envelope, |tx| client.batch_burn(
        &body.owner,
        &ids,
        &amounts,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct ApprovalBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Operator account public key or account-hash.
    #[schema(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub operator: String,
    pub approved: bool,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-approval-for-all",
    request_body = ApprovalBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-approval-for-all")]
pub async fn cep85_set_approval_for_all(
    state: web::Data<AppState>,
    body: web::Json<ApprovalBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.set_approval_for_all(
        &body.operator,
        body.approved,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct SetUriBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = SetUriBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-uri")]
pub async fn cep85_set_uri(
    state: web::Data<AppState>,
    body: web::Json<SetUriBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.set_uri(
        &body.uri,
        body.id.as_deref(),
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct SetTotalSupplyBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
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
    request_body = SetTotalSupplyBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-total-supply-of")]
pub async fn cep85_set_total_supply_of(
    state: web::Data<AppState>,
    body: web::Json<SetTotalSupplyBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    mutate!(state, body.envelope, |tx| client.set_total_supply_of(
        &body.id,
        &body.total_supply,
        tx
    ))
}

#[derive(Deserialize, ToSchema)]
pub struct SetTotalSupplyBatchBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
    pub total_supplies: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-total-supply-of-batch",
    request_body = SetTotalSupplyBatchBody,
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-total-supply-of-batch")]
pub async fn cep85_set_total_supply_of_batch(
    state: web::Data<AppState>,
    body: web::Json<SetTotalSupplyBatchBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    let supplies: Vec<&str> = body.total_supplies.iter().map(String::as_str).collect();
    mutate!(state, body.envelope, |tx| client
        .set_total_supply_of_batch(&ids, &supplies, tx))
}

#[derive(Deserialize, ToSchema)]
pub struct ChangeSecurityBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Admin public keys or account-hashes.
    pub admin_list: Option<Vec<String>>,
    /// Minter public keys or account-hashes.
    pub minter_list: Option<Vec<String>>,
    /// Burner keys.
    pub burner_list: Option<Vec<String>>,
    /// Meta-admin keys.
    pub meta_list: Option<Vec<String>>,
    /// Keys to clear from security lists.
    pub none_list: Option<Vec<String>>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/change-security",
    request_body = ChangeSecurityBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/change-security")]
pub async fn cep85_change_security(
    state: web::Data<AppState>,
    body: web::Json<ChangeSecurityBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let args = ChangeSecurityArgs {
        admin_list: body.admin_list.clone(),
        minter_list: body.minter_list.clone(),
        burner_list: body.burner_list.clone(),
        meta_list: body.meta_list.clone(),
        none_list: body.none_list.clone(),
    };
    mutate!(state, body.envelope, |tx| client.change_security(&args, tx))
}

#[derive(Deserialize, ToSchema)]
pub struct SetModalitiesBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub envelope: MutateEnvelope,
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Enable burn entrypoints.
    #[schema(example = true)]
    pub enable_burn: Option<bool>,
    /// Events mode discriminant (0=NoEvents, 1=CES, ...).
    #[schema(example = 1)]
    pub events_mode: Option<u8>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/set-modalities",
    request_body = SetModalitiesBody,
    responses((status = 200, description = "Pipeline or query outcome", body = crate::tx::PipelineOutcome)),
    tag = "CEP-85"
)]
#[post("/v1/cep85/set-modalities")]
pub async fn cep85_set_modalities(
    state: web::Data<AppState>,
    body: web::Json<SetModalitiesBody>,
) -> Result<HttpResponse, ApiError> {
    build_transaction_params(&state, &body.envelope)?;
    let client = bound(&state, &body.contract)?;
    let mode = match body.events_mode {
        Some(v) => Some(
            EventsMode::from_u8(v)
                .ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {v}")))?,
        ),
        None => None,
    };
    mutate!(state, body.envelope, |tx| client.set_modalities(
        body.enable_burn,
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

#[derive(Deserialize, ToSchema)]
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

#[derive(Deserialize, ToSchema)]
pub struct BatchAccountsIdsBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Account keys for batch queries.
    pub accounts: Vec<String>,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/balance-of-batch",
    request_body = BatchAccountsIdsBody,
    responses((status = 200, description = "Batch balances")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/balance-of-batch")]
pub async fn cep85_balance_of_batch(
    state: web::Data<AppState>,
    body: web::Json<BatchAccountsIdsBody>,
) -> Result<HttpResponse, ApiError> {
    let client = bound(&state, &body.contract)?;
    let accounts: Vec<&str> = body.accounts.iter().map(String::as_str).collect();
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "balances": client.balance_of_batch(&accounts, &ids).await.map_err(ApiError::from_cep)?
    })))
}

#[derive(Deserialize, ToSchema)]
pub struct BatchIdsBody {
    #[serde(flatten)]
    #[schema(inline)]
    pub contract: ContractRef,
    /// Token ids for batch queries.
    pub ids: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/v1/cep85/supply-of-batch",
    request_body = BatchIdsBody,
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/supply-of-batch")]
pub async fn cep85_supply_of_batch(
    state: web::Data<AppState>,
    body: web::Json<BatchIdsBody>,
) -> Result<HttpResponse, ApiError> {
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "supplies": client.supply_of_batch(&ids).await.map_err(ApiError::from_cep)?
    })))
}

#[utoipa::path(
    post,
    path = "/v1/cep85/total-supply-of-batch",
    request_body = BatchIdsBody,
    responses((status = 200, description = "Batch query result")),
    tag = "CEP-85"
)]
#[post("/v1/cep85/total-supply-of-batch")]
pub async fn cep85_total_supply_of_batch(
    state: web::Data<AppState>,
    body: web::Json<BatchIdsBody>,
) -> Result<HttpResponse, ApiError> {
    let client = bound(&state, &body.contract)?;
    let ids: Vec<&str> = body.ids.iter().map(String::as_str).collect();
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

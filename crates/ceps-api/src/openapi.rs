//! `OpenAPI` document aggregation.

use crate::features::CompiledFeatures;
use crate::routes::{health::HealthResult, hello::HelloResult};
use crate::tx::PipelineOutcome;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::hello::hello_handler,
        crate::routes::health::health_handler,
        crate::routes::chain::chain_balance,
        crate::routes::chain::chain_account,
        crate::routes::chain::chain_transaction,
        crate::routes::instances::list_instances,
        crate::routes::instances::register_instance,
        crate::routes::wasm::list_wasm,
    ),
    components(schemas(
        HelloResult,
        HealthResult,
        CompiledFeatures,
        PipelineOutcome,
        crate::registry::InstanceRecord,
        crate::routes::wasm::WasmEntry,
    )),
    tags(
        (name = "Health", description = "Liveness and hello"),
        (name = "KMS", description = "Key management proxy (kms-secp256k1-api)"),
        (name = "Keys", description = "Local custody keys"),
        (name = "Chain", description = "Native CSPR and transaction queries"),
        (name = "Instances", description = "CEP contract instance registry and wasm"),
        (name = "CEP-18", description = "Fungible token"),
        (name = "CEP-78", description = "NFT"),
        (name = "CEP-85", description = "Multi-token"),
        (name = "CEP-95", description = "Odra NFT"),
    ),
    info(
        title = "ceps-rust-api",
        description = "Casper CEP HTTP API. Socle queries and CEP routes; optional local or KMS signing (no PEM in HTTP bodies). Uses Transactions only.",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;

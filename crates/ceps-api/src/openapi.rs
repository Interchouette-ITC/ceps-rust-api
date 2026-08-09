//! `OpenAPI` document aggregation.

use crate::routes::{health::HealthResult, hello::HelloResult};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::hello::hello_handler,
        crate::routes::health::health_handler,
    ),
    components(schemas(HelloResult, HealthResult)),
    tags(
        (name = "Health", description = "Liveness and hello"),
        (name = "KMS", description = "Key management (via kms-secp256k1-api)"),
        (name = "Chain", description = "Native CSPR and account queries"),
        (name = "Instances", description = "CEP contract instance registry"),
        (name = "CEP-18", description = "Fungible token"),
        (name = "CEP-78", description = "NFT"),
        (name = "CEP-85", description = "Multi-token"),
        (name = "CEP-95", description = "Odra NFT"),
    ),
    info(
        title = "ceps-rust-api",
        description = "Casper CEP HTTP API. Socle queries and CEP routes; optional local or KMS signing (no PEM in HTTP bodies).",
        version = "0.1.0"
    )
)]
pub struct ApiDoc;

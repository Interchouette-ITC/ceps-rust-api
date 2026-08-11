//! OpenAPI document aggregation.

use crate::features::CompiledFeatures;
use crate::routes::{health::HealthResult, hello::HelloResult};
use crate::tx::PipelineOutcome;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::hello::hello_handler,
        crate::routes::health::health_handler,
    ),
    components(schemas(
        HelloResult,
        HealthResult,
        CompiledFeatures,
        PipelineOutcome,
        crate::tx::MutateEnvelope,
        crate::tx::SubmitMode,
        crate::tx::WaitMode,
        crate::tx::SignerRef,
    )),
    tags(
        (name = "Health", description = "Liveness and hello"),
        (name = "Chain", description = "Put already-signed Transaction JSON"),
        (name = "CEP-18", description = "Fungible token"),
        (name = "CEP-78", description = "NFT"),
        (name = "CEP-85", description = "Multi-token"),
        (name = "CEP-95", description = "Odra NFT"),
    ),
    info(
        title = "ceps-rust-api",
        description = "Casper CEP HTTP API. CEP routes and optional local or KMS put signing (no PEM in HTTP bodies). Uses Transactions only.",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;

#[cfg(feature = "chain-put")]
#[derive(OpenApi)]
#[openapi(paths(crate::routes::chain::chain_put_transaction))]
struct ApiDocChainPut;

#[cfg(feature = "cep18")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep18::cep18_install,
        crate::routes::cep18::cep18_transfer,
        crate::routes::cep18::cep18_balance_of,
    ),
    components(schemas(
        crate::routes::cep18::InstallBody,
        crate::routes::cep18::TransferBody,
        crate::routes::cep18::ContractQuery,
    ))
)]
struct ApiDocCep18;

#[cfg(feature = "cep78")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep78::cep78_install,
        crate::routes::cep78::cep78_mint,
        crate::routes::cep78::cep78_owner_of,
    ),
    components(schemas(
        crate::routes::cep78::InstallBody,
        crate::routes::cep78::MintBody,
        crate::routes::cep78::ContractRef,
    ))
)]
struct ApiDocCep78;

#[cfg(feature = "cep85")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep85::cep85_install,
        crate::routes::cep85::cep85_mint,
        crate::routes::cep85::cep85_balance_of,
    ),
    components(schemas(
        crate::routes::cep85::InstallBody,
        crate::routes::cep85::MintBody,
        crate::routes::cep85::ContractRef,
    ))
)]
struct ApiDocCep85;

#[cfg(feature = "cep95")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep95::cep95_install,
        crate::routes::cep95::cep95_transfer_from,
        crate::routes::cep95::cep95_bind_odra_install,
        crate::routes::cep95::cep95_owner_of,
    ),
    components(schemas(
        crate::routes::cep95::InstallBody,
        crate::routes::cep95::TransferBody,
        crate::routes::cep95::BindOdraInstallBody,
        crate::routes::cep95::ContractRef,
    ))
)]
struct ApiDocCep95;

/// Build the OpenAPI document for the compiled feature set.
#[must_use]
pub fn build_openapi() -> utoipa::openapi::OpenApi {
    let doc = ApiDoc::openapi();
    #[cfg(not(any(
        feature = "chain-put",
        feature = "cep18",
        feature = "cep78",
        feature = "cep85",
        feature = "cep95"
    )))]
    {
        return doc;
    }
    #[cfg(any(
        feature = "chain-put",
        feature = "cep18",
        feature = "cep78",
        feature = "cep85",
        feature = "cep95"
    ))]
    {
        let mut doc = doc;
        #[cfg(feature = "chain-put")]
        doc.merge(ApiDocChainPut::openapi());
        #[cfg(feature = "cep18")]
        doc.merge(ApiDocCep18::openapi());
        #[cfg(feature = "cep78")]
        doc.merge(ApiDocCep78::openapi());
        #[cfg(feature = "cep85")]
        doc.merge(ApiDocCep85::openapi());
        #[cfg(feature = "cep95")]
        doc.merge(ApiDocCep95::openapi());
        doc
    }
}

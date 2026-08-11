//! OpenAPI document aggregation (full Actix surface).

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
        (name = "CEP-18", description = "Fungible token (full ceps-client surface)"),
        (name = "CEP-78", description = "NFT (full ceps-client surface; migrate not exposed)"),
        (name = "CEP-85", description = "Multi-token (full ceps-client surface)"),
        (name = "CEP-95", description = "Odra NFT (full ceps-client surface)"),
    ),
    info(
        title = "ceps-rust-api",
        description = "Casper CEP HTTP API. Every Actix CEP route is listed here. CEP mutates use MutateEnvelope (submit/wait/signer/payment_amount). Optional local or KMS put signing (no PEM in HTTP bodies). Transactions only.",
        version = "1.0.0"
    )
)]
pub struct ApiDoc;

#[cfg(feature = "chain-put")]
#[derive(OpenApi)]
#[openapi(
    paths(crate::routes::chain::chain_put_transaction),
    components(schemas(crate::routes::chain::PutTransactionBody))
)]
struct ApiDocChainPut;

#[cfg(feature = "cep18")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep18::cep18_install,
        crate::routes::cep18::cep18_upgrade,
        crate::routes::cep18::cep18_transfer,
        crate::routes::cep18::cep18_transfer_from,
        crate::routes::cep18::cep18_approve,
        crate::routes::cep18::cep18_increase_allowance,
        crate::routes::cep18::cep18_decrease_allowance,
        crate::routes::cep18::cep18_mint,
        crate::routes::cep18::cep18_burn,
        crate::routes::cep18::cep18_change_events_mode,
        crate::routes::cep18::cep18_change_security,
        crate::routes::cep18::cep18_name,
        crate::routes::cep18::cep18_symbol,
        crate::routes::cep18::cep18_decimals,
        crate::routes::cep18::cep18_total_supply,
        crate::routes::cep18::cep18_events_mode,
        crate::routes::cep18::cep18_is_mint_and_burn_enabled,
        crate::routes::cep18::cep18_balance_of,
        crate::routes::cep18::cep18_allowances,
        crate::routes::cep18::cep18_security_badge
    ),
    components(schemas(
        crate::routes::cep18::ContractQuery,
        crate::routes::cep18::InstallBody,
        crate::routes::cep18::UpgradeBody,
        crate::routes::cep18::TransferBody,
        crate::routes::cep18::TransferFromBody,
        crate::routes::cep18::ApproveBody,
        crate::routes::cep18::MintBurnBody,
        crate::routes::cep18::ChangeEventsBody,
        crate::routes::cep18::ChangeSecurityBody
    ))
)]
struct ApiDocCep18;

#[cfg(feature = "cep78")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep78::cep78_install,
        crate::routes::cep78::cep78_upgrade,
        crate::routes::cep78::cep78_mint,
        crate::routes::cep78::cep78_transfer,
        crate::routes::cep78::cep78_burn,
        crate::routes::cep78::cep78_register_owner,
        crate::routes::cep78::cep78_approve,
        crate::routes::cep78::cep78_revoke,
        crate::routes::cep78::cep78_set_approval_for_all,
        crate::routes::cep78::cep78_set_token_metadata,
        crate::routes::cep78::cep78_set_variables,
        crate::routes::cep78::cep78_mint_session,
        crate::routes::cep78::cep78_transfer_session,
        crate::routes::cep78::cep78_updated_receipts,
        crate::routes::cep78::cep78_owner_of_session,
        crate::routes::cep78::cep78_balance_of_session,
        crate::routes::cep78::cep78_get_approved_session,
        crate::routes::cep78::cep78_is_approved_for_all_session,
        crate::routes::cep78::cep78_collection_name,
        crate::routes::cep78::cep78_collection_symbol,
        crate::routes::cep78::cep78_total_token_supply,
        crate::routes::cep78::cep78_number_of_minted_tokens,
        crate::routes::cep78::cep78_allow_minting,
        crate::routes::cep78::cep78_operator_burn_mode,
        crate::routes::cep78::cep78_package_operator_mode,
        crate::routes::cep78::cep78_acl_package_mode,
        crate::routes::cep78::cep78_json_schema,
        crate::routes::cep78::cep78_minting_mode,
        crate::routes::cep78::cep78_whitelist_mode,
        crate::routes::cep78::cep78_reporting_mode,
        crate::routes::cep78::cep78_burn_mode,
        crate::routes::cep78::cep78_holder_mode,
        crate::routes::cep78::cep78_identifier_mode,
        crate::routes::cep78::cep78_metadata_mutability,
        crate::routes::cep78::cep78_nft_kind,
        crate::routes::cep78::cep78_nft_metadata_kind,
        crate::routes::cep78::cep78_ownership_mode,
        crate::routes::cep78::cep78_events_mode,
        crate::routes::cep78::cep78_owner_of,
        crate::routes::cep78::cep78_balance_of,
        crate::routes::cep78::cep78_is_approved_for_all,
        crate::routes::cep78::cep78_get_approved,
        crate::routes::cep78::cep78_metadata,
        crate::routes::cep78::cep78_is_acl_whitelisted
    ),
    components(schemas(
        crate::routes::cep78::ContractRef,
        crate::routes::cep78::InstallBody,
        crate::routes::cep78::UpgradeBody,
        crate::routes::cep78::MintBody,
        crate::routes::cep78::TransferBody,
        crate::routes::cep78::BurnBody,
        crate::routes::cep78::RegisterOwnerBody,
        crate::routes::cep78::ApproveBody,
        crate::routes::cep78::ApprovalForAllBody,
        crate::routes::cep78::SetMetaBody,
        crate::routes::cep78::SetVariablesBody,
        crate::routes::cep78::MintSessionBody,
        crate::routes::cep78::TransferSessionBody,
        crate::routes::cep78::UpdatedReceiptsBody,
        crate::routes::cep78::MetadataQuery,
        crate::routes::cep78::OwnerOfSessionBody,
        crate::routes::cep78::BalanceOfSessionBody,
        crate::routes::cep78::GetApprovedSessionBody,
        crate::routes::cep78::IsApprovedForAllSessionBody
    ))
)]
struct ApiDocCep78;

#[cfg(feature = "cep85")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep85::cep85_install,
        crate::routes::cep85::cep85_upgrade,
        crate::routes::cep85::cep85_mint,
        crate::routes::cep85::cep85_batch_mint,
        crate::routes::cep85::cep85_transfer,
        crate::routes::cep85::cep85_batch_transfer,
        crate::routes::cep85::cep85_burn,
        crate::routes::cep85::cep85_batch_burn,
        crate::routes::cep85::cep85_set_approval_for_all,
        crate::routes::cep85::cep85_set_uri,
        crate::routes::cep85::cep85_set_total_supply_of,
        crate::routes::cep85::cep85_set_total_supply_of_batch,
        crate::routes::cep85::cep85_change_security,
        crate::routes::cep85::cep85_set_modalities,
        crate::routes::cep85::cep85_balance_of_batch,
        crate::routes::cep85::cep85_supply_of_batch,
        crate::routes::cep85::cep85_total_supply_of_batch,
        crate::routes::cep85::cep85_collection_name,
        crate::routes::cep85::cep85_collection_uri,
        crate::routes::cep85::cep85_balance_of,
        crate::routes::cep85::cep85_supply_of,
        crate::routes::cep85::cep85_total_supply_of,
        crate::routes::cep85::cep85_total_fungible_supply,
        crate::routes::cep85::cep85_uri,
        crate::routes::cep85::cep85_is_non_fungible,
        crate::routes::cep85::cep85_is_approved_for_all,
        crate::routes::cep85::cep85_enable_burn,
        crate::routes::cep85::cep85_events_mode,
        crate::routes::cep85::cep85_number_of_minted_tokens,
        crate::routes::cep85::cep85_transfer_filter_contract,
        crate::routes::cep85::cep85_transfer_filter_method,
        crate::routes::cep85::cep85_security_badge
    ),
    components(schemas(
        crate::routes::cep85::ContractRef,
        crate::routes::cep85::InstallBody,
        crate::routes::cep85::UpgradeBody,
        crate::routes::cep85::MintBody,
        crate::routes::cep85::BatchMintBody,
        crate::routes::cep85::TransferBody,
        crate::routes::cep85::BatchTransferBody,
        crate::routes::cep85::BurnBody,
        crate::routes::cep85::BatchBurnBody,
        crate::routes::cep85::ApprovalBody,
        crate::routes::cep85::SetUriBody,
        crate::routes::cep85::SetTotalSupplyBody,
        crate::routes::cep85::SetTotalSupplyBatchBody,
        crate::routes::cep85::ChangeSecurityBody,
        crate::routes::cep85::SetModalitiesBody,
        crate::routes::cep85::UriQuery,
        crate::routes::cep85::BatchAccountsIdsBody,
        crate::routes::cep85::BatchIdsBody
    ))
)]
struct ApiDocCep85;

#[cfg(feature = "cep95")]
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::cep95::cep95_install,
        crate::routes::cep95::cep95_transfer_from,
        crate::routes::cep95::cep95_safe_transfer_from,
        crate::routes::cep95::cep95_approve,
        crate::routes::cep95::cep95_revoke_approval,
        crate::routes::cep95::cep95_approve_for_all,
        crate::routes::cep95::cep95_revoke_approval_for_all,
        crate::routes::cep95::cep95_mint,
        crate::routes::cep95::cep95_burn,
        crate::routes::cep95::cep95_transfer_ownership,
        crate::routes::cep95::cep95_name,
        crate::routes::cep95::cep95_symbol,
        crate::routes::cep95::cep95_total_supply,
        crate::routes::cep95::cep95_get_owner,
        crate::routes::cep95::cep95_owner_of,
        crate::routes::cep95::cep95_balance_of,
        crate::routes::cep95::cep95_get_approved,
        crate::routes::cep95::cep95_is_approved_for_all,
        crate::routes::cep95::cep95_token_metadata,
        crate::routes::cep95::cep95_bind_odra_install
    ),
    components(schemas(
        crate::routes::cep95::ContractRef,
        crate::routes::cep95::InstallBody,
        crate::routes::cep95::TransferBody,
        crate::routes::cep95::ApproveBody,
        crate::routes::cep95::ApproveForAllBody,
        crate::routes::cep95::MintBody,
        crate::routes::cep95::BurnBody,
        crate::routes::cep95::TransferOwnershipBody,
        crate::routes::cep95::BindOdraInstallBody
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

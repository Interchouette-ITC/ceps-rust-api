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
        crate::routes::extractors::MutateQuery,
        crate::routes::extractors::ContractQuery,
        crate::routes::extractors::EventsModeParam,
        crate::routes::extractors::EventsMode78Param,
        crate::routes::extractors::OwnershipModeParam,
        crate::routes::extractors::NftMetadataKindParam,
        crate::routes::extractors::IdentifierModeParam,
        crate::routes::extractors::MetadataMutabilityParam,
        crate::routes::extractors::NftKindParam,
        crate::routes::extractors::MintingModeParam,
        crate::routes::extractors::BurnModeParam,
        crate::routes::extractors::WhitelistModeParam,
        crate::routes::extractors::HolderModeParam,
        crate::routes::extractors::OwnerReverseLookupModeParam,
        crate::routes::extractors::NamedKeyConventionModeParam,
        crate::routes::extractors::NameResponse,
        crate::routes::extractors::SymbolResponse,
        crate::routes::extractors::DecimalsResponse,
        crate::routes::extractors::TotalSupplyResponse,
        crate::routes::extractors::EventsModeResponse,
        crate::routes::extractors::EnabledResponse,
        crate::routes::extractors::BalanceResponse,
        crate::routes::extractors::AllowanceResponse,
        crate::routes::extractors::BadgeResponse,
        crate::routes::extractors::BoolResponse,
        crate::routes::extractors::OptionalStringResponse,
        crate::routes::extractors::U64Response,
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
        description = "Casper CEP HTTP API. Mutates take typed query parameters (submit/wait/signer/payment_amount plus operation fields). Selected JSON bodies only for nested CEP data (e.g. token metadata). Optional local or KMS put signing.",
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
        crate::routes::cep18::InstallQuery,
        crate::routes::cep18::UpgradeQuery,
        crate::routes::cep18::TransferQuery,
        crate::routes::cep18::TransferFromQuery,
        crate::routes::cep18::ApproveQuery,
        crate::routes::cep18::MintBurnQuery,
        crate::routes::cep18::ChangeEventsQuery,
        crate::routes::cep18::ChangeSecurityQuery
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
        crate::routes::cep78::InstallQuery,
        crate::routes::cep78::InstallSchemaBody,
        crate::routes::cep78::TokenMetaBody,
        crate::routes::cep78::UpgradeQuery,
        crate::routes::cep78::MintQuery,
        crate::routes::cep78::TransferQuery,
        crate::routes::cep78::BurnQuery,
        crate::routes::cep78::RegisterOwnerQuery,
        crate::routes::cep78::ApproveQuery,
        crate::routes::cep78::ApprovalForAllQuery,
        crate::routes::cep78::SetMetaQuery,
        crate::routes::cep78::SetVariablesQuery,
        crate::routes::cep78::MintSessionQuery,
        crate::routes::cep78::TransferSessionQuery,
        crate::routes::cep78::UpdatedReceiptsQuery,
        crate::routes::cep78::MetadataQuery,
        crate::routes::cep78::OwnerOfSessionQuery,
        crate::routes::cep78::BalanceOfSessionQuery,
        crate::routes::cep78::GetApprovedSessionQuery,
        crate::routes::cep78::IsApprovedForAllSessionQuery
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
        crate::routes::cep85::InstallQuery,
        crate::routes::cep85::UpgradeQuery,
        crate::routes::cep85::MintQuery,
        crate::routes::cep85::BatchMintQuery,
        crate::routes::cep85::TransferQuery,
        crate::routes::cep85::BatchTransferQuery,
        crate::routes::cep85::BurnQuery,
        crate::routes::cep85::BatchBurnQuery,
        crate::routes::cep85::ApprovalQuery,
        crate::routes::cep85::SetUriQuery,
        crate::routes::cep85::SetTotalSupplyQuery,
        crate::routes::cep85::SetTotalSupplyBatchQuery,
        crate::routes::cep85::ChangeSecurityQuery,
        crate::routes::cep85::SetModalitiesQuery,
        crate::routes::cep85::UriQuery,
        crate::routes::cep85::BatchAccountsIdsQuery,
        crate::routes::cep85::BatchIdsQuery
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
        crate::routes::cep95::InstallQuery,
        crate::routes::cep95::TransferQuery,
        crate::routes::cep95::ApproveQuery,
        crate::routes::cep95::ApproveForAllQuery,
        crate::routes::cep95::MintQuery,
        crate::routes::cep95::MintMetadataBody,
        crate::routes::cep95::BurnQuery,
        crate::routes::cep95::TransferOwnershipQuery,
        crate::routes::cep95::BindOdraInstallQuery,
        crate::routes::cep95::OwnerResponse,
        crate::routes::cep95::TokenMetadataResponse,
        crate::routes::cep95::BindOdraInstallResponse
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

#[cfg(test)]
mod tests {
    use super::build_openapi;
    use serde_json::Value;

    #[test]
    fn cep18_approve_params_are_flat_enums() {
        let doc = build_openapi();
        let json = serde_json::to_value(doc).expect("openapi json");
        let params = json
            .pointer("/paths/~1v1~1cep18~1approve/post/parameters")
            .and_then(|v| v.as_array())
            .expect("approve parameters");
        let names: Vec<&str> = params
            .iter()
            .filter_map(|p| p.get("name").and_then(|n| n.as_str()))
            .collect();
        for required in [
            "signer",
            "submit",
            "wait",
            "payment_amount",
            "contract_hash",
            "spender",
            "amount",
        ] {
            assert!(names.contains(&required), "missing {required} in {names:?}");
        }
        assert!(!names.contains(&"mutate"));
        assert!(!names.contains(&"contract"));

        let submit = params
            .iter()
            .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("submit"))
            .expect("submit param");
        assert_enum_select(&json, submit, &["put", "return"]);
    }

    #[test]
    fn cep18_change_events_mode_is_select() {
        let doc = build_openapi();
        let json = serde_json::to_value(doc).expect("openapi json");
        let params = json
            .pointer("/paths/~1v1~1cep18~1change-events-mode/post/parameters")
            .and_then(|v| v.as_array())
            .expect("change-events-mode parameters");
        let events = params
            .iter()
            .find(|p| p.get("name").and_then(|n| n.as_str()) == Some("events_mode"))
            .expect("events_mode param");
        assert_enum_select(&json, events, &["NoEvents", "CES", "Native", "NativeBytes"]);
    }

    #[test]
    fn all_mode_like_query_params_are_enum_selects() {
        let doc = build_openapi();
        let json = serde_json::to_value(doc).expect("openapi json");
        let mode_like = [
            "submit",
            "wait",
            "events_mode",
            "ownership_mode",
            "nft_metadata_kind",
            "nft_kind",
            "identifier_mode",
            "metadata_mutability",
            "minting_mode",
            "burn_mode",
            "whitelist_mode",
            "holder_mode",
            "owner_reverse_lookup_mode",
            "named_key_convention",
            "kind",
        ];
        let mut seen = 0usize;
        for (path, methods) in json.get("paths").and_then(|v| v.as_object()).unwrap() {
            for (_method, op) in methods.as_object().unwrap() {
                let Some(params) = op.get("parameters").and_then(|v| v.as_array()) else {
                    continue;
                };
                for p in params {
                    if p.get("in").and_then(|v| v.as_str()) != Some("query") {
                        continue;
                    }
                    let Some(name) = p.get("name").and_then(|v| v.as_str()) else {
                        continue;
                    };
                    if !mode_like.contains(&name) {
                        continue;
                    }
                    seen += 1;
                    let schema = resolve_schema(&json, p.get("schema").expect("schema"));
                    assert!(
                        schema.get("enum").is_some(),
                        "{path} query `{name}` is not an OpenAPI enum select: {schema}"
                    );
                }
            }
        }
        assert!(
            seen >= 20,
            "expected many mode-like enum params, only saw {seen}"
        );
    }

    fn resolve_schema<'a>(doc: &'a Value, schema: &'a Value) -> Value {
        if let Some(r) = schema.get("$ref").and_then(|v| v.as_str()) {
            let name = r.rsplit('/').next().unwrap();
            return doc
                .pointer(&format!("/components/schemas/{name}"))
                .cloned()
                .unwrap_or_else(|| panic!("missing schema ref {r}"));
        }
        if let Some(all_of) = schema.get("allOf").and_then(|v| v.as_array()) {
            for part in all_of {
                let resolved = resolve_schema(doc, part);
                if resolved.get("enum").is_some() {
                    return resolved;
                }
            }
        }
        schema.clone()
    }

    fn assert_enum_select(doc: &Value, param: &Value, expected: &[&str]) {
        let schema = resolve_schema(doc, param.get("schema").expect("param schema"));
        let vals: Vec<String> = schema
            .get("enum")
            .and_then(|v| v.as_array())
            .expect("enum array")
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        for e in expected {
            assert!(
                vals.iter().any(|v| v.eq_ignore_ascii_case(e)),
                "missing variant {e} in {vals:?} (schema={schema})"
            );
        }
    }
}

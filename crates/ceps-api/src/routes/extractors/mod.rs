//! Shared query extractors, OpenAPI enums, and typed response helpers.

use crate::error::ApiError;
use crate::tx::{MutateEnvelope, SignerRef, SubmitMode, WaitMode};
use ceps_client::{EventsMode, EventsMode78};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

fn default_payment() -> String {
    crate::constants::DEFAULT_CALL_PAYMENT.to_string()
}

/// Shared submit/wait/signer/payment as **flat** query params (no nested object in OpenAPI).
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct MutateQuery {
    /// `put` signs/puts when configured; `return` returns unsigned Transaction JSON.
    #[serde(default)]
    #[param(inline, example = "put")]
    pub submit: SubmitMode,
    /// After put: wait until accepted or processed.
    #[serde(default)]
    #[param(inline, example = "processed")]
    pub wait: WaitMode,
    /// Casper account public key hex (initiator / signer identity).
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub signer: String,
    #[serde(default = "default_payment")]
    #[param(example = "2500000000")]
    pub payment_amount: String,
}

impl MutateQuery {
    #[must_use]
    pub fn envelope(&self) -> MutateEnvelope {
        MutateEnvelope {
            submit: self.submit,
            wait: self.wait,
            signer: SignerRef {
                public_key: self.signer.clone(),
            },
            payment_amount: self.payment_amount.clone(),
        }
    }
}

/// Build envelope from the common flat query fields (inlined on route structs).
#[must_use]
pub fn envelope_from(
    submit: SubmitMode,
    wait: WaitMode,
    signer: &str,
    payment_amount: &str,
) -> MutateEnvelope {
    MutateEnvelope {
        submit,
        wait,
        signer: SignerRef {
            public_key: signer.to_string(),
        },
        payment_amount: payment_amount.to_string(),
    }
}

/// Contract binding as **flat** query params.
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query, style = Form)]
pub struct ContractQuery {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[param(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[serde(default)]
    #[param(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

/// CEP-18 / CEP-85 events mode (OpenAPI select).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum EventsModeParam {
    NoEvents,
    CES,
    Native,
    NativeBytes,
}

impl EventsModeParam {
    pub fn to_client(self) -> EventsMode {
        match self {
            Self::NoEvents => EventsMode::NoEvents,
            Self::CES => EventsMode::CES,
            Self::Native => EventsMode::Native,
            Self::NativeBytes => EventsMode::NativeBytes,
        }
    }
}

/// CEP-78 events mode (OpenAPI select).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum EventsMode78Param {
    NoEvents,
    CEP47,
    CES,
    Native,
    NativeBytes,
}

impl EventsMode78Param {
    pub fn to_client(self) -> EventsMode78 {
        match self {
            Self::NoEvents => EventsMode78::NoEvents,
            Self::CEP47 => EventsMode78::CEP47,
            Self::CES => EventsMode78::CES,
            Self::Native => EventsMode78::Native,
            Self::NativeBytes => EventsMode78::NativeBytes,
        }
    }
}

macro_rules! mode_param {
    ($name:ident => $client:ty { $($var:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
        #[serde(rename_all = "PascalCase")]
        pub enum $name {
            $($var),+
        }

        impl $name {
            #[must_use]
            pub fn to_client(self) -> $client {
                match self {
                    $(Self::$var => <$client>::$var,)+
                }
            }
        }
    };
}

mode_param!(OwnershipModeParam => ceps_client::cep78::OwnershipMode {
    Minter,
    Assigned,
    Transferable
});
mode_param!(NftMetadataKindParam => ceps_client::cep78::NftMetadataKind {
    CEP78,
    Nft721,
    Raw,
    CustomValidated
});
mode_param!(IdentifierModeParam => ceps_client::cep78::IdentifierMode {
    Ordinal,
    Hash
});
mode_param!(MetadataMutabilityParam => ceps_client::cep78::MetadataMutability {
    Immutable,
    Mutable
});
mode_param!(NftKindParam => ceps_client::cep78::NftKind {
    Physical,
    Digital,
    Virtual
});
mode_param!(MintingModeParam => ceps_client::cep78::MintingMode {
    Installer,
    Public,
    Acl
});
mode_param!(BurnModeParam => ceps_client::cep78::BurnMode {
    Burnable,
    NonBurnable
});
mode_param!(WhitelistModeParam => ceps_client::cep78::WhitelistMode {
    Unlocked,
    Locked
});
mode_param!(HolderModeParam => ceps_client::cep78::HolderMode {
    Accounts,
    Contracts,
    Mixed
});
mode_param!(OwnerReverseLookupModeParam => ceps_client::cep78::OwnerReverseLookupMode {
    NoLookup,
    Complete,
    TransfersOnly
});
mode_param!(NamedKeyConventionModeParam => ceps_client::cep78::NamedKeyConventionMode {
    DerivedFromCollectionName,
    V1_0Standard,
    V1_0Custom
});

/// Parse CEP-18/85 `events_mode` from name or decimal u8 string (legacy query).
pub fn parse_events_mode(raw: &str) -> Result<EventsMode, ApiError> {
    let s = raw.trim();
    if let Ok(v) = s.parse::<u8>() {
        return EventsMode::from_u8(v)
            .ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {v}")));
    }
    match s.to_ascii_lowercase().as_str() {
        "noevents" | "no_events" | "none" => Ok(EventsMode::NoEvents),
        "ces" => Ok(EventsMode::CES),
        "native" => Ok(EventsMode::Native),
        "nativebytes" | "native_bytes" => Ok(EventsMode::NativeBytes),
        _ => Err(ApiError::BadRequest(format!(
            "invalid events_mode {s} (use NoEvents|CES|Native|NativeBytes or 0-3)"
        ))),
    }
}

/// Parse CEP-78 `events_mode` from name or decimal u8 string (legacy query).
pub fn parse_events_mode78(raw: &str) -> Result<EventsMode78, ApiError> {
    let s = raw.trim();
    if let Ok(v) = s.parse::<u8>() {
        return EventsMode78::from_u8(v)
            .ok_or_else(|| ApiError::BadRequest(format!("invalid events_mode {v}")));
    }
    match s.to_ascii_lowercase().as_str() {
        "noevents" | "no_events" | "none" => Ok(EventsMode78::NoEvents),
        "cep47" => Ok(EventsMode78::CEP47),
        "ces" => Ok(EventsMode78::CES),
        "native" => Ok(EventsMode78::Native),
        "nativebytes" | "native_bytes" => Ok(EventsMode78::NativeBytes),
        _ => Err(ApiError::BadRequest(format!(
            "invalid events_mode {s} (use NoEvents|CEP47|CES|Native|NativeBytes or 0-4)"
        ))),
    }
}

/// Non-empty repeated query list → `Some(vec)`, else `None`.
pub fn opt_list(v: Vec<String>) -> Option<Vec<String>> {
    let cleaned: Vec<String> = v
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

/// Serialize nested JSON Value to the string CEP entrypoints expect.
pub fn json_value_to_string(v: &serde_json::Value) -> Result<String, ApiError> {
    if v.is_null() {
        return Err(ApiError::BadRequest("JSON value must not be null".into()));
    }
    if let Some(s) = v.as_str() {
        return Ok(s.to_string());
    }
    serde_json::to_string(v).map_err(|e| ApiError::BadRequest(format!("json serialize: {e}")))
}

#[derive(Debug, Serialize, ToSchema)]
pub struct StringFieldResponse {
    pub value: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct NameResponse {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SymbolResponse {
    pub symbol: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DecimalsResponse {
    pub decimals: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TotalSupplyResponse {
    pub total_supply: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventsModeResponse {
    pub events_mode: String,
    pub events_mode_u8: u8,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EnabledResponse {
    pub enabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BalanceResponse {
    pub balance: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AllowanceResponse {
    pub allowance: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BadgeResponse {
    pub badge: Option<String>,
    pub badge_u8: Option<u8>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BoolResponse {
    pub value: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct U64Response {
    pub value: u64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OptionalStringResponse {
    pub value: Option<String>,
}

/// Merge IntoParams lists so OpenAPI shows flat query fields (utoipa 5 does not explode serde flatten).
#[macro_export]
macro_rules! impl_flat_query_params {
    // Mutate + Contract + Rest
    ($ty:ty, mutate_contract, $rest:ty) => {
        impl utoipa::IntoParams for $ty {
            fn into_params(
                parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
            ) -> Vec<utoipa::openapi::path::Parameter> {
                let mut params =
                    <$crate::routes::extractors::MutateQuery as utoipa::IntoParams>::into_params(
                        &parameter_in_provider,
                    );
                params.extend(
                    <$crate::routes::extractors::ContractQuery as utoipa::IntoParams>::into_params(
                        &parameter_in_provider,
                    ),
                );
                params.extend(<$rest as utoipa::IntoParams>::into_params(
                    &parameter_in_provider,
                ));
                params
            }
        }
    };
    // Mutate + Rest (install / no contract)
    ($ty:ty, mutate, $rest:ty) => {
        impl utoipa::IntoParams for $ty {
            fn into_params(
                parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
            ) -> Vec<utoipa::openapi::path::Parameter> {
                let mut params =
                    <$crate::routes::extractors::MutateQuery as utoipa::IntoParams>::into_params(
                        &parameter_in_provider,
                    );
                params.extend(<$rest as utoipa::IntoParams>::into_params(
                    &parameter_in_provider,
                ));
                params
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_mode_named() {
        assert_eq!(parse_events_mode("CES").unwrap(), EventsMode::CES);
        assert_eq!(parse_events_mode("1").unwrap(), EventsMode::CES);
        assert_eq!(parse_events_mode78("CEP47").unwrap(), EventsMode78::CEP47);
        assert_eq!(EventsModeParam::CES.to_client(), EventsMode::CES);
    }

    #[test]
    fn mutate_query_to_envelope() {
        let q = MutateQuery {
            submit: SubmitMode::Put,
            wait: WaitMode::Processed,
            signer: "01aa".into(),
            payment_amount: "1".into(),
        };
        let e = q.envelope();
        assert_eq!(e.signer.public_key, "01aa");
    }

    #[test]
    fn mutate_query_params_are_flat_names() {
        let params = <MutateQuery as utoipa::IntoParams>::into_params(|| {
            Some(utoipa::openapi::path::ParameterIn::Query)
        });
        let names: Vec<_> = params.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"signer"));
        assert!(names.contains(&"submit"));
        assert!(names.contains(&"wait"));
        assert!(names.contains(&"payment_amount"));
        assert!(!names.iter().any(|n| *n == "mutate"));
    }

    #[test]
    fn events_mode_param_is_enum_schema() {
        use utoipa::PartialSchema;
        let schema = EventsModeParam::schema();
        let json = serde_json::to_value(schema).expect("schema json");
        // Inline/component schema should expose enum variants, not free-form string only.
        let enums = json
            .pointer("/Schema/Object/enum")
            .or_else(|| json.pointer("/enum"))
            .cloned();
        assert!(
            enums.is_some(),
            "EventsModeParam OpenAPI schema missing enum: {json}"
        );
    }
}

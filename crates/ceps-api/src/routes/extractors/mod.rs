//! Shared query extractors and typed response helpers for CEP routes.

use crate::error::ApiError;
use crate::tx::{MutateEnvelope, SignerRef, SubmitMode, WaitMode};
use ceps_client::{EventsMode, EventsMode78};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

fn default_payment() -> String {
    crate::constants::DEFAULT_CALL_PAYMENT.to_string()
}

/// Mutate envelope as flat query parameters (`signer` = public key hex).
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct MutateQuery {
    /// `put` signs/puts when a signing backend is configured; `return` returns unsigned Transaction JSON.
    #[serde(default)]
    #[param(example = "put")]
    pub submit: SubmitMode,
    /// After put: wait until accepted or processed.
    #[serde(default)]
    #[param(example = "processed")]
    pub wait: WaitMode,
    /// Casper account public key hex (initiator / signer identity).
    #[param(example = "01aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub signer: String,
    #[serde(default = "default_payment")]
    #[param(example = "2500000000")]
    pub payment_amount: String,
}

impl MutateQuery {
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

/// Contract binding as query parameters.
#[derive(Debug, Clone, Deserialize, IntoParams, ToSchema)]
#[into_params(parameter_in = Query)]
pub struct ContractQuery {
    /// Contract hash hex (64 hex chars, no 0x prefix).
    #[param(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub contract_hash: String,
    /// Optional package hash hex.
    #[serde(default)]
    #[param(example = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")]
    pub package_hash: Option<String>,
}

/// Parse CEP-18/85 `events_mode` from name or decimal u8 string.
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

/// Parse CEP-78 `events_mode` from name or decimal u8 string.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_mode_named() {
        assert_eq!(parse_events_mode("CES").unwrap(), EventsMode::CES);
        assert_eq!(parse_events_mode("1").unwrap(), EventsMode::CES);
        assert_eq!(parse_events_mode78("CEP47").unwrap(), EventsMode78::CEP47);
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
}

//! HTTP error mapping.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use std::fmt;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    NoSigner(String),
    #[error("{0}")]
    Kms(String),
    #[error("{0}")]
    Chain(String),
    #[error("{0}")]
    TxFailed(String),
    #[error("{0}")]
    FeatureDisabled(String),
    #[error("{0}")]
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
    code: &'static str,
}

impl ApiError {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_) => "not_found",
            Self::NoSigner(_) => "no_signer",
            Self::Kms(_) => "kms",
            Self::Chain(_) => "chain",
            Self::TxFailed(_) => "tx_failed",
            Self::FeatureDisabled(_) => "feature_disabled",
            Self::Internal(_) => "internal",
        }
    }

    #[must_use]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) | Self::NoSigner(_) | Self::FeatureDisabled(_) => {
                StatusCode::BAD_REQUEST
            }
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Kms(_) | Self::Chain(_) => StatusCode::BAD_GATEWAY,
            Self::TxFailed(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn from_cep(err: ceps_client::CEPError) -> Self {
        use ceps_client::CEPError;
        match err {
            CEPError::InvalidUrl(m)
            | CEPError::InvalidHash(m)
            | CEPError::MissingArgument(m)
            | CEPError::InvalidArgument(m)
            | CEPError::Decode(m) => Self::BadRequest(m),
            CEPError::ContractHashMissing => Self::BadRequest("contract hash is not set".into()),
            CEPError::EmptyQuery(m) => Self::NotFound(m),
            CEPError::Execution { message, .. } => Self::TxFailed(message),
            CEPError::WaitFailed(m) => Self::Chain(m),
            CEPError::Sdk(e) => Self::Chain(e.to_string()),
            CEPError::Io(e) => Self::Internal(e.to_string()),
            CEPError::Other(m) => {
                let lower = m.to_ascii_lowercase();
                if lower.contains("parse")
                    || lower.contains("invalid")
                    || lower.contains("serialize")
                {
                    Self::BadRequest(m)
                } else {
                    Self::Chain(m)
                }
            }
        }
    }

    pub fn internal(err: impl fmt::Display) -> Self {
        Self::Internal(err.to_string())
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status()
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status()).json(ErrorBody {
            error: self.to_string(),
            code: self.code(),
        })
    }
}

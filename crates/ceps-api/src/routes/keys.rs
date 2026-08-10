//! Custody key listing and KMS proxy routes.
//!
//! Key *creation* is KMS-only (`POST /v1/kms/create-key`). Local signing uses a
//! static keyring from `LOCAL_KEYS_JSON` (for example NCTL user PEMs).

#[cfg(feature = "sign-kms")]
use crate::error::ApiError;
#[cfg(any(feature = "sign-local", feature = "sign-kms"))]
use crate::state::AppState;
#[cfg(any(feature = "sign-local", feature = "sign-kms"))]
use actix_web::{get, post, web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct KeyringList {
    pub public_keys: Vec<String>,
}

#[cfg(feature = "sign-local")]
#[utoipa::path(
    get,
    path = "/v1/keys",
    responses((status = 200, description = "Local keyring public keys", body = KeyringList)),
    tag = "Keys"
)]
#[cfg(feature = "sign-local")]
#[get("/v1/keys")]
pub async fn keys_list(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(KeyringList {
        public_keys: state.keyring.public_keys(),
    })
}

#[cfg(feature = "sign-kms")]
#[utoipa::path(
    post,
    path = "/v1/kms/create-key",
    responses((status = 200, description = "KMS createKey proxy")),
    tag = "KMS"
)]
#[cfg(feature = "sign-kms")]
#[post("/v1/kms/create-key")]
pub async fn kms_create_key(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let kms = state
        .kms
        .as_ref()
        .ok_or_else(|| ApiError::Kms("KMS_URL not configured".into()))?;
    let created = kms.create_key().await?;
    Ok(HttpResponse::Ok().json(created))
}

#[cfg(feature = "sign-kms")]
#[utoipa::path(
    get,
    path = "/v1/kms/list-keys",
    responses((status = 200, description = "KMS listKeys proxy")),
    tag = "KMS"
)]
#[cfg(feature = "sign-kms")]
#[get("/v1/kms/list-keys")]
pub async fn kms_list_keys(state: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    let kms = state
        .kms
        .as_ref()
        .ok_or_else(|| ApiError::Kms("KMS_URL not configured".into()))?;
    Ok(HttpResponse::Ok().json(kms.list_keys().await?))
}

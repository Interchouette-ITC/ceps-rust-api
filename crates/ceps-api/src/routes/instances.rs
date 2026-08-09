//! Instance registry routes.

use crate::error::ApiError;
use crate::registry::InstanceRecord;
use crate::state::AppState;
use actix_web::{delete, get, post, web, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RegisterInstanceBody {
    pub cep: String,
    pub contract_hash: String,
    pub package_hash: Option<String>,
    pub label: Option<String>,
}

#[utoipa::path(
    get,
    path = "/v1/instances",
    responses((status = 200, description = "Registered instances", body = [InstanceRecord])),
    tag = "Instances"
)]
#[get("/v1/instances")]
pub async fn list_instances(state: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().json(state.registry.list())
}

#[utoipa::path(
    post,
    path = "/v1/instances",
    request_body = RegisterInstanceBody,
    responses((status = 200, description = "Registered instance", body = InstanceRecord)),
    tag = "Instances"
)]
#[post("/v1/instances")]
pub async fn register_instance(
    state: web::Data<AppState>,
    body: web::Json<RegisterInstanceBody>,
) -> Result<HttpResponse, ApiError> {
    if body.contract_hash.trim().is_empty() {
        return Err(ApiError::BadRequest("contract_hash required".into()));
    }
    let rec = state.registry.register(
        body.cep.trim(),
        body.contract_hash.trim(),
        body.package_hash.clone(),
        body.label.clone(),
    );
    Ok(HttpResponse::Ok().json(rec))
}

#[utoipa::path(
    get,
    path = "/v1/instances/{id}",
    params(("id" = String, Path, description = "Instance id")),
    responses((status = 200, description = "Instance", body = InstanceRecord)),
    tag = "Instances"
)]
#[get("/v1/instances/{id}")]
pub async fn get_instance(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    state
        .registry
        .get(&path)
        .map(|r| HttpResponse::Ok().json(r))
        .ok_or_else(|| ApiError::NotFound(format!("instance {}", path.as_str())))
}

#[utoipa::path(
    delete,
    path = "/v1/instances/{id}",
    params(("id" = String, Path, description = "Instance id")),
    responses((status = 204, description = "Deleted")),
    tag = "Instances"
)]
#[delete("/v1/instances/{id}")]
pub async fn delete_instance(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    if state.registry.remove(&path) {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Err(ApiError::NotFound(format!("instance {}", path.as_str())))
    }
}

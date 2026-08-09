//! Health endpoint.

use crate::VERSION;
use actix_web::{get, HttpResponse, Responder};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema, Debug)]
pub struct HealthResult {
    pub status: String,
    pub service: String,
    pub version: String,
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Process health", body = HealthResult)),
    tag = "Health"
)]
#[get("/health")]
pub async fn health_handler() -> impl Responder {
    HttpResponse::Ok().json(HealthResult {
        status: "healthy".to_string(),
        service: "ceps-rust-api".to_string(),
        version: VERSION.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    #[actix_web::test]
    async fn health_returns_ok() {
        let app = test::init_service(App::new().service(health_handler)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}

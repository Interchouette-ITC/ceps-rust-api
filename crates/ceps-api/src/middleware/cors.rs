//! CORS for demo clients and Swagger UI.

use actix_cors::Cors;

/// Permissive CORS for local/demo. Tighten for production later.
pub fn demo_cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600)
}

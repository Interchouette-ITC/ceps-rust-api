//! ceps-rust-api library: Actix HTTP application for Casper CEP operations.

pub mod config;
pub mod constants;
pub mod error;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod state;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

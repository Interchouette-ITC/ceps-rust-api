//! ceps-rust-api MCP library (mcpkit tools + Make/Docker helpers + HTTP client).

pub mod client;
pub mod ops;
pub mod paths;
pub mod server;

pub use server::{run, run_http, DEFAULT_HTTP_LISTEN};

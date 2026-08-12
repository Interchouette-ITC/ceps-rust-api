//! ceps-rust-api MCP library (rmcp tools + Make/Docker helpers + HTTP client).

pub mod client;
pub mod ops;
pub mod paths;
pub mod server;
pub mod tool_args;

pub use server::{run, run_http, DEFAULT_HTTP_LISTEN};

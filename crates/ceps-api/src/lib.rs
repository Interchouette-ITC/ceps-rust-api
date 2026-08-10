//! Internal application modules for the `ceps-rust-api` binary and its tests.
//! Not a published library for external consumers (`publish = false` on the package).

#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::ref_option,
    clippy::must_use_candidate,
    clippy::double_must_use
)]

pub mod config;
pub mod constants;
pub mod error;
pub mod features;
pub mod middleware;
pub mod openapi;
pub mod routes;
pub mod server;
pub mod sign;
pub mod state;
pub mod tx;

#[cfg(feature = "sign-kms")]
pub mod kms;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

//! ceps-rust-api library: Actix HTTP application for Casper CEP operations.

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

#[cfg(all(
    feature = "custody",
    not(any(feature = "sign-local", feature = "sign-kms"))
))]
compile_error!("feature `custody` requires `sign-local` and/or `sign-kms`");

pub mod config;
pub mod constants;
pub mod error;
pub mod features;
pub mod middleware;
pub mod openapi;
pub mod registry;
pub mod routes;
pub mod server;
pub mod sign;
pub mod state;
pub mod tx;

#[cfg(feature = "sign-kms")]
pub mod kms;

#[cfg(feature = "custody")]
pub mod custody;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

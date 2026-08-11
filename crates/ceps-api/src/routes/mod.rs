//! HTTP routes.

pub mod chain;
pub mod common;
pub mod extractors;
pub mod health;
pub mod hello;

#[cfg(feature = "cep18")]
pub mod cep18;
#[cfg(feature = "cep78")]
pub mod cep78;
#[cfg(feature = "cep85")]
pub mod cep85;
#[cfg(feature = "cep95")]
pub mod cep95;

pub use extractors::{ContractQuery, MutateQuery};
pub use health::health_handler;
pub use hello::hello_handler;

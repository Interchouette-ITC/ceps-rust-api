//! HTTP routes.

pub mod extractors;
pub mod health;
pub mod hello;

pub use health::health_handler;
pub use hello::hello_handler;

use ceps_rust_api::config::Config;
use ceps_rust_api::server::run_server;
use ceps_rust_api::VERSION;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    info!(version = VERSION, "ceps-rust-api starting");
    let config = Config::from_env();
    run_server(config).await
}

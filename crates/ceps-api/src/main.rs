use ceps_rust_api::config::{load_dotenv, Config};
use ceps_rust_api::server::run_server;
use ceps_rust_api::VERSION;
use tracing::{error, info};
use tracing_subscriber::{fmt, EnvFilter};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_dotenv();
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .init();

    info!(version = VERSION, "ceps-rust-api starting");
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "invalid configuration");
            std::process::exit(1);
        }
    };
    run_server(config).await
}

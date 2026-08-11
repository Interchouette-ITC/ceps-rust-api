//! Repo root resolution and runtime paths for the MCP sidecar.

use std::env;
use std::path::{Path, PathBuf};

/// Default Streamable HTTP bind (host).
pub const DEFAULT_HTTP_LISTEN: &str = "127.0.0.1:4790";
/// Default MCP HTTP URL for docs / status.
pub const DEFAULT_MCP_URL: &str = "http://127.0.0.1:4790/mcp";
/// Default ceps-rust-api HTTP base URL.
pub const DEFAULT_API_URL: &str = "http://127.0.0.1:8080";

pub const API_CONTAINER: &str = "ceps-rust-api";
pub const COMPOSE_PROD: &str = "docker/docker-compose.yml";

/// Resolve the ceps-rust-api repository root.
///
/// Order: `CEPS_API_ROOT` → cwd with `Makefile` → parent of cwd (when run from `mcp/`).
pub fn repo_root() -> PathBuf {
    if let Ok(env_root) = env::var("CEPS_API_ROOT") {
        return PathBuf::from(env_root);
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if cwd.join("Makefile").is_file() && cwd.join("Cargo.toml").is_file() {
        return cwd;
    }
    let parent = cwd.join("..");
    if parent.join("Makefile").is_file() && parent.join("Cargo.toml").is_file() {
        return parent.canonicalize().unwrap_or(parent);
    }
    cwd
}

/// Absolute path on the **Docker host** for `-v` bind sources.
///
/// MCP runs with the repo bind-mounted at `/workspace` and talks to the host
/// Docker socket. Paths inside the MCP container (`CEPS_API_ROOT=/workspace`)
/// must **not** be passed as `-v` sources. Set `CEPS_HOST_ROOT` to the host
/// clone path (launch scripts / compose do this). When unset (host-native MCP),
/// falls back to [`repo_root`].
pub fn host_repo_root() -> PathBuf {
    if let Ok(host) = env::var("CEPS_HOST_ROOT") {
        let host = host.trim();
        if !host.is_empty() {
            return PathBuf::from(host);
        }
    }
    repo_root()
}

/// MCP container layout: in-container root is `/workspace` (host clone bind).
pub fn mcp_uses_workspace_mount() -> bool {
    matches!(
        env::var("CEPS_API_ROOT").ok().as_deref().map(str::trim),
        Some("/workspace")
    )
}

/// True when Docker bind sources must not be used (would hit host `/`, `/workspace`, etc.).
pub fn host_bind_root_is_unsafe() -> bool {
    if mcp_uses_workspace_mount() {
        match env::var("CEPS_HOST_ROOT") {
            Ok(h) if !h.trim().is_empty() => {}
            _ => return true,
        }
    }

    let host = host_repo_root();
    if !host.is_absolute() {
        return true;
    }
    if host == Path::new("/") {
        return true;
    }
    if host == Path::new("/workspace") || host.starts_with("/workspace/") {
        return true;
    }
    false
}

/// Error text when refusing compose starts that would trash the host disk if binds appear.
pub fn unsafe_host_bind_message(tool: &str) -> String {
    format!(
        "REFUSING {tool}: unsafe Docker bind root (would write under host /workspace or /). \
CEPS_HOST_ROOT is required when CEPS_API_ROOT=/workspace and must be an absolute \
host clone path - never /workspace and never /. \
Compose/Cursor launch scripts must pass -e CEPS_HOST_ROOT=<host-path>."
    )
}

/// Directory for host API pid/log files (`mcp/.run/`).
pub fn run_dir() -> PathBuf {
    repo_root().join("mcp").join(".run")
}

pub fn api_pid_path() -> PathBuf {
    run_dir().join("api.pid")
}

pub fn api_log_path() -> PathBuf {
    run_dir().join("api.log")
}

/// HTTP base URL for API tools (`CEPS_API_URL`, else default `:8080`).
pub fn api_base_url() -> String {
    env::var("CEPS_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string())
}

#[cfg(test)]
pub mod test_env {
    use std::sync::Mutex;
    /// Serialize tests that mutate `CEPS_API_ROOT` / `CEPS_HOST_ROOT` / `CEPS_API_URL`.
    pub static ENV_LOCK: Mutex<()> = Mutex::new(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::test_env::ENV_LOCK;
    use std::fs;

    #[test]
    fn repo_root_respects_env() {
        let _g = ENV_LOCK.lock().unwrap();
        let mut dir = env::temp_dir();
        dir.push(format!("ceps-api-mcp-root-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Makefile"), "help:\n\t@echo ok\n").unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname=\"x\"\nversion=\"0.0.0\"\nedition=\"2021\"\n",
        )
        .unwrap();
        env::set_var("CEPS_API_ROOT", &dir);
        assert_eq!(repo_root(), dir);
        env::remove_var("CEPS_API_ROOT");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn host_repo_root_prefers_ceps_host_root() {
        let _g = ENV_LOCK.lock().unwrap();
        env::set_var("CEPS_API_ROOT", "/workspace");
        env::set_var("CEPS_HOST_ROOT", "/tmp/ceps-rust-api-fake");
        assert_eq!(host_repo_root(), PathBuf::from("/tmp/ceps-rust-api-fake"));
        assert!(!host_bind_root_is_unsafe());
        env::remove_var("CEPS_HOST_ROOT");
        assert!(host_bind_root_is_unsafe());
        env::remove_var("CEPS_API_ROOT");
    }

    #[test]
    fn host_bind_refuses_root_and_relative() {
        let _g = ENV_LOCK.lock().unwrap();
        env::remove_var("CEPS_API_ROOT");
        env::set_var("CEPS_HOST_ROOT", "/");
        assert!(host_bind_root_is_unsafe());
        env::set_var("CEPS_HOST_ROOT", "relative/path");
        assert!(host_bind_root_is_unsafe());
        env::set_var("CEPS_HOST_ROOT", "/workspace");
        assert!(host_bind_root_is_unsafe());
        env::set_var("CEPS_HOST_ROOT", "/workspace/foo");
        assert!(host_bind_root_is_unsafe());
        env::set_var("CEPS_HOST_ROOT", "/tmp/ceps-rust-api-fake");
        assert!(!host_bind_root_is_unsafe());
        env::remove_var("CEPS_HOST_ROOT");
    }
}

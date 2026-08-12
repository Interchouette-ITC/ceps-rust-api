//! Make / Docker lifecycle helpers (parity with repository Makefile targets).

use std::fs;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

use crate::paths::{
    api_base_url, api_log_path, api_pid_path, host_bind_root_is_unsafe, host_repo_root, repo_root,
    run_dir, unsafe_host_bind_message, API_CONTAINER, DEFAULT_API_URL, DEFAULT_MCP_URL,
};

fn run(cmd: &str, args: &[&str]) -> (i32, String, String) {
    let host = host_repo_root();
    let host_s = host.to_string_lossy();
    match Command::new(cmd)
        .args(args)
        .current_dir(repo_root())
        .env("CEPS_HOST_ROOT", host_s.as_ref())
        .env("PWD", host_s.as_ref())
        .output()
    {
        Ok(out) => (
            out.status.code().unwrap_or(1),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ),
        Err(e) => (127, String::new(), e.to_string()),
    }
}

fn refuse_unsafe_host_binds(tool: &str) -> Option<String> {
    if host_bind_root_is_unsafe() {
        Some(unsafe_host_bind_message(tool))
    } else {
        None
    }
}

fn format_cmd(label: &str, code: i32, out: &str, err: &str) -> String {
    if code != 0 {
        return format!(
            "{label} failed ({code}):\n{}",
            if err.is_empty() { out } else { err }
        );
    }
    format!("{label} ok.\n{out}{err}").trim().to_string()
}

fn make_args(target: &str, features: Option<&str>) -> Vec<String> {
    let mut args = vec![target.to_string()];
    if let Some(f) = features {
        if !f.is_empty() {
            args.push(format!("FEATURES={f}"));
        }
    }
    args
}

fn make_target(target: &str, features: Option<&str>) -> String {
    let args = make_args(target, features);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let (code, out, err) = run("make", &arg_refs);
    format_cmd(&format!("make {}", args.join(" ")), code, &out, &err)
}

fn docker_ps_filter(name: &str) -> String {
    let (code, out, err) = run(
        "docker",
        &[
            "ps",
            "-a",
            "--filter",
            &format!("name={name}"),
            "--format",
            "{{.Names}}\t{{.Status}}\t{{.Ports}}",
        ],
    );
    if code != 0 {
        return format!("docker ps failed: {err}");
    }
    if out.trim().is_empty() {
        format!("{name}: (not found)")
    } else {
        out.trim().to_string()
    }
}

fn curl_get(url: &str) -> String {
    let (code, out, err) = run(
        "curl",
        &[
            "-sS",
            "-m",
            "3",
            "-o",
            "-",
            "-w",
            "\nHTTP %{http_code}",
            url,
        ],
    );
    if code != 0 {
        return format!("curl {url} failed: {err}");
    }
    out.trim().to_string()
}

/// Make help + MCP tool map.
pub fn help() -> String {
    let make_help = make_target("help", None);
    format!(
        "{make_help}\n\n\
         MCP mirrors local-dev Make (not push/version-bump):\n\
         - ceps_api_build / ceps_api_build_release / ceps_api_check / ceps_api_lint\n\
         - ceps_api_test / ceps_api_verify\n\
         - ceps_api_docker_build / ceps_api_docker_run / ceps_api_docker_run_kms / ceps_api_docker_stop\n\
         - ceps_api_version_show / ceps_api_status\n\
         - ceps_api_start / ceps_api_stop (host cargo)\n\
         HTTP tools: ceps_api_hello / ceps_api_health / ceps_api_openapi /\n\
         ceps_api_cep18_* / ceps_api_cep78_* / ceps_api_cep85_* / ceps_api_cep95_* /\n\
         ceps_api_put_transaction\n\
         MCP HTTP: {DEFAULT_MCP_URL}\n\
         Default API: {DEFAULT_API_URL}"
    )
}

pub fn build(features: Option<&str>) -> String {
    make_target("build", features)
}

pub fn build_release(features: Option<&str>) -> String {
    make_target("build-release", features)
}

pub fn check(features: Option<&str>) -> String {
    make_target("check", features)
}

pub fn lint(features: Option<&str>) -> String {
    make_target("lint", features)
}

pub fn test_suite(features: Option<&str>) -> String {
    make_target("test", features)
}

pub fn verify(features: Option<&str>) -> String {
    make_target("verify", features)
}

pub fn docker_build() -> String {
    make_target("docker-build", None)
}

pub fn docker_run() -> String {
    if let Some(msg) = refuse_unsafe_host_binds("ceps_api_docker_run") {
        return msg;
    }
    make_target("docker-run", None)
}

pub fn docker_run_kms() -> String {
    if let Some(msg) = refuse_unsafe_host_binds("ceps_api_docker_run_kms") {
        return msg;
    }
    make_target("docker-run-kms", None)
}

pub fn docker_stop() -> String {
    make_target("docker-stop", None)
}

pub fn version_show() -> String {
    make_target("version-show", None)
}

/// Start host `ceps-rust-api` in the background (pid under `mcp/.run/`).
pub fn api_start() -> String {
    let _ = fs::create_dir_all(run_dir());
    if api_pid_path().is_file() {
        if let Ok(pid_s) = fs::read_to_string(api_pid_path()) {
            let pid = pid_s.trim();
            if !pid.is_empty() {
                let (code, _, _) = run("kill", &["-0", pid]);
                if code == 0 {
                    return format!("API already running (pid {pid})");
                }
            }
        }
    }

    let root = repo_root();
    let log = api_log_path();
    let log_file = match fs::File::create(&log) {
        Ok(f) => f,
        Err(e) => return format!("cannot create api log: {e}"),
    };
    let child = Command::new("cargo")
        .args(["run", "-p", "ceps-rust-api", "--release"])
        .current_dir(&root)
        .env("DOTENV_DISABLE", "1")
        .stdout(Stdio::from(log_file.try_clone().unwrap_or(log_file)))
        .stderr(Stdio::from(
            fs::OpenOptions::new()
                .append(true)
                .open(&log)
                .unwrap_or_else(|_| fs::File::create(&log).expect("log")),
        ))
        .spawn();

    match child {
        Ok(child) => {
            let pid = child.id();
            let _ = fs::write(api_pid_path(), format!("{pid}\n"));
            // Detach: forget Child so Drop does not kill it.
            std::mem::forget(child);
            thread::sleep(Duration::from_millis(800));
            format!(
                "API starting (pid {pid}). Log: {}\nProbe: {}",
                log.display(),
                curl_get(&format!("{}/health", api_base_url().trim_end_matches('/')))
            )
        }
        Err(e) => format!("failed to spawn cargo run: {e}"),
    }
}

pub fn api_stop() -> String {
    if !api_pid_path().is_file() {
        return "no api.pid — nothing to stop".into();
    }
    let pid = fs::read_to_string(api_pid_path())
        .unwrap_or_default()
        .trim()
        .to_string();
    if pid.is_empty() {
        let _ = fs::remove_file(api_pid_path());
        return "empty api.pid removed".into();
    }
    let (code, out, err) = run("kill", &[&pid]);
    let _ = fs::remove_file(api_pid_path());
    format_cmd(&format!("kill {pid}"), code, &out, &err)
}

pub fn status() -> String {
    let mut lines = vec![
        format!("repo_root={}", repo_root().display()),
        format!("host_root={}", host_repo_root().display()),
        format!("api_url={}", api_base_url()),
        format!("mcp_url={DEFAULT_MCP_URL}"),
        docker_ps_filter(API_CONTAINER),
        docker_ps_filter("ceps-rust-api-mcp"),
        format!(
            "health: {}",
            curl_get(&format!("{}/health", api_base_url().trim_end_matches('/')))
        ),
    ];
    if api_pid_path().is_file() {
        lines.push(format!(
            "host_api_pid={}",
            fs::read_to_string(api_pid_path())
                .unwrap_or_default()
                .trim()
        ));
    }
    lines.join("\n")
}

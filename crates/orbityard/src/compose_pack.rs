//! Shared Get-live compose helpers (Docker PATH · compose CLI · heartbeat · volume recovery).
//! Spec: specs/backend/get-live-compose-packs-from-peers-design.md (G0)

use anyhow::{bail, Context, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// First compose file found in the project root.
pub fn compose_file(project: &Path) -> Option<PathBuf> {
    for name in [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
        "docker-compose.prod.yml",
        "docker-compose.prod.yaml",
    ] {
        let p = project.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn docker_candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut push_dir = |p: PathBuf| {
        if p.is_dir() && !dirs.iter().any(|d| d == &p) {
            dirs.push(p);
        }
    };

    #[cfg(windows)]
    {
        for key in [
            r"HKCU\Environment",
            r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ] {
            if let Some(path) = windows_reg_path(key) {
                for part in path.split(';').filter(|s| !s.is_empty()) {
                    push_dir(PathBuf::from(part));
                }
            }
        }
        for base in [
            std::env::var_os("ProgramFiles").map(PathBuf::from),
            std::env::var_os("ProgramW6432").map(PathBuf::from),
        ]
        .into_iter()
        .flatten()
        {
            push_dir(base.join(r"Docker\Docker\resources\bin"));
        }
        push_dir(PathBuf::from(r"C:\Program Files\Docker\Docker\resources\bin"));
        push_dir(PathBuf::from(r"F:\Docker\resources\bin"));
        push_dir(PathBuf::from(r"D:\Docker\resources\bin"));
    }

    if let Ok(path) = std::env::var("PATH") {
        #[cfg(windows)]
        let parts = path.split(';');
        #[cfg(not(windows))]
        let parts = path.split(':');
        for part in parts.filter(|s| !s.is_empty()) {
            push_dir(PathBuf::from(part));
        }
    }
    dirs
}

#[cfg(windows)]
fn windows_reg_path(key: &str) -> Option<String> {
    let out = Command::new("reg")
        .args(["query", key, "/v", "Path"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let line = line.trim();
        let marker = if line.contains("REG_EXPAND_SZ") {
            "REG_EXPAND_SZ"
        } else if line.contains("REG_SZ") {
            "REG_SZ"
        } else {
            continue;
        };
        let Some((_, rest)) = line.split_once(marker) else {
            continue;
        };
        let val = rest.trim();
        if val.is_empty() {
            continue;
        }
        let mut expanded = val.to_string();
        for (name, repl) in [
            ("%SystemRoot%", std::env::var("SystemRoot").ok()),
            ("%SYSTEMROOT%", std::env::var("SystemRoot").ok()),
            ("%ProgramFiles%", std::env::var("ProgramFiles").ok()),
            ("%ProgramFiles(x86)%", std::env::var("ProgramFiles(x86)").ok()),
            ("%USERPROFILE%", std::env::var("USERPROFILE").ok()),
        ] {
            if let Some(r) = repl {
                expanded = expanded.replace(name, &r);
            }
        }
        return Some(expanded);
    }
    None
}

fn find_in_candidate_dirs(names: &[&str]) -> Option<PathBuf> {
    for dir in docker_candidate_dirs() {
        for name in names {
            let p = dir.join(name);
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

pub fn docker_bin() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(p) = which::which("docker.exe") {
            return Ok(p);
        }
    }
    if let Ok(p) = which::which("docker").or_else(|_| which::which("docker.exe")) {
        return Ok(p);
    }
    find_in_candidate_dirs(&["docker.exe", "docker"]).context(
        "docker not found — install Docker Desktop / engine (Engine running is not enough if docker.exe is missing), then retry deploy",
    )
}

fn docker_compose_bin() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        if let Ok(p) = which::which("docker-compose.exe") {
            return Some(p);
        }
    }
    which::which("docker-compose")
        .or_else(|_| which::which("docker-compose.exe"))
        .ok()
        .or_else(|| find_in_candidate_dirs(&["docker-compose.exe", "docker-compose"]))
}

pub fn compose_up_fail_message(
    compose_label: &str,
    out: &std::process::Output,
    combined: &str,
) -> String {
    let code = out.status.code().unwrap_or(-1);
    let lower = combined.to_ascii_lowercase();
    let daemon_down = lower.contains("cannot connect to the docker")
        || lower.contains("dockerdesktoplinuxengine")
        || lower.contains("the system cannot find the file specified")
        || lower.contains("is the docker daemon running")
        || lower.contains("error during connect");
    if daemon_down {
        return format!(
            "{compose_label} up failed — Docker engine not running. Start Docker Desktop, wait until it is ready, then Deploy again."
        );
    }
    let hint = combined
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .last()
        .unwrap_or("")
        .chars()
        .take(160)
        .collect::<String>();
    if hint.is_empty() {
        format!("{compose_label} up failed (exit {code})")
    } else {
        format!("{compose_label} up failed (exit {code}) — {hint}")
    }
}

pub enum ComposeCli {
    Plugin { docker: PathBuf },
    Standalone { bin: PathBuf },
}

impl ComposeCli {
    pub fn resolve() -> Result<(Self, String)> {
        let docker = docker_bin()?;
        let plugin_ok = Command::new(&docker)
            .args(["compose", "version"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if plugin_ok {
            return Ok((Self::Plugin { docker }, "docker compose".into()));
        }
        let bin = docker_compose_bin().context(
            "neither `docker compose` nor `docker-compose` on PATH — install Docker Compose, then retry deploy",
        )?;
        Ok((Self::Standalone { bin }, "docker-compose".into()))
    }

    pub fn run(&self, project: &Path, args: &[&str]) -> Result<std::process::Output> {
        match self {
            Self::Plugin { docker } => {
                let mut full = vec!["compose"];
                full.extend(args);
                Command::new(docker)
                    .args(&full)
                    .current_dir(project)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .output()
                    .with_context(|| format!("spawn {} {}", docker.display(), full.join(" ")))
            }
            Self::Standalone { bin } => Command::new(bin)
                .args(args)
                .current_dir(project)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .with_context(|| format!("spawn {} {}", bin.display(), args.join(" "))),
        }
    }
}

/// Prefer a named compose file when present (e.g. `docker-compose.prod.yml`).
pub fn compose_up_detached(
    project: &Path,
    file: Option<&Path>,
) -> Result<(String, std::process::Output)> {
    let (cli, label) = ComposeCli::resolve()?;
    let mut args: Vec<&str> = Vec::new();
    let file_s;
    if let Some(f) = file {
        args.push("-f");
        file_s = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("docker-compose.yml")
            .to_string();
        args.push(&file_s);
    }
    args.extend(["up", "-d", "--remove-orphans"]);
    let out = cli.run(project, &args)?;
    Ok((label, out))
}

pub fn compose_logs(project: &Path, service: &str, file: Option<&Path>) -> String {
    let Ok((cli, _)) = ComposeCli::resolve() else {
        return String::new();
    };
    let mut args: Vec<&str> = Vec::new();
    let file_s;
    if let Some(f) = file {
        args.push("-f");
        file_s = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("docker-compose.yml")
            .to_string();
        args.push(&file_s);
    }
    args.extend(["logs", "--no-color", "--tail", "60", service]);
    let Ok(out) = cli.run(project, &args) else {
        return String::new();
    };
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

pub fn compose_down_volumes(
    project: &Path,
    file: Option<&Path>,
    log_prefix: &str,
) -> Result<()> {
    let (cli, label) = ComposeCli::resolve()?;
    println!("{log_prefix} · {label} down -v (reset volumes)");
    let mut args: Vec<&str> = Vec::new();
    let file_s;
    if let Some(f) = file {
        args.push("-f");
        file_s = f
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("docker-compose.yml")
            .to_string();
        args.push(&file_s);
    }
    args.extend(["down", "-v", "--remove-orphans"]);
    let out = cli.run(project, &args)?;
    if !out.status.success() {
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        bail!(
            "{label} down -v failed (exit {}) — {}",
            out.status.code().unwrap_or(-1),
            combined.lines().last().unwrap_or("").trim()
        );
    }
    Ok(())
}

/// Heuristic: Prisma / migrate / generic corrupt-volume crash loops.
pub fn volume_corrupt_from_logs(logs: &str) -> bool {
    let lower = logs.to_ascii_lowercase();
    lower.contains("p3009")
        || lower.contains("failed migrations")
        || lower.contains("migrate found failed")
        || (lower.contains("prisma migrate") && lower.contains("failed"))
}

pub fn http_status(host: &str, port: u16, path: &str) -> Result<u16> {
    let mut stream = TcpStream::connect((host, port))
        .with_context(|| format!("connect {host}:{port}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
    )?;
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;
    let text = String::from_utf8_lossy(&buf);
    text.lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .context("HTTP response missing status")
}

pub enum ProofWait {
    Ok,
    Timeout { last: String, logs: String },
    CorruptVolume { logs: String },
}

/// Poll HTTP proof URL; periodically sniff service logs for corrupt-volume signals.
pub fn wait_http_proof(
    project: &Path,
    host: &str,
    port: u16,
    path: &str,
    log_service: &str,
    compose_file: Option<&Path>,
    timeout: Duration,
) -> ProofWait {
    let start = Instant::now();
    let mut last = String::new();
    let mut next_log_check = start + Duration::from_secs(6);
    while start.elapsed() < timeout {
        match http_status(host, port, path) {
            Ok(200) | Ok(204) => return ProofWait::Ok,
            Ok(code) => last = format!("status {code}"),
            Err(e) => last = format!("{e:#}"),
        }
        if Instant::now() >= next_log_check {
            next_log_check = Instant::now() + Duration::from_secs(8);
            let logs = compose_logs(project, log_service, compose_file);
            if volume_corrupt_from_logs(&logs) {
                return ProofWait::CorruptVolume { logs };
            }
        }
        thread::sleep(Duration::from_millis(800));
    }
    let logs = compose_logs(project, log_service, compose_file);
    if volume_corrupt_from_logs(&logs) {
        return ProofWait::CorruptVolume { logs };
    }
    ProofWait::Timeout { last, logs }
}

pub fn proof_fail_message(prefix: &str, last: &str, logs: &str) -> String {
    let hint = logs
        .lines()
        .map(str::trim)
        .filter(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("error")
                || lower.contains("failed")
                || lower.contains("p3009")
                || lower.contains("migrate")
        })
        .last()
        .unwrap_or(last)
        .chars()
        .take(180)
        .collect::<String>();
    if hint.is_empty() {
        format!("{prefix} ({last})")
    } else {
        format!("{prefix} — {hint}")
    }
}

/// Weak secret heuristic shared by ensure steps (Umami · Flowsint · …).
pub fn is_weak_secret(value: &str) -> bool {
    let v = value.trim();
    if v.is_empty() || v.len() < 16 {
        return true;
    }
    let lower = v.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "umami"
            | "postgres"
            | "password"
            | "flowsint"
            | "supersecret" // Medusa medusa-config default
            | "superscretchangeitplz"
            | "replace-me"
            | "change-me"
            | "change-me-generate-with-orbit"
    ) || lower.contains("replace-me")
        || lower.contains("change-me")
        || lower.contains("changeit")
        || lower.starts_with("base64:qnh") // flowsint .env.example sample vault key prefix
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weak_secret_detects_defaults() {
        assert!(is_weak_secret("password"));
        assert!(is_weak_secret("flowsint"));
        assert!(is_weak_secret("superscretchangeitplz"));
        assert!(!is_weak_secret(&"a".repeat(24)));
    }

    #[test]
    fn volume_corrupt_prisma() {
        assert!(volume_corrupt_from_logs(
            "Error: P3009\nmigrate found failed migrations in the target database"
        ));
        assert!(!volume_corrupt_from_logs("umami · waiting for db"));
    }
}

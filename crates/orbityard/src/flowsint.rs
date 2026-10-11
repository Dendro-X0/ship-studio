//! Flowsint OSINT Get-live — multi-env ensure (F2) · deploy (F3).
//! Spec: specs/backend/flowsint-get-live-design.md

use anyhow::{bail, Context, Result};
use rand::RngCore;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::compose_pack::{
    self, compose_down_volumes, compose_up_detached, compose_up_fail_message, proof_fail_message,
    wait_http_proof, ProofWait,
};
use crate::config;

fn encode_base64(bytes: &[u8]) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= bytes.len() {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8) | (bytes[i + 2] as u32);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push(T[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = bytes.len() - i;
    if rem == 1 {
        let n = (bytes[i] as u32) << 16;
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((bytes[i] as u32) << 16) | ((bytes[i + 1] as u32) << 8);
        out.push(T[((n >> 18) & 63) as usize] as char);
        out.push(T[((n >> 12) & 63) as usize] as char);
        out.push(T[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

#[derive(Debug, Serialize)]
pub struct EnsureFileReport {
    pub path: String,
    pub created: bool,
    pub updated_keys: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EnsureReport {
    pub ok: bool,
    pub project: String,
    pub files: Vec<EnsureFileReport>,
    pub message: String,
}

fn parse_env_map(raw: &str) -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            out.push((String::new(), line.to_string(), true));
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            out.push((k.trim().to_string(), v.trim().to_string(), false));
        } else {
            out.push((String::new(), line.to_string(), true));
        }
    }
    out
}

fn render_env(lines: &[(String, String, bool)]) -> String {
    let mut s = String::new();
    for (k, v, passthrough) in lines {
        if *passthrough {
            s.push_str(v);
            s.push('\n');
        } else {
            s.push_str(k);
            s.push('=');
            s.push_str(v);
            s.push('\n');
        }
    }
    s
}

fn upsert_key(lines: &mut Vec<(String, String, bool)>, key: &str, value: String) -> bool {
    for (k, v, passthrough) in lines.iter_mut() {
        if !*passthrough && k == key {
            if v == &value {
                return false;
            }
            *v = value;
            return true;
        }
    }
    lines.push((key.to_string(), value, false));
    true
}

fn get_key<'a>(lines: &'a [(String, String, bool)], key: &str) -> Option<&'a str> {
    lines
        .iter()
        .find(|(k, _, p)| !*p && k == key)
        .map(|(_, v, _)| v.as_str())
}

fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn random_vault_key() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    format!("base64:{}", encode_base64(&buf))
}

fn is_weak_vault(value: &str) -> bool {
    compose_pack::is_weak_secret(value)
        || value.trim().eq_ignore_ascii_case(
            "base64:qnHTmwYb+uoygIw9MsRMY22vS5YPchY+QOi/E79GAvM=",
        )
}

/// Target `.env` paths: root always; module copies when those dirs exist (official clone).
fn env_targets(project: &Path) -> Vec<PathBuf> {
    let mut out = vec![project.join(".env")];
    for rel in ["flowsint-api", "flowsint-core", "flowsint-app"] {
        let dir = project.join(rel);
        if dir.is_dir() {
            out.push(dir.join(".env"));
        }
    }
    out
}

fn load_template(project: &Path, target: &Path) -> Result<String> {
    let local_example = target
        .parent()
        .map(|p| p.join(".env.example"))
        .unwrap_or_else(|| project.join(".env.example"));
    let root_example = project.join(".env.example");
    if target.is_file() {
        return fs::read_to_string(target).with_context(|| format!("read {}", target.display()));
    }
    if local_example.is_file() {
        return fs::read_to_string(&local_example)
            .with_context(|| format!("read {}", local_example.display()));
    }
    if root_example.is_file() {
        return fs::read_to_string(&root_example)
            .with_context(|| format!("read {}", root_example.display()));
    }
    Ok(
        "# Orbit Yard flowsint ensure\nAUTH_SECRET=\nMASTER_VAULT_KEY_V1=\nNEO4J_PASSWORD=\nPOSTGRES_PASSWORD=\n"
            .into(),
    )
}

fn patch_database_url(lines: &mut Vec<(String, String, bool)>, postgres_password: &str) {
    let user = get_key(lines, "POSTGRES_USER")
        .unwrap_or("flowsint")
        .to_string();
    let db = get_key(lines, "POSTGRES_DB")
        .unwrap_or("flowsint")
        .to_string();
    let Some(url) = get_key(lines, "DATABASE_URL").map(str::to_string) else {
        // Prod compose builds DATABASE_URL from POSTGRES_* — optional.
        return;
    };
    // Rewrite password segment in postgresql://user:pass@host/db
    if let Some(rest) = url.strip_prefix("postgresql://").or_else(|| url.strip_prefix("postgres://"))
    {
        if let Some((creds, host_and_db)) = rest.split_once('@') {
            let user_part = creds.split(':').next().unwrap_or(user.as_str());
            let scheme = if url.starts_with("postgres://") {
                "postgres"
            } else {
                "postgresql"
            };
            let new_url = format!("{scheme}://{user_part}:{postgres_password}@{host_and_db}");
            let _ = upsert_key(lines, "DATABASE_URL", new_url);
            return;
        }
    }
    let _ = upsert_key(
        lines,
        "DATABASE_URL",
        format!("postgresql://{user}:{postgres_password}@postgres:5432/{db}"),
    );
}

struct SecretPlan {
    auth_secret: Option<String>,
    vault_key: Option<String>,
    neo4j_password: Option<String>,
    postgres_password: Option<String>,
}

fn plan_secrets(sample: &[(String, String, bool)]) -> SecretPlan {
    let mut plan = SecretPlan {
        auth_secret: None,
        vault_key: None,
        neo4j_password: None,
        postgres_password: None,
    };
    let auth = get_key(sample, "AUTH_SECRET").unwrap_or("");
    if compose_pack::is_weak_secret(auth) {
        plan.auth_secret = Some(random_hex(32));
    }
    let vault = get_key(sample, "MASTER_VAULT_KEY_V1").unwrap_or("");
    if is_weak_vault(vault) {
        plan.vault_key = Some(random_vault_key());
    }
    let neo = get_key(sample, "NEO4J_PASSWORD").unwrap_or("");
    if compose_pack::is_weak_secret(neo) {
        plan.neo4j_password = Some(random_hex(24));
    }
    let pg = get_key(sample, "POSTGRES_PASSWORD").unwrap_or("");
    if compose_pack::is_weak_secret(pg) {
        plan.postgres_password = Some(random_hex(24));
    }
    plan
}

fn apply_plan(
    lines: &mut Vec<(String, String, bool)>,
    plan: &SecretPlan,
) -> Vec<String> {
    let mut updated = Vec::new();
    if let Some(v) = &plan.auth_secret {
        if upsert_key(lines, "AUTH_SECRET", v.clone()) {
            updated.push("AUTH_SECRET".into());
        }
    }
    if let Some(v) = &plan.vault_key {
        if upsert_key(lines, "MASTER_VAULT_KEY_V1", v.clone()) {
            updated.push("MASTER_VAULT_KEY_V1".into());
        }
    }
    if let Some(v) = &plan.neo4j_password {
        if upsert_key(lines, "NEO4J_PASSWORD", v.clone()) {
            updated.push("NEO4J_PASSWORD".into());
        }
    }
    if let Some(v) = &plan.postgres_password {
        if upsert_key(lines, "POSTGRES_PASSWORD", v.clone()) {
            updated.push("POSTGRES_PASSWORD".into());
        }
        let before = get_key(lines, "DATABASE_URL").map(str::to_string);
        patch_database_url(lines, v);
        let after = get_key(lines, "DATABASE_URL").map(str::to_string);
        if before != after && !updated.iter().any(|k| k == "DATABASE_URL") {
            updated.push("DATABASE_URL".into());
        }
    }
    updated
}

/// Write or repair root (+ module) `.env` files with strong secrets. Never prints values.
pub fn ensure(project: &Path) -> Result<EnsureReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let detected = config::probe(&project);
    if !detected.flowsint {
        bail!(
            "not a Flowsint layout at {} — bind the Flowsint clone (or `.ship/layout` = osint.flowsint / fixtures/flowsint-osint)",
            project.display()
        );
    }

    let targets = env_targets(&project);
    // Plan from root template so all files share the same rotated secrets.
    let root_raw = load_template(&project, &project.join(".env"))?;
    let root_lines = parse_env_map(&root_raw);
    let plan = plan_secrets(&root_lines);

    let mut files = Vec::new();
    let mut any_created = false;
    let mut all_keys: Vec<String> = Vec::new();

    for target in &targets {
        let created = !target.is_file();
        let raw = load_template(&project, target)?;
        let mut lines = parse_env_map(&raw);
        let updated_keys = apply_plan(&mut lines, &plan);
        if created || !updated_keys.is_empty() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("mkdir {}", parent.display()))?;
            }
            let body = render_env(&lines);
            fs::write(target, &body).with_context(|| format!("write {}", target.display()))?;
        }
        if created {
            any_created = true;
        }
        for k in &updated_keys {
            if !all_keys.iter().any(|x| x == k) {
                all_keys.push(k.clone());
            }
        }
        files.push(EnsureFileReport {
            path: target.display().to_string(),
            created,
            updated_keys,
        });
    }

    let message = if any_created && !all_keys.is_empty() {
        format!(
            "created/repaired {} .env file(s); rotated {} — next: flowsint deploy",
            files.len(),
            all_keys.join(" · ")
        )
    } else if all_keys.is_empty() {
        format!(
            "{} .env file(s) already have strong secrets — next: flowsint deploy",
            files.len()
        )
    } else {
        format!(
            "updated {} across {} .env file(s) — next: flowsint deploy",
            all_keys.join(" · "),
            files.len()
        )
    };

    Ok(EnsureReport {
        ok: true,
        project: project.display().to_string(),
        files,
        message,
    })
}

#[derive(Debug, Serialize)]
pub struct DeployReport {
    pub ok: bool,
    pub project: String,
    pub app_url: Option<String>,
    pub api_health_ok: bool,
    pub message: String,
}

fn prod_compose_file(project: &Path) -> Option<PathBuf> {
    for name in ["docker-compose.prod.yml", "docker-compose.prod.yaml"] {
        let p = project.join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn write_last_run(
    project: &Path,
    started: &str,
    ok: bool,
    message: String,
    urls: Vec<String>,
) -> Result<()> {
    config::write_last_run(
        project,
        &config::LastRun {
            finished: true,
            ok,
            started_at: started.to_string(),
            finished_at: config::now_rfc3339(),
            dry_run: false,
            offline: true,
            steps: vec![config::StepResult {
                id: "flowsint.deploy".into(),
                ok,
                exit_code: if ok { 0 } else { 1 },
                detail: message.clone(),
            }],
            message,
            urls,
            host_provider: Some("flowsint".into()),
        },
    )
}

fn finish_deploy_ok(
    project: &Path,
    started: &str,
    app_url: &str,
    api_health_ok: bool,
) -> Result<DeployReport> {
    println!("flowsint · App ready · {app_url}");
    println!("flowsint · Confirm: create account at /register (no default credentials)");
    let message = if api_health_ok {
        format!("flowsint deploy ok · {app_url} · api /health ok")
    } else {
        format!("flowsint deploy ok · {app_url}")
    };
    write_last_run(
        project,
        started,
        true,
        message.clone(),
        vec![app_url.to_string()],
    )?;
    Ok(DeployReport {
        ok: true,
        project: project.display().to_string(),
        app_url: Some(app_url.to_string()),
        api_health_ok,
        message,
    })
}

fn api_health_ok() -> bool {
    matches!(
        compose_pack::http_status("127.0.0.1", 5001, "/health"),
        Ok(200) | Ok(204)
    )
}

fn run_compose_up(
    project: &Path,
    compose: &Path,
    log_label: &str,
) -> Result<(String, std::process::Output), String> {
    match compose_up_detached(project, Some(compose)) {
        Ok((label, out)) => {
            println!(
                "flowsint · {label} -f {} {log_label}",
                compose
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("compose")
            );
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            for line in combined.lines().take(40) {
                println!("flowsint · {line}");
            }
            if !out.status.success() {
                return Err(compose_up_fail_message(&label, &out, &combined));
            }
            Ok((label, out))
        }
        Err(e) => Err(format!("{e:#}")),
    }
}

/// Ensure secrets → `compose -f docker-compose.prod.yml up -d` → wait App URL.
pub fn deploy(project: &Path) -> Result<DeployReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let started = config::now_rfc3339();
    println!("flowsint · ensure secrets");
    let ensured = ensure(&project)?;
    println!("flowsint · {}", ensured.message);

    let Some(compose) = prod_compose_file(&project) else {
        let message: String =
            "no docker-compose.prod.yml — Flowsint Get-live expects the prod compose file".into();
        let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
        return Ok(DeployReport {
            ok: false,
            project: project.display().to_string(),
            app_url: None,
            api_health_ok: false,
            message,
        });
    };

    let app_url = "http://127.0.0.1:5173".to_string();

    if let Err(message) = run_compose_up(&project, &compose, "up -d") {
        println!("flowsint · {message}");
        let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
        return Ok(DeployReport {
            ok: false,
            project: project.display().to_string(),
            app_url: None,
            api_health_ok: false,
            message,
        });
    }

    // Prefer /register (first-login); fall back is same host root.
    println!("flowsint · wait App {app_url}/register");
    let proof = wait_http_proof(
        &project,
        "127.0.0.1",
        5173,
        "/register",
        "api",
        Some(&compose),
        Duration::from_secs(240),
    );
    let proof = match proof {
        ProofWait::Timeout { last, logs } => {
            println!("flowsint · /register not ready ({last}) — trying /");
            match wait_http_proof(
                &project,
                "127.0.0.1",
                5173,
                "/",
                "app",
                Some(&compose),
                Duration::from_secs(60),
            ) {
                ProofWait::Ok => ProofWait::Ok,
                other => {
                    // Prefer richer logs from the first wait when both timed out.
                    match other {
                        ProofWait::Timeout { last: last2, logs: logs2 } => {
                            if logs2.len() > logs.len() {
                                ProofWait::Timeout {
                                    last: last2,
                                    logs: logs2,
                                }
                            } else {
                                ProofWait::Timeout { last, logs }
                            }
                        }
                        v => v,
                    }
                }
            }
        }
        other => other,
    };

    match proof {
        ProofWait::Ok => {
            let api_ok = api_health_ok();
            if api_ok {
                println!("flowsint · api /health ok");
            } else {
                println!("flowsint · api /health not reachable yet (app UI is enough for Get-live)");
            }
            finish_deploy_ok(&project, &started, &app_url, api_ok)
        }
        ProofWait::CorruptVolume { logs } => {
            println!("flowsint · detected failed migrate / corrupt volume — resetting once");
            for line in logs.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev() {
                println!("flowsint · {line}");
            }
            if let Err(e) = compose_down_volumes(&project, Some(&compose), "flowsint") {
                let message = format!(
                    "flowsint deploy: volume reset failed — {e:#}"
                );
                let _ = write_last_run(
                    &project,
                    &started,
                    false,
                    message.clone(),
                    vec![app_url.clone()],
                );
                return Ok(DeployReport {
                    ok: false,
                    project: project.display().to_string(),
                    app_url: Some(app_url),
                    api_health_ok: false,
                    message,
                });
            }
            if let Err(message) = run_compose_up(&project, &compose, "up -d (after volume reset)") {
                let _ = write_last_run(
                    &project,
                    &started,
                    false,
                    message.clone(),
                    vec![app_url.clone()],
                );
                return Ok(DeployReport {
                    ok: false,
                    project: project.display().to_string(),
                    app_url: Some(app_url),
                    api_health_ok: false,
                    message,
                });
            }
            println!("flowsint · wait App {app_url}/register");
            match wait_http_proof(
                &project,
                "127.0.0.1",
                5173,
                "/register",
                "api",
                Some(&compose),
                Duration::from_secs(240),
            ) {
                ProofWait::Ok => {
                    let api_ok = api_health_ok();
                    finish_deploy_ok(&project, &started, &app_url, api_ok)
                }
                ProofWait::CorruptVolume { logs } | ProofWait::Timeout { last: _, logs } => {
                    let message = proof_fail_message(
                        "flowsint deploy: App still unhealthy after volume reset",
                        "proof timeout",
                        &logs,
                    );
                    let _ = write_last_run(
                        &project,
                        &started,
                        false,
                        message.clone(),
                        vec![app_url.clone()],
                    );
                    Ok(DeployReport {
                        ok: false,
                        project: project.display().to_string(),
                        app_url: Some(app_url),
                        api_health_ok: false,
                        message,
                    })
                }
            }
        }
        ProofWait::Timeout { last, logs } => {
            let message = proof_fail_message(
                &format!("flowsint deploy: App proof timeout ({last})"),
                &last,
                &logs,
            );
            let _ = write_last_run(
                &project,
                &started,
                false,
                message.clone(),
                vec![app_url.clone()],
            );
            Ok(DeployReport {
                ok: false,
                project: project.display().to_string(),
                app_url: Some(app_url),
                api_health_ok: false,
                message,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "orbityard-flowsint-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn ensure_fixture_rotates_root_secrets() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/flowsint-osint");
        // Work on a copy so we don't dirty the committed fixture.
        let dir = tmp("fixture-copy");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/layout"), "osint.flowsint\n").unwrap();
        fs::copy(
            fixture.join(".env.example"),
            dir.join(".env.example"),
        )
        .unwrap();
        fs::copy(
            fixture.join("docker-compose.prod.yml"),
            dir.join("docker-compose.prod.yml"),
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert_eq!(r.files.len(), 1);
        assert!(r.files[0].created);
        assert!(r.files[0]
            .updated_keys
            .iter()
            .any(|k| k == "AUTH_SECRET"));
        assert!(r.files[0]
            .updated_keys
            .iter()
            .any(|k| k == "MASTER_VAULT_KEY_V1"));
        assert!(r.files[0]
            .updated_keys
            .iter()
            .any(|k| k == "NEO4J_PASSWORD"));
        assert!(r.files[0]
            .updated_keys
            .iter()
            .any(|k| k == "POSTGRES_PASSWORD"));

        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(!body.contains("superscretchangeitplz"));
        assert!(!body.contains("password\n") && !body.contains("=password"));
        assert!(!body.contains("POSTGRES_PASSWORD=flowsint"));
        assert!(!body.contains("base64:qnHTmwYb"));
        for line in body.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k == "AUTH_SECRET" || k == "NEO4J_PASSWORD" || k == "POSTGRES_PASSWORD" {
                    assert!(!compose_pack::is_weak_secret(v), "weak {k}");
                }
                if k == "MASTER_VAULT_KEY_V1" {
                    assert!(v.starts_with("base64:"));
                    assert!(!is_weak_vault(v), "weak vault");
                }
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_multi_module_dirs() {
        let dir = tmp("multi");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/layout"), "osint.flowsint\n").unwrap();
        for rel in ["flowsint-api", "flowsint-core", "flowsint-app"] {
            fs::create_dir_all(dir.join(rel)).unwrap();
        }
        fs::write(
            dir.join(".env.example"),
            "AUTH_SECRET=superscretchangeitplz\nMASTER_VAULT_KEY_V1=base64:qnHTmwYb+uoygIw9MsRMY22vS5YPchY+QOi/E79GAvM=\nNEO4J_PASSWORD=password\nPOSTGRES_PASSWORD=flowsint\nDATABASE_URL=postgresql://flowsint:flowsint@postgres:5432/flowsint\n",
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert_eq!(r.files.len(), 4);
        let auth_vals: Vec<_> = r
            .files
            .iter()
            .map(|f| {
                let body = fs::read_to_string(&f.path).unwrap();
                body.lines()
                    .find_map(|l| l.strip_prefix("AUTH_SECRET=").map(str::to_string))
                    .unwrap()
            })
            .collect();
        assert!(auth_vals.iter().all(|v| v == &auth_vals[0]));
        assert!(!compose_pack::is_weak_secret(&auth_vals[0]));
        let db_url = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(!db_url.contains("flowsint:flowsint@"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_skips_strong_existing() {
        let dir = tmp("strong");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/layout"), "osint.flowsint\n").unwrap();
        let auth = "a".repeat(64);
        let neo = "b".repeat(24);
        let pg = "c".repeat(24);
        let vault = random_vault_key();
        fs::write(
            dir.join(".env"),
            format!(
                "AUTH_SECRET={auth}\nMASTER_VAULT_KEY_V1={vault}\nNEO4J_PASSWORD={neo}\nPOSTGRES_PASSWORD={pg}\n"
            ),
        )
        .unwrap();
        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(!r.files[0].created);
        assert!(r.files[0].updated_keys.is_empty());
        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(body.contains(&auth));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn prod_compose_resolves_fixture() {
        let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/flowsint-osint");
        let p = prod_compose_file(&fixture).expect("prod compose");
        assert!(p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .contains("prod"));
    }

    #[test]
    fn deploy_rejects_non_flowsint() {
        let dir = tmp("not-flowsint");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let err = deploy(&dir).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.to_ascii_lowercase().contains("flowsint"),
            "unexpected: {msg}"
        );
        let _ = fs::remove_dir_all(&dir);
    }
}

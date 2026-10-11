//! Umami analytics Get-live — ensure secrets · deploy (U2).
//! Spec: specs/backend/umami-get-live-design.md

use anyhow::{bail, Context, Result};
use rand::RngCore;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::compose_pack::{
    self, compose_down_volumes, compose_file, compose_up_detached, compose_up_fail_message,
    proof_fail_message, wait_http_proof, ProofWait,
};
use crate::config;

#[derive(Debug, Serialize)]
pub struct EnsureReport {
    pub ok: bool,
    pub project: String,
    pub env_path: String,
    pub created: bool,
    pub updated_keys: Vec<String>,
    pub message: String,
}

fn env_path(project: &Path) -> PathBuf {
    project.join(".env")
}

fn is_weak(value: &str) -> bool {
    compose_pack::is_weak_secret(value)
}

fn random_secret(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

fn parse_env_map(raw: &str) -> Vec<(String, String, bool)> {
    // (key, value, is_comment_or_blank line preserved as key="")
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

/// Write or repair `.env` with strong `UMAMI_*` secrets. Never prints secret values.
pub fn ensure(project: &Path) -> Result<EnsureReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let detected = config::probe(&project);
    if !detected.umami {
        bail!(
            "not an Umami layout at {} — bind the folder that contains the Umami compose file (or `.ship/layout` = analytics.umami), not the monorepo root",
            project.display()
        );
    }

    let path = env_path(&project);
    let example = project.join(".env.example");
    let created = !path.is_file();
    let mut raw = if path.is_file() {
        fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?
    } else if example.is_file() {
        fs::read_to_string(&example).with_context(|| format!("read {}", example.display()))?
    } else {
        "# Orbit Yard umami ensure\nUMAMI_DB_PASSWORD=\nUMAMI_APP_SECRET=\n".into()
    };

    let mut lines = parse_env_map(&raw);
    let mut updated_keys = Vec::new();

    let db = get_key(&lines, "UMAMI_DB_PASSWORD").unwrap_or("");
    if is_weak(db) {
        let secret = random_secret(24);
        if upsert_key(&mut lines, "UMAMI_DB_PASSWORD", secret) {
            updated_keys.push("UMAMI_DB_PASSWORD".into());
        }
    }

    let app = get_key(&lines, "UMAMI_APP_SECRET").unwrap_or("");
    if is_weak(app) {
        let secret = random_secret(32);
        if upsert_key(&mut lines, "UMAMI_APP_SECRET", secret) {
            updated_keys.push("UMAMI_APP_SECRET".into());
        }
    }

    if !updated_keys.is_empty() || created {
        raw = render_env(&lines);
        fs::write(&path, &raw).with_context(|| format!("write {}", path.display()))?;
    }

    let message = if created {
        format!(
            "created .env with {} secret(s) — not committed; next: umami deploy",
            updated_keys.len()
        )
    } else if updated_keys.is_empty() {
        ".env already has strong UMAMI_* secrets — next: umami deploy".into()
    } else {
        format!(
            "updated {} in .env (placeholders replaced) — next: umami deploy",
            updated_keys.join(" · ")
        )
    };

    Ok(EnsureReport {
        ok: true,
        project: project.display().to_string(),
        env_path: path.display().to_string(),
        created,
        updated_keys,
        message,
    })
}

#[derive(Debug, Serialize)]
pub struct DeployReport {
    pub ok: bool,
    pub project: String,
    pub admin_url: Option<String>,
    pub heartbeat_ok: bool,
    pub message: String,
}

/// Ensure secrets → `docker compose up -d` → wait Admin heartbeat → last-run URL.
pub fn deploy(project: &Path) -> Result<DeployReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let started = config::now_rfc3339();
    println!("umami · ensure secrets");
    let ensured = ensure(&project)?;
    println!("umami · {}", ensured.message);

    let Some(_compose) = compose_file(&project) else {
        bail!("no docker-compose.yml / compose.yml in project");
    };
    let admin_url = "http://127.0.0.1:3000".to_string();

    let (compose_label, out) = match compose_up_detached(&project, None) {
        Ok(v) => v,
        Err(e) => {
            // Soft report so Desktop toast/Output show the real cause (not a bare exit).
            let message = format!("{e:#}");
            println!("umami · {message}");
            let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
            return Ok(DeployReport {
                ok: false,
                project: project.display().to_string(),
                admin_url: None,
                heartbeat_ok: false,
                message,
            });
        }
    };
    println!("umami · {compose_label} up -d");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    for line in combined.lines().take(40) {
        println!("umami · {line}");
    }
    if !out.status.success() {
        let message = compose_up_fail_message(&compose_label, &out, &combined);
        println!("umami · {message}");
        let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
        return Ok(DeployReport {
            ok: false,
            project: project.display().to_string(),
            admin_url: None,
            heartbeat_ok: false,
            message,
        });
    }

    println!("umami · wait heartbeat {admin_url}/api/heartbeat");
    match wait_http_proof(
        &project,
        "127.0.0.1",
        3000,
        "/api/heartbeat",
        "umami",
        None,
        Duration::from_secs(120),
    ) {
        ProofWait::Ok => return finish_deploy_ok(&project, &started, &admin_url),
        ProofWait::CorruptVolume { logs } => {
            println!("umami · detected failed Prisma migrate in DB volume — resetting once");
            for line in logs.lines().rev().take(8).collect::<Vec<_>>().into_iter().rev() {
                println!("umami · {line}");
            }
            if let Err(e) = compose_down_volumes(&project, None, "umami") {
                let message = format!(
                    "umami deploy: DB volume has a failed migration, and reset failed — {e:#}"
                );
                let _ = write_last_run(
                    &project,
                    &started,
                    false,
                    message.clone(),
                    vec![admin_url.clone()],
                );
                return Ok(DeployReport {
                    ok: false,
                    project: project.display().to_string(),
                    admin_url: Some(admin_url),
                    heartbeat_ok: false,
                    message,
                });
            }
            let (compose_label, out) = match compose_up_detached(&project, None) {
                Ok(v) => v,
                Err(e) => {
                    let message = format!("umami deploy: after volume reset, compose failed — {e:#}");
                    let _ = write_last_run(
                        &project,
                        &started,
                        false,
                        message.clone(),
                        vec![admin_url.clone()],
                    );
                    return Ok(DeployReport {
                        ok: false,
                        project: project.display().to_string(),
                        admin_url: Some(admin_url),
                        heartbeat_ok: false,
                        message,
                    });
                }
            };
            println!("umami · {compose_label} up -d (after volume reset)");
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            for line in combined.lines().take(40) {
                println!("umami · {line}");
            }
            if !out.status.success() {
                let message = compose_up_fail_message(&compose_label, &out, &combined);
                let _ = write_last_run(
                    &project,
                    &started,
                    false,
                    message.clone(),
                    vec![admin_url.clone()],
                );
                return Ok(DeployReport {
                    ok: false,
                    project: project.display().to_string(),
                    admin_url: Some(admin_url),
                    heartbeat_ok: false,
                    message,
                });
            }
            println!("umami · wait heartbeat {admin_url}/api/heartbeat");
            match wait_http_proof(
                &project,
                "127.0.0.1",
                3000,
                "/api/heartbeat",
                "umami",
                None,
                Duration::from_secs(120),
            ) {
                ProofWait::Ok => return finish_deploy_ok(&project, &started, &admin_url),
                ProofWait::CorruptVolume { logs } | ProofWait::Timeout { last: _, logs } => {
                    let message = proof_fail_message(
                        "umami deploy: Admin still unhealthy after DB volume reset",
                        "heartbeat timeout",
                        &logs,
                    );
                    let _ = write_last_run(
                        &project,
                        &started,
                        false,
                        message.clone(),
                        vec![admin_url.clone()],
                    );
                    return Ok(DeployReport {
                        ok: false,
                        project: project.display().to_string(),
                        admin_url: Some(admin_url),
                        heartbeat_ok: false,
                        message,
                    });
                }
            }
        }
        ProofWait::Timeout { last, logs } => {
            let message = proof_fail_message(
                &format!("umami deploy: heartbeat timeout after 120s ({last})"),
                &last,
                &logs,
            );
            let _ = write_last_run(
                &project,
                &started,
                false,
                message.clone(),
                vec![admin_url.clone()],
            );
            Ok(DeployReport {
                ok: false,
                project: project.display().to_string(),
                admin_url: Some(admin_url),
                heartbeat_ok: false,
                message,
            })
        }
    }
}

fn finish_deploy_ok(project: &Path, started: &str, admin_url: &str) -> Result<DeployReport> {
    println!("umami · Admin ready · {admin_url}");
    println!("umami · Confirm: change default admin password in the UI");
    let message = format!("umami deploy ok · {admin_url}");
    write_last_run(
        project,
        started,
        true,
        message.clone(),
        vec![admin_url.to_string()],
    )?;
    Ok(DeployReport {
        ok: true,
        project: project.display().to_string(),
        admin_url: Some(admin_url.to_string()),
        heartbeat_ok: true,
        message,
    })
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
                id: "umami.deploy".into(),
                ok,
                exit_code: if ok { 0 } else { 1 },
                detail: message.clone(),
            }],
            message,
            urls,
            host_provider: Some("umami".into()),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "orbityard-umami-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn ensure_creates_strong_env() {
        let dir = tmp("ensure");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/layout"), "analytics.umami\n").unwrap();
        fs::write(
            dir.join("docker-compose.yml"),
            "services:\n  umami:\n    image: ghcr.io/umami-software/umami:postgresql-latest\n",
        )
        .unwrap();
        fs::write(
            dir.join(".env.example"),
            "UMAMI_DB_PASSWORD=replace-me\nUMAMI_APP_SECRET=replace-me\n",
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(r.created);
        assert!(r.updated_keys.contains(&"UMAMI_DB_PASSWORD".into()));
        assert!(r.updated_keys.contains(&"UMAMI_APP_SECRET".into()));

        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(!body.contains("replace-me"));
        for line in body.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k == "UMAMI_DB_PASSWORD" || k == "UMAMI_APP_SECRET" {
                    assert!(!is_weak(v), "weak {k}");
                }
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_skips_strong_existing() {
        let dir = tmp("strong");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("docker-compose.yml"),
            "services:\n  umami:\n    image: ghcr.io/umami-software/umami:postgresql-latest\n",
        )
        .unwrap();
        let strong_db = "a".repeat(24);
        let strong_app = "b".repeat(32);
        fs::write(
            dir.join(".env"),
            format!("UMAMI_DB_PASSWORD={strong_db}\nUMAMI_APP_SECRET={strong_app}\n"),
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(!r.created);
        assert!(r.updated_keys.is_empty());
        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(body.contains(&strong_db));
        let _ = fs::remove_dir_all(&dir);
    }
}

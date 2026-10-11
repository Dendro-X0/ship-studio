//! Medusa commerce Get-live — ensure env secrets (M2) · deploy proof (M3).
//! Spec: specs/backend/medusa-get-live-design.md

use anyhow::{bail, Context, Result};
use rand::RngCore;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::compose_pack::{
    self, compose_file, compose_logs, compose_up_detached, compose_up_fail_message,
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
    /// Keys that still need a human Put (e.g. empty DATABASE_URL). Never secret values.
    pub needs_put: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct DeployReport {
    pub ok: bool,
    pub project: String,
    pub admin_url: Option<String>,
    pub heartbeat_ok: bool,
    pub message: String,
}

fn env_path(project: &Path) -> PathBuf {
    project.join(".env")
}

fn is_weak(value: &str) -> bool {
    compose_pack::is_weak_secret(value)
}

fn is_weak_url(value: &str) -> bool {
    let v = value.trim();
    v.is_empty()
        || v.eq_ignore_ascii_case("replace-me")
        || v.contains("change-me")
        || v.contains("YOUR_")
        || v.contains("<")
}

fn random_secret(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    buf.iter().map(|b| format!("{b:02x}")).collect()
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

fn ensure_key_present(lines: &mut Vec<(String, String, bool)>, key: &str) -> bool {
    if get_key(lines, key).is_some() {
        return false;
    }
    lines.push((key.to_string(), String::new(), false));
    true
}

fn get_key<'a>(lines: &'a [(String, String, bool)], key: &str) -> Option<&'a str> {
    lines
        .iter()
        .find(|(k, _, p)| !*p && k == key)
        .map(|(_, v, _)| v.as_str())
}

fn compose_has_postgres(project: &Path) -> bool {
    let Some(path) = compose_file(project) else {
        return false;
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return false;
    };
    let lower = raw.to_ascii_lowercase();
    lower.contains("postgres:") || lower.contains("image: postgres")
}

fn compose_has_medusa_service(project: &Path) -> bool {
    let Some(path) = compose_file(project) else {
        return false;
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return false;
    };
    // Service key `medusa:` (not just a comment).
    raw.lines().any(|l| {
        let t = l.trim();
        t == "medusa:" || t.starts_with("medusa:")
    })
}

/// Write or repair `.env` with strong Medusa secrets. Never prints secret values.
/// With compose Postgres, wires `DATABASE_URL` from `POSTGRES_PASSWORD` (host localhost).
pub fn ensure(project: &Path) -> Result<EnsureReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let detected = config::probe(&project);
    if !detected.medusa {
        bail!(
            "not a Medusa layout at {} — bind the folder with medusa-config / @medusajs/* (or `.ship/layout` = commerce.medusa)",
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
        "# Orbit Yard medusa ensure\nPOSTGRES_PASSWORD=\nDATABASE_URL=\nREDIS_URL=\nJWT_SECRET=\nCOOKIE_SECRET=\nSTORE_CORS=\nADMIN_CORS=\nAUTH_CORS=\n".into()
    };

    let mut lines = parse_env_map(&raw);
    let mut updated_keys = Vec::new();

    // Secrets — rotate when weak; keep JWT ≠ COOKIE.
    let jwt_existing = get_key(&lines, "JWT_SECRET")
        .unwrap_or("")
        .to_string();
    let cookie_existing = get_key(&lines, "COOKIE_SECRET")
        .unwrap_or("")
        .to_string();
    let jwt_val = if is_weak(&jwt_existing) {
        let secret = random_secret(32);
        if upsert_key(&mut lines, "JWT_SECRET", secret.clone()) {
            updated_keys.push("JWT_SECRET".into());
        }
        secret
    } else {
        jwt_existing
    };
    let cookie_weak = is_weak(&cookie_existing) || cookie_existing == jwt_val;
    if cookie_weak {
        let mut secret = random_secret(32);
        while secret == jwt_val {
            secret = random_secret(32);
        }
        if upsert_key(&mut lines, "COOKIE_SECRET", secret) {
            updated_keys.push("COOKIE_SECRET".into());
        }
    }

    if ensure_key_present(&mut lines, "DATABASE_URL") {
        updated_keys.push("DATABASE_URL".into());
    }
    if ensure_key_present(&mut lines, "REDIS_URL") {
        updated_keys.push("REDIS_URL".into());
    }

    // Compose Postgres → rotate password and wire host DATABASE_URL / REDIS_URL.
    if compose_has_postgres(&project) {
        let pg = get_key(&lines, "POSTGRES_PASSWORD")
            .unwrap_or("")
            .to_string();
        let pg_pass = if is_weak(&pg) {
            let secret = random_secret(24);
            if upsert_key(&mut lines, "POSTGRES_PASSWORD", secret.clone()) {
                updated_keys.push("POSTGRES_PASSWORD".into());
            }
            secret
        } else {
            pg
        };
        let db_url = format!("postgres://medusa:{pg_pass}@localhost:5432/medusa");
        if upsert_key(&mut lines, "DATABASE_URL", db_url) {
            if !updated_keys.iter().any(|k| k == "DATABASE_URL") {
                updated_keys.push("DATABASE_URL".into());
            }
        }
        let redis = get_key(&lines, "REDIS_URL").unwrap_or("");
        if is_weak_url(redis) {
            if upsert_key(&mut lines, "REDIS_URL", "redis://localhost:6379".into()) {
                if !updated_keys.iter().any(|k| k == "REDIS_URL") {
                    updated_keys.push("REDIS_URL".into());
                }
            }
        }
    }

    const CORS: &[(&str, &str)] = &[
        ("STORE_CORS", "http://localhost:8000"),
        ("ADMIN_CORS", "http://localhost:9000"),
        ("AUTH_CORS", "http://localhost:9000,http://localhost:8000"),
    ];
    for (key, default) in CORS {
        let cur = get_key(&lines, key).unwrap_or("");
        if cur.trim().is_empty() {
            if upsert_key(&mut lines, key, (*default).into()) {
                updated_keys.push((*key).into());
            }
        }
    }

    if get_key(&lines, "PORT").is_none() {
        if upsert_key(&mut lines, "PORT", "9000".into()) {
            updated_keys.push("PORT".into());
        }
    }

    let mut needs_put = Vec::new();
    let db = get_key(&lines, "DATABASE_URL").unwrap_or("");
    if is_weak_url(db) {
        needs_put.push("DATABASE_URL".into());
    }

    if !updated_keys.is_empty() || created {
        raw = render_env(&lines);
        fs::write(&path, &raw).with_context(|| format!("write {}", path.display()))?;
    }

    let message = if !needs_put.is_empty() {
        format!(
            "{} — Put {} (your Postgres) before deploy; soft-no: payment plugins are vendor dashboards",
            if created {
                format!("created .env with {} key update(s)", updated_keys.len())
            } else if updated_keys.is_empty() {
                ".env secrets ok".into()
            } else {
                format!("updated {} in .env", updated_keys.join(" · "))
            },
            needs_put.join(" · ")
        )
    } else if created {
        format!(
            "created .env with {} key update(s) — next: medusa deploy",
            updated_keys.len()
        )
    } else if updated_keys.is_empty() {
        ".env already has strong Medusa secrets + DATABASE_URL — next: medusa deploy".into()
    } else {
        format!(
            "updated {} in .env — next: medusa deploy",
            updated_keys.join(" · ")
        )
    };

    Ok(EnsureReport {
        ok: true,
        project: project.display().to_string(),
        env_path: path.display().to_string(),
        created,
        updated_keys,
        needs_put,
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
                id: "medusa.deploy".into(),
                ok,
                exit_code: if ok { 0 } else { 1 },
                detail: message.clone(),
            }],
            message,
            urls,
            host_provider: Some("medusa".into()),
        },
    )
}

fn finish_deploy_ok(project: &Path, started: &str, admin_url: &str) -> Result<DeployReport> {
    println!("medusa · Admin ready · {admin_url}");
    println!("medusa · Confirm: create/sign-in admin (soft-no: payment plugins stay vendor dashboards)");
    let message = format!("medusa deploy ok · {admin_url}");
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

/// Ensure secrets → compose up (when present) → wait Admin `/health` → last-run URL.
pub fn deploy(project: &Path) -> Result<DeployReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    let started = config::now_rfc3339();
    println!("medusa · ensure secrets");
    let ensured = ensure(&project)?;
    println!("medusa · {}", ensured.message);

    let admin_url = "http://127.0.0.1:9000/app".to_string();
    let has_compose = compose_file(&project).is_some();

    if !ensured.needs_put.is_empty() && !has_compose {
        let message = format!(
            "medusa deploy: Put {} first (or add docker-compose.yml with Postgres) — soft-no: Integrations is not the path",
            ensured.needs_put.join(" · ")
        );
        println!("medusa · {message}");
        let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
        return Ok(DeployReport {
            ok: false,
            project: project.display().to_string(),
            admin_url: None,
            heartbeat_ok: false,
            message,
        });
    }

    if has_compose {
        let (compose_label, out) = match compose_up_detached(&project, None) {
            Ok(v) => v,
            Err(e) => {
                let message = format!("{e:#}");
                println!("medusa · {message}");
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
        println!("medusa · {compose_label} up -d");
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        for line in combined.lines().take(40) {
            println!("medusa · {line}");
        }
        if !out.status.success() {
            let message = compose_up_fail_message(&compose_label, &out, &combined);
            println!("medusa · {message}");
            let _ = write_last_run(&project, &started, false, message.clone(), vec![]);
            return Ok(DeployReport {
                ok: false,
                project: project.display().to_string(),
                admin_url: None,
                heartbeat_ok: false,
                message,
            });
        }
    } else {
        println!("medusa · no compose — expecting Medusa already listening on :9000");
    }

    println!("medusa · wait health http://127.0.0.1:9000/health");
    let log_service = if compose_has_medusa_service(&project) {
        "medusa"
    } else {
        "postgres"
    };
    match wait_http_proof(
        &project,
        "127.0.0.1",
        9000,
        "/health",
        log_service,
        None,
        Duration::from_secs(if has_compose { 90 } else { 30 }),
    ) {
        ProofWait::Ok => finish_deploy_ok(&project, &started, &admin_url),
        ProofWait::CorruptVolume { logs } => {
            let message = proof_fail_message(
                "medusa deploy: DB volume looks corrupt",
                "corrupt volume",
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
        ProofWait::Timeout { last, logs } => {
            let message = if has_compose && !compose_has_medusa_service(&project) {
                format!(
                    "medusa deploy: deps compose is up but Admin never answered on :9000 ({last}) — named gate: add a `medusa` service (build: .) or run `medusa develop` against DATABASE_URL; Soft-L3 needs a real Medusa clone (fixture is deps-only)"
                )
            } else {
                proof_fail_message(
                    &format!("medusa deploy: Admin health timeout after wait ({last})"),
                    &last,
                    &logs,
                )
            };
            // Include a snip of logs when a medusa service exists.
            let message = if compose_has_medusa_service(&project) && !logs.trim().is_empty() {
                let snip = compose_logs(&project, "medusa", None);
                let hint = snip.lines().rev().take(3).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(" | ");
                if hint.is_empty() {
                    message
                } else {
                    format!("{message} — {hint}")
                }
            } else {
                message
            };
            println!("medusa · {message}");
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "orbityard-medusa-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn ensure_creates_secrets_and_needs_db() {
        let dir = tmp("ensure");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("medusa-config.ts"), "module.exports = {}\n").unwrap();
        fs::write(
            dir.join(".env.example"),
            "DATABASE_URL=\nJWT_SECRET=supersecret\nCOOKIE_SECRET=supersecret\n",
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(r.created);
        assert!(r.updated_keys.contains(&"JWT_SECRET".into()));
        assert!(r.updated_keys.contains(&"COOKIE_SECRET".into()));
        assert!(r.needs_put.contains(&"DATABASE_URL".into()));
        assert!(!r.message.contains("supersecret"));

        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(!body.contains("supersecret"));
        assert!(!body.to_ascii_lowercase().contains("stripe"));
        let mut jwt = String::new();
        let mut cookie = String::new();
        for line in body.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if k == "JWT_SECRET" {
                    assert!(!is_weak(v), "weak JWT");
                    jwt = v.to_string();
                }
                if k == "COOKIE_SECRET" {
                    assert!(!is_weak(v), "weak COOKIE");
                    cookie = v.to_string();
                }
            }
        }
        assert_ne!(jwt, cookie);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_wires_database_from_compose_postgres() {
        let dir = tmp("compose-db");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("medusa-config.ts"), "module.exports = {}\n").unwrap();
        fs::write(
            dir.join("docker-compose.yml"),
            "services:\n  postgres:\n    image: postgres:15-alpine\n  redis:\n    image: redis:7-alpine\n",
        )
        .unwrap();
        fs::write(
            dir.join(".env.example"),
            "POSTGRES_PASSWORD=change-me\nDATABASE_URL=\nJWT_SECRET=supersecret\nCOOKIE_SECRET=supersecret\n",
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(!r.needs_put.contains(&"DATABASE_URL".into()));
        assert!(r.updated_keys.contains(&"POSTGRES_PASSWORD".into()));
        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(body.contains("postgres://medusa:"));
        assert!(body.contains("@localhost:5432/medusa"));
        assert!(body.contains("REDIS_URL=redis://localhost:6379"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_skips_strong_existing() {
        let dir = tmp("strong");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("medusa-config.ts"), "module.exports = {}\n").unwrap();
        let jwt = "a".repeat(32);
        let cookie = "b".repeat(32);
        fs::write(
            dir.join(".env"),
            format!(
                "DATABASE_URL=postgres://medusa:strongpass@localhost:5432/medusa\nREDIS_URL=redis://localhost:6379\nJWT_SECRET={jwt}\nCOOKIE_SECRET={cookie}\nSTORE_CORS=http://localhost:8000\nADMIN_CORS=http://localhost:9000\nAUTH_CORS=http://localhost:9000\nPORT=9000\n"
            ),
        )
        .unwrap();

        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(!r.created);
        assert!(r.updated_keys.is_empty());
        assert!(!r.needs_put.contains(&"DATABASE_URL".into()));
        let body = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(body.contains(&jwt));
        assert!(body.contains(&cookie));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_rejects_non_medusa() {
        let dir = tmp("reject");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("package.json"), r#"{"name":"x"}"#).unwrap();
        let err = ensure(&dir).unwrap_err();
        assert!(err.to_string().contains("not a Medusa layout"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ensure_fixture_wires_compose_db() {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/medusa-commerce");
        let dir = tmp("fixture-copy");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for name in [
            "medusa-config.ts",
            "package.json",
            ".env.example",
            "docker-compose.yml",
        ] {
            let src = fixture.join(name);
            if src.is_file() {
                fs::copy(&src, dir.join(name)).unwrap();
            }
        }
        let r = ensure(&dir).unwrap();
        assert!(r.ok);
        assert!(
            !r.needs_put.contains(&"DATABASE_URL".into()),
            "compose fixture should wire DATABASE_URL"
        );
        let json = serde_json::to_string(&r).unwrap();
        let body = fs::read_to_string(dir.join(".env")).unwrap();
        for line in body.lines() {
            if let Some((k, v)) = line.split_once('=') {
                if (k == "JWT_SECRET" || k == "COOKIE_SECRET" || k == "POSTGRES_PASSWORD")
                    && v.len() >= 16
                {
                    assert!(!json.contains(v), "ensure JSON leaked {k} value");
                }
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn deploy_fails_without_db_when_no_compose() {
        let dir = tmp("deploy-no-db");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("medusa-config.ts"), "module.exports = {}\n").unwrap();
        let r = deploy(&dir).unwrap();
        assert!(!r.ok);
        assert!(r.message.contains("Put DATABASE_URL") || r.message.contains("DATABASE_URL"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn compose_medusa_service_detect() {
        let dir = tmp("svc");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("docker-compose.yml"),
            "services:\n  postgres:\n    image: postgres:15-alpine\n",
        )
        .unwrap();
        assert!(!compose_has_medusa_service(&dir));
        fs::write(
            dir.join("docker-compose.yml"),
            "services:\n  medusa:\n    build: .\n  postgres:\n    image: postgres:15-alpine\n",
        )
        .unwrap();
        assert!(compose_has_medusa_service(&dir));
        let _ = fs::remove_dir_all(&dir);
    }
}

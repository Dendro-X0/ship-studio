//! Guided launch workflow — one step at a time: open → verify → next → launch.

use crate::adapters;
use crate::config;
use crate::flow;
use crate::human;
use crate::portal::{self, ProviderId};
use crate::secrets;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

fn resolve_bin(name: &str) -> Result<PathBuf> {
    which(name)
        .or_else(|_| which(format!("{name}.cmd")))
        .or_else(|_| which(format!("{name}.exe")))
        .with_context(|| format!("program not found: {name} (is it on PATH?)"))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    Auto,
    Oauth,
    Paste,
    Deploy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {
    Pending,
    Done,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchStep {
    pub id: String,
    pub title: String,
    pub kind: StepKind,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verify_hint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub put_provider: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub put_name: Option<String>,
    pub status: StepStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchState {
    pub schema: String,
    pub project: String,
    pub current: usize,
    pub steps: Vec<LaunchStep>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct LaunchView {
    pub schema: String,
    pub project: String,
    pub current_index: usize,
    pub total: usize,
    pub done_count: usize,
    pub finished: bool,
    pub current: Option<LaunchStep>,
    pub steps: Vec<LaunchStep>,
    pub actions: Vec<String>,
    pub notes: Vec<String>,
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn state_path(project: &Path) -> PathBuf {
    config::ship_dir(project).join("launch.json")
}

fn load_saved(project: &Path) -> Option<LaunchState> {
    let path = state_path(project);
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_state(project: &Path, state: &LaunchState) -> Result<()> {
    let dir = config::ship_dir(project);
    fs::create_dir_all(&dir)?;
    fs::write(state_path(project), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn merge_statuses(fresh: &mut [LaunchStep], saved: &LaunchState) {
    for step in fresh.iter_mut() {
        if let Some(prev) = saved.steps.iter().find(|s| s.id == step.id) {
            step.status = prev.status.clone();
            step.verified_at = prev.verified_at.clone();
        }
    }
}

fn build_plan(project: &Path) -> Result<Vec<LaunchStep>> {
    let doctor = adapters::doctor(project)?;
    let portal = portal::plan_for(project, None)?;
    let secrets_plan = secrets::plan_for(project, None)?;
    let put_queue = human::put_queue_public(&secrets_plan.hints);

    let mut steps = Vec::new();
    steps.push(LaunchStep {
        id: "doctor".into(),
        title: "Doctor — local Signet + Orbit".into(),
        kind: StepKind::Auto,
        detail: if doctor.ok {
            "Tools found.".into()
        } else {
            "Fix doctor failures before continuing.".into()
        },
        entry_url: None,
        verify_hint: Some("shipctl doctor".into()),
        put_provider: None,
        put_name: None,
        status: StepStatus::Pending,
        verified_at: None,
    });

    for id in &portal.providers {
        let Ok(pid) = ProviderId::parse(id) else {
            continue;
        };
        let (title, hint, url) = match pid {
            ProviderId::Cloudflare => (
                "Cloudflare — Wrangler OAuth".into(),
                "wrangler whoami (prefer OAuth over API tokens)".into(),
                None, // Open runs `wrangler login`
            ),
            ProviderId::Vercel => (
                "Vercel — CLI login".into(),
                "vercel whoami".into(),
                None,
            ),
            ProviderId::Netlify => (
                "Netlify — CLI login".into(),
                "netlify status".into(),
                None,
            ),
            ProviderId::Github => (
                "GitHub — gh auth (optional for PAT create)".into(),
                "gh auth status".into(),
                Some("https://github.com/settings/tokens/new".into()),
            ),
            ProviderId::Polar => (
                "Polar — open dashboard (no CLI OAuth)".into(),
                "Confirm in UI after copying checkout/webhook values".into(),
                Some("https://polar.sh/dashboard".into()),
            ),
        };
        // Polar oauth step is weak — skip as oauth; paste steps cover it.
        if matches!(pid, ProviderId::Polar) {
            continue;
        }
        // Skip GitHub oauth as required gate — PAT paste is the real gate.
        if matches!(pid, ProviderId::Github) {
            continue;
        }
        steps.push(LaunchStep {
            id: format!("oauth.{}", pid.as_str()),
            title,
            kind: StepKind::Oauth,
            detail: "Complete login on the official site / CLI — then Verify.".into(),
            entry_url: url,
            verify_hint: Some(hint),
            put_provider: None,
            put_name: None,
            status: StepStatus::Pending,
            verified_at: None,
        });
    }

    for h in &put_queue {
        steps.push(LaunchStep {
            id: format!("paste.{}", h.name),
            title: format!("Paste secret — {} → {}", h.name, h.provider),
            kind: StepKind::Paste,
            detail: format!(
                "Open the source page, copy the value, then put via `{}`. Confirm when done.",
                h.put_cli.join(" ")
            ),
            entry_url: h.entry_url.clone(),
            verify_hint: Some(format!(
                "confirm after paste, or wrangler secret list | find {}",
                h.name
            )),
            put_provider: Some(h.provider.clone()),
            put_name: Some(h.name.clone()),
            status: StepStatus::Pending,
            verified_at: None,
        });
    }

    steps.push(LaunchStep {
        id: "configure".into(),
        title: "Configure — write .ship/studio.json".into(),
        kind: StepKind::Auto,
        detail: "Persist sign/deploy intent for flow.".into(),
        entry_url: None,
        verify_hint: Some(".ship/studio.json exists".into()),
        put_provider: None,
        put_name: None,
        status: StepStatus::Pending,
        verified_at: None,
    });

    steps.push(LaunchStep {
        id: "flow_dry_run".into(),
        title: "Flow dry-run — preview sign → deploy".into(),
        kind: StepKind::Auto,
        detail: "Offline plan check before network deploy.".into(),
        entry_url: None,
        verify_hint: Some("shipctl flow --dry-run --offline --skip-deploy".into()),
        put_provider: None,
        put_name: None,
        status: StepStatus::Pending,
        verified_at: None,
    });

    steps.push(LaunchStep {
        id: "deploy".into(),
        title: "Deploy — Orbit ship (network)".into(),
        kind: StepKind::Deploy,
        detail: "Run deploy when ready. Confirm or check last-run.".into(),
        entry_url: None,
        verify_hint: Some("shipctl deploy / last-run ok, or confirm".into()),
        put_provider: None,
        put_name: None,
        status: StepStatus::Pending,
        verified_at: None,
    });

    Ok(steps)
}

pub fn load_or_build(project: &Path) -> Result<LaunchState> {
    load_state(project, true)
}

fn load_state(project: &Path, snap_to_pending: bool) -> Result<LaunchState> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let mut steps = build_plan(&project)?;
    let mut current = 0;
    if let Some(saved) = load_saved(&project) {
        merge_statuses(&mut steps, &saved);
        current = saved.current.min(steps.len().saturating_sub(1));
    }
    if snap_to_pending {
        let cur_pending = steps
            .get(current)
            .is_some_and(|s| s.status == StepStatus::Pending);
        if !cur_pending {
            if let Some(i) = steps.iter().position(|s| s.status == StepStatus::Pending) {
                current = i;
            } else if !steps.is_empty() {
                current = steps.len() - 1;
            }
        }
    }
    let state = LaunchState {
        schema: "ship-studio/launch/v1".into(),
        project: project.display().to_string(),
        current,
        steps,
        notes: vec![
            "Work on official platforms; shipctl only sequences and verifies.".into(),
            "Paste steps: Open → copy on vendor site → put CLI → Confirm → Next.".into(),
        ],
    };
    save_state(&project, &state)?;
    Ok(state)
}

pub fn view(state: &LaunchState) -> LaunchView {
    let done_count = state
        .steps
        .iter()
        .filter(|s| s.status == StepStatus::Done || s.status == StepStatus::Skipped)
        .count();
    let finished = !state.steps.is_empty()
        && state
            .steps
            .iter()
            .all(|s| s.status == StepStatus::Done || s.status == StepStatus::Skipped);
    let current = state.steps.get(state.current).cloned();
    let mut actions = vec![
        "shipctl launch".into(),
        "shipctl launch open".into(),
        "shipctl launch verify".into(),
        "shipctl launch confirm".into(),
        "shipctl launch next".into(),
    ];
    if let Some(cur) = &current {
        if cur.kind == StepKind::Paste {
            if let (Some(p), Some(n)) = (&cur.put_provider, &cur.put_name) {
                actions.push(format!(
                    "shipctl secrets put --provider {p} --name {n}"
                ));
            }
        }
    }
    LaunchView {
        schema: state.schema.clone(),
        project: state.project.clone(),
        current_index: state.current,
        total: state.steps.len(),
        done_count,
        finished,
        current,
        steps: state.steps.clone(),
        actions,
        notes: state.notes.clone(),
    }
}

pub fn open_current(project: &Path) -> Result<LaunchView> {
    let state = load_state(project, true)?;
    let Some(step) = state.steps.get(state.current).cloned() else {
        bail!("no launch steps");
    };
    if let Some(url) = &step.entry_url {
        portal::open_url(url)?;
    }
    if step.kind == StepKind::Oauth {
        let work = if step.id.contains("cloudflare") {
            secrets::wrangler_workdir(project)
        } else {
            project.to_path_buf()
        };
        let (bin, args): (PathBuf, &[&str]) = if step.id.contains("cloudflare") {
            (resolve_bin("wrangler")?, &["login"])
        } else if step.id.contains("vercel") {
            (resolve_bin("vercel")?, &["login"])
        } else if step.id.contains("netlify") {
            (resolve_bin("netlify")?, &["login"])
        } else {
            return Ok(view(&state));
        };
        let _ = Command::new(&bin)
            .args(args)
            .current_dir(&work)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
    }
    if step.kind == StepKind::Paste {
        if let (Some(provider), Some(name)) = (&step.put_provider, &step.put_name) {
            eprintln!(
                "After copying the value, put with:\n  shipctl secrets put --project {} --provider {} --name {}",
                project.display(),
                provider,
                name
            );
            if std::io::stdin().is_terminal() {
                eprint!("Run put now? [y/N] ");
                let _ = std::io::Write::flush(&mut std::io::stderr());
                let mut line = String::new();
                let _ = std::io::stdin().read_line(&mut line);
                if line.trim().eq_ignore_ascii_case("y") {
                    let id = ProviderId::parse(provider)?;
                    let code = secrets::put_secret(project, id, name)?;
                    eprintln!("put exited {code}");
                }
            }
        }
    }
    save_state(project, &state)?;
    Ok(view(&state))
}

fn run_capture(bin: &str, args: &[&str], cwd: &Path) -> Result<(i32, String)> {
    use std::io::Read;
    let path = resolve_bin(bin)?;
    let mut child = Command::new(&path)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("run {bin}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("missing stdout"))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("missing stderr"))?;
    let t_out = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let t_err = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });
    let timeout = std::time::Duration::from_secs(20);
    let start = std::time::Instant::now();
    let status = loop {
        match child.try_wait()? {
            Some(status) => break status,
            None if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                bail!(
                    "{bin} timed out after {}s — if already logged in: shipctl launch confirm",
                    timeout.as_secs()
                );
            }
            None => std::thread::sleep(std::time::Duration::from_millis(200)),
        }
    };
    let mut text = t_out.join().unwrap_or_default();
    text.push_str(&t_err.join().unwrap_or_default());
    Ok((status.code().unwrap_or(1), text))
}

pub fn verify_current(project: &Path) -> Result<(bool, String, LaunchView)> {
    let mut state = load_state(project, true)?;
    let idx = state.current;
    let step = state
        .steps
        .get(idx)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no current step"))?;

    let (ok, msg) = match step.kind {
        StepKind::Auto if step.id == "doctor" => {
            let report = adapters::doctor(project)?;
            (report.ok, if report.ok { "doctor ok".into() } else { "doctor failed".into() })
        }
        StepKind::Auto if step.id == "configure" => {
            let path = config::ship_dir(project).join("studio.json");
            if path.is_file() {
                (true, "studio.json present".into())
            } else {
                let _ = config::configure(project)?;
                (true, "configure wrote studio.json".into())
            }
        }
        StepKind::Auto if step.id == "flow_dry_run" => {
            let plan = flow::plan(project, false, true, true)?;
            (
                !plan.steps.is_empty(),
                format!("dry-run {} step(s)", plan.steps.len()),
            )
        }
        StepKind::Oauth if step.id.contains("cloudflare") => {
            let (code, text) = run_capture("wrangler", &["whoami"], project)?;
            (
                code == 0 && !text.to_lowercase().contains("not logged"),
                if code == 0 {
                    "wrangler whoami ok".into()
                } else {
                    format!("wrangler whoami failed — run Open / wrangler login: {text}")
                },
            )
        }
        StepKind::Oauth if step.id.contains("vercel") => {
            let (code, _) = run_capture("vercel", &["whoami"], project)?;
            (
                code == 0,
                if code == 0 {
                    "vercel whoami ok".into()
                } else {
                    "vercel whoami failed — run Open / vercel login".into()
                },
            )
        }
        StepKind::Oauth if step.id.contains("netlify") => {
            let (code, _) = run_capture("netlify", &["status"], project)?;
            (code == 0, if code == 0 { "netlify status ok".into() } else { "netlify not logged in".into() })
        }
        StepKind::Paste => {
            // Prefer soft confirm; optional network list.
            if let Some(name) = &step.put_name {
                if let Ok((code, text)) = run_capture(
                    "wrangler",
                    &["secret", "list"],
                    &secrets::wrangler_workdir(project),
                ) {
                    if code == 0 && text.contains(name) {
                        (true, format!("{name} present in wrangler secret list"))
                    } else {
                        (
                            false,
                            format!(
                                "{name} not seen in secret list yet — paste then Confirm, or retry Verify"
                            ),
                        )
                    }
                } else {
                    (
                        false,
                        "Could not list secrets — after paste use: shipctl launch confirm".into(),
                    )
                }
            } else {
                (false, "missing put name".into())
            }
        }
        StepKind::Deploy => {
            match config::read_last_run(project) {
                Ok(v) if v.get("ok").and_then(|x| x.as_bool()) == Some(true) => {
                    (true, "last-run ok".into())
                }
                _ => (
                    false,
                    "No successful last-run — deploy then Confirm, or retry Verify".into(),
                ),
            }
        }
        _ => (false, "use confirm for this step".into()),
    };

    if ok {
        if let Some(s) = state.steps.get_mut(idx) {
            s.status = StepStatus::Done;
            s.verified_at = Some(now_rfc3339());
        }
        save_state(project, &state)?;
    }
    Ok((ok, msg, view(&state)))
}

pub fn confirm_current(project: &Path) -> Result<LaunchView> {
    let mut state = load_state(project, true)?;
    let idx = state.current;
    let Some(step) = state.steps.get_mut(idx) else {
        bail!("no current step");
    };
    step.status = StepStatus::Done;
    step.verified_at = Some(now_rfc3339());
    // Park cursor on this done step so next() can advance from here.
    save_state(project, &state)?;
    Ok(view(&state))
}

pub fn next(project: &Path, force: bool) -> Result<LaunchView> {
    let mut state = load_state(project, false)?;
    let idx = state.current;
    let Some(step) = state.steps.get(idx) else {
        bail!("no current step");
    };
    if !force && step.status == StepStatus::Pending {
        bail!(
            "current step '{}' is still pending — verify, confirm, or next --force",
            step.id
        );
    }
    if force {
        if let Some(s) = state.steps.get_mut(idx) {
            if s.status == StepStatus::Pending {
                s.status = StepStatus::Skipped;
                s.verified_at = Some(now_rfc3339());
            }
        }
    }
    let advanced = state
        .steps
        .iter()
        .enumerate()
        .skip(idx + 1)
        .find(|(_, s)| s.status == StepStatus::Pending)
        .map(|(i, _)| i)
        .unwrap_or_else(|| state.steps.len().saturating_sub(1));
    state.current = advanced;
    save_state(project, &state)?;
    Ok(view(&state))
}

pub fn reset(project: &Path) -> Result<LaunchView> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let _ = fs::remove_file(state_path(&project));
    let state = load_or_build(&project)?;
    Ok(view(&state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_includes_doctor_and_deploy() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n# Secrets\n# - GITHUB_TOKEN\n").unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(state.steps.iter().any(|s| s.id == "doctor"));
        assert!(state.steps.iter().any(|s| s.id == "deploy"));
        assert!(state.steps.iter().any(|s| s.id.starts_with("paste.")));
        let v = view(&state);
        assert!(!v.finished);
        assert_eq!(v.current.as_ref().unwrap().id, "doctor");
    }

    #[test]
    fn confirm_then_next_advances() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-next-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let _ = load_or_build(&dir).unwrap();
        let v = confirm_current(&dir).unwrap();
        assert_eq!(v.current.as_ref().unwrap().status, StepStatus::Done);
        let v2 = next(&dir, false).unwrap();
        assert_ne!(v2.current_index, 0);
    }
}

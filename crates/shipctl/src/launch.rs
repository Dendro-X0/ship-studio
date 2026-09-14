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
    Sign,
    List,
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
    /// Local CLI to run on Open/Run, e.g. `["signet","build"]` or `["shipctl","deploy"]`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<Vec<String>>,
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

fn step(
    id: &str,
    title: &str,
    kind: StepKind,
    detail: &str,
    entry_url: Option<String>,
    verify_hint: Option<String>,
    run: Option<Vec<String>>,
) -> LaunchStep {
    LaunchStep {
        id: id.into(),
        title: title.into(),
        kind,
        detail: detail.into(),
        entry_url,
        verify_hint,
        put_provider: None,
        put_name: None,
        run,
        status: StepStatus::Pending,
        verified_at: None,
    }
}

fn build_plan(project: &Path) -> Result<Vec<LaunchStep>> {
    let doctor = adapters::doctor(project)?;
    let detected = config::probe(project);
    let portal = portal::plan_for(project, None)?;
    let secrets_plan = secrets::plan_for(project, None)?;
    let put_queue = human::put_queue_public(&secrets_plan.hints);
    let wants_signet = detected.tauri || detected.signet_toml;

    let mut steps = Vec::new();
    steps.push(step(
        "doctor",
        "Doctor — local Signet + Orbit",
        StepKind::Auto,
        if doctor.ok {
            "Tools found."
        } else {
            "Fix doctor failures before continuing."
        },
        None,
        Some("shipctl doctor".into()),
        None,
    ));

    for id in &portal.providers {
        let Ok(pid) = ProviderId::parse(id) else {
            continue;
        };
        if matches!(pid, ProviderId::Polar | ProviderId::Github) {
            continue;
        }
        let (title, hint) = match pid {
            ProviderId::Cloudflare => (
                "Cloudflare — Wrangler OAuth",
                "wrangler whoami (prefer OAuth over API tokens)",
            ),
            ProviderId::Vercel => ("Vercel — CLI login", "vercel whoami"),
            ProviderId::Netlify => ("Netlify — CLI login", "netlify status"),
            _ => continue,
        };
        steps.push(step(
            &format!("oauth.{}", pid.as_str()),
            title,
            StepKind::Oauth,
            "Complete login on the official site / CLI — then Verify.",
            None,
            Some(hint.into()),
            None,
        ));
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
            run: None,
            status: StepStatus::Pending,
            verified_at: None,
        });
    }

    steps.push(step(
        "configure",
        "Configure — write .ship/studio.json",
        StepKind::Auto,
        "Persist sign/deploy intent for flow.",
        None,
        Some(".ship/studio.json exists".into()),
        Some(vec![
            "shipctl".into(),
            "configure".into(),
            "--project".into(),
            ".".into(),
        ]),
    ));

    steps.push(step(
        "intent",
        "Ship intent — set sign_args / deploy_args for real ship",
        StepKind::Auto,
        if wants_signet {
            "Recommend sign_args=[build] and provider deploy_args for desktop+web ship."
        } else {
            "Recommend provider deploy_args (Worker/docs). Signet build skipped unless signet.toml appears."
        },
        None,
        Some("studio.json sign_args/deploy_args look ship-ready".into()),
        None,
    ));

    if wants_signet {
        if !detected.signet_toml {
            steps.push(step(
                "signet.scan",
                "Signet — scan/init project signing config",
                StepKind::Sign,
                "Create signet.toml from repo scan (or init).",
                None,
                Some("signet.toml exists".into()),
                Some(vec!["signet".into(), "scan".into(), "--apply".into()]),
            ));
        }
        steps.push(step(
            "signet.identity",
            "Signet — signing identity",
            StepKind::Sign,
            "Ensure a local signing identity exists (create once).",
            None,
            Some("signet identity list shows an identity".into()),
            Some(vec!["signet".into(), "identity".into(), "list".into()]),
        ));
        steps.push(step(
            "signet.build",
            "Signet — build & sign artifacts",
            StepKind::Sign,
            "Build and sign desktop/mobile artifacts. Network may be used for timestamps.",
            None,
            Some("signet build exit 0 (or confirm)".into()),
            Some(vec!["signet".into(), "build".into()]),
        ));
        steps.push(step(
            "signet.ship_plan",
            "Signet — multi-platform ship plan",
            StepKind::Sign,
            "Review coverage plan before CI/collect/release.",
            None,
            Some("signet ship --plan exit 0".into()),
            Some(vec!["signet".into(), "ship".into(), "--plan".into()]),
        ));
        steps.push(step(
            "signet.release_dry",
            "Signet — release dry-run (checksums / GitHub payload)",
            StepKind::Sign,
            "Offline-ish check of release packaging. Set tag via SIGNET_RELEASE_TAG or confirm.",
            None,
            Some("signet release --dry-run exit 0".into()),
            Some(vec![
                "signet".into(),
                "release".into(),
                "--dry-run".into(),
                "--tag".into(),
                std::env::var("SIGNET_RELEASE_TAG").unwrap_or_else(|_| "v0.1.0".into()),
            ]),
        ));
        steps.push(step(
            "signet.release",
            "Signet — publish GitHub Release (network)",
            StepKind::Sign,
            "Live release upload. Requires gh auth. Confirm after success.",
            Some("https://github.com/releases/new".into()),
            Some("confirm after signet release succeeds".into()),
            Some(vec![
                "signet".into(),
                "release".into(),
                "--tag".into(),
                std::env::var("SIGNET_RELEASE_TAG").unwrap_or_else(|_| "v0.1.0".into()),
            ]),
        ));
    }

    if detected.polar {
        steps.push(step(
            "listing.polar",
            "Listing — Polar product / checkout (official dashboard)",
            StepKind::List,
            "Update Polar product listing & checkout URL on polar.sh — Ship Studio only opens the door.",
            Some("https://polar.sh/dashboard".into()),
            Some("confirm listing/checkout updated".into()),
            None,
        ));
    }

    steps.push(step(
        "flow_dry_run",
        "Flow dry-run — preview configure → sign → deploy",
        StepKind::Auto,
        "Offline plan check before network deploy.",
        None,
        Some("shipctl flow --dry-run --offline --skip-deploy".into()),
        Some(vec![
            "shipctl".into(),
            "flow".into(),
            "--project".into(),
            ".".into(),
            "--dry-run".into(),
            "--offline".into(),
            "--skip-deploy".into(),
        ]),
    ));

    steps.push(step(
        "deploy",
        "Deploy — Orbit (network)",
        StepKind::Deploy,
        "Run Orbit deploy with studio.json deploy_args. Confirm or check last-run.",
        None,
        Some("shipctl deploy / last-run ok, or confirm".into()),
        Some(vec![
            "shipctl".into(),
            "deploy".into(),
            "--project".into(),
            ".".into(),
        ]),
    ));

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
            "Paste: Open → copy on vendor site → put → Confirm → Next.".into(),
            "Sign/release/deploy: Open/Run executes local CLI; Confirm after live network steps.".into(),
        ],
    };
    save_state(&project, &state)?;
    Ok(state)
}

fn resolve_run_bin(name: &str) -> Result<PathBuf> {
    if name == "shipctl" {
        if let Ok(exe) = std::env::current_exe() {
            return Ok(exe);
        }
    }
    resolve_bin(name)
}

/// Whether a successful Open/Run may auto-mark the step done (no live network publish).
fn auto_done_after_run(step: &LaunchStep) -> bool {
    match step.kind {
        StepKind::Deploy | StepKind::List | StepKind::Paste | StepKind::Oauth => false,
        StepKind::Sign if step.id == "signet.release" => false,
        StepKind::Auto | StepKind::Sign => true,
    }
}

fn execute_run(project: &Path, argv: &[String]) -> Result<i32> {
    let Some((bin_name, rest)) = argv.split_first() else {
        bail!("empty run argv");
    };
    let bin = resolve_run_bin(bin_name)?;
    let work = if bin_name == "wrangler" {
        secrets::wrangler_workdir(project)
    } else {
        project.to_path_buf()
    };
    eprintln!("$ {} {}", bin.display(), rest.join(" "));
    let status = Command::new(&bin)
        .args(rest)
        .current_dir(&work)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("run {}", argv.join(" ")))?;
    Ok(status.code().unwrap_or(1))
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
        "shipctl launch run".into(),
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
        if let Some(run) = &cur.run {
            actions.push(format!("run: {}", run.join(" ")));
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
    let mut state = load_state(project, true)?;
    let idx = state.current;
    let Some(step) = state.steps.get(idx).cloned() else {
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
    if let Some(argv) = &step.run {
        let code = execute_run(project, argv)?;
        if code == 0 && auto_done_after_run(&step) {
            if let Some(s) = state.steps.get_mut(idx) {
                s.status = StepStatus::Done;
                s.verified_at = Some(now_rfc3339());
            }
            eprintln!("run ok — step marked done");
        } else if code != 0 {
            eprintln!("run exited {code} — fix, retry Open/Run, or Confirm if already done");
        } else {
            eprintln!("run ok — Confirm after you verify the live result, then Next");
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
        StepKind::Auto if step.id == "intent" => {
            let path = config::ship_dir(project).join("studio.json");
            if !path.is_file() {
                (false, "studio.json missing — Open/Run configure first".into())
            } else {
                match config::read_studio(project)? {
                    Some(s) => (
                        true,
                        format!(
                            "intent ok · sign_args={:?} · deploy_args={:?}",
                            s.sign_args, s.deploy_args
                        ),
                    ),
                    None => (false, "studio.json unreadable".into()),
                }
            }
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
                    "No successful last-run — Open/Run deploy then Confirm, or retry Verify".into(),
                ),
            }
        }
        StepKind::Sign if step.id == "signet.scan" => {
            let ok = project.join("signet.toml").is_file();
            (
                ok,
                if ok {
                    "signet.toml present".into()
                } else {
                    "signet.toml missing — Open/Run signet scan --apply".into()
                },
            )
        }
        StepKind::Sign if step.id == "signet.identity" => {
            match run_capture("signet", &["identity", "list"], project) {
                Ok((code, text)) => {
                    let has = code == 0
                        && text.lines().any(|l| {
                            let t = l.trim();
                            !t.is_empty()
                                && !t.starts_with('{')
                                && !t.to_ascii_lowercase().contains("no identit")
                        });
                    (
                        has,
                        if has {
                            "signet identity list ok".into()
                        } else {
                            "no identity yet — create one, then Confirm / Verify".into()
                        },
                    )
                }
                Err(e) => (false, format!("signet identity list failed: {e:#}")),
            }
        }
        StepKind::Sign | StepKind::List => (
            false,
            "after Open/Run succeeds on this step: shipctl launch confirm".into(),
        ),
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
        assert!(state.steps.iter().any(|s| s.id == "intent"));
        assert!(state.steps.iter().any(|s| s.id.starts_with("paste.")));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("signet.")));
        let deploy = state.steps.iter().find(|s| s.id == "deploy").unwrap();
        assert!(deploy.run.is_some());
        let v = view(&state);
        assert!(!v.finished);
        assert_eq!(v.current.as_ref().unwrap().id, "doctor");
    }

    #[test]
    fn tauri_plan_includes_signet_ship_steps() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-tauri-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src-tauri")).unwrap();
        fs::write(dir.join("package.json"), "{}\n").unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(state.steps.iter().any(|s| s.id == "signet.scan"));
        assert!(state.steps.iter().any(|s| s.id == "signet.build"));
        assert!(state.steps.iter().any(|s| s.id == "signet.release_dry"));
        assert!(state.steps.iter().any(|s| s.id == "signet.release"));
        assert!(state.steps.iter().any(|s| s.id == "deploy"));
        let build = state.steps.iter().find(|s| s.id == "signet.build").unwrap();
        assert_eq!(build.kind, StepKind::Sign);
        assert_eq!(build.run.as_ref().unwrap()[0], "signet");
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

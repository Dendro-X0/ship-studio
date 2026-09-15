//! Publish portal — minute-oriented wizard for the full manual publishing path.

use crate::adapters;
use crate::config;
use crate::flow;
use crate::human;
use crate::portal::{self, ProviderId};
use crate::scopes;
use crate::secrets;
use crate::signpath;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum StudioMode {
    #[default]
    General,
    Advanced,
}

impl StudioMode {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "advanced" | "adv" => Self::Advanced,
            _ => Self::General,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Advanced => "advanced",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PubKind {
    Auto,
    Human,
    Oauth,
    Sign,
    List,
    Deploy,
    Check,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PubStatus {
    Pending,
    Done,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubStep {
    pub id: String,
    pub title: String,
    pub kind: PubKind,
    pub detail: String,
    pub minutes: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop_view: Option<String>,
    pub status: PubStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishState {
    pub schema: String,
    pub project: String,
    #[serde(default)]
    pub mode: StudioMode,
    pub current: usize,
    pub steps: Vec<PubStep>,
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PublishView {
    pub schema: String,
    pub project: String,
    pub mode: StudioMode,
    pub current_index: usize,
    pub total: usize,
    pub done_count: usize,
    pub minutes_remaining: u32,
    pub minutes_total: u32,
    pub finished: bool,
    pub current: Option<PubStep>,
    pub steps: Vec<PubStep>,
    pub actions: Vec<String>,
    pub notes: Vec<String>,
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into())
}

fn state_path(project: &Path) -> PathBuf {
    config::ship_dir(project).join("publish.json")
}

fn resolve_bin(name: &str) -> Result<PathBuf> {
    which(name)
        .or_else(|_| which(format!("{name}.cmd")))
        .or_else(|_| which(format!("{name}.exe")))
        .with_context(|| format!("program not found: {name}"))
}

fn resolve_run_bin(name: &str) -> Result<PathBuf> {
    if name == "shipctl" {
        if let Ok(exe) = std::env::current_exe() {
            return Ok(exe);
        }
    }
    resolve_bin(name)
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

fn step(
    id: impl Into<String>,
    title: impl Into<String>,
    kind: PubKind,
    detail: impl Into<String>,
    minutes: u32,
    entry_url: Option<String>,
    run: Option<Vec<String>>,
    desktop_view: Option<&str>,
) -> PubStep {
    PubStep {
        id: id.into(),
        title: title.into(),
        kind,
        detail: detail.into(),
        minutes,
        entry_url,
        run,
        desktop_view: desktop_view.map(|s| s.into()),
        status: PubStatus::Pending,
        verified_at: None,
    }
}

fn auto_done_after_run(step: &PubStep) -> bool {
    match step.kind {
        PubKind::Deploy | PubKind::List | PubKind::Human | PubKind::Oauth | PubKind::Check => false,
        PubKind::Sign if step.id.contains("release") && !step.id.contains("dry") => false,
        PubKind::Auto | PubKind::Sign => true,
    }
}

fn is_general_step(step: &PubStep) -> bool {
    let id = step.id.as_str();
    id == "doctor"
        || id == "scopes"
        || id == "env.sprint"
        || id == "configure"
        || id == "dry_run"
        || id == "live_check"
        || id == "deploy"
        || id.starts_with("deploy.")
        || id == "sign.self.build"
}

fn build_plan_for(project: &Path, mode: StudioMode) -> Result<Vec<PubStep>> {
    let doctor = adapters::doctor(project)?;
    let detected = config::probe(project);
    let portal = portal::plan_for(project, None)?;
    let secrets_plan = secrets::plan_for(project, None)?;
    let put_queue = human::put_queue_public(&secrets_plan.hints);
    let sc = scopes::plan_for(project);
    let wants_signet = detected.tauri || detected.signet_toml;
    let sign_mode = config::read_studio(project)
        .ok()
        .flatten()
        .map(|s| s.sign_path)
        .unwrap_or_else(|| {
            if wants_signet {
                "self_then_official".into()
            } else {
                "official_listing_only".into()
            }
        });

    let mut steps = Vec::new();
    steps.push(step(
        "doctor",
        "Doctor — Signet + Orbit on PATH",
        PubKind::Auto,
        if doctor.ok {
            "Tools found. Confirm and continue."
        } else {
            "Install or PATH-fix Signet/Orbit, then Verify."
        },
        1,
        None,
        Some(vec![
            "shipctl".into(),
            "doctor".into(),
            "--project".into(),
            ".".into(),
        ]),
        Some("tools"),
    ));

    if sc.scopes.len() > 1
        || sc
            .scopes
            .iter()
            .any(|s| s.relative != "." && !s.relative.is_empty())
    {
        steps.push(step(
            "scopes",
            "Scopes — pick Web / API / Desktop / Mobile / Container",
            PubKind::Human,
            format!(
                "{} detected · {} active. Save selection in Scopes, then Confirm.",
                sc.scopes.len(),
                sc.active.len()
            ),
            1,
            None,
            Some(vec![
                "shipctl".into(),
                "scopes".into(),
                "--project".into(),
                ".".into(),
            ]),
            Some("scopes"),
        ));
    }

    for id in &portal.providers {
        let Ok(pid) = ProviderId::parse(id) else {
            continue;
        };
        if matches!(pid, ProviderId::Polar | ProviderId::Github) {
            continue;
        }
        let (title, hint) = match pid {
            ProviderId::Cloudflare => ("Cloudflare — Wrangler login", "wrangler login → whoami"),
            ProviderId::Vercel => ("Vercel — CLI login", "vercel login → whoami"),
            ProviderId::Netlify => ("Netlify — CLI login", "netlify login"),
            _ => continue,
        };
        steps.push(step(
            format!("oauth.{}", pid.as_str()),
            title,
            PubKind::Oauth,
            hint,
            2,
            None,
            None,
            Some("portal"),
        ));
    }

    if !put_queue.is_empty() {
        steps.push(step(
            "env.sprint",
            "ENV & tokens — create / paste / put",
            PubKind::Human,
            format!(
                "{} secret(s) to put. Open Env portal, paste on vendor sites, put via CLI, then Confirm.",
                put_queue.len()
            ),
            5,
            put_queue.first().and_then(|h| h.entry_url.clone()),
            None,
            Some("env"),
        ));
    }

    if detected.d1 || detected.neon || detected.supabase || detected.turso {
        let entry = if detected.neon {
            Some("https://console.neon.tech".into())
        } else if detected.supabase {
            Some("https://supabase.com/dashboard".into())
        } else if detected.turso {
            Some("https://turso.tech/app".into())
        } else {
            Some("https://dash.cloudflare.com/?to=/:account/workers/d1".into())
        };
        let mut bits = Vec::new();
        if detected.d1 {
            bits.push("D1");
        }
        if detected.neon {
            bits.push("Neon");
        }
        if detected.supabase {
            bits.push("Supabase");
        }
        if detected.turso {
            bits.push("Turso");
        }
        steps.push(step(
            "db.provision",
            format!("Database — provision ({})", bits.join(" · ")),
            PubKind::Human,
            "Create the DB on the vendor console, copy the connection string, put on the deploy target, then Confirm.",
            4,
            entry,
            None,
            Some("env"),
        ));
    }

    if detected.ci_release {
        let files = detected.release_workflows.join(", ");
        steps.push(step(
            "ci.release",
            "CI — GitHub Actions release",
            PubKind::Check,
            format!(
                "Workflow(s): {files}. After tag/Signet release, confirm the Actions run looks green."
            ),
            2,
            config::github_actions_url(project)
                .or_else(|| Some("https://github.com/actions".into())),
            None,
            Some("dashboard"),
        ));
    }

    if detected.container {
        let detail = if detected.compose {
            "Compose + image layout detected. Build/tag/push locally; open registry docs, then Confirm."
        } else {
            "Dockerfile detected. Build/tag/push locally; open registry docs, then Confirm."
        };
        steps.push(step(
            "container.deploy",
            "Container — build / push (docs)",
            PubKind::Human,
            detail,
            4,
            Some(config::container_docs_url(project).into()),
            None,
            Some("portal"),
        ));
    }

    steps.push(step(
        "configure",
        "Configure — write .ship/studio.json",
        PubKind::Auto,
        "Persist sign/deploy intent for this publish.",
        1,
        None,
        Some(vec![
            "shipctl".into(),
            "configure".into(),
            "--project".into(),
            ".".into(),
        ]),
        Some("ritual"),
    ));

    if wants_signet && sign_mode != "official" {
        if !detected.signet_toml {
            steps.push(step(
                "sign.self.scan",
                "Self-sign — scan / init signet.toml",
                PubKind::Sign,
                "Create signing config from repo layout.",
                2,
                None,
                Some(vec!["signet".into(), "scan".into(), "--apply".into()]),
                Some("sign"),
            ));
        }
        steps.push(step(
            "sign.self.build",
            "Self-sign — Signet build",
            PubKind::Sign,
            "Build & sign local artifacts.",
            8,
            None,
            Some(vec!["signet".into(), "build".into()]),
            Some("sign"),
        ));
        steps.push(step(
            "sign.self.release_dry",
            "Self-sign — release dry-run",
            PubKind::Sign,
            "Checksum / GitHub payload check before live release.",
            2,
            None,
            Some(vec![
                "signet".into(),
                "release".into(),
                "--dry-run".into(),
                "--tag".into(),
                std::env::var("SIGNET_RELEASE_TAG").unwrap_or_else(|_| "v0.1.0".into()),
            ]),
            Some("sign"),
        ));
    }

    if wants_signet && sign_mode != "self" {
        for p in signpath::plan_for(project).paths {
            if p.kind != "official" {
                continue;
            }
            if p.id == "official.github" && sign_mode != "official" {
                continue;
            }
            steps.push(step(
                format!("sign.{}", p.id),
                p.title.clone(),
                PubKind::Sign,
                p.detail.clone(),
                3,
                p.entry_url.clone(),
                p.run.clone(),
                Some("sign"),
            ));
        }
    }

    if wants_signet && sign_mode != "official" {
        steps.push(step(
            "sign.self.release",
            "Publish — GitHub Release (network)",
            PubKind::Sign,
            "Live `signet release` after gh auth. Confirm when the release is up.",
            3,
            Some("https://github.com/releases/new".into()),
            Some(vec![
                "signet".into(),
                "release".into(),
                "--tag".into(),
                std::env::var("SIGNET_RELEASE_TAG").unwrap_or_else(|_| "v0.1.0".into()),
            ]),
            Some("sign"),
        ));
    }

    if detected.polar {
        steps.push(step(
            "listing.polar",
            "Listing — Polar product / checkout",
            PubKind::List,
            "Update product, pricing, checkout URL on polar.sh — then Confirm.",
            3,
            Some("https://polar.sh/dashboard".into()),
            None,
            Some("portal"),
        ));
    }

    if detected.steam {
        steps.push(step(
            "listing.steam",
            "Listing — Steamworks partner",
            PubKind::List,
            "Steam store presence / depots stay on partner.steamgames.com — then Confirm.",
            3,
            Some("https://partner.steamgames.com/".into()),
            None,
            Some("portal"),
        ));
    }

    if detected.itch {
        steps.push(step(
            "listing.itch",
            "Listing — itch.io dashboard",
            PubKind::List,
            "Upload / page / pricing on itch.io — then Confirm.",
            3,
            Some("https://itch.io/dashboard".into()),
            None,
            Some("portal"),
        ));
    }

    if detected.epic {
        steps.push(step(
            "listing.epic",
            "Listing — Epic Games Store portal",
            PubKind::List,
            "Epic product listing stays on the developer portal — then Confirm.",
            3,
            Some("https://dev.epicgames.com/portal".into()),
            None,
            Some("portal"),
        ));
    }

    if detected.android || detected.expo || (detected.mobile && !detected.ios) {
        steps.push(step(
            "listing.play",
            "Listing — Google Play Console",
            PubKind::List,
            "Store listing, screenshots, and release track on Play Console — then Confirm.",
            3,
            Some("https://play.google.com/console".into()),
            None,
            Some("sign"),
        ));
    }

    if detected.ios || detected.expo || (detected.mobile && !detected.android) {
        steps.push(step(
            "listing.app_store",
            "Listing — App Store Connect",
            PubKind::List,
            "App record, metadata, and pricing on App Store Connect — then Confirm.",
            3,
            Some("https://appstoreconnect.apple.com".into()),
            None,
            Some("sign"),
        ));
    }

    // Store submission (send for review) — Advanced only; Desktop / Mobile.
    if detected.android || detected.expo || (detected.mobile && !detected.ios) {
        steps.push(step(
            "submit.play",
            "Submit — Play production / review",
            PubKind::List,
            "Promote the release track and send for review on Play Console — then Confirm.",
            3,
            Some("https://play.google.com/console".into()),
            None,
            Some("sign"),
        ));
    }
    if detected.ios || detected.expo || detected.tauri || (detected.mobile && !detected.android) {
        steps.push(step(
            "submit.app_store",
            "Submit — App Store review",
            PubKind::List,
            "Submit for Review on App Store Connect after listing + build — then Confirm.",
            3,
            Some("https://appstoreconnect.apple.com".into()),
            None,
            Some("sign"),
        ));
    }
    if detected.tauri {
        steps.push(step(
            "submit.microsoft",
            "Submit — Microsoft Store",
            PubKind::List,
            "Partner Center product submission / certification — then Confirm.",
            3,
            Some("https://partner.microsoft.com/dashboard/products".into()),
            None,
            Some("sign"),
        ));
    }

    steps.push(step(
        "dry_run",
        "Dry-run — configure → sign → deploy plan",
        PubKind::Auto,
        "Offline preview before network deploy.",
        1,
        None,
        Some(vec![
            "shipctl".into(),
            "flow".into(),
            "--project".into(),
            ".".into(),
            "--dry-run".into(),
            "--offline".into(),
            "--skip-deploy".into(),
        ]),
        Some("tools"),
    ));

    let selected = scopes::selected(project);
    let deploy_scopes: Vec<_> = selected
        .iter()
        .filter(|s| s.provider.is_some())
        .cloned()
        .collect();
    if deploy_scopes.is_empty() {
        steps.push(step(
            "deploy",
            "Deploy — Orbit (network)",
            PubKind::Deploy,
            "Run Orbit with studio.json deploy_args. Confirm after success.",
            3,
            None,
            Some(vec![
                "shipctl".into(),
                "deploy".into(),
                "--project".into(),
                ".".into(),
            ]),
            Some("launch"),
        ));
    } else {
        for s in deploy_scopes {
            let mut run = vec![
                "shipctl".into(),
                "deploy".into(),
                "--project".into(),
                s.relative.clone(),
            ];
            run.extend(s.deploy_args.clone());
            steps.push(step(
                format!("deploy.{}", s.id),
                format!("Deploy — {}", s.label),
                PubKind::Deploy,
                format!("Orbit deploy in `{}`.", s.relative),
                3,
                None,
                Some(run),
                Some("launch"),
            ));
        }
    }

    steps.push(step(
        "live_check",
        "Live check — smoke the published surface",
        PubKind::Check,
        "Open your site/API/release page and Confirm when it looks right.",
        2,
        if detected.polar {
            Some("https://polar.sh/dashboard".into())
        } else {
            None
        },
        None,
        Some("dashboard"),
    ));

    if mode == StudioMode::General {
        steps.retain(is_general_step);
    }

    Ok(steps)
}

fn load_saved(project: &Path) -> Option<PublishState> {
    let raw = fs::read_to_string(state_path(project)).ok()?;
    serde_json::from_str(&raw).ok()
}

fn save_state(project: &Path, state: &PublishState) -> Result<()> {
    let dir = config::ship_dir(project);
    fs::create_dir_all(&dir)?;
    fs::write(state_path(project), serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn merge_statuses(fresh: &mut [PubStep], saved: &PublishState) {
    for step in fresh.iter_mut() {
        if let Some(prev) = saved.steps.iter().find(|s| s.id == step.id) {
            step.status = prev.status.clone();
            step.verified_at = prev.verified_at.clone();
        }
    }
}

/// Mark deploy* + live_check done when Orbit/last-run already prove a live ship.
fn apply_live_deploy_skips(project: &Path, steps: &mut [PubStep]) {
    let deploy = crate::pulse::inspect_deploy(project);
    if !crate::pulse::deploy_is_live(&deploy) {
        return;
    }
    let reason = if let Some(u) = deploy.urls.first() {
        format!("skipped — already live ({u})")
    } else {
        format!("skipped — already live ({})", deploy.signal)
    };
    let stamp = Some(config::now_rfc3339());
    for step in steps.iter_mut() {
        let is_deploy = step.id == "deploy" || step.id.starts_with("deploy.");
        let is_live = step.id == "live_check";
        if !(is_deploy || is_live) {
            continue;
        }
        if step.status == PubStatus::Pending {
            step.status = PubStatus::Done;
            step.verified_at = stamp.clone();
            step.detail = format!("{} · {}", step.detail, reason);
        }
        if is_live && step.entry_url.is_none() {
            step.entry_url = deploy.urls.first().cloned();
        }
    }
}

pub fn load_or_build(project: &Path) -> Result<PublishState> {
    load_state(project, true, None)
}

pub fn load_or_build_with_mode(project: &Path, mode: StudioMode) -> Result<PublishState> {
    load_state(project, true, Some(mode))
}

fn load_state(
    project: &Path,
    snap_to_pending: bool,
    mode_override: Option<StudioMode>,
) -> Result<PublishState> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let saved = load_saved(&project);
    let mode = mode_override.unwrap_or_else(|| {
        saved
            .as_ref()
            .map(|s| s.mode)
            .unwrap_or(StudioMode::General)
    });
    let mode_changed = saved.as_ref().is_some_and(|s| s.mode != mode);
    let mut steps = build_plan_for(&project, mode)?;
    let mut current = 0;
    if let Some(saved) = &saved {
        if !mode_changed {
            merge_statuses(&mut steps, saved);
            current = saved.current.min(steps.len().saturating_sub(1));
        }
    }
    apply_live_deploy_skips(&project, &mut steps);
    if snap_to_pending {
        let cur_pending = steps
            .get(current)
            .is_some_and(|s| s.status == PubStatus::Pending);
        if !cur_pending {
            if let Some(i) = steps.iter().position(|s| s.status == PubStatus::Pending) {
                current = i;
            } else if !steps.is_empty() {
                current = steps.len() - 1;
            }
        }
    }
    let minutes_total: u32 = steps.iter().map(|s| s.minutes).sum();
    let mode_note = match mode {
        StudioMode::General => "Mode: general — minimal publish spine.",
        StudioMode::Advanced => "Mode: advanced — full OAuth / official sign / listing path.",
    };
    let state = PublishState {
        schema: "ship-studio/publish/v1".into(),
        project: project.display().to_string(),
        mode,
        current,
        steps,
        notes: vec![
            mode_note.into(),
            format!("~{minutes_total} min guided publish — you finish vendor UIs; shipctl sequences."),
            "Open/Run → work on official platform or local CLI → Confirm → Next.".into(),
            "Network deploy & live release require your initiation.".into(),
        ],
    };
    save_state(&project, &state)?;
    Ok(state)
}

pub fn view(state: &PublishState) -> PublishView {
    let done_count = state
        .steps
        .iter()
        .filter(|s| s.status == PubStatus::Done || s.status == PubStatus::Skipped)
        .count();
    let finished = !state.steps.is_empty()
        && state
            .steps
            .iter()
            .all(|s| s.status == PubStatus::Done || s.status == PubStatus::Skipped);
    let current = state.steps.get(state.current).cloned();
    let minutes_total: u32 = state.steps.iter().map(|s| s.minutes).sum();
    let minutes_remaining: u32 = state
        .steps
        .iter()
        .skip(state.current)
        .filter(|s| s.status == PubStatus::Pending)
        .map(|s| s.minutes)
        .sum();
    let mut actions = vec![
        format!("shipctl publish --mode {}", state.mode.as_str()),
        "shipctl publish open".into(),
        "shipctl publish confirm".into(),
        "shipctl publish next".into(),
    ];
    if let Some(cur) = &current {
        if let Some(run) = &cur.run {
            actions.push(format!("run: {}", run.join(" ")));
        }
        if let Some(view) = &cur.desktop_view {
            actions.push(format!("desktop view: {view}"));
        }
    }
    PublishView {
        schema: state.schema.clone(),
        project: state.project.clone(),
        mode: state.mode,
        current_index: state.current,
        total: state.steps.len(),
        done_count,
        minutes_remaining,
        minutes_total,
        finished,
        current,
        steps: state.steps.clone(),
        actions,
        notes: state.notes.clone(),
    }
}

pub fn open_current(project: &Path) -> Result<PublishView> {
    let mut state = load_state(project, true, None)?;
    let idx = state.current;
    let Some(step) = state.steps.get(idx).cloned() else {
        bail!("no publish steps");
    };
    if let Some(url) = &step.entry_url {
        portal::open_url(url)?;
    }
    if step.kind == PubKind::Oauth {
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
    if step.id == "env.sprint" && std::io::stdin().is_terminal() {
        eprintln!("Tip: shipctl env --project {}  ·  or Desktop → Env / tokens", project.display());
        eprint!("Run human paste put now? [y/N] ");
        let _ = std::io::Write::flush(&mut std::io::stderr());
        let mut line = String::new();
        let _ = std::io::stdin().read_line(&mut line);
        if line.trim().eq_ignore_ascii_case("y") {
            let _ = crate::human::run_with_options(project, false, true, false);
        }
    }
    if let Some(argv) = &step.run {
        let code = execute_run(project, argv)?;
        if code == 0 && auto_done_after_run(&step) {
            if let Some(s) = state.steps.get_mut(idx) {
                s.status = PubStatus::Done;
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

pub fn verify_current(project: &Path) -> Result<(bool, String, PublishView)> {
    let mut state = load_state(project, true, None)?;
    let idx = state.current;
    let step = state
        .steps
        .get(idx)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("no current step"))?;

    let (ok, msg) = match step.kind {
        PubKind::Auto if step.id == "doctor" => {
            let report = adapters::doctor(project)?;
            (
                report.ok,
                if report.ok {
                    "doctor ok".into()
                } else {
                    "doctor failed".into()
                },
            )
        }
        PubKind::Auto if step.id == "configure" => {
            let path = config::ship_dir(project).join("studio.json");
            if path.is_file() {
                (true, "studio.json present".into())
            } else {
                let _ = config::configure(project)?;
                (true, "configure wrote studio.json".into())
            }
        }
        PubKind::Auto if step.id == "dry_run" => {
            let plan = flow::plan(project, false, true, true)?;
            (
                !plan.steps.is_empty(),
                format!("dry-run {} step(s)", plan.steps.len()),
            )
        }
        PubKind::Human if step.id == "scopes" => {
            let plan = scopes::plan_for(project);
            (
                !plan.active.is_empty(),
                format!("{} active scope(s)", plan.active.len()),
            )
        }
        PubKind::Oauth if step.id.contains("cloudflare") => {
            match run_capture("wrangler", &["whoami"], project) {
                Ok((code, text)) => (
                    code == 0 && !text.to_lowercase().contains("not logged"),
                    if code == 0 {
                        "wrangler whoami ok".into()
                    } else {
                        "wrangler not logged in".into()
                    },
                ),
                Err(e) => (false, format!("{e:#}")),
            }
        }
        PubKind::Oauth if step.id.contains("vercel") => {
            match run_capture("vercel", &["whoami"], project) {
                Ok((code, _)) => (
                    code == 0,
                    if code == 0 {
                        "vercel whoami ok".into()
                    } else {
                        "vercel not logged in".into()
                    },
                ),
                Err(e) => (false, format!("{e:#}")),
            }
        }
        PubKind::Deploy => {
            let deploy = crate::pulse::inspect_deploy(project);
            if crate::pulse::deploy_is_live(&deploy) {
                let msg = if let Some(u) = deploy.urls.first() {
                    format!("already live — {u} (Confirm skips redeploy)")
                } else {
                    format!("already live — {} (Confirm skips redeploy)", deploy.signal)
                };
                (true, msg)
            } else {
                match config::read_last_run(project) {
                    Ok(v) if v.get("ok").and_then(|x| x.as_bool()) == Some(true) => {
                        (true, "last-run ok".into())
                    }
                    _ => (
                        false,
                        "No successful deploy yet — Open/Run deploy then Confirm".into(),
                    ),
                }
            }
        }
        PubKind::Check => {
            let deploy = crate::pulse::inspect_deploy(project);
            if crate::pulse::deploy_is_live(&deploy) {
                (
                    true,
                    deploy
                        .urls
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "already live — Confirm to finish".into()),
                )
            } else {
                (
                    false,
                    "after Open/Run succeeds: shipctl publish confirm".into(),
                )
            }
        }
        PubKind::Sign if step.id.contains("scan") => {
            let ok = project.join("signet.toml").is_file();
            (
                ok,
                if ok {
                    "signet.toml present".into()
                } else {
                    "signet.toml missing".into()
                },
            )
        }
        PubKind::Human | PubKind::List | PubKind::Sign => (
            false,
            "after Open/Run succeeds: shipctl publish confirm".into(),
        ),
        _ => (false, "use confirm for this step".into()),
    };

    if ok {
        if let Some(s) = state.steps.get_mut(idx) {
            s.status = PubStatus::Done;
            s.verified_at = Some(now_rfc3339());
        }
        save_state(project, &state)?;
    }
    Ok((ok, msg, view(&state)))
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
    let mut stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("stdout"))?;
    let mut stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("stderr"))?;
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
                bail!("{bin} timed out — use publish confirm if already done");
            }
            None => std::thread::sleep(std::time::Duration::from_millis(200)),
        }
    };
    let mut text = t_out.join().unwrap_or_default();
    text.push_str(&t_err.join().unwrap_or_default());
    Ok((status.code().unwrap_or(1), text))
}

pub fn confirm_current(project: &Path) -> Result<PublishView> {
    let mut state = load_state(project, true, None)?;
    let idx = state.current;
    let Some(step) = state.steps.get_mut(idx) else {
        bail!("no current step");
    };
    step.status = PubStatus::Done;
    step.verified_at = Some(now_rfc3339());
    save_state(project, &state)?;
    Ok(view(&state))
}

pub fn next(project: &Path, force: bool) -> Result<PublishView> {
    let mut state = load_state(project, false, None)?;
    let idx = state.current;
    let Some(step) = state.steps.get(idx) else {
        bail!("no current step");
    };
    if !force && step.status == PubStatus::Pending {
        bail!(
            "current step '{}' is still pending — verify, confirm, or next --force",
            step.id
        );
    }
    if force {
        if let Some(s) = state.steps.get_mut(idx) {
            if s.status == PubStatus::Pending {
                s.status = PubStatus::Skipped;
                s.verified_at = Some(now_rfc3339());
            }
        }
    }
    let advanced = state
        .steps
        .iter()
        .enumerate()
        .skip(idx + 1)
        .find(|(_, s)| s.status == PubStatus::Pending)
        .map(|(i, _)| i)
        .unwrap_or_else(|| state.steps.len().saturating_sub(1));
    state.current = advanced;
    save_state(project, &state)?;
    Ok(view(&state))
}

#[allow(dead_code)]
pub fn reset(project: &Path) -> Result<PublishView> {
    reset_with_mode(project, StudioMode::General)
}

pub fn reset_with_mode(project: &Path, mode: StudioMode) -> Result<PublishView> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let _ = fs::remove_file(state_path(&project));
    let state = load_or_build_with_mode(&project, mode)?;
    Ok(view(&state))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_plan_includes_doctor_and_live_check() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("wrangler.toml"),
            "name = \"x\"\n# Secrets\n# - GITHUB_TOKEN\n",
        )
        .unwrap();
        let state = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(state.steps.iter().any(|s| s.id == "doctor"));
        assert!(state.steps.iter().any(|s| s.id == "live_check"));
        assert!(state.steps.iter().any(|s| s.id == "deploy" || s.id.starts_with("deploy.")));
        let v = view(&state);
        assert!(v.minutes_total >= 5);
        assert!(!v.finished);
        assert_eq!(v.mode, StudioMode::Advanced);
    }

    #[test]
    fn general_mode_omits_oauth_and_listing() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-general-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("wrangler.toml"),
            "name = \"x\"\n# Secrets\n# - GITHUB_TOKEN\n",
        )
        .unwrap();
        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(general.steps.iter().all(is_general_step));
        assert!(!general.steps.iter().any(|s| s.id.starts_with("oauth.")));
        assert!(general.steps.iter().any(|s| s.id == "doctor"));
        assert!(general.steps.iter().any(|s| s.id == "live_check"));
        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(advanced.steps.len() >= general.steps.len());
    }

    #[test]
    fn general_mode_skips_deploy_when_orbit_live() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-skip-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let run = dir.join(".orbit/runs/2026-09-12T19-33-55Z");
        fs::create_dir_all(&run).unwrap();
        fs::write(
            run.join("summary.json"),
            r#"{"ok":true,"provider":"cloudflare","url":"https://x.workers.dev"}"#,
        )
        .unwrap();
        let state = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        let deploy = state
            .steps
            .iter()
            .find(|s| s.id == "deploy" || s.id.starts_with("deploy."))
            .expect("deploy step");
        assert_eq!(deploy.status, PubStatus::Done);
        assert!(deploy.detail.contains("already live"));
        let live = state.steps.iter().find(|s| s.id == "live_check").unwrap();
        assert_eq!(live.status, PubStatus::Done);
        assert_eq!(
            live.entry_url.as_deref(),
            Some("https://x.workers.dev")
        );
    }

    #[test]
    fn confirm_then_next_advances() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-next-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let _ = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        let v = confirm_current(&dir).unwrap();
        assert_eq!(v.current.as_ref().unwrap().status, PubStatus::Done);
        let v2 = next(&dir, false).unwrap();
        assert_ne!(v2.current_index, 0);
    }

    #[test]
    fn mobile_fixture_gets_store_listing_in_advanced_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-mobile-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::create_dir_all(dir.join("ios")).unwrap();
        fs::write(
            dir.join("app.json"),
            r#"{"expo":{"name":"demo","slug":"demo"}}"#,
        )
        .unwrap();
        fs::write(dir.join("android/build.gradle"), "// stub\n").unwrap();

        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(
            advanced.steps.iter().any(|s| s.id == "listing.play"),
            "advanced missing listing.play"
        );
        assert!(
            advanced
                .steps
                .iter()
                .any(|s| s.id == "listing.app_store"),
            "advanced missing listing.app_store"
        );
        assert!(
            advanced.steps.iter().any(|s| s.id == "submit.play"),
            "advanced missing submit.play"
        );
        assert!(
            advanced.steps.iter().any(|s| s.id == "submit.app_store"),
            "advanced missing submit.app_store"
        );
        let play = advanced
            .steps
            .iter()
            .find(|s| s.id == "listing.play")
            .unwrap();
        assert_eq!(
            play.entry_url.as_deref(),
            Some("https://play.google.com/console")
        );
        assert_eq!(play.desktop_view.as_deref(), Some("sign"));

        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id == "listing.play"));
        assert!(!general.steps.iter().any(|s| s.id == "listing.app_store"));
        assert!(!general.steps.iter().any(|s| s.id.starts_with("listing.")));
        assert!(!general.steps.iter().any(|s| s.id.starts_with("submit.")));
    }

    #[test]
    fn tauri_fixture_gets_microsoft_submit_in_advanced() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-tauri-submit-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("src-tauri")).unwrap();
        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(advanced.steps.iter().any(|s| s.id == "submit.microsoft"));
        assert!(advanced.steps.iter().any(|s| s.id == "submit.app_store"));
        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id.starts_with("submit.")));
    }

    #[test]
    fn db_fixture_gets_provision_in_advanced_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-db-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("wrangler.toml"),
            "name = \"x\"\n[[d1_databases]]\nbinding = \"DB\"\ndatabase_name = \"x\"\ndatabase_id = \"…\"\n",
        )
        .unwrap();
        fs::write(dir.join(".env"), "DATABASE_URL=\n").unwrap();
        // Force neon via empty NEON key as well.
        fs::write(dir.join(".env.local"), "NEON_DATABASE_URL=\n").unwrap();

        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(
            advanced.steps.iter().any(|s| s.id == "db.provision"),
            "advanced missing db.provision"
        );
        let db = advanced
            .steps
            .iter()
            .find(|s| s.id == "db.provision")
            .unwrap();
        assert_eq!(db.desktop_view.as_deref(), Some("env"));
        assert!(db.entry_url.is_some());

        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id == "db.provision"));
    }

    #[test]
    fn ci_release_fixture_advanced_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-ci-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(
            dir.join(".github/workflows/release.yml"),
            "name: release\non: push\n",
        )
        .unwrap();
        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(advanced.steps.iter().any(|s| s.id == "ci.release"));
        let step = advanced
            .steps
            .iter()
            .find(|s| s.id == "ci.release")
            .unwrap();
        assert!(step.entry_url.is_some());
        assert_eq!(step.desktop_view.as_deref(), Some("dashboard"));
        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id == "ci.release"));
    }

    #[test]
    fn container_fixture_advanced_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-docker-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Dockerfile"), "FROM alpine\n").unwrap();
        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(advanced.steps.iter().any(|s| s.id == "container.deploy"));
        let step = advanced
            .steps
            .iter()
            .find(|s| s.id == "container.deploy")
            .unwrap();
        assert!(step.entry_url.is_some());
        assert_eq!(step.desktop_view.as_deref(), Some("portal"));
        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id == "container.deploy"));
    }

    #[test]
    fn steam_and_markets_listing_advanced_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-publish-markets-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(dir.join("steam_appid.txt"), "480\n").unwrap();
        fs::write(
            dir.join(".ship/markets.json"),
            r#"["itch","epic"]"#,
        )
        .unwrap();
        let advanced = load_or_build_with_mode(&dir, StudioMode::Advanced).unwrap();
        assert!(advanced.steps.iter().any(|s| s.id == "listing.steam"));
        assert!(advanced.steps.iter().any(|s| s.id == "listing.itch"));
        assert!(advanced.steps.iter().any(|s| s.id == "listing.epic"));
        let steam = advanced
            .steps
            .iter()
            .find(|s| s.id == "listing.steam")
            .unwrap();
        assert_eq!(
            steam.entry_url.as_deref(),
            Some("https://partner.steamgames.com/")
        );
        let general = load_or_build_with_mode(&dir, StudioMode::General).unwrap();
        assert!(!general.steps.iter().any(|s| s.id.starts_with("listing.")));
    }
}

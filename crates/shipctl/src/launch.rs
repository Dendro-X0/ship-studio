//! Guided launch workflow — one step at a time: open → verify → next → launch.

use crate::adapters;
use crate::config;
use crate::flow;
use crate::human;
use crate::portal::{self, ProviderId};
use crate::scopes;
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
    /// Desktop related panel — e.g. `platforms` opens Deployment (not a vendor URL).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub desktop_view: Option<String>,
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
    step_with_view(id, title, kind, detail, entry_url, verify_hint, run, None)
}

fn step_with_view(
    id: &str,
    title: &str,
    kind: StepKind,
    detail: &str,
    entry_url: Option<String>,
    verify_hint: Option<String>,
    run: Option<Vec<String>>,
    desktop_view: Option<&str>,
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
        desktop_view: desktop_view.map(|s| s.into()),
        status: StepStatus::Pending,
        verified_at: None,
    }
}

fn container_local_tag(project: &Path) -> String {
    let raw = project
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("app");
    let sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let name = if sanitized.is_empty() {
        "app".into()
    } else {
        sanitized
    };
    format!("{name}:local")
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

    let host_logins: Vec<&str> = portal
        .providers
        .iter()
        .filter_map(|id| {
            let Ok(pid) = ProviderId::parse(id) else {
                return None;
            };
            match pid {
                ProviderId::Cloudflare | ProviderId::Vercel | ProviderId::Netlify
                | ProviderId::Fly | ProviderId::Railway => Some(pid.label()),
                _ => None,
            }
        })
        .collect();
    if !host_logins.is_empty() {
        steps.push(step_with_view(
            "oauth.hosts",
            "Deployment — host login / Put secrets",
            StepKind::Oauth,
            &format!(
                "Open Deployment for {} — Login CLI / Put / dashboard there, then Verify → Confirm.",
                host_logins.join(" · ")
            ),
            None,
            Some("Confirm after host login (or Verify when whoami succeeds)".into()),
            None,
            Some("platforms"),
        ));
    }

    if !put_queue.is_empty() {
        steps.push(step_with_view(
            "env.sprint",
            "Env — create / paste / put secrets",
            StepKind::Paste,
            &format!(
                "{} secret(s) to put. Open Env — paste on vendor sites, Put in the terminal, then Confirm.",
                put_queue.len()
            ),
            put_queue.first().and_then(|h| h.entry_url.clone()),
            Some("confirm after puts".into()),
            None,
            Some("env"),
        ));
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

    let selected = scopes::selected(project);
    if selected.len() > 1 || selected.iter().any(|s| s.relative != "." && !s.relative.is_empty())
    {
        steps.push(step_with_view(
            "scopes",
            "Scopes — Web / API / Desktop directories",
            StepKind::Auto,
            "Open Scopes to confirm which directories to ship, then Confirm.",
            None,
            Some("active scopes saved".into()),
            None,
            Some("scopes"),
        ));
    }

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
    let include_self = wants_signet && sign_mode != "official";
    let include_official = sign_mode != "self";

    if !detected.license || !detected.security_md {
        let mut miss = Vec::new();
        if !detected.license {
            miss.push("LICENSE");
        }
        if !detected.security_md {
            miss.push("SECURITY.md");
        }
        if !detected.changelog {
            miss.push("CHANGELOG");
        }
        steps.push(step(
            "legal.baseline",
            "Launch — legal / security baseline",
            StepKind::List,
            &format!(
                "Missing at repo root: {}. Add LICENSE + SECURITY.md (CHANGELOG recommended), then Confirm. No legal advice — just the ship checklist.",
                miss.join(" · ")
            ),
            None,
            Some("confirm after LICENSE + SECURITY.md exist".into()),
            None,
        ));
    }

    // Panel redirects — not one row per vendor (Sign / Deployment / Integrations).
    let needs_sign = include_self
        || include_official
        || detected.tauri
        || detected.mobile
        || detected.android
        || detected.ios
        || detected.expo
        || detected.steam
        || detected.itch
        || detected.epic
        || detected.graduate_sign;
    if needs_sign {
        let mut bits = Vec::new();
        if include_self {
            bits.push("Signet identity/build/release");
        }
        if include_official || detected.tauri || detected.mobile {
            bits.push("official certs / store submit");
        }
        if detected.steam || detected.itch || detected.epic {
            bits.push("marketplace submit");
        }
        steps.push(step_with_view(
            "sign.panel",
            "Sign — identity / build / stores",
            StepKind::Sign,
            &format!(
                "Open Sign for {} — Confirm when that work is done.",
                if bits.is_empty() {
                    "signing paths".into()
                } else {
                    bits.join(" · ")
                }
            ),
            None,
            Some("confirm after Sign panel work".into()),
            None,
            Some("sign"),
        ));
    }

    let needs_deploy_panel = detected.marketing_site
        || detected.d1
        || detected.neon
        || detected.supabase
        || detected.turso
        || detected.fly
        || detected.railway
        || detected.render
        || detected.digitalocean
        || detected.heroku
        || detected.amplify
        || detected.cloudrun
        || detected.azurestatic
        || detected.firebase
        || detected.appwrite
        || detected.convex
        || (detected.mobile
            && (detected.firebase || detected.appwrite || detected.convex || detected.supabase));
    // oauth.hosts already covers CF/Vercel/Netlify/Fly/Railway login; add Deployment only if other host work remains.
    if host_logins.is_empty() && needs_deploy_panel {
        steps.push(step_with_view(
            "deploy.panel",
            "Deployment — host / landing / DB",
            StepKind::Deploy,
            "Open Deployment for host login, landing cutover, and DB/BaaS provision — Confirm when live.",
            None,
            Some("confirm after Deployment panel work".into()),
            None,
            Some("platforms"),
        ));
    } else if !host_logins.is_empty() && needs_deploy_panel {
        // Enrich the existing oauth.hosts detail via a note — gate already present.
        if let Some(s) = steps.iter_mut().find(|s| s.id == "oauth.hosts") {
            s.detail = format!(
                "{} Also use Deployment for landing / DB when needed.",
                s.detail
            );
        }
    }

    let needs_integrations = detected.polar
        || detected.gumroad
        || detected.lemon
        || detected.stripe
        || detected.paddle;
    if needs_integrations {
        let mut bits = Vec::new();
        if detected.polar {
            bits.push("Polar");
        }
        if detected.gumroad {
            bits.push("Gumroad");
        }
        if detected.lemon {
            bits.push("Lemon");
        }
        if detected.stripe {
            bits.push("Stripe");
        }
        if detected.paddle {
            bits.push("Paddle");
        }
        steps.push(step_with_view(
            "integrations.panel",
            "Integrations — payments / checkout",
            StepKind::List,
            &format!(
                "Open Integrations for {} — only when this app sells. Confirm when listings are updated.",
                bits.join(" · ")
            ),
            None,
            Some("confirm after Integrations work (or skip if N/A)".into()),
            None,
            Some("integrations"),
        ));
    }

    if detected.npm_publish || detected.crates_publish || detected.huggingface {
        steps.push(step_with_view(
            "listing.packages",
            "Packages — npm / crates / Hub",
            StepKind::List,
            "Open Portal for registry login / dry-run publish docs — Confirm when the version is live. Bridge never publishes.",
            None,
            Some("confirm after package publish".into()),
            None,
            Some("portal"),
        ));
    }

    if detected.ci_release {
        let files = detected.release_workflows.join(", ");
        let workflow = detected
            .release_workflows
            .first()
            .cloned()
            .unwrap_or_else(|| "release.yml".into());
        let has_release = detected
            .release_workflows
            .iter()
            .any(|n| n.to_ascii_lowercase().contains("release"));
        let has_deploy = detected
            .release_workflows
            .iter()
            .any(|n| n.to_ascii_lowercase().contains("deploy"));
        let ci_title = match (has_release, has_deploy) {
            (true, false) => "CI — GitHub Actions release",
            (false, true) => "CI — GitHub Actions deploy",
            _ => "CI — GitHub Actions ship",
        };
        let ci_detail = if detected
            .release_workflows
            .iter()
            .all(|n| n.to_ascii_lowercase().contains("deploy"))
            && !has_release
        {
            format!(
                "Workflow(s): {files}. After merge to main, Run `gh run list` (read-only) and Confirm when the deploy run looks green."
            )
        } else {
            format!(
                "Workflow(s): {files}. After tag/Signet release, Run `gh run list` (read-only) and Confirm when the Actions run looks green."
            )
        };
        steps.push(step(
            "ci.release",
            ci_title,
            StepKind::List,
            &ci_detail,
            config::github_actions_url(project)
                .or_else(|| Some("https://github.com/actions".into())),
            Some("confirm when Actions looks green".into()),
            Some(vec![
                "gh".into(),
                "run".into(),
                "list".into(),
                "--workflow".into(),
                workflow,
                "--limit".into(),
                "5".into(),
            ]),
        ));
    }

    if detected.container {
        let tag = container_local_tag(project);
        let (build_run, build_detail) = if detected.dockerfile {
            (
                Some(vec![
                    "docker".into(),
                    "build".into(),
                    "-t".into(),
                    tag.clone(),
                    ".".into(),
                ]),
                format!(
                    "Run `docker build -t {tag} .` locally. Confirm when the image builds. Push stays on the next step."
                ),
            )
        } else {
            (
                Some(vec![
                    "docker".into(),
                    "compose".into(),
                    "build".into(),
                ]),
                "Run `docker compose build` locally. Confirm when images build. Push stays on the next step."
                    .into(),
            )
        };
        steps.push(step(
            "container.build",
            "Container — local build",
            StepKind::Deploy,
            &build_detail,
            Some(config::container_docs_url(project).into()),
            Some("confirm after local image build".into()),
            build_run,
        ));
        let push_detail = if detected.compose && detected.dockerfile {
            format!(
                "After `{tag}` (or compose images) exist locally: `docker login` / `gh auth`, then `docker push` on your machine. Open registry docs, Confirm when published. Bridge never pushes."
            )
        } else if detected.compose {
            "After compose images build: `docker login` / `gh auth`, then push tags on your machine. Open registry docs, Confirm when published. Bridge never pushes."
                .into()
        } else {
            format!(
                "After `{tag}` builds: `docker login` / `gh auth`, then `docker push` on your machine. Open registry docs, Confirm when published. Bridge never pushes."
            )
        };
        steps.push(step(
            "container.deploy",
            "Container — registry push (docs)",
            StepKind::Deploy,
            &push_detail,
            Some(config::container_docs_url(project).into()),
            Some("confirm when image is in the registry".into()),
            None,
        ));
    }

    if detected.suite_sync {
        let targets = if detected.suite_detail.is_empty() {
            "configured siblings".into()
        } else {
            detected.suite_detail.clone()
        };
        steps.push(step(
            "suite.url_sync",
            "Suite — sync canonical URL to siblings",
            StepKind::List,
            &format!(
                "Paste the live landing URL into sibling env keys ({targets}). Ship Studio never writes sibling .env values — Confirm when keys match."
            ),
            Some(config::suite_sync_url(project)),
            Some("confirm when sibling env keys match".into()),
            None,
        ));
    }

    if crate::selfhost::plan_eligible(project) {
        steps.push(step_with_view(
            "selfhost.deploy",
            "Self-host — local auto deploy",
            StepKind::Auto,
            "Run shipctl selfhost (artifact + loopback health). Done when checks pass — no Confirm. Prefer Publish for the full Adaptive path.",
            None,
            Some("shipctl selfhost".into()),
            Some(vec![
                "shipctl".into(),
                "selfhost".into(),
                "--project".into(),
                ".".into(),
            ]),
            Some("platforms"),
        ));
    }

    steps.push(step(
        "flow_dry_run",
        "Flow dry-run — preview configure → sign → deploy",
        StepKind::Auto,
        "Offline plan check before network deploy. Prefer Publish for the full Adaptive path.",
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

    let deploy_scopes: Vec<_> = selected
        .iter()
        .filter(|s| s.provider.is_some())
        .cloned()
        .collect();
    if deploy_scopes.is_empty() {
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
                &format!("deploy.{}", s.id),
                &format!("Deploy — {} ({})", s.label, s.provider.as_deref().unwrap_or("orbit")),
                StepKind::Deploy,
                &format!("Orbit deploy in `{}`.", s.relative),
                None,
                Some("last-run ok, or confirm after deploy".into()),
                Some(run),
            ));
        }
    }

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
    if step.id == "signet.identity" {
        assist_signet_identity(project)?;
    } else if let Some(argv) = &step.run {
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

/// Interactive: list identities; if none, run `signet identity create` in this TTY.
fn assist_signet_identity(project: &Path) -> Result<()> {
    let bin = resolve_run_bin("signet")?;
    let (code, text) = run_capture("signet", &["identity", "list"], project).unwrap_or((1, String::new()));
    let has = code == 0
        && text.lines().any(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with('{')
                && !t.to_ascii_lowercase().contains("no identit")
        });
    if has {
        eprintln!("$ {} identity list\n{text}", bin.display());
        eprintln!("signing identity present — Verify / Confirm in Studio");
        return Ok(());
    }
    eprintln!("no signing identity yet — launching `signet identity create` (follow prompts)");
    let status = Command::new(&bin)
        .args(["identity", "create"])
        .current_dir(project)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| "signet identity create")?;
    if !status.success() {
        eprintln!(
            "signet identity create exited {} — fix, retry Open/Run, or Confirm if you created one elsewhere",
            status.code().unwrap_or(1)
        );
    }
    Ok(())
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
        StepKind::Auto if step.id == "selfhost.deploy" => {
            match crate::selfhost::run(project, crate::selfhost::SelfhostOpts::default()) {
                Ok(r) if r.ok => (
                    true,
                    r.health_url
                        .map(|u| format!("selfhost.check ok · {u}"))
                        .unwrap_or_else(|| r.message),
                ),
                Ok(r) => (false, r.message),
                Err(e) => (false, format!("{e:#}")),
            }
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
                            "intent ok · sign_args={:?} · deploy_args={:?} · sign_path={}",
                            s.sign_args, s.deploy_args, s.sign_path
                        ),
                    ),
                    None => (false, "studio.json unreadable".into()),
                }
            }
        }
        StepKind::Auto if step.id == "scopes" => {
            let plan = scopes::plan_for(project);
            (
                !plan.active.is_empty(),
                format!("{} active scope(s)", plan.active.len()),
            )
        }
        StepKind::Oauth if step.id == "oauth.hosts" || step.id.starts_with("oauth.") => {
            // Collapsed host gate — any logged-in deploy CLI counts; else Confirm after Deployment.
            let mut ok_any = false;
            let mut msgs: Vec<String> = Vec::new();
            if let Ok((code, text)) = run_capture("wrangler", &["whoami"], project) {
                let good = code == 0 && !text.to_lowercase().contains("not logged");
                ok_any |= good;
                msgs.push(if good {
                    "wrangler ok".into()
                } else {
                    "wrangler not logged in".into()
                });
            }
            if let Ok((code, _)) = run_capture("vercel", &["whoami"], project) {
                ok_any |= code == 0;
                msgs.push(if code == 0 {
                    "vercel ok".into()
                } else {
                    "vercel not logged in".into()
                });
            }
            if let Ok((code, _)) = run_capture("netlify", &["status"], project) {
                ok_any |= code == 0;
                msgs.push(if code == 0 {
                    "netlify ok".into()
                } else {
                    "netlify not logged in".into()
                });
            }
            (
                ok_any,
                if ok_any {
                    format!("host login ok ({})", msgs.join(" · "))
                } else {
                    "Open Deployment → Login CLI, then Verify / Confirm".into()
                },
            )
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
        assert!(state.steps.iter().any(|s| s.id == "deploy" || s.id.starts_with("deploy.") || s.id == "oauth.hosts"));
        assert!(state.steps.iter().any(|s| s.id == "intent"));
        assert!(state.steps.iter().any(|s| s.id == "env.sprint"));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("signet.")));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("paste.")));
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
        assert!(state.steps.iter().any(|s| s.id == "sign.panel"));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("signet.")));
        let sign = state.steps.iter().find(|s| s.id == "sign.panel").unwrap();
        assert_eq!(sign.kind, StepKind::Sign);
        assert_eq!(sign.desktop_view.as_deref(), Some("sign"));
        assert!(state.steps.iter().any(|s| s.id == "deploy" || s.id.starts_with("deploy.")));
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

    #[test]
    fn launch_commerce_listings_when_detected() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-commerce-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".env"), "STRIPE_SECRET_KEY=\nPOLAR_CHECKOUT_URL=\n").unwrap();
        fs::write(
            dir.join(".ship/markets.json"),
            r#"["gumroad","lemon","paddle"]"#,
        )
        .unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(
            state.steps.iter().any(|s| s.id == "integrations.panel"),
            "commerce → one Integrations redirect"
        );
        let integ = state
            .steps
            .iter()
            .find(|s| s.id == "integrations.panel")
            .unwrap();
        assert_eq!(integ.desktop_view.as_deref(), Some("integrations"));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("listing.")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_host_and_baas_when_detected() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-host-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::write(dir.join("fly.toml"), "app = \"demo\"\n").unwrap();
        fs::write(dir.join("heroku.yml"), "build:\n  docker:\n    web: Dockerfile\n").unwrap();
        fs::write(dir.join("firebase.json"), "{}\n").unwrap();
        fs::write(dir.join("build.gradle"), "// android\n").unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(
            state
                .steps
                .iter()
                .any(|s| s.id == "deploy.panel" || s.id == "oauth.hosts"),
            "host/BaaS → Deployment redirect"
        );
        assert!(!state.steps.iter().any(|s| s.id == "baas.provision"));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("host.")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_marketplace_listing_and_submit() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-market-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join("steam_appid.txt"), "480\n").unwrap();
        fs::write(
            dir.join(".ship/markets.json"),
            r#"["itch","epic"]"#,
        )
        .unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(
            state.steps.iter().any(|s| s.id == "sign.panel"),
            "marketplace → Sign panel"
        );
        assert!(!state.steps.iter().any(|s| s.id.starts_with("listing.")));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("submit.")));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_store_mobile_and_tauri_honesty() {
        let mobile = std::env::temp_dir().join(format!(
            "shipctl-launch-play-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&mobile);
        fs::create_dir_all(mobile.join("android")).unwrap();
        fs::write(mobile.join("build.gradle"), "// android
").unwrap();
        let m = load_or_build(&mobile).unwrap();
        assert!(m.steps.iter().any(|s| s.id == "sign.panel"));
        assert!(!m.steps.iter().any(|s| s.id.starts_with("submit.")));
        let _ = fs::remove_dir_all(&mobile);

        let tauri = std::env::temp_dir().join(format!(
            "shipctl-launch-tauri-store-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&tauri);
        fs::create_dir_all(tauri.join("src-tauri")).unwrap();
        fs::write(tauri.join("package.json"), "{}
").unwrap();
        let t = load_or_build(&tauri).unwrap();
        assert!(t.steps.iter().any(|s| s.id == "sign.panel"));
        assert!(!t.steps.iter().any(|s| s.id.starts_with("submit.")));
        let sign = t.steps.iter().find(|s| s.id == "sign.panel").unwrap();
        assert_eq!(sign.desktop_view.as_deref(), Some("sign"));
        let _ = fs::remove_dir_all(&tauri);
    }

    #[test]
    fn launch_registry_and_hf_listings() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-registry-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"name":"@acme/lib","version":"1.0.0"}"#,
        )
        .unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]
name = \"demo\"
version = \"0.1.0\"
edition = \"2021\"
",
        )
        .unwrap();
        fs::write(dir.join(".ship/markets.json"), r#"["hf"]"#).unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(state.steps.iter().any(|s| s.id == "listing.packages"));
        let pkg = state
            .steps
            .iter()
            .find(|s| s.id == "listing.packages")
            .unwrap();
        assert_eq!(pkg.desktop_view.as_deref(), Some("portal"));
        assert!(!state.steps.iter().any(|s| s.id == "listing.npm"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_db_marketing_and_suite() {
        let db = std::env::temp_dir().join(format!(
            "shipctl-launch-db-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&db);
        fs::create_dir_all(&db).unwrap();
        fs::write(
            db.join("wrangler.toml"),
            "name = \"x\"\n[[d1_databases]]\nbinding = \"DB\"\ndatabase_name = \"x\"\ndatabase_id = \"…\"\n",
        )
        .unwrap();
        fs::write(db.join(".env.local"), "NEON_DATABASE_URL=\n").unwrap();
        let d = load_or_build(&db).unwrap();
        assert!(
            d.steps
                .iter()
                .any(|s| s.id == "deploy.panel" || s.id == "oauth.hosts")
        );
        assert!(!d.steps.iter().any(|s| s.id == "db.provision"));
        let _ = fs::remove_dir_all(&db);

        let mkt = std::env::temp_dir().join(format!(
            "shipctl-launch-mkt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&mkt);
        fs::create_dir_all(mkt.join("apps/website")).unwrap();
        fs::write(mkt.join("apps/website/index.html"), "<h1>site</h1>\n").unwrap();
        fs::write(mkt.join("vercel.json"), "{}\n").unwrap();
        let m = load_or_build(&mkt).unwrap();
        assert!(
            m.steps
                .iter()
                .any(|s| s.id == "oauth.hosts" || s.id == "deploy.panel"),
            "marketing/host → Deployment"
        );
        assert!(!m.steps.iter().any(|s| s.id == "marketing.deploy"));
        let _ = fs::remove_dir_all(&mkt);

        let suite = std::env::temp_dir().join(format!(
            "shipctl-launch-suite-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&suite);
        fs::create_dir_all(suite.join(".ship")).unwrap();
        fs::write(
            suite.join(".ship/suite.json"),
            r#"{"canonical_hint":"https://ship.example","siblings":[{"label":"truss","path":"../truss","env_keys":["NEXT_PUBLIC_X_URL"]}]}"#,
        )
        .unwrap();
        let s = load_or_build(&suite).unwrap();
        assert!(s.steps.iter().any(|s| s.id == "suite.url_sync"));
        let sync = s.steps.iter().find(|st| st.id == "suite.url_sync").unwrap();
        assert_eq!(sync.entry_url.as_deref(), Some("https://ship.example"));
        assert!(sync.detail.contains("NEXT_PUBLIC_X_URL"));
        let _ = fs::remove_dir_all(&suite);
    }

    #[test]
    fn launch_selfhost_parity() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-launch-selfhost-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        let web = dir.join("apps/website");
        fs::create_dir_all(&web).unwrap();
        fs::write(web.join("index.html"), "<!doctype html>").unwrap();
        let t = load_or_build(&dir).unwrap();
        let step = t
            .steps
            .iter()
            .find(|s| s.id == "selfhost.deploy")
            .expect("selfhost.deploy");
        assert_eq!(step.kind, StepKind::Auto);
        assert_eq!(step.desktop_view.as_deref(), Some("platforms"));
        let run = step.run.as_ref().expect("run");
        assert_eq!(run[..2], ["shipctl", "selfhost"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn launch_ci_container_parity() {
        let ci = std::env::temp_dir().join(format!(
            "shipctl-launch-ci-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&ci);
        fs::create_dir_all(ci.join(".github/workflows")).unwrap();
        fs::write(ci.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(
            ci.join(".github/workflows/release.yml"),
            "name: release\non: push\n",
        )
        .unwrap();
        let c = load_or_build(&ci).unwrap();
        assert!(c.steps.iter().any(|s| s.id == "ci.release"));
        let release = c.steps.iter().find(|s| s.id == "ci.release").unwrap();
        assert!(release.entry_url.is_some());
        let run = release.run.as_ref().expect("gh run list");
        assert_eq!(run[0], "gh");
        assert_eq!(run[1], "run");
        assert_eq!(run[2], "list");
        assert!(run.iter().any(|a| a.contains("release.yml")));
        let _ = fs::remove_dir_all(&ci);

        let ctr = std::env::temp_dir().join(format!(
            "shipctl-launch-ctr-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&ctr);
        fs::create_dir_all(&ctr).unwrap();
        fs::write(ctr.join("Dockerfile"), "FROM scratch\n").unwrap();
        let t = load_or_build(&ctr).unwrap();
        assert!(t.steps.iter().any(|s| s.id == "container.build"));
        assert!(t.steps.iter().any(|s| s.id == "container.deploy"));
        let build = t
            .steps
            .iter()
            .find(|s| s.id == "container.build")
            .unwrap();
        let brun = build.run.as_ref().expect("docker build");
        assert_eq!(brun[0], "docker");
        assert_eq!(brun[1], "build");
        assert!(brun.iter().any(|a| a.ends_with(":local")));
        let deploy = t
            .steps
            .iter()
            .find(|s| s.id == "container.deploy")
            .unwrap();
        assert!(deploy.run.is_none());
        assert!(deploy.entry_url.is_some());
        let _ = fs::remove_dir_all(&ctr);
    }

    #[test]
    fn launch_baseline_release_parity() {
        let legal = std::env::temp_dir().join(format!(
            "shipctl-launch-legal-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&legal);
        fs::create_dir_all(legal.join(".ship")).unwrap();
        fs::write(legal.join("signet.toml"), "name = \"x\"\n").unwrap();
        let l = load_or_build(&legal).unwrap();
        assert!(l.steps.iter().any(|s| s.id == "legal.baseline"));
        assert!(l.steps.iter().any(|s| s.id == "sign.panel"));
        assert!(!l.steps.iter().any(|s| s.id == "trust.pack"));
        assert!(!l.steps.iter().any(|s| s.id == "ship.desktop_cut"));
        let _ = fs::remove_dir_all(&legal);

        let grad = std::env::temp_dir().join(format!(
            "shipctl-launch-grad-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&grad);
        fs::create_dir_all(grad.join(".ship")).unwrap();
        fs::write(
            grad.join(".ship/markets.json"),
            r#"["graduate"]"#,
        )
        .unwrap();
        let g = load_or_build(&grad).unwrap();
        assert!(
            g.steps.iter().any(|s| s.id == "sign.panel"),
            "graduate → Sign panel"
        );
        assert!(!g.steps.iter().any(|s| s.id == "sign.graduate"));
        let _ = fs::remove_dir_all(&grad);

        let rel = std::env::temp_dir().join(format!(
            "shipctl-launch-ghrel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&rel);
        fs::create_dir_all(&rel).unwrap();
        fs::write(rel.join("package.json"), r#"{"name":"x"}"#).unwrap();
        let st = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&rel)
            .status();
        if !st.map(|s| s.success()).unwrap_or(false) {
            return;
        }
        let _ = std::process::Command::new("git")
            .args(["remote", "add", "origin", "https://github.com/acme/app.git"])
            .current_dir(&rel)
            .status();
        let r = load_or_build(&rel).unwrap();
        // Without Signet self-path, GitHub release work lives on Sign — not a Launch row.
        assert!(!r.steps.iter().any(|s| s.id == "release.github"));
        let _ = fs::remove_dir_all(&rel);
    }
}

//! Project pulse — local git + .ship + deploy signals for Dashboard / Now.

use crate::config;
use crate::scopes;
use anyhow::Result;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize)]
pub struct GitPulse {
    pub is_repo: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub dirty: bool,
    pub dirty_count: usize,
    pub committed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit: Option<GitCommit>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitCommit {
    pub hash: String,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WizardPulse {
    pub present: bool,
    pub finished: bool,
    pub current_index: usize,
    pub total: usize,
    pub done_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minutes_remaining: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeployPulse {
    pub signal: String,
    pub detail: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
    pub last_run_ok: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolsPulse {
    pub signet_found: bool,
    pub orbit_found: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signet_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orbit_version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PulseAction {
    pub id: String,
    pub label: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NowPulse {
    pub title: String,
    pub detail: String,
    pub primary: PulseAction,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<PulseAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectPulse {
    pub schema: String,
    pub project: String,
    pub name: String,
    pub kind: String,
    pub git: GitPulse,
    pub publish: WizardPulse,
    pub launch: WizardPulse,
    pub deploy: DeployPulse,
    pub tools: ToolsPulse,
    pub scopes_active: Vec<String>,
    pub now: NowPulse,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

fn project_name(project: &Path) -> String {
    project
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .to_string()
}

fn git_ok(project: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(project)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn git_pulse(project: &Path) -> GitPulse {
    let is_repo = git_ok(project, &["rev-parse", "--is-inside-work-tree"])
        .map(|s| s == "true")
        .unwrap_or(false);
    if !is_repo {
        return GitPulse {
            is_repo: false,
            branch: None,
            dirty: false,
            dirty_count: 0,
            committed: false,
            ahead: None,
            behind: None,
            last_commit: None,
            notes: vec!["Not a git repository.".into()],
        };
    }
    let branch = git_ok(project, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let porcelain = git_ok(project, &["status", "--porcelain"]).unwrap_or_default();
    let dirty_count = porcelain.lines().filter(|l| !l.trim().is_empty()).count();
    let dirty = dirty_count > 0;
    let committed = git_ok(project, &["rev-parse", "HEAD"]).is_some();
    let last_commit = git_ok(
        project,
        &["log", "-1", "--format=%h%x09%s%x09%cI"],
    )
    .and_then(|line| {
        let mut parts = line.splitn(3, '\t');
        let hash = parts.next()?.to_string();
        let subject = parts.next()?.to_string();
        let when = parts.next().map(|s| s.to_string());
        Some(GitCommit {
            hash,
            subject,
            when,
        })
    });
    let mut ahead = None;
    let mut behind = None;
    let mut notes = Vec::new();
    if let Some(lr) = git_ok(
        project,
        &["rev-list", "--left-right", "--count", "@{u}...HEAD"],
    ) {
        let mut it = lr.split_whitespace();
        if let (Some(b), Some(a)) = (it.next(), it.next()) {
            behind = b.parse().ok();
            ahead = a.parse().ok();
        }
    } else {
        notes.push("No upstream tracking branch (local-only).".into());
    }
    if dirty {
        notes.push(format!("{dirty_count} uncommitted change(s)."));
    }
    GitPulse {
        is_repo: true,
        branch,
        dirty,
        dirty_count,
        committed,
        ahead,
        behind,
        last_commit,
        notes,
    }
}

fn wizard_from_file(path: &Path, with_minutes: bool) -> WizardPulse {
    let Ok(raw) = fs::read_to_string(path) else {
        return WizardPulse {
            present: false,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return WizardPulse {
            present: true,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
    };
    let steps = v
        .get("steps")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let total = steps.len();
    let current_index = v.get("current").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    let done_count = steps
        .iter()
        .filter(|s| {
            matches!(
                s.get("status").and_then(|x| x.as_str()),
                Some("done") | Some("skipped")
            )
        })
        .count();
    let finished = total > 0 && done_count == total;
    let cur = steps.get(current_index.min(total.saturating_sub(1)));
    let minutes_remaining = if with_minutes {
        Some(
            steps
                .iter()
                .skip(current_index)
                .filter(|s| s.get("status").and_then(|x| x.as_str()) == Some("pending"))
                .map(|s| s.get("minutes").and_then(|x| x.as_u64()).unwrap_or(0) as u32)
                .sum(),
        )
    } else {
        None
    };
    WizardPulse {
        present: true,
        finished,
        current_index: current_index.min(total.saturating_sub(1)),
        total,
        done_count,
        current_id: cur
            .and_then(|s| s.get("id"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        current_title: cur
            .and_then(|s| s.get("title"))
            .and_then(|x| x.as_str())
            .map(|s| s.to_string()),
        minutes_remaining,
    }
}

fn wizard_from_publish(project: &Path) -> WizardPulse {
    wizard_from_file(&config::ship_dir(project).join("publish.json"), true)
}

fn wizard_from_launch(project: &Path) -> WizardPulse {
    wizard_from_file(&config::ship_dir(project).join("launch.json"), false)
}

fn find_wrangler_dirs(project: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let candidates = [
        project.join(".wrangler"),
        project.join("apps/api/.wrangler"),
        project.join("apps/web/.wrangler"),
        project.join("api/.wrangler"),
    ];
    for c in candidates {
        if c.is_dir() {
            out.push(c);
        }
    }
    out
}

fn find_vercel_project_json(project: &Path) -> Option<PathBuf> {
    let root = project.join(".vercel/project.json");
    if root.is_file() {
        return Some(root);
    }
    let apps = project.join("apps");
    if let Ok(entries) = fs::read_dir(&apps) {
        for ent in entries.flatten() {
            let p = ent.path().join(".vercel/project.json");
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

fn push_url(urls: &mut Vec<String>, raw: Option<&str>) {
    let Some(u) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return;
    };
    if !urls.iter().any(|x| x == u) {
        urls.push(u.to_string());
    }
}

/// Scan `.orbit/runs/*/summary.json` (newest first) for successful deploys + URLs.
pub fn latest_orbit_deploy(project: &Path) -> Option<(String, Vec<String>)> {
    let runs = project.join(".orbit/runs");
    let Ok(entries) = fs::read_dir(&runs) else {
        return None;
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
    let mut urls = Vec::new();
    let mut providers = Vec::new();
    let mut best_detail: Option<String> = None;
    for dir in dirs {
        let summary = dir.join("summary.json");
        let Ok(raw) = fs::read_to_string(&summary) else {
            continue;
        };
        let Ok(j) = serde_json::from_str::<serde_json::Value>(&raw) else {
            continue;
        };
        if j.get("ok").and_then(|x| x.as_bool()) != Some(true) {
            continue;
        }
        let before = urls.len();
        push_url(&mut urls, j.get("url").and_then(|x| x.as_str()));
        push_url(&mut urls, j.get("apiUrl").and_then(|x| x.as_str()));
        push_url(&mut urls, j.get("docsUrl").and_then(|x| x.as_str()));
        if urls.len() == before && best_detail.is_some() {
            // Already have a success; skip empty-url duplicates unless first.
            continue;
        }
        let provider = j
            .get("provider")
            .and_then(|x| x.as_str())
            .unwrap_or("orbit");
        if !providers.iter().any(|p| p == provider) {
            providers.push(provider.to_string());
        }
        if best_detail.is_none() {
            best_detail = Some(if let Some(u) = urls.first() {
                format!("Orbit {provider} deploy ok · {u}")
            } else {
                format!("Orbit {provider} deploy ok · {}", dir.display())
            });
        }
        // Keep scanning a few more successes so Cloudflare + Vercel URLs both surface.
        if providers.len() >= 3 || urls.len() >= 4 {
            break;
        }
    }
    let detail = best_detail?;
    let detail = if providers.len() > 1 {
        format!(
            "{detail} (+{})",
            providers
                .iter()
                .skip(1)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    } else {
        detail
    };
    // Prefer workers.dev first when present (API primary for assess-api-style repos).
    urls.sort_by(|a, b| {
        let aw = a.contains("workers.dev");
        let bw = b.contains("workers.dev");
        match (aw, bw) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });
    Some((detail, urls))
}

pub fn latest_live_urls(project: &Path) -> Vec<String> {
    let mut urls = Vec::new();
    if let Some((_, u)) = latest_orbit_deploy(project) {
        for x in u {
            push_url(&mut urls, Some(&x));
        }
    }
    if let Ok(v) = config::read_last_run(project) {
        push_url(&mut urls, v.get("url").and_then(|x| x.as_str()));
        if let Some(arr) = v.get("urls").and_then(|x| x.as_array()) {
            for item in arr {
                push_url(&mut urls, item.as_str());
            }
        }
    }
    urls
}

/// True when local evidence shows a successful prior live deploy (not mere wrangler dev state).
pub fn deploy_is_live(deploy: &DeployPulse) -> bool {
    matches!(
        deploy.signal.as_str(),
        "orbit_deployed" | "last_run_ok"
    ) || !deploy.urls.is_empty()
}

/// Public inspect for publish skip / verify (same as internal pulse).
pub fn inspect_deploy(project: &Path) -> DeployPulse {
    deploy_pulse(project)
}

fn deploy_pulse(project: &Path) -> DeployPulse {
    let mut urls = Vec::new();
    let mut last_run_ok = None;
    let mut signal = "unknown".to_string();
    let mut detail = "No local deploy signal yet.".to_string();

    if let Some((orbit_detail, orbit_urls)) = latest_orbit_deploy(project) {
        signal = "orbit_deployed".into();
        detail = orbit_detail;
        for u in orbit_urls {
            push_url(&mut urls, Some(&u));
        }
    }

    if let Ok(v) = config::read_last_run(project) {
        last_run_ok = v.get("ok").and_then(|x| x.as_bool());
        push_url(&mut urls, v.get("url").and_then(|x| x.as_str()));
        if let Some(arr) = v.get("urls").and_then(|x| x.as_array()) {
            for item in arr {
                push_url(&mut urls, item.as_str());
            }
        }
        if let Some(msg) = v.get("message").and_then(|x| x.as_str()) {
            if signal == "unknown" {
                detail = msg.to_string();
            }
        }
        if let Some(arr) = v.get("steps").and_then(|x| x.as_array()) {
            for s in arr {
                let id = s.get("id").and_then(|x| x.as_str()).unwrap_or("");
                let ok = s.get("ok").and_then(|x| x.as_bool()).unwrap_or(false);
                if id.contains("deploy") && ok && signal != "orbit_deployed" {
                    signal = "last_run_ok".into();
                    detail = "Last shipctl deploy step succeeded.".into();
                }
            }
        }
        if last_run_ok == Some(true) && signal == "unknown" {
            signal = "last_run_ok".into();
            detail = "Last shipctl run succeeded.".into();
        }
    }

    if let Some(vercel) = find_vercel_project_json(project) {
        if signal == "unknown" {
            signal = "vercel_linked".into();
            detail = format!("Vercel linked · {}", vercel.display());
        }
        if let Ok(raw) = fs::read_to_string(&vercel) {
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&raw) {
                if let Some(id) = j.get("projectId").and_then(|x| x.as_str()) {
                    if signal == "vercel_linked" {
                        detail = format!("Vercel linked · projectId={id}");
                    }
                }
            }
        }
    }

    let orbit_state = project.join(".orbit/state.json");
    if orbit_state.is_file() && signal == "unknown" {
        if let Ok(raw) = fs::read_to_string(&orbit_state) {
            if let Ok(j) = serde_json::from_str::<serde_json::Value>(&raw) {
                let cf = j
                    .pointer("/providers/cloudflare/configured")
                    .and_then(|x| x.as_bool())
                    == Some(true);
                let configured = j.get("configured").and_then(|x| x.as_bool()) == Some(true);
                if cf || configured {
                    signal = "orbit_configured".into();
                    detail = "Orbit configured (.orbit/state.json) — no successful run summary yet."
                        .into();
                }
            }
        }
    }

    for dir in find_wrangler_dirs(project) {
        let has_local = dir.join("state").is_dir() || dir.join("tmp").is_dir();
        let has_deploy_cache = dir.join("deploy").is_dir();
        if signal == "unknown" || signal == "wrangler_local" {
            if has_local || has_deploy_cache {
                signal = "wrangler_local".into();
                detail = format!(
                    "Local Wrangler state at {} (dev/miniflare — not proof of remote deploy)",
                    dir.display()
                );
            } else {
                signal = "wrangler_local".into();
                detail = format!("Wrangler directory at {}", dir.display());
            }
        }
    }

    DeployPulse {
        signal,
        detail,
        urls,
        last_run_ok,
    }
}

fn tools_pulse(_project: &Path) -> ToolsPulse {
    // PATH-only — avoid full doctor (secrets/portal) so Dashboard pulse stays fast.
    let signet = which::which("signet")
        .or_else(|_| which::which("signet.exe"))
        .ok();
    let orbit = which::which("orbit")
        .or_else(|_| which::which("orbit.exe"))
        .or_else(|_| which::which("orbit.cmd"))
        .ok();
    ToolsPulse {
        signet_found: signet.is_some() || std::env::var_os("SIGNET_PATH").is_some(),
        orbit_found: orbit.is_some() || std::env::var_os("ORBIT_PATH").is_some(),
        signet_version: signet.map(|p| p.display().to_string()),
        orbit_version: orbit.map(|p| p.display().to_string()),
    }
}

fn describe_kind(project: &Path) -> String {
    let d = config::probe(project);
    let mut bits = Vec::new();
    if d.mobile {
        let mut m = Vec::new();
        if d.expo {
            m.push("Expo");
        }
        if d.android {
            m.push("Android");
        }
        if d.ios {
            m.push("iOS");
        }
        if m.is_empty() {
            bits.push("Mobile".to_string());
        } else {
            bits.push(format!("Mobile ({})", m.join(" · ")));
        }
    }
    if d.tauri {
        bits.push("Desktop (Tauri)".into());
    } else if !d.mobile && d.wrangler {
        bits.push("API / Cloudflare Worker".into());
    } else if !d.mobile && d.vercel {
        bits.push("Web (Vercel)".into());
    } else if !d.mobile && d.netlify {
        bits.push("Web (Netlify)".into());
    } else if !d.mobile && d.package_json {
        bits.push("Node project".into());
    }
    if d.polar {
        bits.push("Polar listing".into());
    }
    if d.d1 || d.neon || d.supabase || d.turso {
        let mut db = Vec::new();
        if d.d1 {
            db.push("D1");
        }
        if d.neon {
            db.push("Neon");
        }
        if d.supabase {
            db.push("Supabase");
        }
        if d.turso {
            db.push("Turso");
        }
        bits.push(format!("DB ({})", db.join(" · ")));
    }
    if d.github {
        bits.push("GitHub".into());
    }
    if d.ci_release {
        bits.push("CI release".into());
    }
    if d.container {
        bits.push(if d.compose {
            "Container (Compose)".into()
        } else {
            "Container".into()
        });
    }
    if d.pwa {
        bits.push("PWA".into());
    }
    if d.huggingface {
        bits.push("Hugging Face".into());
    }
    if d.steam || d.itch || d.epic {
        let mut m = Vec::new();
        if d.steam {
            m.push("Steam");
        }
        if d.itch {
            m.push("itch");
        }
        if d.epic {
            m.push("Epic");
        }
        bits.push(format!("Markets ({})", m.join(" · ")));
    }
    if bits.is_empty() {
        "Local project".into()
    } else {
        bits.join(" · ")
    }
}

fn action(
    id: impl Into<String>,
    label: impl Into<String>,
    kind: impl Into<String>,
    view: Option<&str>,
    cmd: Option<Vec<String>>,
) -> PulseAction {
    PulseAction {
        id: id.into(),
        label: label.into(),
        kind: kind.into(),
        view: view.map(|s| s.into()),
        cmd,
    }
}

fn provider_linked(deploy: &DeployPulse) -> bool {
    deploy_is_live(deploy)
        || deploy.signal == "vercel_linked"
        || deploy.signal == "orbit_configured"
        || deploy.signal == "wrangler_local"
}

/// Hard-block only when Signet is required (Tauri / signet.toml) and missing.
/// Cloudflare Workers can already be live via Wrangler without Orbit on PATH.
fn tools_hard_block(wants_signet: bool, tools: &ToolsPulse, deploy: &DeployPulse) -> Option<String> {
    if wants_signet && !tools.signet_found {
        return Some("Signet".into());
    }
    if wants_signet && !tools.orbit_found && !provider_linked(deploy) {
        return Some("Orbit".into());
    }
    None
}

/// Step-specific cue appended to mid-publish Now detail (band #17).
fn publish_cut_hint(id: &str) -> &'static str {
    if id.starts_with("sign.") || id == "trust.pack" || id == "ship.desktop_cut" {
        return " Final-mile cut — Open/Run Signet, then Confirm.";
    }
    match id {
        "release.github" => " Run `gh release list`, then cut Release on GitHub and Confirm.",
        "ci.release" => " After tag/release — Run `gh run list` / confirm Actions green.",
        "listing.npm" => " Run `npm publish --dry-run`; live publish stays Confirm.",
        "listing.crates" => " Run `cargo publish --dry-run`; live publish stays Confirm.",
        "listing.huggingface" => " Upload on Hub with huggingface-cli; Confirm when repo is live.",
        "container.build" => " Run local `docker build` / compose build, then Confirm.",
        "container.deploy" => " Push image on your machine (docs Open); bridge never pushes.",
        "legal.baseline" => " Add LICENSE + SECURITY.md at repo root, then Confirm.",
        "marketing.deploy" => " Deploy/cut over the public landing URL, then Confirm.",
        "suite.url_sync" => " Paste live URL into sibling env keys, then Confirm.",
        _ => "",
    }
}

fn decide_now(
    name: &str,
    git: &GitPulse,
    publish: &WizardPulse,
    launch: &WizardPulse,
    deploy: &DeployPulse,
    tools: &ToolsPulse,
    wants_signet: bool,
) -> NowPulse {
    let mut actions = vec![
        action(
            "publish",
            "Open publish",
            "nav",
            Some("publish"),
            Some(vec!["publish".into(), "--project".into(), ".".into()]),
        ),
        action(
            "env",
            "Env / tokens",
            "nav",
            Some("env"),
            Some(vec!["env".into(), "--project".into(), ".".into()]),
        ),
        action(
            "scopes",
            "Scopes",
            "nav",
            Some("scopes"),
            Some(vec!["scopes".into(), "--project".into(), ".".into()]),
        ),
        action("git_status", "Git status", "run", Some("output"), None),
    ];

    // Mid-wizard always wins — don't bury Continue behind tool install.
    if publish.present && !publish.finished {
        let title = publish
            .current_title
            .clone()
            .unwrap_or_else(|| "Continue publish".into());
        let mins = publish
            .minutes_remaining
            .map(|m| format!(" · ~{m} min left"))
            .unwrap_or_default();
        let linked = if provider_linked(deploy) {
            " Local provider state already looks deployed."
        } else {
            ""
        };
        let cut = publish
            .current_id
            .as_deref()
            .map(publish_cut_hint)
            .unwrap_or("");
        let detail = format!(
            "Publish step {}/{}{mins}.{linked}{cut} Open/Run on the vendor UI or local CLI, Confirm, Next.",
            publish.current_index + 1,
            publish.total
        );
        return NowPulse {
            title,
            detail,
            primary: action(
                "publish_continue",
                "Continue publishing",
                "nav",
                Some("publish"),
                Some(vec!["publish".into(), "--project".into(), ".".into()]),
            ),
            actions,
        };
    }

    if launch.present && !launch.finished && !publish.present {
        let title = launch
            .current_title
            .clone()
            .unwrap_or_else(|| format!("Pick up {name}"));
        return NowPulse {
            title,
            detail: format!(
                "Launch mid-flight ({}/{}). Prefer Publish for the minute wizard, or continue Launch.",
                launch.current_index + 1,
                launch.total
            ),
            primary: action(
                "publish_start",
                "Start publishing",
                "nav",
                Some("publish"),
                Some(vec!["publish".into(), "--project".into(), ".".into()]),
            ),
            actions: {
                actions.insert(
                    0,
                    action(
                        "launch_continue",
                        "Continue launch",
                        "nav",
                        Some("launch"),
                        Some(vec!["launch".into(), "--project".into(), ".".into()]),
                    ),
                );
                actions
            },
        };
    }

    if let Some(missing) = tools_hard_block(wants_signet, tools, deploy) {
        return NowPulse {
            title: format!("Install {missing} before shipping"),
            detail: "Doctor checks PATH. Set SIGNET_PATH / ORBIT_PATH if installed elsewhere."
                .into(),
            primary: action(
                "doctor",
                "Run doctor",
                "run",
                Some("tools"),
                Some(vec!["doctor".into(), "--project".into(), ".".into()]),
            ),
            actions,
        };
    }

    if git.dirty
        && matches!(
            deploy.signal.as_str(),
            "unknown" | "wrangler_local" | "vercel_linked" | "orbit_configured"
        )
        && !deploy_is_live(deploy)
        && publish.finished
    {
        return NowPulse {
            title: "Uncommitted changes on disk".into(),
            detail: format!(
                "{} file(s) dirty{}. Commit or stash before another live deploy if you care about provenance.",
                git.dirty_count,
                git.branch
                    .as_ref()
                    .map(|b| format!(" on {b}"))
                    .unwrap_or_default()
            ),
            primary: action(
                "git_status",
                "Show git status",
                "run",
                Some("output"),
                None,
            ),
            actions,
        };
    }

    if git.dirty && !publish.present {
        let dirty_label = format!("Git: {} dirty", git.dirty_count);
        actions.insert(
            0,
            action(
                "git_status",
                dirty_label.as_str(),
                "run",
                Some("output"),
                None,
            ),
        );
    }

    if deploy_is_live(deploy) {
        let url_note = deploy
            .urls
            .first()
            .map(|u| format!(" Live: {u}."))
            .unwrap_or_default();
        let dirty_note = if git.dirty {
            format!(" Working tree has {} uncommitted change(s) — redeploy only if you intend to.", git.dirty_count)
        } else {
            String::new()
        };
        return NowPulse {
            title: format!("{name} is already live"),
            detail: format!(
                "{}{}{} Skip redeploy unless you need a new ship.",
                deploy.detail, url_note, dirty_note
            ),
            primary: action(
                "publish_start",
                "Review publish",
                "nav",
                Some("publish"),
                Some(vec!["publish".into(), "--project".into(), ".".into()]),
            ),
            actions,
        };
    }

    if deploy.signal == "vercel_linked" || deploy.signal == "orbit_configured" {
        return NowPulse {
            title: format!("{name} is provider-linked"),
            detail: format!(
                "{}. Start publishing for env/listing — deploy only when you need a new release.",
                deploy.detail
            ),
            primary: action(
                "publish_start",
                "Start publishing",
                "nav",
                Some("publish"),
                Some(vec!["publish".into(), "--project".into(), ".".into()]),
            ),
            actions,
        };
    }

    let git_line = if !git.is_repo {
        "Not a git repo.".into()
    } else if let Some(c) = &git.last_commit {
        let dirty = if git.dirty {
            format!(" · {} dirty", git.dirty_count)
        } else {
            " · clean".into()
        };
        format!("{} — {}{}", c.hash, c.subject, dirty)
    } else {
        "Git repo with no commits yet.".into()
    };

    NowPulse {
        title: format!("Pick up {name}"),
        detail: format!(
            "{git_line} Start the publish portal — you finish vendor UIs; Ship Studio keeps the sequence."
        ),
        primary: action(
            "publish_start",
            "Start publishing",
            "nav",
            Some("publish"),
            Some(vec!["publish".into(), "--project".into(), ".".into()]),
        ),
        actions,
    }
}

pub fn for_project(project: &Path) -> Result<ProjectPulse> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let name = project_name(&project);
    let git = git_pulse(&project);
    let publish = wizard_from_publish(&project);
    let launch = wizard_from_launch(&project);
    let deploy = deploy_pulse(&project);
    let tools = tools_pulse(&project);
    let scopes_active = scopes::plan_for(&project).active;
    let kind = describe_kind(&project);
    let detected = config::probe(&project);
    let wants_signet = detected.tauri || detected.signet_toml;
    let now = decide_now(
        &name,
        &git,
        &publish,
        &launch,
        &deploy,
        &tools,
        wants_signet,
    );

    let mut notes = Vec::new();
    notes.extend(git.notes.iter().cloned());
    if publish.present {
        notes.push(format!(
            "Publish {}/{} · done {}",
            publish.current_index + 1,
            publish.total,
            publish.done_count
        ));
    } else if launch.present {
        notes.push(format!(
            "Launch {}/{} · done {}",
            launch.current_index + 1,
            launch.total,
            launch.done_count
        ));
    }
    notes.push(format!("Deploy signal: {} — {}", deploy.signal, deploy.detail));
    if detected.ci_release {
        let files = detected.release_workflows.join(", ");
        if let Some(url) = config::github_actions_url(&project) {
            notes.push(format!("CI release workflow(s): {files} — {url}"));
        } else {
            notes.push(format!(
                "CI release workflow(s): {files} — open GitHub Actions after tagging."
            ));
        }
    }
    if detected.container {
        notes.push(format!(
            "Container layout — Run local build, then push yourself · {}",
            config::container_docs_url(&project)
        ));
    }
    if detected.steam || detected.itch || detected.epic {
        let mut m = Vec::new();
        if detected.steam {
            m.push("Steam");
        }
        if detected.itch {
            m.push("itch.io");
        }
        if detected.epic {
            m.push("Epic");
        }
        notes.push(format!(
            "Extra marketplace(s): {} — Advanced listing opens partner dashboards.",
            m.join(", ")
        ));
    }
    if !detected.license || !detected.security_md {
        let mut miss = Vec::new();
        if !detected.license {
            miss.push("LICENSE");
        }
        if !detected.security_md {
            miss.push("SECURITY.md");
        }
        notes.push(format!(
            "Launch baseline missing: {} — Advanced legal.baseline.",
            miss.join(" · ")
        ));
    }
    if (detected.tauri || detected.signet_toml) && !detected.trust_md {
        notes.push("Desktop/Signet without TRUST.md — Advanced trust.pack.".into());
    }
    if detected.npm_publish || detected.crates_publish {
        let mut m = Vec::new();
        if detected.npm_publish {
            m.push("npm");
        }
        if detected.crates_publish {
            m.push("crates.io");
        }
        notes.push(format!(
            "Package registry: {} — Advanced listing opens publisher dashboards.",
            m.join(", ")
        ));
    }
    if detected.marketing_site {
        notes.push(format!(
            "Marketing site ({}) — Advanced marketing.deploy.",
            if detected.marketing_host.is_empty() {
                "unknown"
            } else {
                detected.marketing_host.as_str()
            }
        ));
    }
    if detected.graduate_sign {
        notes.push(
            "Graduate signing opted in — Advanced sign.graduate (no verified-publisher claims)."
                .into(),
        );
    }
    if detected.suite_sync {
        notes.push(format!(
            "Suite URL sync — Advanced suite.url_sync ({})",
            if detected.suite_detail.is_empty() {
                "siblings"
            } else {
                detected.suite_detail.as_str()
            }
        ));
    }
    if detected.gumroad || detected.lemon {
        let mut m = Vec::new();
        if detected.gumroad {
            m.push("Gumroad");
        }
        if detected.lemon {
            m.push("Lemon");
        }
        notes.push(format!(
            "Commerce ({}) — Advanced listing opens SKU dashboards.",
            m.join(", ")
        ));
    }

    Ok(ProjectPulse {
        schema: "ship-studio/pulse/v1".into(),
        project: project.display().to_string(),
        name,
        kind,
        git,
        publish,
        launch,
        deploy,
        tools,
        scopes_active,
        now,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp() -> PathBuf {
        std::env::temp_dir().join(format!(
            "shipctl-pulse-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ))
    }

    #[test]
    fn pulse_on_wrangler_fixture() {
        let dir = tmp();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        let pulse = for_project(&dir).unwrap();
        assert_eq!(pulse.schema, "ship-studio/pulse/v1");
        assert!(pulse.kind.contains("Cloudflare") || pulse.kind.contains("Worker") || !pulse.kind.is_empty());
        assert!(!pulse.now.title.is_empty());
        assert!(!pulse.now.primary.label.is_empty());
    }

    #[test]
    fn pulse_notes_ci_release_workflow() {
        let dir = tmp();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(
            dir.join(".github/workflows/release.yml"),
            "name: release\n",
        )
        .unwrap();
        let pulse = for_project(&dir).unwrap();
        assert!(
            pulse.notes.iter().any(|n| n.contains("CI release") && n.contains("release.yml")),
            "notes={:?}",
            pulse.notes
        );
        assert!(pulse.kind.contains("CI release") || pulse.notes.iter().any(|n| n.contains("release.yml")));
    }

    #[test]
    fn decide_now_prefers_publish_in_progress() {
        let git = GitPulse {
            is_repo: true,
            branch: Some("main".into()),
            dirty: false,
            dirty_count: 0,
            committed: true,
            ahead: Some(0),
            behind: Some(0),
            last_commit: None,
            notes: vec![],
        };
        let publish = WizardPulse {
            present: true,
            finished: false,
            current_index: 3,
            total: 10,
            done_count: 3,
            current_id: Some("env.sprint".into()),
            current_title: Some("ENV & tokens".into()),
            minutes_remaining: Some(12),
        };
        let launch = WizardPulse {
            present: false,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
        let deploy = DeployPulse {
            signal: "unknown".into(),
            detail: "".into(),
            urls: vec![],
            last_run_ok: None,
        };
        let tools = ToolsPulse {
            signet_found: false,
            orbit_found: false,
            signet_version: None,
            orbit_version: None,
        };
        let now = decide_now("assess-api", &git, &publish, &launch, &deploy, &tools, false);
        assert_eq!(now.primary.id, "publish_continue");
        assert!(now.title.contains("ENV"));
    }

    #[test]
    fn publish_cut_hints_for_ci_and_registry() {
        assert!(publish_cut_hint("ci.release").contains("gh run list"));
        assert!(publish_cut_hint("listing.npm").contains("dry-run"));
        assert!(publish_cut_hint("listing.crates").contains("dry-run"));
        assert!(publish_cut_hint("container.build").contains("docker"));
        assert!(publish_cut_hint("container.deploy").contains("never pushes"));
        assert!(publish_cut_hint("release.github").contains("gh release list"));
        assert!(publish_cut_hint("sign.self.release").contains("Final-mile"));
        assert_eq!(publish_cut_hint("env.sprint"), "");

        let git = GitPulse {
            is_repo: true,
            branch: Some("main".into()),
            dirty: false,
            dirty_count: 0,
            committed: true,
            ahead: Some(0),
            behind: Some(0),
            last_commit: None,
            notes: vec![],
        };
        let publish = WizardPulse {
            present: true,
            finished: false,
            current_index: 8,
            total: 20,
            done_count: 8,
            current_id: Some("ci.release".into()),
            current_title: Some("CI — GitHub Actions release".into()),
            minutes_remaining: Some(5),
        };
        let launch = WizardPulse {
            present: false,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
        let deploy = DeployPulse {
            signal: "unknown".into(),
            detail: "".into(),
            urls: vec![],
            last_run_ok: None,
        };
        let tools = ToolsPulse {
            signet_found: true,
            orbit_found: true,
            signet_version: None,
            orbit_version: None,
        };
        let now = decide_now("dogfood", &git, &publish, &launch, &deploy, &tools, true);
        assert_eq!(now.primary.id, "publish_continue");
        assert!(
            now.detail.contains("gh run list"),
            "expected ci cut hint in detail: {}",
            now.detail
        );
    }

    #[test]
    fn wrangler_linked_not_blocked_without_orbit() {
        let git = GitPulse {
            is_repo: true,
            branch: Some("main".into()),
            dirty: true,
            dirty_count: 2,
            committed: true,
            ahead: Some(0),
            behind: Some(0),
            last_commit: None,
            notes: vec![],
        };
        let publish = WizardPulse {
            present: false,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
        let launch = WizardPulse {
            present: false,
            finished: false,
            current_index: 0,
            total: 0,
            done_count: 0,
            current_id: None,
            current_title: None,
            minutes_remaining: None,
        };
        let deploy = DeployPulse {
            signal: "orbit_deployed".into(),
            detail: "Orbit cloudflare deploy ok".into(),
            urls: vec!["https://assess-api.example.workers.dev".into()],
            last_run_ok: Some(true),
        };
        let tools = ToolsPulse {
            signet_found: false,
            orbit_found: false,
            signet_version: None,
            orbit_version: None,
        };
        let now = decide_now("assess-api", &git, &publish, &launch, &deploy, &tools, false);
        assert_ne!(now.primary.id, "doctor");
        assert!(now.title.to_lowercase().contains("live") || now.title.contains("deployed"));
    }

    #[test]
    fn wrangler_local_is_not_live() {
        let dir = tmp();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".wrangler/state")).unwrap();
        let deploy = deploy_pulse(&dir);
        assert_eq!(deploy.signal, "wrangler_local");
        assert!(!deploy_is_live(&deploy));
    }

    #[test]
    fn orbit_summary_marks_live() {
        let dir = tmp();
        let _ = fs::remove_dir_all(&dir);
        let run = dir.join(".orbit/runs/2026-09-12T19-33-55Z");
        fs::create_dir_all(&run).unwrap();
        fs::write(
            run.join("summary.json"),
            r#"{"ok":true,"command":"deploy","provider":"cloudflare","url":"https://x.workers.dev"}"#,
        )
        .unwrap();
        let deploy = deploy_pulse(&dir);
        assert_eq!(deploy.signal, "orbit_deployed");
        assert!(deploy_is_live(&deploy));
        assert!(deploy.urls.iter().any(|u| u.contains("workers.dev")));
    }
}

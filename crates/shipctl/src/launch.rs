//! Guided launch workflow — one step at a time: open → verify → next → launch.

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

    let selected = scopes::selected(project);
    if selected.len() > 1 || selected.iter().any(|s| s.relative != "." && !s.relative.is_empty())
    {
        steps.push(step(
            "scopes",
            "Scopes — Web / API / Desktop directories",
            StepKind::Auto,
            "Confirm which directories to ship. `shipctl scopes set --ids …` then Verify.",
            None,
            Some("active scopes saved".into()),
            Some(vec![
                "shipctl".into(),
                "scopes".into(),
                "--project".into(),
                ".".into(),
            ]),
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

    if include_self {
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

    if include_official && wants_signet {
        for p in signpath::plan_for(project).paths {
            if p.kind != "official" {
                continue;
            }
            steps.push(step(
                &format!("sign.{}", p.id),
                &p.title,
                StepKind::Sign,
                &p.detail,
                p.entry_url.clone(),
                Some("confirm after vendor UI, or skip with next --force".into()),
                p.run.clone(),
            ));
        }
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
            &format!("Database — provision ({})", bits.join(" · ")),
            StepKind::Deploy,
            "Create the DB on the vendor console, copy the connection string, put on the deploy target, then Confirm. Studio never creates databases.",
            entry,
            Some("confirm after DB provision + put".into()),
            None,
        ));
    }

    if detected.mobile
        && (detected.firebase || detected.appwrite || detected.convex || detected.supabase)
    {
        let entry = if detected.firebase {
            Some("https://console.firebase.google.com/".into())
        } else if detected.appwrite {
            Some("https://cloud.appwrite.io/".into())
        } else if detected.convex {
            Some("https://dashboard.convex.dev/".into())
        } else {
            Some("https://supabase.com/dashboard".into())
        };
        let mut bits = Vec::new();
        if detected.firebase {
            bits.push("Firebase");
        }
        if detected.appwrite {
            bits.push("Appwrite");
        }
        if detected.convex {
            bits.push("Convex");
        }
        if detected.supabase {
            bits.push("Supabase Auth");
        }
        steps.push(step(
            "baas.provision",
            &format!("Mobile BaaS — provision ({})", bits.join(" · ")),
            StepKind::Deploy,
            "Open the vendor console, create Auth / client keys for the mobile app, put values on the host, then Confirm. Studio never calls the BaaS APIs.",
            entry,
            Some("confirm after BaaS console setup".into()),
            None,
        ));
    }
    if detected.fly {
        steps.push(step(
            "host.fly",
            "Host — Fly.io dashboard",
            StepKind::Deploy,
            "Launch/scale the app on Fly (dashboard or flyctl). Studio only opens the official page — never deploys for you.",
            Some("https://fly.io/dashboard".into()),
            Some("confirm when the app is live".into()),
            None,
        ));
    }
    if detected.railway {
        steps.push(step(
            "host.railway",
            "Host — Railway dashboard",
            StepKind::Deploy,
            "Create/deploy the service on Railway. Studio only opens the official page.",
            Some("https://railway.app/dashboard".into()),
            Some("confirm when the service is live".into()),
            None,
        ));
    }
    if detected.render {
        steps.push(step(
            "host.render",
            "Host — Render dashboard",
            StepKind::Deploy,
            "Create/deploy the service on Render. Studio only opens the official page.",
            Some("https://dashboard.render.com/".into()),
            Some("confirm when the service is live".into()),
            None,
        ));
    }
    if detected.digitalocean {
        steps.push(step(
            "host.digitalocean",
            "Host — DigitalOcean App Platform",
            StepKind::Deploy,
            "Create/deploy the app on DigitalOcean App Platform. Studio only opens the official page.",
            Some("https://cloud.digitalocean.com/apps".into()),
            Some("confirm when the app is live".into()),
            None,
        ));
    }
    if detected.heroku {
        steps.push(step(
            "host.heroku",
            "Host — Heroku dashboard",
            StepKind::Deploy,
            "Create/deploy the app on Heroku. Studio only opens the official page — never deploys for you.",
            Some("https://dashboard.heroku.com/apps".into()),
            Some("confirm when the app is live".into()),
            None,
        ));
    }
    if detected.amplify {
        steps.push(step(
            "host.amplify",
            "Host — AWS Amplify console",
            StepKind::Deploy,
            "Create/deploy the app on AWS Amplify. Studio only opens the official page — never deploys for you.",
            Some("https://console.aws.amazon.com/amplify/home".into()),
            Some("confirm when the app is live".into()),
            None,
        ));
    }
    if detected.cloudrun {
        steps.push(step(
            "host.cloudrun",
            "Host — Google Cloud Run",
            StepKind::Deploy,
            "Create/deploy the service on Cloud Run. Studio only opens the official page — never deploys for you.",
            Some("https://console.cloud.google.com/run".into()),
            Some("confirm when the service is live".into()),
            None,
        ));
    }
    if detected.azurestatic {
        steps.push(step(
            "host.azurestatic",
            "Host — Azure Static Web Apps",
            StepKind::Deploy,
            "Create/deploy the static web app on Azure. Studio only opens the official page — never deploys for you.",
            Some("https://portal.azure.com/#view/HubsExtension/BrowseResource/resourceType/Microsoft.Web%2FstaticSites".into()),
            Some("confirm when the app is live".into()),
            None,
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
    if detected.gumroad {
        steps.push(step(
            "listing.gumroad",
            "Listing — Gumroad product / checkout",
            StepKind::List,
            "Create/update the Gumroad product on app.gumroad.com — Studio only opens the door.",
            Some("https://app.gumroad.com/".into()),
            Some("confirm listing/checkout updated".into()),
            None,
        ));
    }
    if detected.lemon {
        steps.push(step(
            "listing.lemon",
            "Listing — Lemon Squeezy product / checkout",
            StepKind::List,
            "Create/update the Lemon product on app.lemonsqueezy.com — Studio only opens the door.",
            Some("https://app.lemonsqueezy.com/".into()),
            Some("confirm listing/checkout updated".into()),
            None,
        ));
    }
    if detected.stripe {
        steps.push(step(
            "listing.stripe",
            "Listing — Stripe product / Payment Link",
            StepKind::List,
            "Create/update Stripe products or Payment Links on the dashboard — Studio never creates charges.",
            Some("https://dashboard.stripe.com/".into()),
            Some("confirm listing/checkout updated".into()),
            None,
        ));
    }
    if detected.paddle {
        steps.push(step(
            "listing.paddle",
            "Listing — Paddle product / price",
            StepKind::List,
            "Create/update Paddle products on the vendor dashboard — Studio never creates transactions.",
            Some("https://vendors.paddle.com/".into()),
            Some("confirm listing/checkout updated".into()),
            None,
        ));
    }
    if detected.npm_publish {
        steps.push(step(
            "listing.npm",
            "Listing — npm publish",
            StepKind::List,
            "Run `npm publish --dry-run` first. Live publish (OTP) stays on your machine — Confirm when the version is live. Bridge never publishes.",
            Some("https://www.npmjs.com/login".into()),
            Some("confirm after dry-run / live publish".into()),
            Some(vec![
                "npm".into(),
                "publish".into(),
                "--dry-run".into(),
            ]),
        ));
    }
    if detected.crates_publish {
        steps.push(step(
            "listing.crates",
            "Listing — crates.io publish",
            StepKind::List,
            "Run `cargo publish --dry-run` first. Live publish stays on your machine — Confirm when crates.io shows the version. Bridge never publishes.",
            Some("https://crates.io/me".into()),
            Some("confirm after dry-run / live publish".into()),
            Some(vec![
                "cargo".into(),
                "publish".into(),
                "--dry-run".into(),
            ]),
        ));
    }
    if detected.huggingface {
        steps.push(step(
            "listing.huggingface",
            "Listing — Hugging Face Hub",
            StepKind::List,
            "Create/update the model or dataset repo on huggingface.co; upload with huggingface-cli on your machine. Bridge never uploads.",
            Some("https://huggingface.co/docs/hub/repositories-getting-started".into()),
            Some("confirm when the Hub repo is public".into()),
            None,
        ));
    }
    if detected.steam {
        steps.push(step(
            "listing.steam",
            "Listing — Steamworks partner",
            StepKind::List,
            "Steam store presence stays on partner.steamgames.com — depots/builds are the next Submit step.",
            Some("https://partner.steamgames.com/".into()),
            Some("confirm listing updated".into()),
            None,
        ));
        steps.push(step(
            "submit.steam",
            "Submit — Steam depots / builds",
            StepKind::List,
            "Upload the build and set depots on Steamworks. Studio only opens the docs — never Steam API upload.",
            Some("https://partner.steamgames.com/doc/sdk/uploading".into()),
            Some("confirm when the build is live".into()),
            None,
        ));
    }
    if detected.itch {
        steps.push(step(
            "listing.itch",
            "Listing — itch.io dashboard",
            StepKind::List,
            "Store page / pricing on itch.io — build push is the next Submit step (butler).",
            Some("https://itch.io/dashboard".into()),
            Some("confirm listing updated".into()),
            None,
        ));
        steps.push(step(
            "submit.itch",
            "Submit — itch.io butler push",
            StepKind::List,
            "Push the build with butler (or the itch dashboard). Studio only opens the docs — never runs butler.",
            Some("https://itch.io/docs/butler/".into()),
            Some("confirm when the build is live".into()),
            None,
        ));
    }
    if detected.epic {
        steps.push(step(
            "listing.epic",
            "Listing — Epic Games Store portal",
            StepKind::List,
            "Epic product listing stays on the developer portal — binary upload is the next Submit step.",
            Some("https://dev.epicgames.com/portal".into()),
            Some("confirm listing updated".into()),
            None,
        ));
        steps.push(step(
            "submit.epic",
            "Submit — Epic binary / artifacts",
            StepKind::List,
            "Upload binaries on Epic publishing tools. Studio only opens the docs — never uploads for you.",
            Some("https://dev.epicgames.com/docs/epic-games-store/".into()),
            Some("confirm when the build is submitted".into()),
            None,
        ));
    }
    if detected.android || detected.expo || (detected.mobile && !detected.ios) {
        steps.push(step(
            "listing.play",
            "Listing — Google Play Console",
            StepKind::List,
            "Store listing, screenshots, and release track on Play Console — then Confirm.",
            Some("https://play.google.com/console".into()),
            Some("confirm listing updated".into()),
            None,
        ));
        steps.push(step(
            "submit.play",
            "Submit — Play production / review",
            StepKind::List,
            "Promote the release track and send for review on Play Console — Studio never uploads APKs/AABs.",
            Some("https://play.google.com/console".into()),
            Some("confirm after review submit".into()),
            None,
        ));
    }
    if detected.ios || detected.expo || (detected.mobile && !detected.android) {
        steps.push(step(
            "listing.app_store",
            "Listing — App Store Connect",
            StepKind::List,
            "App record, metadata, and pricing on App Store Connect — then Confirm.",
            Some("https://appstoreconnect.apple.com".into()),
            Some("confirm listing updated".into()),
            None,
        ));
    }
    if detected.ios || detected.expo || detected.tauri || (detected.mobile && !detected.android) {
        steps.push(step(
            "submit.app_store",
            "Submit — App Store review",
            StepKind::List,
            "Submit for Review on App Store Connect after listing + build — Studio never uploads binaries.",
            Some("https://appstoreconnect.apple.com".into()),
            Some("confirm after review submit".into()),
            None,
        ));
    }
    if detected.tauri {
        steps.push(step(
            "submit.microsoft",
            "Submit — Microsoft Store",
            StepKind::List,
            "Partner Center product submission / certification — Studio never uploads packages.",
            Some("https://partner.microsoft.com/dashboard/products".into()),
            Some("confirm after Partner Center submit".into()),
            None,
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

    if detected.marketing_site {
        let host = if detected.marketing_host.is_empty() {
            "your host".into()
        } else {
            detected.marketing_host.clone()
        };
        steps.push(step(
            "marketing.deploy",
            "Marketing — public landing deploy",
            StepKind::Deploy,
            &format!(
                "Deploy or cut over the download / HOOK / docs landing ({host}). Confirm when the canonical URL serves this build. Bridge does not touch DNS."
            ),
            Some(config::marketing_deploy_url(project)),
            Some("confirm when the landing URL is live".into()),
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
        assert!(state.steps.iter().any(|s| s.id == "deploy" || s.id.starts_with("deploy.")));
        assert!(state.steps.iter().any(|s| s.id == "intent"));
        assert!(state.steps.iter().any(|s| s.id.starts_with("paste.")));
        assert!(!state.steps.iter().any(|s| s.id.starts_with("signet.")));
        let deploy = state
            .steps
            .iter()
            .find(|s| s.id == "deploy" || s.id.starts_with("deploy."))
            .unwrap();
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
        assert!(state.steps.iter().any(|s| s.id == "deploy" || s.id.starts_with("deploy.")));
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
        assert!(state.steps.iter().any(|s| s.id == "listing.polar"));
        assert!(state.steps.iter().any(|s| s.id == "listing.gumroad"));
        assert!(state.steps.iter().any(|s| s.id == "listing.lemon"));
        assert!(state.steps.iter().any(|s| s.id == "listing.stripe"));
        assert!(state.steps.iter().any(|s| s.id == "listing.paddle"));
        let stripe = state
            .steps
            .iter()
            .find(|s| s.id == "listing.stripe")
            .unwrap();
        assert_eq!(
            stripe.entry_url.as_deref(),
            Some("https://dashboard.stripe.com/")
        );
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
        assert!(state.steps.iter().any(|s| s.id == "baas.provision"));
        let baas = state
            .steps
            .iter()
            .find(|s| s.id == "baas.provision")
            .unwrap();
        assert_eq!(
            baas.entry_url.as_deref(),
            Some("https://console.firebase.google.com/")
        );
        let fly = state.steps.iter().find(|s| s.id == "host.fly").unwrap();
        assert_eq!(fly.entry_url.as_deref(), Some("https://fly.io/dashboard"));
        let heroku = state.steps.iter().find(|s| s.id == "host.heroku").unwrap();
        assert_eq!(
            heroku.entry_url.as_deref(),
            Some("https://dashboard.heroku.com/apps")
        );
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
        assert!(state.steps.iter().any(|s| s.id == "listing.steam"));
        assert!(state.steps.iter().any(|s| s.id == "submit.steam"));
        assert!(state.steps.iter().any(|s| s.id == "listing.itch"));
        assert!(state.steps.iter().any(|s| s.id == "submit.itch"));
        assert!(state.steps.iter().any(|s| s.id == "listing.epic"));
        assert!(state.steps.iter().any(|s| s.id == "submit.epic"));
        let submit = state
            .steps
            .iter()
            .find(|s| s.id == "submit.steam")
            .unwrap();
        assert_eq!(
            submit.entry_url.as_deref(),
            Some("https://partner.steamgames.com/doc/sdk/uploading")
        );
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
        fs::write(mobile.join("build.gradle"), "// android\n").unwrap();
        let m = load_or_build(&mobile).unwrap();
        assert!(m.steps.iter().any(|s| s.id == "listing.play"));
        assert!(m.steps.iter().any(|s| s.id == "submit.play"));
        assert!(!m.steps.iter().any(|s| s.id == "submit.microsoft"));
        let play = m.steps.iter().find(|s| s.id == "listing.play").unwrap();
        assert_eq!(
            play.entry_url.as_deref(),
            Some("https://play.google.com/console")
        );
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
        fs::write(tauri.join("package.json"), "{}\n").unwrap();
        let t = load_or_build(&tauri).unwrap();
        assert!(
            !t.steps.iter().any(|s| s.id == "listing.play" || s.id == "submit.play"),
            "Tauri-only must not get Play lanes"
        );
        assert!(t.steps.iter().any(|s| s.id == "submit.app_store"));
        assert!(t.steps.iter().any(|s| s.id == "submit.microsoft"));
        let ms = t.steps.iter().find(|s| s.id == "submit.microsoft").unwrap();
        assert_eq!(
            ms.entry_url.as_deref(),
            Some("https://partner.microsoft.com/dashboard/products")
        );
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
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        fs::write(dir.join(".ship/markets.json"), r#"["hf"]"#).unwrap();
        let state = load_or_build(&dir).unwrap();
        assert!(state.steps.iter().any(|s| s.id == "listing.npm"));
        assert!(state.steps.iter().any(|s| s.id == "listing.crates"));
        assert!(state.steps.iter().any(|s| s.id == "listing.huggingface"));
        let npm = state.steps.iter().find(|s| s.id == "listing.npm").unwrap();
        assert_eq!(
            npm.run.as_ref().map(|r| r.as_slice()),
            Some(
                [
                    "npm".to_string(),
                    "publish".to_string(),
                    "--dry-run".to_string()
                ]
                .as_slice()
            )
        );
        assert_eq!(
            npm.entry_url.as_deref(),
            Some("https://www.npmjs.com/login")
        );
        let crates = state
            .steps
            .iter()
            .find(|s| s.id == "listing.crates")
            .unwrap();
        assert_eq!(
            crates.run.as_ref().map(|r| r.as_slice()),
            Some(
                [
                    "cargo".to_string(),
                    "publish".to_string(),
                    "--dry-run".to_string()
                ]
                .as_slice()
            )
        );
        let hf = state
            .steps
            .iter()
            .find(|s| s.id == "listing.huggingface")
            .unwrap();
        assert_eq!(
            hf.entry_url.as_deref(),
            Some("https://huggingface.co/docs/hub/repositories-getting-started")
        );
        assert!(hf.run.is_none());
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
        assert!(d.steps.iter().any(|s| s.id == "db.provision"));
        let provision = d.steps.iter().find(|s| s.id == "db.provision").unwrap();
        assert_eq!(
            provision.entry_url.as_deref(),
            Some("https://console.neon.tech")
        );
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
        assert!(m.steps.iter().any(|s| s.id == "marketing.deploy"));
        let marketing = m
            .steps
            .iter()
            .find(|s| s.id == "marketing.deploy")
            .unwrap();
        assert!(marketing
            .entry_url
            .as_ref()
            .is_some_and(|u| u.contains("vercel") || u.contains("pages") || u.contains("netlify")));
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
}

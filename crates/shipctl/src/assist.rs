//! Deploy assist wizard — ordered checklist across bind, scopes, env, sign, deploy.

use crate::config;
use crate::envx;
use crate::launch;
use crate::publish;
use crate::scopes;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct AssistStep {
    pub id: String,
    pub title: String,
    pub detail: String,
    pub view: String,
    pub ready: bool,
}

#[derive(Debug, Serialize)]
pub struct AssistPlan {
    pub schema: String,
    pub project: String,
    pub sign_path: String,
    pub scopes: Vec<String>,
    pub steps: Vec<AssistStep>,
    pub notes: Vec<String>,
}

pub fn plan_for(project: &Path) -> Result<AssistPlan> {
    let detected = config::probe(project);
    let sc = scopes::plan_for(project);
    let env = envx::plan_for(project)?;
    let has_studio = config::ship_dir(project).join("studio.json").is_file();
    let sign_path = if detected.tauri || detected.signet_toml {
        "self_then_official"
    } else {
        "skip_or_official_only"
    };

    let retrieve = env
        .actions
        .iter()
        .filter(|a| a.kind == "retrieve")
        .count();

    let mut steps = vec![
        AssistStep {
            id: "bind".into(),
            title: "Bind project directory".into(),
            detail: "Switch target folder from the titlebar or sidebar recents.".into(),
            view: "dashboard".into(),
            ready: true,
        },
        AssistStep {
            id: "scopes".into(),
            title: "Select scopes (Web / API / Desktop)".into(),
            detail: format!(
                "{} scope(s) detected · {} active.",
                sc.scopes.len(),
                sc.active.len()
            ),
            view: "scopes".into(),
            ready: !sc.active.is_empty(),
        },
        AssistStep {
            id: "env".into(),
            title: "ENV & tokens — create / retrieve / put".into(),
            detail: format!("{retrieve} paste-put hint(s). Create stays on vendor dashboards."),
            view: "env".into(),
            ready: retrieve == 0,
        },
        AssistStep {
            id: "sign".into(),
            title: "Signing path — self-sign or official wizard".into(),
            detail: if detected.tauri || detected.signet_toml {
                "Self-sign with Signet build; official stores via vendor URLs.".into()
            } else {
                "No desktop Signet target — skip or use official listing/release only.".into()
            },
            view: "sign".into(),
            ready: has_studio || !(detected.tauri || detected.signet_toml),
        },
        AssistStep {
            id: "providers".into(),
            title: "Providers — OAuth / dashboards".into(),
            detail: "Cloudflare, Vercel, Netlify, GitHub, Polar, DB, BaaS, and alt hosts (Fly/Railway/Render/DO) as detected.".into(),
            view: "portal".into(),
            ready: true,
        },
        AssistStep {
            id: "publish".into(),
            title: "Publish portal — minute wizard".into(),
            detail: "Open/Run → Confirm → Next — final-mile sign → release → deploy. Optional Watch: CLI `publish watch`, Desktop toggle, TUI `w`, MCP `ship_publish_watch`.".into(),
            view: "publish".into(),
            ready: has_studio,
        },
    ];

    if detected.ci_release {
        let files = detected.release_workflows.join(", ");
        let detail = if let Some(url) = config::github_actions_url(project) {
            format!(
                "Workflow(s): {files}. After tag/Signet release, Run `gh run list` or confirm green on {url}"
            )
        } else {
            format!(
                "Workflow(s): {files}. After tag/Signet release, Run `gh run list` / open GitHub Actions."
            )
        };
        steps.push(AssistStep {
            id: "ci.release".into(),
            title: "CI — GitHub Actions release".into(),
            detail,
            view: "dashboard".into(),
            ready: false,
        });
    }

    let mut notes = vec![
        "Assist sequences local CLIs + official URLs. Network deploy is operator-initiated."
            .into(),
        "Shipping hub: final-mile is sign → release → deploy; docs/demos stay outside Studio."
            .into(),
    ];
    if detected.graduate_sign {
        notes.push(
            "Graduate opted in — Advanced sign.graduate Runs `signet graduate notes`."
                .into(),
        );
    }
    if (detected.tauri || detected.signet_toml)
        && !(detected.wrangler || detected.vercel || detected.netlify)
    {
        notes.push(
            "Desktop-only — cut via Signet release (+ marketing); Orbit is not the desktop deploy."
                .into(),
        );
    }
    if detected.ci_release {
        notes.push(format!(
            "Release CI: {} — Advanced Run `gh run list` after tag/Signet release, then Confirm.",
            detected.release_workflows.join(", ")
        ));
    }
    if detected.container {
        notes.push(
            "Container Dockerfile/Compose — Advanced Run `container.build`; push is Confirm-only (no docker push from bridge)."
                .into(),
        );
    }
    if detected.npm_publish {
        notes.push(
            "npm package — Advanced listing.npm Runs `npm publish --dry-run`; live publish stays Confirm."
                .into(),
        );
    }
    if detected.crates_publish {
        notes.push(
            "crates.io package — Advanced listing.crates Runs `cargo publish --dry-run`; live publish stays Confirm."
                .into(),
        );
    }
    if detected.pwa {
        notes.push(
            "PWA manifest — deploy with web host; confirm manifest + service worker on canonical URL."
                .into(),
        );
    }
    if detected.huggingface {
        notes.push(
            "Hugging Face — Advanced listing.huggingface opens Hub docs; weight upload stays Confirm-only."
                .into(),
        );
    }
    // Non-Signet-self GitHub releases use release.github (same gate as publish plan).
    let wants_signet = detected.tauri || detected.signet_toml;
    if detected.github && !(wants_signet) {
        notes.push(
            "GitHub remote — Advanced release.github Runs `gh release list` (read-only); create stays on GitHub UI."
                .into(),
        );
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
            "Extra markets ({}) — Advanced listing + submit.* open vendor portals/docs; upload stays Confirm.",
            m.join(" · ")
        ));
    }
    if detected.fly
        || detected.railway
        || detected.render
        || detected.digitalocean
        || detected.heroku
        || detected.amplify
        || detected.cloudrun
        || detected.azurestatic
    {
        let mut m = Vec::new();
        if detected.fly {
            m.push("Fly");
        }
        if detected.railway {
            m.push("Railway");
        }
        if detected.render {
            m.push("Render");
        }
        if detected.digitalocean {
            m.push("DigitalOcean");
        }
        if detected.heroku {
            m.push("Heroku");
        }
        if detected.amplify {
            m.push("Amplify");
        }
        if detected.cloudrun {
            m.push("Cloud Run");
        }
        if detected.azurestatic {
            m.push("Azure Static");
        }
        notes.push(format!(
            "Alt hosts ({}) — Advanced host.* opens dashboards; deploy stays on their CLI/UI (no Orbit).",
            m.join(" · ")
        ));
    }
    if detected.mobile
        && (detected.firebase || detected.appwrite || detected.convex || detected.supabase)
    {
        notes.push(
            "Mobile BaaS — Advanced baas.provision opens Firebase/Appwrite/Convex/Supabase console (Auth/keys)."
                .into(),
        );
    }
    if detected.stripe || detected.paddle {
        let mut m = Vec::new();
        if detected.stripe {
            m.push("Stripe");
        }
        if detected.paddle {
            m.push("Paddle");
        }
        notes.push(format!(
            "Commerce ({}) — Advanced listing.* opens SKU dashboards; no Payment Link creation from Studio.",
            m.join(" · ")
        ));
    }

    Ok(AssistPlan {
        schema: "ship-studio/assist/v1".into(),
        project: project.display().to_string(),
        sign_path: sign_path.into(),
        scopes: sc.active,
        notes,
        steps,
    })
}

/// Ensure launch state exists so assist can jump into it.
pub fn start_launch(project: &Path) -> Result<launch::LaunchView> {
    let state = launch::load_or_build(project)?;
    Ok(launch::view(&state))
}

/// Ensure publish portal state exists (preferred minute wizard).
pub fn start_publish(project: &Path) -> Result<publish::PublishView> {
    let state = publish::load_or_build(project)?;
    Ok(publish::view(&state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn assist_notes_release_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-assist-ci-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(
            dir.join(".github/workflows/release.yml"),
            "name: release\n",
        )
        .unwrap();
        let plan = plan_for(&dir).unwrap();
        assert!(plan.steps.iter().any(|s| s.id == "ci.release"));
        assert!(plan.notes.iter().any(|n| n.contains("gh run list")));
        let step = plan.steps.iter().find(|s| s.id == "ci.release").unwrap();
        assert!(step.detail.contains("gh run list"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn assist_notes_registry_dry_run_and_release_list() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-assist-pkg-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("package.json"),
            r#"{"name":"@demo/pkg","version":"0.1.0","publishConfig":{"access":"public"}}"#,
        )
        .unwrap();
        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo_pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        let st = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&dir)
            .status();
        if st.map(|s| s.success()).unwrap_or(false) {
            let _ = std::process::Command::new("git")
                .args(["remote", "add", "origin", "https://github.com/acme/pkg.git"])
                .current_dir(&dir)
                .status();
        }
        let plan = plan_for(&dir).unwrap();
        assert!(plan.notes.iter().any(|n| n.contains("npm publish --dry-run")));
        assert!(plan.notes.iter().any(|n| n.contains("cargo publish --dry-run")));
        if config::probe(&dir).github {
            assert!(plan.notes.iter().any(|n| n.contains("gh release list")));
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn assist_notes_hosts_baas_commerce_and_submit() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-assist-expand-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join("fly.toml"), "app = \"demo\"\n").unwrap();
        fs::write(dir.join("firebase.json"), "{}\n").unwrap();
        fs::write(dir.join("build.gradle"), "// android\n").unwrap();
        fs::write(dir.join(".env"), "STRIPE_SECRET_KEY=\n").unwrap();
        fs::write(dir.join("steam_appid.txt"), "480\n").unwrap();
        let plan = plan_for(&dir).unwrap();
        assert!(plan.notes.iter().any(|n| n.contains("host.*") || n.contains("Fly")));
        assert!(plan.notes.iter().any(|n| n.contains("baas.provision")));
        assert!(plan.notes.iter().any(|n| n.contains("listing.") && n.contains("Stripe")));
        assert!(plan.notes.iter().any(|n| n.contains("submit")));
        let publish = plan.steps.iter().find(|s| s.id == "publish").unwrap();
        assert!(publish.detail.contains("Watch"));
        let _ = fs::remove_dir_all(&dir);
    }
}

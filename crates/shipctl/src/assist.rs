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
            detail: "Cloudflare, Vercel, Netlify, GitHub, Polar, DB hosts as detected.".into(),
            view: "portal".into(),
            ready: true,
        },
        AssistStep {
            id: "publish".into(),
            title: "Publish portal — minute wizard".into(),
            detail: "Open/Run → Confirm → Next through env, sign, listing, deploy.".into(),
            view: "publish".into(),
            ready: has_studio,
        },
    ];

    if detected.ci_release {
        let files = detected.release_workflows.join(", ");
        let detail = if let Some(url) = config::github_actions_url(project) {
            format!("Workflow(s): {files}. Confirm green on {url}")
        } else {
            format!("Workflow(s): {files}. Open GitHub Actions after tagging.")
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
    ];
    if detected.ci_release {
        notes.push(format!(
            "Release CI: {} — confirm Actions after tag/Signet release.",
            detected.release_workflows.join(", ")
        ));
    }
    if detected.container {
        notes.push(
            "Container Dockerfile/Compose detected — Advanced publish opens registry docs; no remote build."
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
            "Extra markets ({}) — enable via markers or `.ship/markets`; Advanced listing is URL + confirm.",
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
        assert!(plan.notes.iter().any(|n| n.contains("Release CI")));
    }
}

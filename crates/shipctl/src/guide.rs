//! Unified shipping guide — offline checklist (portal + secrets + flow plan).

use crate::adapters;
use crate::config;
use crate::flow;
use crate::portal;
use crate::secrets;
use anyhow::Result;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct GuideStep {
    pub id: String,
    pub title: String,
    pub human: bool,
    pub command: Vec<String>,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct GuidePlan {
    pub schema: String,
    pub project: String,
    pub providers: Vec<String>,
    pub secret_hint_count: usize,
    pub secret_names: Vec<String>,
    pub entry_urls: Vec<String>,
    pub doctor_ok: bool,
    pub steps: Vec<GuideStep>,
    pub notes: Vec<String>,
}

pub fn plan_for(project: &Path) -> Result<GuidePlan> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let doctor = adapters::doctor(&project)?;
    let portal = portal::plan_for(&project, None)?;
    let secrets = secrets::plan_for(&project, None)?;
    let flow = flow::plan(&project, false, true, true)?;

    let mut secret_names: Vec<String> = secrets
        .hints
        .iter()
        .filter(|h| h.name != "<NAME>")
        .map(|h| format!("{}/{}", h.provider, h.name))
        .collect();
    secret_names.sort();
    secret_names.dedup();

    let mut entry_urls = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for step in &portal.steps {
        if let Some(url) = &step.entry_url {
            if seen.insert(url.clone()) {
                entry_urls.push(url.clone());
            }
        }
    }
    for hint in &secrets.hints {
        if let Some(url) = &hint.entry_url {
            if seen.insert(url.clone()) {
                entry_urls.push(url.clone());
            }
        }
    }

    let mut steps = vec![
        GuideStep {
            id: "doctor".into(),
            title: "Doctor — local tools".into(),
            human: false,
            command: vec![
                "shipctl".into(),
                "doctor".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: if doctor.ok {
                "Signet + Orbit found.".into()
            } else {
                "Fix missing Signet/Orbit/PATH before deploy.".into()
            },
        },
        GuideStep {
            id: "portal".into(),
            title: "Portal — open provider entry points".into(),
            human: true,
            command: vec![
                "shipctl".into(),
                "portal".into(),
                "--project".into(),
                project.display().to_string(),
                "--open".into(),
            ],
            detail: format!(
                "Providers: {}. OAuth stays manual in the browser.",
                if portal.providers.is_empty() {
                    "(none detected — full catalog)".into()
                } else {
                    portal.providers.join(", ")
                }
            ),
        },
        GuideStep {
            id: "secrets".into(),
            title: "Secrets — paste into provider CLIs".into(),
            human: true,
            command: vec![
                "shipctl".into(),
                "secrets".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: if secret_names.is_empty() {
                "No named hints — add wrangler `# Secrets:` or empty .dev.vars keys.".into()
            } else {
                format!(
                    "{} hint(s): {}. Put via `shipctl secrets put --provider … --name …`.",
                    secret_names.len(),
                    secret_names.join(", ")
                )
            },
        },
        GuideStep {
            id: "vault".into(),
            title: "Vault — optional encrypted .km backup (Clavis)".into(),
            human: true,
            command: vec![
                "shipctl".into(),
                "vault".into(),
                "export".into(),
                "--out".into(),
                "ship-secrets.km".into(),
                "--from-hints".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: "Argon2id + AES-256-GCM kmvault. Passphrase via TTY or SHIP_VAULT_PASSPHRASE. Open in Clavis / Keys Manager.".into(),
        },
        GuideStep {
            id: "configure".into(),
            title: "Configure — write .ship/studio.json".into(),
            human: false,
            command: vec![
                "shipctl".into(),
                "configure".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: "Merges detected deploy_args / sign_args.".into(),
        },
        GuideStep {
            id: "publish".into(),
            title: "Publish — minute Open/Run → Confirm → Next".into(),
            human: true,
            command: vec![
                "shipctl".into(),
                "publish".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: "Preferred final-mile spine. Optional Watch: `publish watch`, Desktop toggle, TUI `w`, MCP `ship_publish_watch`.".into(),
        },
        GuideStep {
            id: "flow_dry_run".into(),
            title: "Flow dry-run — preview sign → deploy args".into(),
            human: false,
            command: vec![
                "shipctl".into(),
                "flow".into(),
                "--project".into(),
                project.display().to_string(),
                "--dry-run".into(),
                "--offline".into(),
                "--skip-deploy".into(),
            ],
            detail: format!(
                "{} planned step(s): {}",
                flow.steps.len(),
                flow.steps
                    .iter()
                    .map(|s| s.id.as_str())
                    .collect::<Vec<_>>()
                    .join(" → ")
            ),
        },
        GuideStep {
            id: "flow".into(),
            title: "Flow — sign then deploy (when ready)".into(),
            human: true,
            command: vec![
                "shipctl".into(),
                "flow".into(),
                "--project".into(),
                project.display().to_string(),
            ],
            detail: "Legacy companion — prefer Publish for the full Adaptive path.".into(),
        },
    ];

    // Surface first portal OAuth/dashboard as concrete follow-up.
    if let Some(human) = portal
        .steps
        .iter()
        .find(|s| s.kind == "oauth" || s.kind == "dashboard")
    {
        steps.insert(
            2,
            GuideStep {
                id: format!("portal.{}", human.id),
                title: human.title.clone(),
                human: true,
                command: human
                    .cli
                    .clone()
                    .unwrap_or_else(|| vec!["shipctl".into(), "portal".into(), "--open".into()]),
                detail: human.detail.clone(),
            },
        );
    }

    let intent = config::intent_for(&project)?;
    let detected = config::probe(&project);
    let mut notes = vec![
        "Guide is offline-safe JSON — open/login/put are operator-initiated.".into(),
        "Prefer Publish (`shipctl publish`) over Flow for Adaptive Open → Confirm → Next.".into(),
        format!("Suggested deploy_args: {:?}", intent.deploy_args),
        format!("Suggested sign_args: {:?}", intent.sign_args),
        format!(
            "{} unique entry URL(s) — `shipctl guide --open` launches them.",
            entry_urls.len()
        ),
        "Desktop/TUI Wizard follows the same step ids.".into(),
        "Optional vault: shipctl vault export --out ship-secrets.km --from-hints".into(),
    ];
    if detected.fly || detected.railway || detected.render || detected.digitalocean || detected.heroku || detected.amplify {
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
        notes.push(format!(
            "Alt hosts ({}) — Advanced Publish host.* opens dashboards; deploy stays on their CLI/UI.",
            m.join(" · ")
        ));
    }
    if detected.mobile
        && (detected.firebase || detected.appwrite || detected.convex || detected.supabase)
    {
        notes.push(
            "Mobile BaaS — Advanced Publish baas.provision opens Firebase/Appwrite/Convex/Supabase console."
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
    notes.push(
        "Progress nudge: `shipctl publish watch` · Desktop Watch · TUI `w` · MCP `ship_publish_watch`."
            .into(),
    );
    // Prefer cut-readiness / portal cues from doctor over early PATH noise.
    for n in doctor.notes.iter().rev().take(8) {
        if !notes.iter().any(|existing| existing == n) {
            notes.push(n.clone());
        }
    }

    Ok(GuidePlan {
        schema: "ship-studio/guide/v1".into(),
        project: project.display().to_string(),
        providers: portal.providers,
        secret_hint_count: secret_names.len(),
        secret_names,
        entry_urls,
        doctor_ok: doctor.ok,
        steps,
        notes,
    })
}

/// Open all unique human entry URLs from the guide (operator-initiated).
pub fn open_entries(plan: &GuidePlan) -> Result<Vec<String>> {
    let mut opened = Vec::new();
    for url in &plan.entry_urls {
        portal::open_url(url)?;
        opened.push(url.clone());
    }
    Ok(opened)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn guide_includes_core_steps() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-guide-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name=\"x\"\n").unwrap();
        let plan = plan_for(&dir).unwrap();
        assert!(plan.steps.iter().any(|s| s.id == "doctor"));
        assert!(plan.steps.iter().any(|s| s.id == "portal"));
        assert!(plan.steps.iter().any(|s| s.id == "secrets"));
        assert!(plan.steps.iter().any(|s| s.id == "vault"));
        assert!(plan.steps.iter().any(|s| s.id == "configure"));
        assert!(plan.steps.iter().any(|s| s.id == "publish"));
        assert!(plan.providers.iter().any(|p| p == "cloudflare"));
        assert!(!plan.entry_urls.is_empty());
    }

    #[test]
    fn guide_notes_hosts_baas_commerce_and_watch() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-guide-expand-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::write(dir.join("fly.toml"), "app = \"demo\"\n").unwrap();
        fs::write(dir.join("firebase.json"), "{}\n").unwrap();
        fs::write(dir.join("build.gradle"), "// android\n").unwrap();
        fs::write(dir.join(".env"), "STRIPE_SECRET_KEY=\n").unwrap();
        let plan = plan_for(&dir).unwrap();
        assert!(plan.steps.iter().any(|s| s.id == "publish"));
        let publish = plan.steps.iter().find(|s| s.id == "publish").unwrap();
        assert!(publish.detail.contains("Watch"));
        assert!(plan
            .notes
            .iter()
            .any(|n| n.contains("host.*") || n.contains("Fly")));
        assert!(plan.notes.iter().any(|n| n.contains("baas.provision")));
        assert!(plan
            .notes
            .iter()
            .any(|n| n.contains("Stripe") || n.contains("listing.*")));
        assert!(plan.notes.iter().any(|n| n.contains("publish watch")));
        let _ = fs::remove_dir_all(&dir);
    }
}

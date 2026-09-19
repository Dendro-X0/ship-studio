//! Human portal sprint — open entry pages in priority order, then paste secrets.

use crate::guide;
use crate::portal::{self, ProviderId};
use crate::secrets::{self, SecretHint};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashSet;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct HumanSprint {
    pub schema: String,
    pub project: String,
    pub minutes_hint: String,
    pub opened: Vec<String>,
    pub open_order: Vec<String>,
    pub put_queue: Vec<SecretHint>,
    pub put_results: Vec<serde_json::Value>,
    pub checklist: Vec<String>,
}

/// Fast path for the operator: navigate → copy from dashboards → paste into put CLIs.
pub fn run(project: &Path, open: bool, put: bool) -> Result<HumanSprint> {
    run_with_options(project, open, put, false)
}

/// `open_all_entries`: also open every guide entry URL (CF/Vercel dashboards), not only paste sources.
pub fn run_with_options(
    project: &Path,
    open: bool,
    put: bool,
    open_all_entries: bool,
) -> Result<HumanSprint> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let guide = guide::plan_for(&project)?;
    let secrets = secrets::plan_for(&project, None)?;

    let put_queue = put_queue_from_hints(&secrets.hints);

    // Default open set = value-source pages for the paste queue only (Polar/GitHub…),
    // prioritized — not the full CF API-token catalog unless --open-all.
    let mut source_urls: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    for h in &put_queue {
        if let Some(url) = &h.entry_url {
            if seen.insert(url.clone()) {
                source_urls.push(url.clone());
            }
        }
    }
    let mut open_order = prioritize_urls(&source_urls);
    if open_all_entries {
        for url in prioritize_urls(&guide.entry_urls) {
            if seen.insert(url.clone()) {
                open_order.push(url);
            }
        }
    }

    let mut opened = Vec::new();
    if open {
        for url in &open_order {
            portal::open_url(url)?;
            opened.push(url.clone());
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    let mut put_results = Vec::new();

    let mut checklist = vec![
        "1) Copy values from the opened source pages (GitHub PAT / Polar checkout+webhook).".into(),
        "2) IMPORTANT: Cloudflare API Tokens cannot be Viewed after create — only Create or ⋯→Roll shows a new value once. Prefer wrangler login for CF auth.".into(),
        "3) Paste into each secrets put prompt (shipctl never stores the value).".into(),
        "4) Optionally encrypt a backup: shipctl vault export --out ./ship-secrets.km --from-hints (Clavis-compatible .km).".into(),
    ];
    if put_queue.is_empty() {
        checklist.push("No paste queue — add empty POLAR_*/GITHUB_TOKEN in .dev.vars or wrangler # Secrets.".into());
    } else {
        checklist.push(format!(
            "Paste queue ({}): {}",
            put_queue.len(),
            put_queue
                .iter()
                .map(|h| format!("{}/{}", h.provider, h.name))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    if put {
        if !io::stdin().is_terminal() {
            checklist.push(
                "TTY required for --put; re-run in a real terminal: shipctl human --project . --put"
                    .into(),
            );
        } else {
            eprintln!("\n=== Ship Studio human portal — paste when each CLI prompts ===\n");
            for (i, hint) in put_queue.iter().enumerate() {
                eprintln!(
                    "[{}/{}] {} / {}  →  {}",
                    i + 1,
                    put_queue.len(),
                    hint.provider,
                    hint.name,
                    hint.put_cli.join(" ")
                );
                if let Some(url) = &hint.entry_url {
                    eprintln!("    source page: {url}");
                    // Re-focus the right tab before each paste.
                    let _ = portal::open_url(url);
                }
                let _ = io::stderr().flush();
                let id = ProviderId::parse(&hint.provider)?;
                match secrets::put_secret(&project, id, &hint.name) {
                    Ok(0) => {
                        eprintln!("    ok\n");
                        put_results.push(serde_json::json!({
                            "provider": hint.provider,
                            "name": hint.name,
                            "ok": true,
                        }));
                    }
                    Ok(code) => {
                        eprintln!("    exit {code}\n");
                        put_results.push(serde_json::json!({
                            "provider": hint.provider,
                            "name": hint.name,
                            "ok": false,
                            "exit_code": code,
                        }));
                    }
                    Err(e) => {
                        eprintln!("    failed: {e:#}\n");
                        put_results.push(serde_json::json!({
                            "provider": hint.provider,
                            "name": hint.name,
                            "ok": false,
                            "error": format!("{e:#}"),
                        }));
                    }
                }
            }
            checklist.push("Put pass finished — verify with wrangler/orbit or shipctl flow.".into());
        }
    } else if !put_queue.is_empty() {
        checklist.push(
            "Next: shipctl human --project . --put   (opens each source page, then wrangler secret put)"
                .into(),
        );
        for h in &put_queue {
            checklist.push(format!(
                "  · {} / {}  ← {}",
                h.provider,
                h.name,
                h.entry_url.as_deref().unwrap_or("(no url)")
            ));
        }
    }

    // Persist sprint for desktop.
    let dir = crate::config::ship_dir(&project);
    let _ = std::fs::create_dir_all(&dir);
    let sprint = HumanSprint {
        schema: "ship-studio/human/v1".into(),
        project: project.display().to_string(),
        minutes_hint: "Aim: 2–3 source tabs → copy → paste into put prompts (~few minutes).".into(),
        opened,
        open_order,
        put_queue,
        put_results,
        checklist,
    };
    let _ = std::fs::write(
        dir.join("last-human.json"),
        serde_json::to_string_pretty(&sprint)?,
    );
    Ok(sprint)
}

fn prioritize_urls(urls: &[String]) -> Vec<String> {
    let mut scored: Vec<(i32, String)> = urls
        .iter()
        .cloned()
        .map(|u| {
            let score = if u.contains("polar.sh") {
                0
            } else if u.contains("neon.tech") {
                1
            } else if u.contains("supabase.com") {
                2
            } else if u.contains("turso.tech") {
                3
            } else if u.contains("/workers/d1") || (u.contains("d1") && u.contains("cloudflare")) {
                4
            } else if u.contains("github.com") {
                5
            } else if u.contains("vercel.com") {
                6
            } else if u.contains("cloudflare.com") {
                7
            } else if u.contains("netlify.com") {
                8
            } else {
                9
            };
            (score, u)
        })
        .collect();
    scored.sort_by_key(|(s, _)| *s);
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for (_, u) in scored {
        if seen.insert(u.clone()) {
            out.push(u);
        }
    }
    out
}

/// Put destinations for launch / human sprint (skips catalog stubs + pepper).
pub fn put_queue_public(hints: &[SecretHint]) -> Vec<SecretHint> {
    put_queue_from_hints(hints)
}

fn put_queue_from_hints(hints: &[SecretHint]) -> Vec<SecretHint> {
    // Prefer putting on Cloudflare Worker when both catalog and CF list the same name.
    let mut by_name: std::collections::BTreeMap<String, SecretHint> =
        std::collections::BTreeMap::new();
    for h in hints {
        if h.name == "<NAME>" || h.name == "API_KEY_PEPPER" {
            continue;
        }
        // Skip catalog-only put stubs — real put is cloudflare/vercel/netlify.
        if matches!(
            h.provider.as_str(),
            "polar" | "github" | "neon" | "supabase" | "d1" | "turso" | "container"
                | "firebase" | "appwrite" | "convex" | "fly" | "railway" | "render"
                | "digitalocean"
        ) {
            continue;
        }
        if h.provider == "cloudflare" {
            by_name.insert(h.name.clone(), h.clone());
        } else {
            by_name.entry(h.name.clone()).or_insert_with(|| h.clone());
        }
    }
    // Stable priority: GITHUB_TOKEN, then POLAR_*, then rest.
    let mut names: Vec<String> = by_name.keys().cloned().collect();
    names.sort_by(|a, b| {
        let rank = |n: &str| {
            if n == "GITHUB_TOKEN" {
                0
            } else if n.starts_with("POLAR_") {
                1
            } else {
                2
            }
        };
        rank(a).cmp(&rank(b)).then(a.cmp(b))
    });
    names
        .into_iter()
        .filter_map(|n| by_name.remove(&n))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polar_before_github_in_open_order() {
        let urls = vec![
            "https://github.com/settings/tokens".into(),
            "https://polar.sh/dashboard".into(),
            "https://dash.cloudflare.com/profile/api-tokens".into(),
        ];
        let ordered = prioritize_urls(&urls);
        assert!(ordered[0].contains("polar"));
        assert!(ordered[1].contains("github"));
    }

    #[test]
    fn human_default_open_order_is_paste_sources_only() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-human-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("wrangler.toml"),
            "# Secrets\n# - GITHUB_TOKEN\n# - POLAR_WEBHOOK_SECRET\nname = \"x\"\n",
        )
        .unwrap();
        // Touch .git so guide also lists github entry URLs.
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        let sprint = run_with_options(&dir, false, false, false).unwrap();
        assert!(
            !sprint.open_order.iter().any(|u| u.contains("cloudflare.com")),
            "default open should not include CF token page: {:?}",
            sprint.open_order
        );
        assert!(
            sprint.open_order.iter().any(|u| u.contains("github.com"))
                || sprint.open_order.iter().any(|u| u.contains("polar.sh")),
            "expected paste sources: {:?}",
            sprint.open_order
        );
        if sprint.open_order.len() >= 2 {
            let polar_i = sprint
                .open_order
                .iter()
                .position(|u| u.contains("polar.sh"));
            let gh_i = sprint
                .open_order
                .iter()
                .position(|u| u.contains("github.com"));
            if let (Some(p), Some(g)) = (polar_i, gh_i) {
                assert!(p < g, "polar should sort before github: {:?}", sprint.open_order);
            }
        }
    }

    #[test]
    fn source_url_github_not_cloudflare() {
        assert!(portal::source_url_for_secret_name("GITHUB_TOKEN").contains("github.com"));
        assert!(portal::source_url_for_secret_name("POLAR_WEBHOOK_SECRET").contains("polar.sh"));
    }

    #[test]
    fn entry_url_vercel_put_not_cloudflare_default() {
        assert!(portal::entry_url_for_secret("ANTHROPIC_API_KEY", Some(ProviderId::Vercel))
            .unwrap()
            .contains("anthropic.com"));
        assert!(portal::entry_url_for_secret("CRON_SECRET", Some(ProviderId::Vercel))
            .unwrap()
            .contains("vercel.com"));
        assert!(portal::entry_url_for_secret("RESEND_API_KEY", Some(ProviderId::Vercel))
            .unwrap()
            .contains("resend.com"));
        assert!(
            !portal::entry_url_for_secret("CRON_SECRET", Some(ProviderId::Vercel))
                .unwrap()
                .contains("cloudflare")
        );
        assert!(portal::entry_url_for_secret("CF_API_TOKEN", None)
            .unwrap()
            .contains("cloudflare"));
    }

    #[test]
    fn put_queue_prefers_cloudflare_and_skips_pepper() {
        let hints = vec![
            SecretHint {
                provider: "polar".into(),
                name: "POLAR_CHECKOUT_URL".into(),
                source: "catalog".into(),
                put_cli: vec![],
                work_dir: ".".into(),
                entry_url: None,
                detail: "".into(),
            },
            SecretHint {
                provider: "cloudflare".into(),
                name: "API_KEY_PEPPER".into(),
                source: "wrangler".into(),
                put_cli: vec![
                    "wrangler".into(),
                    "secret".into(),
                    "put".into(),
                    "API_KEY_PEPPER".into(),
                ],
                work_dir: ".".into(),
                entry_url: None,
                detail: "".into(),
            },
            SecretHint {
                provider: "cloudflare".into(),
                name: "GITHUB_TOKEN".into(),
                source: "wrangler".into(),
                put_cli: vec![
                    "wrangler".into(),
                    "secret".into(),
                    "put".into(),
                    "GITHUB_TOKEN".into(),
                ],
                work_dir: ".".into(),
                entry_url: None,
                detail: "".into(),
            },
            SecretHint {
                provider: "cloudflare".into(),
                name: "POLAR_CHECKOUT_URL".into(),
                source: "wrangler".into(),
                put_cli: vec![
                    "wrangler".into(),
                    "secret".into(),
                    "put".into(),
                    "POLAR_CHECKOUT_URL".into(),
                ],
                work_dir: ".".into(),
                entry_url: None,
                detail: "".into(),
            },
        ];
        let q = put_queue_from_hints(&hints);
        assert_eq!(q.len(), 2);
        assert_eq!(q[0].name, "GITHUB_TOKEN");
        assert_eq!(q[1].name, "POLAR_CHECKOUT_URL");
        assert!(q.iter().all(|h| h.provider == "cloudflare"));
    }
}

//! One-shot offline shipping prep: guide → configure → flow dry-run.

use crate::config;
use crate::flow;
use crate::guide::{self, GuidePlan};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct ShipPrep {
    pub schema: String,
    pub project: String,
    pub guide: GuidePlan,
    pub configured: bool,
    pub dry_run: serde_json::Value,
    pub opened: Vec<String>,
    pub next: Vec<String>,
}

pub fn run(project: &Path, open: bool) -> Result<ShipPrep> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let guide_plan = guide::plan_for(&project)?;

    // Persist for desktop / later status.
    let dir = config::ship_dir(&project);
    fs::create_dir_all(&dir).context("mkdir .ship")?;
    fs::write(
        dir.join("last-guide.json"),
        serde_json::to_string_pretty(&guide_plan)?,
    )
    .context("write .ship/last-guide.json")?;

    config::configure(&project)?;

    let dry = flow::plan(&project, false, true, true)?;
    let dry_run = serde_json::to_value(&dry)?;

    let opened = if open {
        guide::open_entries(&guide_plan)?
    } else {
        Vec::new()
    };

    let next = vec![
        "Human: complete OAuth / marketplace pages (or re-run with --open)".into(),
        "Human: shipctl secrets put --provider cloudflare --name <HINT>".into(),
        "Then: shipctl tui   or   shipctl flow --project .".into(),
        "Desktop: Wizard / Portal / Secrets".into(),
    ];

    Ok(ShipPrep {
        schema: "ship-studio/ship/v1".into(),
        project: project.display().to_string(),
        guide: guide_plan,
        configured: true,
        dry_run,
        opened,
        next,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn ship_writes_last_guide() {
        static N: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "shipctl-ship-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("wrangler.toml"), "name=\"x\"\n").unwrap();
        let prep = run(&dir, false).unwrap();
        assert!(prep.configured);
        assert!(dir.join(".ship/last-guide.json").is_file());
        assert!(prep.guide.providers.iter().any(|p| p == "cloudflare"));
    }
}

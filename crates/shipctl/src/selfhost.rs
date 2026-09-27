//! Local self-host deploy — Studio-owned lane (stream + last-run).
//! Slice 2: detect static surface, stream phases, write `.ship/last-run.json`.
//! Long-lived serve / health probes are Slice 3+.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config;

#[derive(Debug, Clone, Serialize)]
pub struct SelfhostTarget {
    pub kind: &'static str,
    pub path: PathBuf,
    pub rel: String,
}

#[derive(Debug, Serialize)]
pub struct SelfhostReport {
    pub ok: bool,
    pub project: String,
    pub target: Option<SelfhostTarget>,
    pub message: String,
}

fn rel_display(project: &Path, path: &Path) -> String {
    path.strip_prefix(project)
        .map(|p| p.display().to_string().replace('\\', "/"))
        .unwrap_or_else(|_| path.display().to_string().replace('\\', "/"))
}

/// Prefer documented static roots; never steals Dockerfile layouts for serve.
pub fn resolve_target(project: &Path) -> Option<SelfhostTarget> {
    let candidates: &[&str] = &[
        "apps/website/index.html",
        "apps/docs/index.html",
        "public/index.html",
        "dist/index.html",
        "index.html",
    ];
    for rel in candidates {
        let path = project.join(rel);
        if path.is_file() {
            let dir = path.parent().unwrap_or(project).to_path_buf();
            let rel = rel_display(project, &dir);
            return Some(SelfhostTarget {
                kind: "static",
                path: dir,
                rel,
            });
        }
    }
    None
}

pub fn run(project: &Path) -> Result<SelfhostReport> {
    let project = project
        .canonicalize()
        .with_context(|| format!("canonicalize {}", project.display()))?;
    println!("selfhost · starting local auto lane");
    println!("selfhost · project {}", project.display());
    println!("selfhost · detecting static surfaces…");

    let target = resolve_target(&project);
    let started = config::now_rfc3339();

    let report = match &target {
        Some(t) => {
            println!("selfhost · found {} ({})", t.rel, t.kind);
            println!("selfhost · validating readable root…");
            let probe = t.path.join("index.html");
            if !probe.is_file() {
                // Directory without index — still accept if we resolved via index path above.
                let _ = fs::read_dir(&t.path).with_context(|| format!("read {}", t.path.display()))?;
            } else {
                let _ = fs::metadata(&probe).with_context(|| format!("stat {}", probe.display()))?;
            }
            println!("selfhost · stream path live — serve/health checks land in Slice 3");
            println!("selfhost · ready · {}", t.rel);
            SelfhostReport {
                ok: true,
                project: project.display().to_string(),
                target: Some(t.clone()),
                message: format!("selfhost ok · static · {}", t.rel),
            }
        }
        None => {
            println!("selfhost · no static index yet (tried apps/website, public, dist, root)");
            println!("selfhost · add an index.html under one of those paths, then Deploy again");
            SelfhostReport {
                ok: false,
                project: project.display().to_string(),
                target: None,
                message: "selfhost: no static surface detected".into(),
            }
        }
    };

    let finished = config::now_rfc3339();
    let urls = report
        .target
        .as_ref()
        .map(|t| {
            let index = t.path.join("index.html");
            if index.is_file() {
                vec![format!("file:///{}", index.display().to_string().replace('\\', "/"))]
            } else {
                vec![]
            }
        })
        .unwrap_or_default();

    config::write_last_run(
        &project,
        &config::LastRun {
            finished: true,
            ok: report.ok,
            started_at: started,
            finished_at: finished,
            dry_run: false,
            offline: true,
            steps: vec![config::StepResult {
                id: "selfhost".into(),
                ok: report.ok,
                exit_code: if report.ok { 0 } else { 1 },
                detail: report.message.clone(),
            }],
            message: report.message.clone(),
            urls,
        },
    )?;

    if !report.ok {
        bail!("{}", report.message);
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp(label: &str) -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("shipctl-selfhost-{label}-{n}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolves_harbor_style_website() {
        let dir = tmp("site");
        let web = dir.join("apps/website");
        fs::create_dir_all(&web).unwrap();
        fs::write(web.join("index.html"), "<!doctype html><title>ok</title>").unwrap();
        let t = resolve_target(&dir).expect("target");
        assert_eq!(t.kind, "static");
        assert!(t.rel.contains("apps/website"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn run_writes_last_run() {
        let dir = tmp("run");
        let web = dir.join("apps/website");
        fs::create_dir_all(&web).unwrap();
        fs::write(web.join("index.html"), "<!doctype html>").unwrap();
        let report = run(&dir).expect("selfhost");
        assert!(report.ok);
        let last = config::read_last_run(&dir).unwrap();
        assert_eq!(last["ok"], true);
        assert!(last["message"].as_str().unwrap_or("").contains("selfhost"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_surface_fails() {
        let dir = tmp("empty");
        assert!(run(&dir).is_err());
        let _ = fs::remove_dir_all(&dir);
    }
}

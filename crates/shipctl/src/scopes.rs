//! Deploy scopes — directories inside a bound project (web / api / desktop / docs / mobile / container).

use crate::config;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScopeKind {
    Web,
    Api,
    Desktop,
    Docs,
    Mobile,
    Container,
    Root,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    pub id: String,
    pub kind: ScopeKind,
    pub label: String,
    pub root: String,
    pub relative: String,
    pub provider: Option<String>,
    pub deploy_args: Vec<String>,
    pub signals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopesPlan {
    pub schema: String,
    pub project: String,
    pub scopes: Vec<Scope>,
    pub active: Vec<String>,
    pub notes: Vec<String>,
}

fn rel(project: &Path, p: &Path) -> String {
    p.strip_prefix(project)
        .map(|x| x.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".".into())
}

fn has(dir: &Path, name: &str) -> bool {
    dir.join(name).is_file()
}

fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        "node_modules" | ".git" | "target" | "dist" | ".next" | ".turbo" | "vendor"
    )
}

fn kind_from_name(name: &str, signals: &[String]) -> ScopeKind {
    let n = name.to_ascii_lowercase();
    if signals.iter().any(|s| {
        matches!(
            s.as_str(),
            "mobile" | "android" | "ios" | "expo" | "flutter" | "capacitor"
        )
    }) || n.contains("mobile")
        || n.contains("android")
        || n == "ios"
        || n.contains("expo")
        || n.contains("flutter")
    {
        return ScopeKind::Mobile;
    }
    if signals.iter().any(|s| s == "docker" || s == "compose")
        || n.contains("docker")
        || n.contains("container")
    {
        return ScopeKind::Container;
    }
    if n.contains("desktop") || n.contains("tauri") || signals.iter().any(|s| s == "tauri") {
        return ScopeKind::Desktop;
    }
    if n.contains("api")
        || n.contains("worker")
        || n.contains("server")
        || signals.iter().any(|s| s == "wrangler")
    {
        return ScopeKind::Api;
    }
    if n.contains("doc") || n.contains("site") {
        return ScopeKind::Docs;
    }
    if n.contains("web")
        || n.contains("app")
        || n.contains("frontend")
        || signals.iter().any(|s| s == "vercel" || s == "netlify")
    {
        return ScopeKind::Web;
    }
    if signals.iter().any(|s| s == "wrangler") {
        ScopeKind::Api
    } else {
        ScopeKind::Web
    }
}

fn signals_for(dir: &Path) -> Vec<String> {
    let mut s = Vec::new();
    if has(dir, "wrangler.toml") || has(dir, "wrangler.json") || has(dir, "wrangler.jsonc") {
        s.push("wrangler".into());
    }
    if has(dir, "vercel.json") || dir.join(".vercel").is_dir() {
        s.push("vercel".into());
    }
    if has(dir, "netlify.toml") || dir.join(".netlify").is_dir() {
        s.push("netlify".into());
    }
    if dir.join("src-tauri").is_dir() {
        s.push("tauri".into());
    }
    if has(dir, "package.json") {
        s.push("node".into());
    }
    // Mobile — local layout only (no vendor HTTPS).
    let name = dir
        .file_name()
        .map(|x| x.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name == "android"
        || dir.join("android").is_dir()
        || has(dir, "build.gradle")
        || has(dir, "build.gradle.kts")
    {
        s.push("android".into());
        s.push("mobile".into());
    }
    if name == "ios" || dir.join("ios").is_dir() || dir.join("ios").join("Podfile").is_file() {
        s.push("ios".into());
        s.push("mobile".into());
    }
    if has(dir, "app.json") || has(dir, "app.config.js") || has(dir, "app.config.ts") {
        if file_mentions(dir, &["app.json", "app.config.js", "app.config.ts"], "expo") {
            s.push("expo".into());
            s.push("mobile".into());
        }
    }
    if has(dir, "pubspec.yaml") {
        s.push("flutter".into());
        s.push("mobile".into());
    }
    if has(dir, "capacitor.config.json")
        || has(dir, "capacitor.config.ts")
        || has(dir, "capacitor.config.js")
    {
        s.push("capacitor".into());
        s.push("mobile".into());
    }
    if has(dir, "Dockerfile") || has(dir, "Containerfile") || has(dir, "dockerfile") {
        s.push("docker".into());
    }
    if has(dir, "docker-compose.yml")
        || has(dir, "docker-compose.yaml")
        || has(dir, "compose.yml")
        || has(dir, "compose.yaml")
    {
        s.push("compose".into());
        s.push("docker".into());
    }
    s
}

fn file_mentions(dir: &Path, names: &[&str], needle: &str) -> bool {
    let n = needle.to_ascii_lowercase();
    for name in names {
        let Ok(raw) = fs::read_to_string(dir.join(name)) else {
            continue;
        };
        if raw.to_ascii_lowercase().contains(&n) {
            return true;
        }
    }
    false
}

fn deploy_for(signals: &[String]) -> (Option<String>, Vec<String>) {
    if signals.iter().any(|s| s == "wrangler") {
        return (
            Some("cloudflare".into()),
            vec!["deploy".into(), "--provider".into(), "cloudflare".into()],
        );
    }
    if signals.iter().any(|s| s == "vercel") {
        return (
            Some("vercel".into()),
            vec!["deploy".into(), "--provider".into(), "vercel".into()],
        );
    }
    if signals.iter().any(|s| s == "netlify") {
        return (
            Some("netlify".into()),
            vec!["deploy".into(), "--provider".into(), "netlify".into()],
        );
    }
    (None, vec!["status".into()])
}

fn make_scope(project: &Path, dir: &Path, id: &str, kind: ScopeKind, label: &str) -> Scope {
    let signals = signals_for(dir);
    let (provider, deploy_args) = deploy_for(&signals);
    Scope {
        id: id.into(),
        kind,
        label: label.into(),
        root: dir.display().to_string(),
        relative: rel(project, dir),
        provider,
        deploy_args,
        signals,
    }
}

fn consider(project: &Path, dir: &Path, id_hint: &str, out: &mut Vec<Scope>) {
    let signals = signals_for(dir);
    if signals.is_empty() {
        return;
    }
    if signals.len() == 1 && signals[0] == "node" && !has(dir, "vercel.json") {
        // Bare Node apps at the repo root are too noisy. Workspace packages
        // (apps/* , packages/*) are real deploy/release targets.
        let rel_path = rel(project, dir);
        let workspace_pkg = rel_path.starts_with("apps/") || rel_path.starts_with("packages/");
        if !workspace_pkg {
            return;
        }
    }
    let name = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| id_hint.to_string());
    let kind = kind_from_name(&name, &signals);
    let id = format!(
        "{}.{}",
        match kind {
            ScopeKind::Web => "web",
            ScopeKind::Api => "api",
            ScopeKind::Desktop => "desktop",
            ScopeKind::Docs => "docs",
            ScopeKind::Mobile => "mobile",
            ScopeKind::Container => "container",
            ScopeKind::Root => "root",
        },
        name.to_ascii_lowercase().replace([' ', '_'], "-")
    );
    if out.iter().any(|s| s.id == id || s.root == dir.display().to_string()) {
        return;
    }
    let label = match kind {
        ScopeKind::Web => format!("Web — {name}"),
        ScopeKind::Api => format!("API — {name}"),
        ScopeKind::Desktop => format!("Desktop — {name}"),
        ScopeKind::Docs => format!("Docs — {name}"),
        ScopeKind::Mobile => format!("Mobile — {name}"),
        ScopeKind::Container => format!("Container — {name}"),
        ScopeKind::Root => "Root".into(),
    };
    out.push(make_scope(project, dir, &id, kind, &label));
}

pub fn detect(project: &Path) -> Vec<Scope> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let mut out = Vec::new();

    consider(&project, &project, "root", &mut out);
    if !out.is_empty() {
        if let Some(s) = out.first_mut() {
            // Prefer deploy-host / desktop signals over mobile/container when remapping root.
            // (Expo monorepos often have android/ + wrangler.toml at root — API ships; Mobile stays on android/ios dirs.)
            s.kind = if s.signals.iter().any(|x| x == "tauri") {
                ScopeKind::Desktop
            } else if s.signals.iter().any(|x| x == "wrangler") {
                ScopeKind::Api
            } else if s.signals.iter().any(|x| x == "vercel" || x == "netlify") {
                ScopeKind::Web
            } else if s.signals.iter().any(|x| {
                matches!(
                    x.as_str(),
                    "mobile" | "android" | "ios" | "expo" | "flutter" | "capacitor"
                )
            }) {
                ScopeKind::Mobile
            } else if s.signals.iter().any(|x| x == "docker" || x == "compose") {
                ScopeKind::Container
            } else {
                s.kind.clone()
            };
            if s.relative.is_empty() || s.relative == "." {
                s.id = match s.kind {
                    ScopeKind::Api => "api.root".into(),
                    ScopeKind::Desktop => "desktop.root".into(),
                    ScopeKind::Docs => "docs.root".into(),
                    ScopeKind::Mobile => "mobile.root".into(),
                    ScopeKind::Container => "container.root".into(),
                    _ => "web.root".into(),
                };
                s.label = match s.kind {
                    ScopeKind::Api => "API — project root".into(),
                    ScopeKind::Desktop => "Desktop — project root".into(),
                    ScopeKind::Mobile => "Mobile — project root".into(),
                    ScopeKind::Container => "Container — project root".into(),
                    _ => "Web — project root".into(),
                };
            }
        }
    }

    let apps = project.join("apps");
    if apps.is_dir() {
        if let Ok(entries) = fs::read_dir(&apps) {
            for ent in entries.flatten() {
                let p = ent.path();
                if p.is_dir() && !skip_dir(&ent.file_name().to_string_lossy()) {
                    consider(&project, &p, &ent.file_name().to_string_lossy(), &mut out);
                }
            }
        }
    }

    let packages = project.join("packages");
    if packages.is_dir() {
        if let Ok(entries) = fs::read_dir(&packages) {
            for ent in entries.flatten() {
                let p = ent.path();
                if p.is_dir() && !skip_dir(&ent.file_name().to_string_lossy()) {
                    consider(&project, &p, &ent.file_name().to_string_lossy(), &mut out);
                }
            }
        }
    }

    // Top-level native mobile trees (Expo/RN/Flutter often use ./android · ./ios).
    for name in ["android", "ios", "mobile"] {
        let p = project.join(name);
        if p.is_dir() {
            consider(&project, &p, name, &mut out);
        }
    }

    if let Ok(entries) = fs::read_dir(&project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let name = ent.file_name().to_string_lossy().to_string();
            if skip_dir(&name) || name == "apps" {
                continue;
            }
            consider(&project, &p, &name, &mut out);
        }
    }

    if out.is_empty() {
        out.push(Scope {
            id: "root".into(),
            kind: ScopeKind::Root,
            label: "Project root".into(),
            root: project.display().to_string(),
            relative: ".".into(),
            provider: None,
            deploy_args: vec!["status".into()],
            signals: vec![],
        });
    }
    out
}

fn active_path(project: &Path) -> PathBuf {
    config::ship_dir(project).join("scopes.json")
}

pub fn load_active(project: &Path) -> Vec<String> {
    let Ok(raw) = fs::read_to_string(active_path(project)) else {
        return Vec::new();
    };
    serde_json::from_str::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| {
            v.get("active")
                .and_then(|a| a.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str().map(|s| s.to_string()))
                        .collect()
                })
        })
        .unwrap_or_default()
}

pub fn save_active(project: &Path, ids: &[String]) -> Result<()> {
    let plan = detect(project);
    for id in ids {
        if !plan.iter().any(|s| s.id == *id) {
            bail!("unknown scope id: {id}");
        }
    }
    let dir = config::ship_dir(project);
    fs::create_dir_all(&dir)?;
    fs::write(
        active_path(project),
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "ship-studio/scopes/v1",
            "active": ids,
        }))?,
    )?;
    Ok(())
}

pub fn plan_for(project: &Path) -> ScopesPlan {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let scopes = detect(&project);
    let mut active = load_active(&project);
    if active.is_empty() {
        active = scopes.iter().map(|s| s.id.clone()).collect();
    }
    active.retain(|id| scopes.iter().any(|s| s.id == *id));
    if active.is_empty() {
        if let Some(first) = scopes.first() {
            active.push(first.id.clone());
        }
    }
    ScopesPlan {
        schema: "ship-studio/scopes/v1".into(),
        project: project.display().to_string(),
        notes: vec![
            "Select Web / API / Desktop / Mobile / Container scopes; deploy Open/Run uses that directory.".into(),
            "Token create stays on official dashboards — this list is local layout only.".into(),
        ],
        scopes,
        active,
    }
}

pub fn selected(project: &Path) -> Vec<Scope> {
    let plan = plan_for(project);
    plan.scopes
        .into_iter()
        .filter(|s| plan.active.iter().any(|id| id == &s.id))
        .collect()
}

pub fn set_active(project: &Path, ids: Vec<String>) -> Result<ScopesPlan> {
    if ids.is_empty() {
        bail!("at least one scope required");
    }
    save_active(project, &ids)?;
    Ok(plan_for(project))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_web_and_api() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-scopes-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("apps/web")).unwrap();
        fs::create_dir_all(dir.join("apps/api")).unwrap();
        fs::write(dir.join("apps/web/vercel.json"), "{}\n").unwrap();
        fs::write(dir.join("apps/api/wrangler.toml"), "name = \"api\"\n").unwrap();
        let plan = plan_for(&dir);
        assert!(plan.scopes.iter().any(|s| s.kind == ScopeKind::Web));
        assert!(plan.scopes.iter().any(|s| s.kind == ScopeKind::Api));
        assert!(plan.scopes.iter().any(|s| s.provider.as_deref() == Some("cloudflare")));
        assert!(plan.scopes.iter().any(|s| s.provider.as_deref() == Some("vercel")));
    }

    #[test]
    fn detects_workspace_node_apps() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-scopes-ws-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/website/package.json"), "{}\n").unwrap();
        fs::create_dir_all(dir.join("noise")).unwrap();
        fs::write(dir.join("noise/package.json"), "{}\n").unwrap();
        let plan = plan_for(&dir);
        assert!(
            plan.scopes.iter().any(|s| s.relative.replace('\\', "/") == "apps/website"),
            "expected apps/website, got {:?}",
            plan.scopes.iter().map(|s| s.relative.clone()).collect::<Vec<_>>()
        );
        assert!(
            plan.scopes.iter().all(|s| s.relative.replace('\\', "/") != "noise"),
            "root-level node dirs should stay hidden"
        );
    }

    #[test]
    fn detects_mobile_android_and_expo() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-scopes-mobile-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::write(
            dir.join("app.json"),
            r#"{"expo":{"name":"demo","slug":"demo"}}"#,
        )
        .unwrap();
        fs::write(dir.join("android/build.gradle"), "// stub\n").unwrap();
        let plan = plan_for(&dir);
        assert!(
            plan.scopes
                .iter()
                .any(|s| s.kind == ScopeKind::Mobile),
            "expected Mobile scope, got {:?}",
            plan.scopes.iter().map(|s| (&s.id, &s.kind)).collect::<Vec<_>>()
        );
        assert!(plan
            .scopes
            .iter()
            .any(|s| s.signals.iter().any(|x| x == "android" || x == "expo")));
    }

    #[test]
    fn root_prefers_api_when_wrangler_and_mobile_coexist() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-scopes-mixed-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("android")).unwrap();
        fs::write(dir.join("wrangler.toml"), "name = \"x\"\n").unwrap();
        fs::write(
            dir.join("app.json"),
            r#"{"expo":{"name":"demo","slug":"demo"}}"#,
        )
        .unwrap();
        fs::write(dir.join("android/build.gradle"), "// stub\n").unwrap();
        let plan = plan_for(&dir);
        let root = plan
            .scopes
            .iter()
            .find(|s| s.relative == "." || s.relative.is_empty())
            .expect("root scope");
        assert_eq!(root.kind, ScopeKind::Api, "root should be API when wrangler present");
        assert!(
            plan.scopes.iter().any(|s| s.kind == ScopeKind::Mobile),
            "android/ should still yield Mobile"
        );
    }

    #[test]
    fn detects_container_dockerfile() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-scopes-docker-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Dockerfile"), "FROM alpine\n").unwrap();
        let plan = plan_for(&dir);
        assert!(
            plan.scopes
                .iter()
                .any(|s| s.kind == ScopeKind::Container),
            "expected Container scope, got {:?}",
            plan.scopes.iter().map(|s| (&s.id, &s.kind)).collect::<Vec<_>>()
        );
    }
}

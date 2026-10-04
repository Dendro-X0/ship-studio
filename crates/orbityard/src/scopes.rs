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
    if dir.join("src-tauri").is_dir() || has(dir, "tauri.conf.json") {
        s.push("tauri".into());
    }
    if has(dir, "electron-builder.yml")
        || has(dir, "electron-builder.json")
        || dir.join("electron").is_dir()
    {
        s.push("electron".into());
    }
    if has(dir, "wails.json") || dir.join("frontend").join("wailsjs").is_dir() {
        s.push("wails".into());
    }
    if has_dotnet(dir) {
        s.push("dotnet".into());
    }
    if has(dir, "next.config.js")
        || has(dir, "next.config.ts")
        || has(dir, "next.config.mjs")
        || has(dir, "next.config.cjs")
    {
        s.push("next".into());
    }
    if has(dir, "astro.config.mjs")
        || has(dir, "astro.config.ts")
        || has(dir, "astro.config.js")
    {
        s.push("astro".into());
    }
    if has(dir, "nuxt.config.ts") || has(dir, "nuxt.config.js") {
        s.push("nuxt".into());
    }
    if has(dir, "svelte.config.js") || has(dir, "svelte.config.ts") {
        s.push("svelte".into());
    }
    if has(dir, "package.json") {
        s.push("node".into());
        push_npm_framework_signals(dir, &mut s);
    }
    if has(dir, "index.html")
        && !s.iter().any(|x| {
            matches!(
                x.as_str(),
                "next" | "react" | "vue" | "svelte" | "astro" | "nuxt" | "remix"
            )
        })
    {
        s.push("static".into());
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
    s.sort();
    s.dedup();
    s
}

fn has_dotnet(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|ent| {
        ent.path()
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e, "csproj" | "fsproj" | "vbproj" | "sln"))
    })
}

fn push_npm_framework_signals(dir: &Path, s: &mut Vec<String>) {
    let Ok(raw) = fs::read_to_string(dir.join("package.json")) else {
        return;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return;
    };
    let mut deps = serde_json::Map::new();
    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(obj) = v.get(key).and_then(|x| x.as_object()) {
            for (k, val) in obj {
                deps.insert(k.clone(), val.clone());
            }
        }
    }
    let has_dep = |name: &str| deps.contains_key(name);
    if has_dep("next") {
        s.push("next".into());
    }
    if has_dep("react-native") || has_dep("react-native-web") {
        s.push("react-native".into());
        s.push("mobile".into());
    }
    if has_dep("expo") || has_dep("expo-router") {
        s.push("expo".into());
        s.push("mobile".into());
    }
    if has_dep("@remix-run/react") || has_dep("@remix-run/node") {
        s.push("remix".into());
    }
    if has_dep("astro") {
        s.push("astro".into());
    }
    if has_dep("nuxt") {
        s.push("nuxt".into());
    }
    if has_dep("vue") || has_dep("nuxt") {
        s.push("vue".into());
    }
    if has_dep("svelte") || has_dep("@sveltejs/kit") {
        s.push("svelte".into());
    }
    if has_dep("react") && !has_dep("react-native") && !has_dep("next") {
        s.push("react".into());
    }
    if has_dep("express") {
        s.push("express".into());
    }
    if has_dep("@nestjs/core") {
        s.push("nest".into());
    }
    if has_dep("fastify") {
        s.push("fastify".into());
    }
    if has_dep("hono") {
        s.push("hono".into());
    }
    if has_dep("electron") {
        s.push("electron".into());
    }
    if has_dep("@tauri-apps/api") || has_dep("@tauri-apps/cli") {
        s.push("tauri".into());
    }
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

/// `{Surface} – {Framework} | {Framework}` from detected signals (sidebar groups by kind).
fn scope_label(kind: ScopeKind, signals: &[String]) -> String {
    let surface = match kind {
        ScopeKind::Desktop => "Desktop",
        ScopeKind::Web | ScopeKind::Docs => "Website",
        ScopeKind::Api => "API",
        ScopeKind::Mobile => "Mobile",
        ScopeKind::Container => "Container",
        ScopeKind::Root => "Project",
    };
    let stack = framework_stack(kind, signals);
    if stack.is_empty() {
        return surface.into();
    }
    format!("{surface} – {}", stack.join(" | "))
}

fn framework_stack(kind: ScopeKind, signals: &[String]) -> Vec<&'static str> {
    let has = |id: &str| signals.iter().any(|s| s == id);
    let mut out: Vec<&'static str> = Vec::new();
    match kind {
        ScopeKind::Desktop => {
            if has("tauri") {
                out.push("Tauri");
            }
            if has("electron") {
                out.push("Electron");
            }
            if has("wails") {
                out.push("Wails");
            }
            if has("dotnet") {
                out.push(".NET");
            }
        }
        ScopeKind::Web | ScopeKind::Docs => {
            if has("next") {
                out.push("Next.js");
            }
            if has("remix") {
                out.push("Remix");
            }
            if has("astro") {
                out.push("Astro");
            }
            if has("nuxt") {
                out.push("Nuxt");
            }
            if has("svelte") {
                out.push("Svelte");
            }
            if has("vue") && !has("nuxt") {
                out.push("Vue");
            }
            if has("react") && !has("next") && !has("remix") {
                out.push("React");
            }
            if out.is_empty() && has("static") {
                out.push("Static");
            }
            if out.is_empty() && has("node") {
                out.push("Node");
            }
        }
        ScopeKind::Api => {
            if has("wrangler") {
                out.push("Workers");
            }
            if has("express") {
                out.push("Express");
            }
            if has("nest") {
                out.push("NestJS");
            }
            if has("fastify") {
                out.push("Fastify");
            }
            if has("hono") {
                out.push("Hono");
            }
            if out.is_empty() && has("node") {
                out.push("Node.js");
            }
        }
        ScopeKind::Mobile => {
            if has("react-native") {
                out.push("React Native");
            }
            if has("expo") && !has("react-native") {
                out.push("Expo");
            }
            if has("flutter") {
                out.push("Flutter");
            }
            if has("capacitor") {
                out.push("Capacitor");
            }
            if out.is_empty() && has("android") {
                out.push("Android");
            }
            if out.is_empty() && has("ios") {
                out.push("iOS");
            }
        }
        ScopeKind::Container => {
            if has("compose") {
                out.push("Compose");
            } else if has("docker") {
                out.push("Docker");
            }
        }
        ScopeKind::Root => {}
    }
    out
}

fn consider(project: &Path, dir: &Path, id_hint: &str, out: &mut Vec<Scope>) {
    let signals = signals_for(dir);
    if signals.is_empty() {
        return;
    }
    // Bare Node at repo root is noise. Workspace packages + framework/static signals stay.
    let only_node = signals.iter().all(|x| x == "node");
    if only_node && !has(dir, "vercel.json") {
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
    let label = scope_label(kind.clone(), &signals);
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
                s.label = scope_label(s.kind.clone(), &s.signals);
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
            "schema": "orbit-yard/scopes/v1",
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
        schema: "orbit-yard/scopes/v1".into(),
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
    fn labels_use_framework_stack() {
        let dir = std::env::temp_dir().join(format!(
            "orbityard-scopes-labels-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("apps/desktop/src-tauri")).unwrap();
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/desktop/package.json"), "{}\n").unwrap();
        fs::write(dir.join("apps/website/package.json"), "{}\n").unwrap();
        fs::write(dir.join("apps/website/index.html"), "<html></html>\n").unwrap();
        fs::create_dir_all(dir.join("apps/api")).unwrap();
        fs::write(
            dir.join("apps/api/package.json"),
            r#"{"dependencies":{"express":"^4.0.0"}}"#,
        )
        .unwrap();
        fs::create_dir_all(dir.join("apps/web")).unwrap();
        fs::write(
            dir.join("apps/web/package.json"),
            r#"{"dependencies":{"next":"^14.0.0","react":"^18.0.0"}}"#,
        )
        .unwrap();
        let plan = plan_for(&dir);
        let desk = plan
            .scopes
            .iter()
            .find(|s| s.kind == ScopeKind::Desktop)
            .expect("desktop");
        let docs = plan
            .scopes
            .iter()
            .find(|s| s.relative.replace('\\', "/") == "apps/website")
            .expect("website");
        let api = plan
            .scopes
            .iter()
            .find(|s| s.kind == ScopeKind::Api)
            .expect("api");
        let web = plan
            .scopes
            .iter()
            .find(|s| s.relative.replace('\\', "/") == "apps/web")
            .expect("web");
        assert_eq!(desk.label, "Desktop – Tauri");
        assert_eq!(docs.label, "Website – Static");
        assert_eq!(api.label, "API – Express");
        assert_eq!(web.label, "Website – Next.js");
    }

    #[test]
    fn detects_web_and_api() {
        let dir = std::env::temp_dir().join(format!(
            "orbityard-scopes-{}",
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
            "orbityard-scopes-ws-{}",
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
            "orbityard-scopes-mobile-{}",
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
            "orbityard-scopes-mixed-{}",
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
            "orbityard-scopes-docker-{}",
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

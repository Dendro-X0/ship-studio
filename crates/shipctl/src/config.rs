use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioIntent {
    pub schema: String,
    pub project: String,
    pub workflow: Vec<String>,
    pub adapters: Adapters,
    #[serde(default = "default_true")]
    pub offline_bridge: bool,
    /// Args passed to `signet` during `flow` / default Sign action.
    #[serde(default = "default_sign_args")]
    pub sign_args: Vec<String>,
    /// Args passed to `orbit` during `flow` / default Deploy action.
    #[serde(default = "default_deploy_args")]
    pub deploy_args: Vec<String>,
    #[serde(default)]
    pub detected: Detected,
    #[serde(default)]
    pub notes: Vec<String>,
    /// Selected deploy scopes (`web.*` / `api.*` / `desktop.*`).
    #[serde(default)]
    pub active_scopes: Vec<String>,
    /// `self` | `official` | `self_then_official`
    #[serde(default = "default_sign_path")]
    pub sign_path: String,
}

fn default_sign_path() -> String {
    "self_then_official".into()
}

fn default_true() -> bool {
    true
}

fn default_sign_args() -> Vec<String> {
    vec!["doctor".into(), "--json".into()]
}

fn default_deploy_args() -> Vec<String> {
    // Non-interactive default — `orbit ship` is a TUI.
    vec!["status".into()]
}

fn suggested_deploy_args(detected: &Detected) -> Vec<String> {
    if detected.wrangler {
        return vec![
            "deploy".into(),
            "--provider".into(),
            "cloudflare".into(),
        ];
    }
    if detected.vercel {
        return vec!["deploy".into(), "--provider".into(), "vercel".into()];
    }
    if detected.netlify {
        return vec!["deploy".into(), "--provider".into(), "netlify".into()];
    }
    if detected.orbit_configured {
        return vec!["status".into()];
    }
    default_deploy_args()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Adapters {
    #[serde(default = "default_signet")]
    pub signet: String,
    #[serde(default = "default_orbit")]
    pub orbit: String,
}

impl Default for Adapters {
    fn default() -> Self {
        Self {
            signet: default_signet(),
            orbit: default_orbit(),
        }
    }
}

fn default_signet() -> String {
    "signet".into()
}
fn default_orbit() -> String {
    "orbit".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Detected {
    pub signet_toml: bool,
    pub package_json: bool,
    pub tauri: bool,
    pub wrangler: bool,
    pub vercel: bool,
    #[serde(default)]
    pub netlify: bool,
    #[serde(default)]
    pub github: bool,
    #[serde(default)]
    pub polar: bool,
    #[serde(default)]
    pub orbit_configured: bool,
    /// Any mobile layout (Android / iOS / Expo / Flutter / Capacitor).
    #[serde(default)]
    pub mobile: bool,
    #[serde(default)]
    pub android: bool,
    #[serde(default)]
    pub ios: bool,
    #[serde(default)]
    pub expo: bool,
    /// Cloudflare D1 binding in wrangler.
    #[serde(default)]
    pub d1: bool,
    #[serde(default)]
    pub neon: bool,
    #[serde(default)]
    pub supabase: bool,
    #[serde(default)]
    pub turso: bool,
    /// GitHub Actions workflow filename(s) matching `*release*`.
    #[serde(default)]
    pub ci_release: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub release_workflows: Vec<String>,
    /// Dockerfile / Compose present.
    #[serde(default)]
    pub container: bool,
    #[serde(default)]
    pub dockerfile: bool,
    #[serde(default)]
    pub compose: bool,
    /// Opt-in extra marketplaces (Steam / itch / Epic).
    #[serde(default)]
    pub steam: bool,
    #[serde(default)]
    pub itch: bool,
    #[serde(default)]
    pub epic: bool,
    /// Root LICENSE / COPYING present.
    #[serde(default)]
    pub license: bool,
    /// Root TRUST.md (Signet-style honesty).
    #[serde(default)]
    pub trust_md: bool,
    /// Root SECURITY.md.
    #[serde(default)]
    pub security_md: bool,
    /// Root CHANGELOG*.
    #[serde(default)]
    pub changelog: bool,
    /// Publishable npm package (not private app).
    #[serde(default)]
    pub npm_publish: bool,
    /// Publishable crates.io package.
    #[serde(default)]
    pub crates_publish: bool,
    /// Installable PWA (web manifest / vite-plugin-pwa).
    #[serde(default)]
    pub pwa: bool,
    /// Hugging Face Hub model/repo lane (opt-in or model card).
    #[serde(default)]
    pub huggingface: bool,
    /// Marketing / landing / GitHub Pages site present.
    #[serde(default)]
    pub marketing_site: bool,
    /// Hint for marketing host (pages | vercel | netlify | unknown).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub marketing_host: String,
    /// Graduate / OV / notarization signing path opted in.
    #[serde(default)]
    pub graduate_sign: bool,
    /// Gumroad commerce markers.
    #[serde(default)]
    pub gumroad: bool,
    /// Lemon Squeezy commerce markers.
    #[serde(default)]
    pub lemon: bool,
    /// Cross-suite URL sync configured (`.ship/suite.json` or markets).
    #[serde(default)]
    pub suite_sync: bool,
    /// Human-readable sibling targets for suite.url_sync detail.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub suite_detail: String,
    /// Optional canonical URL hint from suite.json.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub suite_canonical: String,
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub id: String,
    pub ok: bool,
    pub exit_code: i32,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastRun {
    pub finished: bool,
    pub ok: bool,
    pub started_at: String,
    pub finished_at: String,
    pub dry_run: bool,
    pub offline: bool,
    pub steps: Vec<StepResult>,
    pub message: String,
    /// Known live URLs from Orbit summaries / operator confirm (no secrets).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<String>,
}

pub fn ship_dir(project: &Path) -> PathBuf {
    project.join(".ship")
}

pub fn studio_path(project: &Path) -> PathBuf {
    ship_dir(project).join("studio.json")
}

pub fn read_studio(project: &Path) -> Result<Option<StudioIntent>> {
    let path = studio_path(project);
    if !path.is_file() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&raw)?))
}

fn any_named(project: &Path, names: &[&str]) -> bool {
    for name in names {
        if project.join(name).is_file() {
            return true;
        }
    }
    // One-level nested (e.g. apps/api/wrangler.toml)
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let name = ent.file_name().to_string_lossy().to_lowercase();
            if name == "node_modules" || name == ".git" || name == "target" || name == "dist" {
                continue;
            }
            for n in names {
                if p.join(n).is_file() {
                    return true;
                }
            }
            // apps/*/wrangler.toml
            if name == "apps" {
                if let Ok(apps) = fs::read_dir(&p) {
                    for app in apps.flatten() {
                        let ap = app.path();
                        if !ap.is_dir() {
                            continue;
                        }
                        for n in names {
                            if ap.join(n).is_file() {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

pub fn probe(project: &Path) -> Detected {
    let mut d = Detected::default();
    d.signet_toml = project.join("signet.toml").is_file();
    d.package_json = project.join("package.json").is_file();
    d.tauri = project.join("src-tauri").is_dir()
        || project.join("apps/desktop/src-tauri").is_dir();
    d.wrangler = any_named(
        project,
        &["wrangler.toml", "wrangler.json", "wrangler.jsonc"],
    );
    d.vercel = any_named(project, &["vercel.json"])
        || project.join(".vercel").is_dir();
    d.netlify = any_named(project, &["netlify.toml"])
        || project.join(".netlify").is_dir();
    d.github = project.join(".git").is_dir() || project.join(".git").is_file();
    d.polar = detect_polar(project);
    d.orbit_configured = project.join(".orbit/state.json").is_file();
    detect_mobile(project, &mut d);
    detect_db(project, &mut d);
    detect_ci_release(project, &mut d);
    detect_container(project, &mut d);
    detect_markets(project, &mut d);
    detect_launch_baseline(project, &mut d);
    detect_package_registries(project, &mut d);
    detect_pwa(project, &mut d);
    detect_huggingface(project, &mut d);
    detect_marketing_site(project, &mut d);
    detect_graduate_commerce(project, &mut d);
    detect_suite_sync(project, &mut d);
    let orbit_configured = d.orbit_configured;

    if d.signet_toml {
        d.hints
            .push("signet.toml found — flow sign uses doctor; set sign_args for build/release.".into());
    } else {
        d.hints
            .push("No signet.toml — run `signet init` or `signet scan --apply` in the project.".into());
    }
    if d.tauri {
        d.hints
            .push("Tauri layout detected — Signet build/ship can target desktop.".into());
    }
    if d.wrangler {
        d.hints
            .push("wrangler config detected — Orbit can deploy Cloudflare.".into());
    }
    if d.vercel {
        d.hints
            .push("vercel.json / .vercel detected — Orbit can deploy Vercel.".into());
    }
    if d.netlify {
        d.hints
            .push("netlify.toml / .netlify detected — Orbit can deploy Netlify.".into());
    }
    if d.github {
        d.hints
            .push("git repo detected — portal can open GitHub token / gh auth login.".into());
    }
    if d.polar {
        d.hints
            .push("Polar markers detected — portal opens polar.sh dashboard for checkout/webhook.".into());
    }
    if d.mobile {
        let mut bits = Vec::new();
        if d.android {
            bits.push("Android");
        }
        if d.ios {
            bits.push("iOS");
        }
        if d.expo {
            bits.push("Expo");
        }
        d.hints.push(format!(
            "Mobile layout detected ({}) — Advanced publish can open Play / App Store Connect listing.",
            if bits.is_empty() {
                "native".to_string()
            } else {
                bits.join(" · ")
            }
        ));
    }
    if d.d1 || d.neon || d.supabase || d.turso {
        let mut bits = Vec::new();
        if d.d1 {
            bits.push("D1");
        }
        if d.neon {
            bits.push("Neon");
        }
        if d.supabase {
            bits.push("Supabase");
        }
        if d.turso {
            bits.push("Turso");
        }
        d.hints.push(format!(
            "Database hosting markers ({}) — portal opens create URLs; put connection strings on the deploy target.",
            bits.join(" · ")
        ));
    }
    if d.ci_release {
        d.hints.push(format!(
            "CI ship workflow(s): {} — open GitHub Actions after tag/deploy.",
            d.release_workflows.join(", ")
        ));
    }
    if d.container {
        let mut bits = Vec::new();
        if d.dockerfile {
            bits.push("Dockerfile");
        }
        if d.compose {
            bits.push("Compose");
        }
        d.hints.push(format!(
            "Container layout ({}) — Advanced `container.build` Runs local docker; `container.deploy` is push docs + Confirm (never auto-push).",
            if bits.is_empty() {
                "docker".into()
            } else {
                bits.join(" · ")
            }
        ));
    }
    if d.steam || d.itch || d.epic {
        let mut bits = Vec::new();
        if d.steam {
            bits.push("Steam");
        }
        if d.itch {
            bits.push("itch.io");
        }
        if d.epic {
            bits.push("Epic");
        }
        d.hints.push(format!(
            "Extra marketplace(s) ({}) — Advanced listing steps open partner dashboards (URL + confirm).",
            bits.join(" · ")
        ));
    }
    if !d.license || !d.security_md {
        let mut miss = Vec::new();
        if !d.license {
            miss.push("LICENSE");
        }
        if !d.security_md {
            miss.push("SECURITY.md");
        }
        d.hints.push(format!(
            "Launch baseline missing ({}) — Advanced legal.baseline before a public cut.",
            miss.join(" · ")
        ));
    }
    if (d.tauri || d.signet_toml) && !d.trust_md {
        d.hints
            .push("Desktop/Signet without TRUST.md — Advanced trust.pack for checksum honesty.".into());
    }
    if d.npm_publish || d.crates_publish {
        let mut bits = Vec::new();
        if d.npm_publish {
            bits.push("npm");
        }
        if d.crates_publish {
            bits.push("crates.io");
        }
        d.hints.push(format!(
            "Package registry ({}) — Advanced listing opens publisher dashboards (URL + confirm).",
            bits.join(" · ")
        ));
    }
    if d.pwa {
        d.hints.push(
            "PWA manifest detected — deploy host + marketing.deploy; ensure manifest/service worker live on canonical URL."
                .into(),
        );
    }
    if d.huggingface {
        d.hints.push(
            "Hugging Face lane — Advanced listing.huggingface opens Hub docs; upload stays on huggingface-cli (Confirm only)."
                .into(),
        );
    }
    if d.marketing_site {
        let host = if d.marketing_host.is_empty() {
            "host dashboard".into()
        } else {
            d.marketing_host.clone()
        };
        d.hints.push(format!(
            "Marketing / landing site detected — Advanced marketing.deploy opens {host} (URL + confirm)."
        ));
    }
    if d.graduate_sign {
        d.hints.push(
            "Graduate signing opted in — Advanced sign.graduate for OV / Authenticode / notarization (no verified-publisher claims)."
                .into(),
        );
    }
    if d.gumroad || d.lemon {
        let mut bits = Vec::new();
        if d.gumroad {
            bits.push("Gumroad");
        }
        if d.lemon {
            bits.push("Lemon");
        }
        d.hints.push(format!(
            "Commerce ({}) — Advanced listing opens SKU dashboards (URL + confirm).",
            bits.join(" · ")
        ));
    }
    if d.suite_sync {
        d.hints.push(format!(
            "Suite URL sync — Advanced suite.url_sync ({})",
            if d.suite_detail.is_empty() {
                "add .ship/suite.json siblings".into()
            } else {
                d.suite_detail.clone()
            }
        ));
    }
    if orbit_configured {
        d.hints
            .push(".orbit/state.json found — Orbit already configured for this repo.".into());
    }
    if !d.wrangler && !d.vercel && !d.netlify && !orbit_configured {
        d.hints
            .push("No wrangler/vercel/netlify/Orbit config yet — run `shipctl portal` then `orbit configure`.".into());
    } else {
        d.hints
            .push("Run `shipctl portal` to open OAuth / token entry points for detected providers.".into());
    }
    d
}

fn detect_mobile(project: &Path, d: &mut Detected) {
    d.android = project.join("android").is_dir()
        || project.join("apps/android").is_dir()
        || project.join("build.gradle").is_file()
        || project.join("build.gradle.kts").is_file()
        || any_named(project, &["build.gradle", "build.gradle.kts"]);
    d.ios = project.join("ios").is_dir()
        || project.join("apps/ios").is_dir()
        || project.join("ios/Podfile").is_file();
    d.expo = file_mentions_any(
        project,
        &["app.json", "app.config.js", "app.config.ts"],
        "expo",
    ) || nested_file_mentions(project, &["app.json", "app.config.js", "app.config.ts"], "expo");
    let flutter = project.join("pubspec.yaml").is_file()
        || any_named(project, &["pubspec.yaml"]);
    let capacitor = any_named(
        project,
        &[
            "capacitor.config.json",
            "capacitor.config.ts",
            "capacitor.config.js",
        ],
    );
    d.mobile = d.android || d.ios || d.expo || flutter || capacitor;
}

fn detect_db(project: &Path, d: &mut Detected) {
    d.d1 = wrangler_mentions(project, &["[[d1_databases]]", "d1_databases"]);
    d.neon = env_key_prefix(project, "NEON_")
        || env_value_contains(project, "neon.tech")
        || package_mentions(project, &["@neondatabase"]);
    d.supabase = project.join("supabase/config.toml").is_file()
        || env_key_prefix(project, "SUPABASE_")
        || env_value_contains(project, "supabase.co");
    d.turso = env_key_prefix(project, "TURSO_")
        || env_key_prefix(project, "LIBSQL_")
        || package_mentions(project, &["@libsql", "@tursodatabase"]);
}

fn detect_ci_release(project: &Path, d: &mut Detected) {
    let dir = project.join(".github").join("workflows");
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let mut names = Vec::new();
    for ent in entries.flatten() {
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        let name = ent.file_name().to_string_lossy().to_string();
        let lower = name.to_ascii_lowercase();
        if !(lower.ends_with(".yml") || lower.ends_with(".yaml")) {
            continue;
        }
        if lower.contains("release") || lower.contains("deploy") {
            names.push(name);
        }
    }
    names.sort();
    d.ci_release = !names.is_empty();
    d.release_workflows = names;
}

fn detect_container(project: &Path, d: &mut Detected) {
    d.dockerfile = has_dockerfile(project);
    d.compose = has_compose(project);
    d.container = d.dockerfile || d.compose;
}

fn detect_markets(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    d.steam = opted.iter().any(|m| m == "steam")
        || project.join("steam_appid.txt").is_file()
        || env_key_prefix(project, "STEAM_");
    d.itch = opted.iter().any(|m| m == "itch" || m == "itch.io")
        || project.join("itch.toml").is_file()
        || project.join(".itch").is_dir();
    d.epic = opted.iter().any(|m| m == "epic" || m == "egs");
}

fn detect_launch_baseline(project: &Path, d: &mut Detected) {
    d.license = root_has_any(
        project,
        &[
            "license",
            "license.md",
            "license.txt",
            "copying",
            "license-mit",
            "license-apache",
            "license-mit.md",
            "license-apache.md",
        ],
    );
    d.trust_md = root_has_any(project, &["trust.md"]);
    d.security_md = root_has_any(project, &["security.md"]);
    d.changelog = root_has_any(
        project,
        &["changelog.md", "changelog", "changes.md", "history.md"],
    );
}

fn detect_package_registries(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    let npm_opt = opted.iter().any(|m| m == "npm");
    let crates_opt = opted
        .iter()
        .any(|m| m == "crates" || m == "crates.io" || m == "cargo");

    d.npm_publish = npm_opt || npm_looks_publishable(project);
    d.crates_publish = crates_opt || crates_looks_publishable(project);
}

fn npm_looks_publishable(project: &Path) -> bool {
    let path = project.join("package.json");
    let Ok(raw) = fs::read_to_string(&path) else {
        return false;
    };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    let Some(obj) = v.as_object() else {
        return false;
    };
    if obj.get("private").and_then(|p| p.as_bool()) == Some(true) {
        return false;
    }
    if obj.get("private").and_then(|p| p.as_bool()) == Some(false) {
        return true;
    }
    if obj.get("publishConfig").map(|p| p.is_object()).unwrap_or(false) {
        return true;
    }
    obj.get("name")
        .and_then(|n| n.as_str())
        .map(|n| n.starts_with('@'))
        .unwrap_or(false)
}

fn crates_looks_publishable(project: &Path) -> bool {
    let path = project.join("Cargo.toml");
    let Ok(raw) = fs::read_to_string(&path) else {
        return false;
    };
    let mut in_package = false;
    let mut saw_package = false;
    let mut publish_blocked = false;
    for line in raw.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            if in_package {
                saw_package = true;
            }
            continue;
        }
        if !in_package {
            continue;
        }
        let lower = t.to_ascii_lowercase();
        if lower.starts_with("publish") {
            // publish = false | publish = [] | publish = ["restricted"]
            if lower.contains("false") || lower.contains('[') {
                publish_blocked = true;
            }
        }
    }
    saw_package && !publish_blocked
}

fn detect_pwa(project: &Path, d: &mut Detected) {
    let manifest_paths = [
        project.join("manifest.webmanifest"),
        project.join("public/manifest.webmanifest"),
        project.join("static/manifest.webmanifest"),
        project.join("public/manifest.json"),
        project.join("manifest.json"),
        project.join("apps/web/public/manifest.webmanifest"),
        project.join("apps/website/public/manifest.webmanifest"),
    ];
    for path in manifest_paths {
        if path.is_file() && manifest_looks_pwa(&path) {
            d.pwa = true;
            return;
        }
    }
    let pkg = project.join("package.json");
    if pkg.is_file() {
        if let Ok(raw) = fs::read_to_string(&pkg) {
            if raw.contains("vite-plugin-pwa") {
                d.pwa = true;
            }
        }
    }
}

fn manifest_looks_pwa(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let lower = raw.to_ascii_lowercase();
    (lower.contains("\"display\"")
        && (lower.contains("standalone")
            || lower.contains("fullscreen")
            || lower.contains("minimal-ui")))
        || (lower.contains("\"start_url\"") && lower.contains("\"name\""))
}

fn detect_huggingface(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    d.huggingface = opted.iter().any(|m| {
        matches!(
            m.as_str(),
            "hf" | "huggingface" | "huggingface_hub" | "model"
        )
    }) || project.join("modelcard.md").is_file()
        || project.join("MODEL_CARD.md").is_file()
        || project.join(".huggingface").is_dir();
}

fn detect_marketing_site(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    let opt_in = opted.iter().any(|m| {
        matches!(
            m.as_str(),
            "marketing" | "site" | "pages" | "hook" | "landing"
        )
    });

    let dir_hit = [
        "apps/website",
        "website",
        "apps/marketing",
        "marketing",
    ]
    .iter()
    .any(|rel| project.join(rel).is_dir());

    let pages_file = project.join("CNAME").is_file() || project.join(".nojekyll").is_file();
    let pages_workflow = marketing_pages_workflow(project);
    let preview = project
        .join("docs/launch/preview/index.html")
        .is_file();

    d.marketing_site = opt_in || dir_hit || pages_file || pages_workflow || preview;
    if !d.marketing_site {
        d.marketing_host.clear();
        return;
    }

    if pages_file || pages_workflow || preview || opted.iter().any(|m| m == "pages" || m == "hook")
    {
        d.marketing_host = "pages".into();
    } else if d.vercel
        || project.join("apps/website/vercel.json").is_file()
        || project.join("website/vercel.json").is_file()
    {
        d.marketing_host = "vercel".into();
    } else if d.netlify {
        d.marketing_host = "netlify".into();
    } else {
        d.marketing_host = "unknown".into();
    }
}

fn marketing_pages_workflow(project: &Path) -> bool {
    let dir = project.join(".github").join("workflows");
    let Ok(entries) = fs::read_dir(&dir) else {
        return false;
    };
    for ent in entries.flatten() {
        let name = ent.file_name().to_string_lossy().to_ascii_lowercase();
        if !(name.ends_with(".yml") || name.ends_with(".yaml")) {
            continue;
        }
        if name.contains("pages") || name.contains("gh-pages") {
            return true;
        }
    }
    false
}

/// Best-effort marketing host dashboard URL (no network).
pub fn marketing_deploy_url(project: &Path) -> String {
    let d = probe(project);
    match d.marketing_host.as_str() {
        "pages" => github_repo_web_url(project)
            .map(|base| format!("{base}/settings/pages"))
            .unwrap_or_else(|| "https://docs.github.com/pages".into()),
        "netlify" => "https://app.netlify.com/".into(),
        "vercel" => "https://vercel.com/dashboard".into(),
        _ => {
            if d.vercel {
                "https://vercel.com/dashboard".into()
            } else if d.netlify {
                "https://app.netlify.com/".into()
            } else {
                github_repo_web_url(project)
                    .map(|base| format!("{base}/settings/pages"))
                    .unwrap_or_else(|| "https://vercel.com/dashboard".into())
            }
        }
    }
}

fn detect_graduate_commerce(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    d.graduate_sign = opted.iter().any(|m| {
        matches!(
            m.as_str(),
            "graduate" | "authenticode" | "notarize" | "notarisation"
        )
    }) || project.join(".ship/graduate").is_file()
        || env_key_prefix(project, "SIGNET_OV_")
        || env_key_prefix(project, "SIGNET_AZURE_")
        || env_key_prefix(project, "SIGNET_NOTARY_")
        || env_key_prefix(project, "WIN_CERT_")
        || env_key_prefix(project, "APPLE_API_KEY")
        || env_key_prefix(project, "NOTARY_")
        || signet_toml_mentions_graduate(project);

    d.gumroad = opted.iter().any(|m| m == "gumroad")
        || env_key_prefix(project, "GUMROAD_");

    d.lemon = opted
        .iter()
        .any(|m| m == "lemon" || m == "lemonsqueezy" || m == "lemon_squeezy")
        || env_key_prefix(project, "LEMON_")
        || env_key_prefix(project, "LEMONSQUEEZY_");
}

#[derive(Debug, Deserialize)]
struct SuiteFile {
    #[serde(default)]
    canonical_hint: String,
    #[serde(default)]
    siblings: Vec<SuiteSibling>,
}

#[derive(Debug, Deserialize)]
struct SuiteSibling {
    #[serde(default)]
    label: String,
    #[serde(default)]
    path: String,
    #[serde(default)]
    env_keys: Vec<String>,
}

fn detect_suite_sync(project: &Path, d: &mut Detected) {
    let opted = read_markets_opt_in(project);
    let markets_suite = opted
        .iter()
        .any(|m| m == "suite" || m == "suite-sync" || m == "suite_sync");

    let path = ship_dir(project).join("suite.json");
    if path.is_file() {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(file) = serde_json::from_str::<SuiteFile>(&raw) {
                let mut bits = Vec::new();
                for sib in &file.siblings {
                    let keys: Vec<_> = sib
                        .env_keys
                        .iter()
                        .map(|k| k.trim())
                        .filter(|k| !k.is_empty())
                        .collect();
                    if sib.path.trim().is_empty() || keys.is_empty() {
                        continue;
                    }
                    let label = if sib.label.trim().is_empty() {
                        sib.path.trim().to_string()
                    } else {
                        sib.label.trim().to_string()
                    };
                    bits.push(format!("{label} ← {}", keys.join(", ")));
                }
                if !bits.is_empty() {
                    d.suite_sync = true;
                    d.suite_detail = bits.join(" · ");
                    d.suite_canonical = file.canonical_hint.trim().to_string();
                    return;
                }
            }
        }
    }

    if markets_suite {
        d.suite_sync = true;
        d.suite_detail = "markets suite — add .ship/suite.json with siblings".into();
    }
}

/// Best-effort URL for suite sync Open (canonical hint or marketing host).
pub fn suite_sync_url(project: &Path) -> String {
    let d = probe(project);
    let hint = d.suite_canonical.trim();
    if hint.starts_with("http://") || hint.starts_with("https://") {
        return hint.to_string();
    }
    marketing_deploy_url(project)
}

fn signet_toml_mentions_graduate(project: &Path) -> bool {
    let path = project.join("signet.toml");
    let Ok(raw) = fs::read_to_string(&path) else {
        return false;
    };
    let lower = raw.to_ascii_lowercase();
    lower.contains("graduate")
        || (lower.contains("ship.path") && lower.contains("official"))
        || (lower.contains("declared_tier") && lower.contains("graduate"))
}

/// Case-insensitive match of a root file's name (not recursive).
fn root_has_any(project: &Path, names_lower: &[&str]) -> bool {
    let Ok(entries) = fs::read_dir(project) else {
        return false;
    };
    for ent in entries.flatten() {
        let Ok(ft) = ent.file_type() else {
            continue;
        };
        if !ft.is_file() {
            continue;
        }
        let name = ent.file_name().to_string_lossy().to_ascii_lowercase();
        if names_lower.iter().any(|n| *n == name) {
            return true;
        }
    }
    false
}

/// Opt-in list from `.ship/markets` (one id per line) or `.ship/markets.json` (array of strings).
fn read_markets_opt_in(project: &Path) -> Vec<String> {
    let ship = ship_dir(project);
    let json_path = ship.join("markets.json");
    if json_path.is_file() {
        if let Ok(raw) = fs::read_to_string(&json_path) {
            if let Ok(arr) = serde_json::from_str::<Vec<String>>(&raw) {
                return arr
                    .into_iter()
                    .map(|s| s.trim().to_ascii_lowercase())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
    }
    let txt_path = ship.join("markets");
    if txt_path.is_file() {
        if let Ok(raw) = fs::read_to_string(&txt_path) {
            return raw
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .map(|l| l.to_ascii_lowercase())
                .collect();
        }
    }
    Vec::new()
}

fn has_dockerfile(project: &Path) -> bool {
    for name in ["Dockerfile", "Containerfile", "dockerfile"] {
        if project.join(name).is_file() {
            return true;
        }
    }
    // Nested: apps/*/Dockerfile or one-level */
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let name = ent.file_name().to_string_lossy().to_lowercase();
            if name == "node_modules" || name == ".git" || name == "target" || name == "dist" {
                continue;
            }
            for fname in ["Dockerfile", "Containerfile", "dockerfile"] {
                if p.join(fname).is_file() {
                    return true;
                }
            }
            if name == "apps" {
                if let Ok(apps) = fs::read_dir(&p) {
                    for app in apps.flatten() {
                        let ap = app.path();
                        if !ap.is_dir() {
                            continue;
                        }
                        for fname in ["Dockerfile", "Containerfile", "dockerfile"] {
                            if ap.join(fname).is_file() {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn has_compose(project: &Path) -> bool {
    for name in [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        if project.join(name).is_file() {
            return true;
        }
    }
    any_named(
        project,
        &[
            "docker-compose.yml",
            "docker-compose.yaml",
            "compose.yml",
            "compose.yaml",
        ],
    )
}

/// Docs / registry entry for container projects (no vendor HTTPS from bridge).
pub fn container_docs_url(project: &Path) -> &'static str {
    if github_repo_web_url(project).is_some() || project.join(".git").exists() {
        "https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry"
    } else {
        "https://docs.docker.com/get-started/docker-concepts/building-images/build-tag-and-publish-an-image/"
    }
}

/// Best-effort GitHub Actions URL from `origin` (no network).
pub fn github_actions_url(project: &Path) -> Option<String> {
    github_repo_web_url(project).map(|base| format!("{base}/actions"))
}

/// Best-effort “create release” URL from `origin` (no network).
pub fn github_releases_new_url(project: &Path) -> Option<String> {
    github_repo_web_url(project).map(|base| format!("{base}/releases/new"))
}

/// `https://github.com/owner/repo` from `git remote get-url origin`, if parseable.
pub fn github_repo_web_url(project: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(project)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    parse_github_remote(&raw)
}

fn parse_github_remote(raw: &str) -> Option<String> {
    let s = raw.trim().trim_end_matches('/').trim_end_matches(".git");
    if let Some(rest) = s.strip_prefix("https://github.com/") {
        let rest = rest.trim_end_matches('/');
        if rest.split('/').count() >= 2 {
            return Some(format!("https://github.com/{rest}"));
        }
    }
    if let Some(rest) = s.strip_prefix("http://github.com/") {
        let rest = rest.trim_end_matches('/');
        if rest.split('/').count() >= 2 {
            return Some(format!("https://github.com/{rest}"));
        }
    }
    if let Some(rest) = s.strip_prefix("git@github.com:") {
        let rest = rest.trim_end_matches('/');
        if rest.split('/').count() >= 2 {
            return Some(format!("https://github.com/{rest}"));
        }
    }
    if let Some(rest) = s.strip_prefix("ssh://git@github.com/") {
        let rest = rest.trim_end_matches('/');
        if rest.split('/').count() >= 2 {
            return Some(format!("https://github.com/{rest}"));
        }
    }
    None
}

fn wrangler_mentions(project: &Path, needles: &[&str]) -> bool {
    let names = ["wrangler.toml", "wrangler.json", "wrangler.jsonc"];
    let check = |dir: &Path| -> bool {
        for name in names {
            let Ok(raw) = fs::read_to_string(dir.join(name)) else {
                continue;
            };
            let lower = raw.to_ascii_lowercase();
            if needles
                .iter()
                .any(|n| lower.contains(&n.to_ascii_lowercase()))
            {
                return true;
            }
        }
        false
    };
    if check(project) {
        return true;
    }
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            let name = ent.file_name().to_string_lossy().to_lowercase();
            if name == "node_modules" || name == ".git" || name == "target" || name == "dist" {
                continue;
            }
            if check(&p) {
                return true;
            }
            if name == "apps" {
                if let Ok(apps) = fs::read_dir(&p) {
                    for app in apps.flatten() {
                        let ap = app.path();
                        if ap.is_dir() && check(&ap) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn env_files(project: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for name in [".env", ".env.local", ".dev.vars"] {
        let p = project.join(name);
        if p.is_file() {
            out.push(p);
        }
    }
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            for name in [".env", ".env.local", ".dev.vars"] {
                let f = p.join(name);
                if f.is_file() {
                    out.push(f);
                }
            }
        }
    }
    out
}

fn env_key_prefix(project: &Path, prefix: &str) -> bool {
    let pref = prefix.to_ascii_uppercase();
    for path in env_files(project) {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        for line in raw.lines() {
            let t = line.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            let key = t.split_once('=').map(|(k, _)| k.trim()).unwrap_or(t);
            if key.to_ascii_uppercase().starts_with(&pref) {
                return true;
            }
        }
    }
    false
}

fn env_value_contains(project: &Path, needle: &str) -> bool {
    let n = needle.to_ascii_lowercase();
    for path in env_files(project) {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        if raw.to_ascii_lowercase().contains(&n) {
            return true;
        }
    }
    false
}

fn package_mentions(project: &Path, needles: &[&str]) -> bool {
    let check = |path: &Path| -> bool {
        let Ok(raw) = fs::read_to_string(path) else {
            return false;
        };
        let lower = raw.to_ascii_lowercase();
        needles
            .iter()
            .any(|n| lower.contains(&n.to_ascii_lowercase()))
    };
    if check(&project.join("package.json")) {
        return true;
    }
    let apps = project.join("apps");
    if apps.is_dir() {
        if let Ok(entries) = fs::read_dir(&apps) {
            for ent in entries.flatten() {
                let p = ent.path().join("package.json");
                if p.is_file() && check(&p) {
                    return true;
                }
            }
        }
    }
    false
}

fn file_mentions_any(project: &Path, names: &[&str], needle: &str) -> bool {
    let n = needle.to_ascii_lowercase();
    for name in names {
        let Ok(raw) = fs::read_to_string(project.join(name)) else {
            continue;
        };
        if raw.to_ascii_lowercase().contains(&n) {
            return true;
        }
    }
    false
}

fn nested_file_mentions(project: &Path, names: &[&str], needle: &str) -> bool {
    let apps = project.join("apps");
    if !apps.is_dir() {
        return false;
    }
    let Ok(entries) = fs::read_dir(&apps) else {
        return false;
    };
    for ent in entries.flatten() {
        let p = ent.path();
        if !p.is_dir() {
            continue;
        }
        if file_mentions_any(&p, names, needle) {
            return true;
        }
    }
    false
}

fn detect_polar(project: &Path) -> bool {
    for name in [".dev.vars", ".env", ".env.local"] {
        if env_file_has_polar_key(&project.join(name)) {
            return true;
        }
    }
    if let Ok(entries) = fs::read_dir(project) {
        for ent in entries.flatten() {
            let p = ent.path();
            if !p.is_dir() {
                continue;
            }
            for name in [".dev.vars", ".env", "wrangler.toml"] {
                let f = p.join(name);
                if env_file_has_polar_key(&f) || file_mentions_polar(&f) {
                    return true;
                }
            }
            if ent.file_name().to_string_lossy() == "apps" {
                if let Ok(apps) = fs::read_dir(&p) {
                    for app in apps.flatten() {
                        let ap = app.path();
                        for name in [".dev.vars", ".env", "wrangler.toml"] {
                            let f = ap.join(name);
                            if env_file_has_polar_key(&f) || file_mentions_polar(&f) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn env_file_has_polar_key(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    raw.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && t.to_ascii_uppercase().starts_with("POLAR_")
    })
}

fn file_mentions_polar(path: &Path) -> bool {
    let Ok(raw) = fs::read_to_string(path) else {
        return false;
    };
    let lower = raw.to_ascii_lowercase();
    lower.contains("polar_") || lower.contains("polar.sh")
}

/// Build intent without writing (for dry-run plan when `.ship` is missing).
pub fn intent_for(project: &Path) -> Result<StudioIntent> {
    let project = fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let detected = probe(&project);
    let existing = read_studio(&project).ok().flatten();

    let mut sign_args = existing
        .as_ref()
        .map(|e| e.sign_args.clone())
        .unwrap_or_else(default_sign_args);
    if sign_args.is_empty() {
        sign_args = default_sign_args();
    }
    if existing.is_none() {
        if detected.tauri || detected.signet_toml {
            sign_args = vec!["build".into()];
        } else if !detected.signet_toml {
            sign_args = vec!["scan".into(), "--json".into()];
        }
    }

    let deploy_args = existing
        .as_ref()
        .map(|e| e.deploy_args.clone())
        .filter(|a| !a.is_empty())
        .unwrap_or_else(|| suggested_deploy_args(&detected));

    // Migrate stale interactive default when operator never customized.
    let deploy_args = if existing
        .as_ref()
        .map(|e| e.deploy_args == vec!["ship".to_string()])
        .unwrap_or(false)
        && (detected.wrangler || detected.vercel || detected.netlify)
    {
        suggested_deploy_args(&detected)
    } else {
        deploy_args
    };

    let mut notes = vec![
        "Put provider tokens in each tool's own local config — shipctl does not store secrets."
            .into(),
        format!(
            "Default sign_args: {:?} — edit .ship/studio.json to change.",
            sign_args
        ),
        "Run: shipctl flow --project . --dry-run".into(),
        "Then: shipctl flow --project .   (add --skip-deploy while offline)".into(),
    ];
    notes.extend(detected.hints.clone());
    let wants_signet = detected.tauri || detected.signet_toml;
    let existing_path = existing.as_ref().map(|e| e.sign_path.clone());

    Ok(StudioIntent {
        schema: "ship-studio/v0".into(),
        project: project.display().to_string(),
        workflow: vec![
            "doctor".into(),
            "configure".into(),
            "sign (signet)".into(),
            "deploy (orbit)".into(),
        ],
        adapters: Adapters::default(),
        offline_bridge: true,
        sign_args,
        deploy_args,
        detected,
        notes,
        active_scopes: crate::scopes::plan_for(&project).active,
        sign_path: if wants_signet {
            existing_path
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "self_then_official".into())
        } else {
            existing_path
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "official_listing_only".into())
        },
    })
}

pub fn configure(project: &Path) -> Result<StudioIntent> {
    let intent = intent_for(project)?;
    let dir = ship_dir(Path::new(&intent.project));
    fs::create_dir_all(&dir).context("mkdir .ship")?;
    let path = studio_path(Path::new(&intent.project));
    fs::write(&path, serde_json::to_string_pretty(&intent)?)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(intent)
}

pub fn write_last_run(project: &Path, run: &LastRun) -> Result<()> {
    let dir = ship_dir(project);
    fs::create_dir_all(&dir)?;
    let path = dir.join("last-run.json");
    fs::write(path, serde_json::to_string_pretty(run)?)?;
    Ok(())
}

pub fn read_last_run(project: &Path) -> Result<serde_json::Value> {
    let path = ship_dir(project).join("last-run.json");
    if !path.is_file() {
        return Ok(serde_json::json!({
            "finished": false,
            "ok": false,
            "message": "no last-run.json yet — run shipctl flow"
        }));
    }
    let raw = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&raw)?)
}

pub fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_github_remote_https_and_ssh() {
        assert_eq!(
            parse_github_remote("https://github.com/acme/app.git"),
            Some("https://github.com/acme/app".into())
        );
        assert_eq!(
            parse_github_remote("git@github.com:acme/app.git"),
            Some("https://github.com/acme/app".into())
        );
        assert_eq!(
            parse_github_remote("ssh://git@github.com/acme/app"),
            Some("https://github.com/acme/app".into())
        );
        assert_eq!(parse_github_remote("https://gitlab.com/acme/app.git"), None);
    }

    #[test]
    fn detects_release_workflow_filename() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-ci-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(dir.join(".github/workflows/ci.yml"), "name: ci\n").unwrap();
        fs::write(
            dir.join(".github/workflows/release.yml"),
            "name: release\non: push\n",
        )
        .unwrap();
        fs::write(
            dir.join(".github/workflows/Release-Publish.yaml"),
            "name: publish\n",
        )
        .unwrap();
        let d = probe(&dir);
        assert!(d.ci_release);
        assert!(d.release_workflows.iter().any(|n| n == "release.yml"));
        assert!(d
            .release_workflows
            .iter()
            .any(|n| n == "Release-Publish.yaml"));
        assert!(!d.release_workflows.iter().any(|n| n == "ci.yml"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_deploy_workflow_filename() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-deploy-wf-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".github/workflows")).unwrap();
        fs::write(
            dir.join(".github/workflows/deploy.yml"),
            "name: Deploy\non: push\n",
        )
        .unwrap();
        let d = probe(&dir);
        assert!(d.ci_release);
        assert!(d.release_workflows.iter().any(|n| n == "deploy.yml"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_dockerfile() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-docker-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("Dockerfile"), "FROM scratch\n").unwrap();
        let d = probe(&dir);
        assert!(d.container);
        assert!(d.dockerfile);
        assert!(!d.compose);
    }

    #[test]
    fn detects_steam_appid_and_markets_file() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-markets-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join("steam_appid.txt"), "480\n").unwrap();
        fs::write(dir.join(".ship/markets"), "itch\nepic\n").unwrap();
        let d = probe(&dir);
        assert!(d.steam);
        assert!(d.itch);
        assert!(d.epic);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_launch_baseline_files() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-legal-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let empty = probe(&dir);
        assert!(!empty.license);
        assert!(!empty.trust_md);
        assert!(!empty.security_md);
        assert!(!empty.changelog);

        fs::write(dir.join("LICENSE"), "MIT\n").unwrap();
        fs::write(dir.join("TRUST.md"), "# trust\n").unwrap();
        fs::write(dir.join("SECURITY.md"), "# sec\n").unwrap();
        fs::write(dir.join("CHANGELOG.md"), "# changes\n").unwrap();
        let d = probe(&dir);
        assert!(d.license);
        assert!(d.trust_md);
        assert!(d.security_md);
        assert!(d.changelog);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_npm_and_crates_publishable() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-pkg-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(
            dir.join("package.json"),
            r#"{"name":"app","private":true}"#,
        )
        .unwrap();
        assert!(!probe(&dir).npm_publish);

        fs::write(
            dir.join("package.json"),
            r#"{"name":"@acme/lib","version":"1.0.0"}"#,
        )
        .unwrap();
        assert!(probe(&dir).npm_publish);

        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        assert!(probe(&dir).crates_publish);

        fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\npublish = false\n",
        )
        .unwrap();
        assert!(!probe(&dir).crates_publish);

        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/markets"), "npm\ncrates\n").unwrap();
        fs::write(dir.join("package.json"), r#"{"name":"x","private":true}"#).unwrap();
        fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = []\n").unwrap();
        let opted = probe(&dir);
        assert!(opted.npm_publish);
        assert!(opted.crates_publish);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_marketing_website_and_pages() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-mkt-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("apps/website")).unwrap();
        fs::write(dir.join("apps/website/index.html"), "<h1>hi</h1>\n").unwrap();
        let d = probe(&dir);
        assert!(d.marketing_site);
        assert!(!d.marketing_host.is_empty());

        let dir2 = std::env::temp_dir().join(format!(
            "shipctl-mkt2-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir2);
        fs::create_dir_all(dir2.join(".ship")).unwrap();
        fs::write(dir2.join(".ship/markets"), "marketing\n").unwrap();
        assert!(probe(&dir2).marketing_site);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&dir2);
    }

    #[test]
    fn detects_graduate_and_commerce_opt_in() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-grad-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(
            dir.join(".ship/markets"),
            "graduate\ngumroad\nlemon\n",
        )
        .unwrap();
        let d = probe(&dir);
        assert!(d.graduate_sign);
        assert!(d.gumroad);
        assert!(d.lemon);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_suite_json_siblings() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-suite-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(
            dir.join(".ship/suite.json"),
            r#"{
              "canonical_hint": "https://example.com",
              "siblings": [
                {"label": "strata", "path": "../strata", "env_keys": ["NEXT_PUBLIC_VELOCITY_URL"]}
              ]
            }"#,
        )
        .unwrap();
        let d = probe(&dir);
        assert!(d.suite_sync);
        assert!(d.suite_detail.contains("strata"));
        assert!(d.suite_detail.contains("NEXT_PUBLIC_VELOCITY_URL"));
        assert_eq!(d.suite_canonical, "https://example.com");
        assert_eq!(suite_sync_url(&dir), "https://example.com");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_pwa_manifest_and_vite_plugin() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-pwa-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("public")).unwrap();
        fs::write(
            dir.join("public/manifest.webmanifest"),
            r#"{"name":"demo","display":"standalone","start_url":"/"}"#,
        )
        .unwrap();
        let d = probe(&dir);
        assert!(d.pwa);

        fs::write(
            dir.join("package.json"),
            r#"{"devDependencies":{"vite-plugin-pwa":"^0.20.0"}}"#,
        )
        .unwrap();
        let d2 = probe(&dir);
        assert!(d2.pwa);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn detects_huggingface_markets_and_modelcard() {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-hf-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".ship/markets.json"), r#"["hf"]"#).unwrap();
        let d = probe(&dir);
        assert!(d.huggingface);

        let dir2 = std::env::temp_dir().join(format!(
            "shipctl-hf-card-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir2);
        fs::create_dir_all(&dir2).unwrap();
        fs::write(dir2.join("modelcard.md"), "# Model\n").unwrap();
        let d2 = probe(&dir2);
        assert!(d2.huggingface);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&dir2);
    }
}

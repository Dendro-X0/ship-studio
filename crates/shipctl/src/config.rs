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
    if existing.is_none() && !detected.signet_toml {
        sign_args = vec!["scan".into(), "--json".into()];
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

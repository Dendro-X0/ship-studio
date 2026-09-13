//! Shipping portal — provider entry catalog + wizard plan (no vendor HTTPS).

use crate::config::{self, Detected};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Cloudflare,
    Vercel,
    Netlify,
    Github,
    Polar,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Vercel => "vercel",
            Self::Netlify => "netlify",
            Self::Github => "github",
            Self::Polar => "polar",
        }
    }

    pub fn label(self) -> &'static str {
        catalog_entry(self).label
    }

    pub fn all() -> [ProviderId; 5] {
        [
            ProviderId::Cloudflare,
            ProviderId::Vercel,
            ProviderId::Netlify,
            ProviderId::Github,
            ProviderId::Polar,
        ]
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cloudflare" | "cf" | "wrangler" => Ok(Self::Cloudflare),
            "vercel" => Ok(Self::Vercel),
            "netlify" => Ok(Self::Netlify),
            "github" | "gh" => Ok(Self::Github),
            "polar" => Ok(Self::Polar),
            other => {
                bail!("unknown provider '{other}' (cloudflare|vercel|netlify|github|polar)")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalStep {
    pub id: String,
    pub provider: String,
    pub kind: String,
    pub title: String,
    pub human: bool,
    pub entry_url: Option<String>,
    pub cli: Option<Vec<String>>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalPlan {
    pub schema: String,
    pub project: String,
    pub providers: Vec<String>,
    pub steps: Vec<PortalStep>,
    pub notes: Vec<String>,
}

pub fn catalog_entry(id: ProviderId) -> &'static ProviderCatalog {
    match id {
        ProviderId::Cloudflare => &CLOUDFLARE,
        ProviderId::Vercel => &VERCEL,
        ProviderId::Netlify => &NETLIFY,
        ProviderId::Github => &GITHUB,
        ProviderId::Polar => &POLAR,
    }
}

pub struct ProviderCatalog {
    pub label: &'static str,
    pub oauth_cli: &'static [&'static str],
    pub orbit_login: Option<&'static [&'static str]>,
    pub token_url: &'static str,
    /// Deep link / same page where Create Token is clicked (may equal token_url).
    pub create_url: &'static str,
    pub docs_url: &'static str,
    pub oauth_hint: &'static str,
    pub env_hint: &'static str,
    /// Platform shows secret only once at create/roll time.
    pub secret_shown_once: bool,
    pub once_hint: &'static str,
}

static CLOUDFLARE: ProviderCatalog = ProviderCatalog {
    label: "Cloudflare",
    oauth_cli: &["wrangler", "login"],
    orbit_login: Some(&["login", "cloudflare"]),
    token_url: "https://dash.cloudflare.com/profile/api-tokens",
    create_url: "https://dash.cloudflare.com/profile/api-tokens",
    docs_url: "https://developers.cloudflare.com/fundamentals/api/get-started/create-token/",
    oauth_hint: "Prefer Wrangler OAuth — no long-lived token value to store. Browser opens Cloudflare; authorize Wrangler.",
    env_hint: "Worker secrets: wrangler secret put <NAME> (or Orbit secrets wizard).",
    secret_shown_once: true,
    once_hint: "Cloudflare API Tokens show the secret ONLY once at Create/Roll. If you already created a token and lost the value: open ⋯ on that row → Roll (new value shown once), or Create Token again and copy immediately. Existing tokens cannot be Viewed. Prefer wrangler login instead of API tokens when possible.",
};

static VERCEL: ProviderCatalog = ProviderCatalog {
    label: "Vercel",
    oauth_cli: &["vercel", "login"],
    orbit_login: Some(&["login", "vercel"]),
    token_url: "https://vercel.com/account/settings/tokens",
    create_url: "https://vercel.com/account/settings/tokens",
    docs_url: "https://vercel.com/docs/rest-api#creating-an-access-token",
    oauth_hint: "Prefer Vercel CLI login. Browser opens Vercel; complete sign-in, then return here.",
    env_hint: "Project env: vercel env add <NAME> (or dashboard → Settings → Environment Variables).",
    secret_shown_once: true,
    once_hint: "Vercel access tokens are shown once at creation. If lost, create a new token and revoke the old one.",
};

static NETLIFY: ProviderCatalog = ProviderCatalog {
    label: "Netlify",
    oauth_cli: &["netlify", "login"],
    orbit_login: Some(&["login", "netlify"]),
    token_url: "https://app.netlify.com/user/applications#personal-access-tokens",
    create_url: "https://app.netlify.com/user/applications#personal-access-tokens",
    docs_url: "https://docs.netlify.com/cli/get-started/#authentication",
    oauth_hint: "Prefer Netlify CLI login. Browser opens Netlify; authorize CLI, then return here.",
    env_hint: "Site env: netlify env:set <NAME> <value> (or Site configuration → Environment variables).",
    secret_shown_once: true,
    once_hint: "Netlify personal access tokens are shown once. If lost, generate a new token.",
};

static GITHUB: ProviderCatalog = ProviderCatalog {
    label: "GitHub",
    oauth_cli: &["gh", "auth", "login"],
    orbit_login: None,
    token_url: "https://github.com/settings/tokens",
    create_url: "https://github.com/settings/tokens/new",
    docs_url: "https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token",
    oauth_hint: "Prefer gh auth login. Or create a PAT — value shown once; copy immediately.",
    env_hint: "For CI/API: create a PAT at the tokens page, then set GITHUB_TOKEN in the deploy target (never commit it).",
    secret_shown_once: true,
    once_hint: "GitHub PATs are shown once at creation. If you lost the value: generate a new token (classic or fine-grained) and update GITHUB_TOKEN everywhere it was used. There is no View for an old PAT.",
};

static POLAR: ProviderCatalog = ProviderCatalog {
    label: "Polar",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://polar.sh/dashboard",
    create_url: "https://polar.sh/dashboard",
    docs_url: "https://docs.polar.sh/",
    oauth_hint: "Open the Polar dashboard — copy checkout URL and webhook signing secret from product/webhook settings.",
    env_hint: "Put POLAR_CHECKOUT_URL and POLAR_WEBHOOK_SECRET on the deploy target (e.g. wrangler secret put).",
    secret_shown_once: false,
    once_hint: "Checkout URL is usually visible again in the product settings. Webhook secrets may need regeneration if lost — check Polar webhook settings.",
};

/// Where to copy the *value* for a named secret (not where to put it).
pub fn source_url_for_secret_name(name: &str) -> &'static str {
    let upper = name.to_ascii_uppercase();
    if upper == "GITHUB_TOKEN" || upper.starts_with("GH_") {
        return GITHUB.create_url;
    }
    if upper.starts_with("POLAR_") {
        return POLAR.token_url;
    }
    if upper.contains("CLOUDFLARE") || upper == "CF_API_TOKEN" || upper == "CF_API_KEY" {
        return CLOUDFLARE.create_url;
    }
    if upper.starts_with("VERCEL_") {
        return VERCEL.create_url;
    }
    if upper.starts_with("NETLIFY_") {
        return NETLIFY.create_url;
    }
    CLOUDFLARE.token_url
}

pub fn once_hint_for_secret_name(name: &str) -> &'static str {
    let upper = name.to_ascii_uppercase();
    if upper == "GITHUB_TOKEN" || upper.starts_with("GH_") {
        return GITHUB.once_hint;
    }
    if upper.starts_with("POLAR_") {
        return POLAR.once_hint;
    }
    if upper.contains("CLOUDFLARE") || upper == "CF_API_TOKEN" {
        return CLOUDFLARE.once_hint;
    }
    "Copy the value when the provider shows it — many platforms never display it again."
}

pub fn detected_providers(detected: &Detected) -> Vec<ProviderId> {
    let mut out = Vec::new();
    if detected.wrangler {
        out.push(ProviderId::Cloudflare);
    }
    if detected.vercel {
        out.push(ProviderId::Vercel);
    }
    if detected.netlify {
        out.push(ProviderId::Netlify);
    }
    if detected.github {
        out.push(ProviderId::Github);
    }
    if detected.polar {
        out.push(ProviderId::Polar);
    }
    out
}

pub fn plan_for(project: &Path, filter: Option<ProviderId>) -> Result<PortalPlan> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let detected = config::probe(&project);
    let mut providers = detected_providers(&detected);
    if let Some(f) = filter {
        providers.retain(|p| *p == f);
        if providers.is_empty() {
            providers.push(f);
        }
    }
    if providers.is_empty() {
        providers = ProviderId::all().to_vec();
    }
    plan_for_providers(&project, &providers)
}

/// Build a portal plan for an explicit provider set (wizard / multi-select).
pub fn plan_for_providers(project: &Path, providers: &[ProviderId]) -> Result<PortalPlan> {
    let project = std::fs::canonicalize(project).unwrap_or_else(|_| project.to_path_buf());
    let providers: Vec<ProviderId> = if providers.is_empty() {
        ProviderId::all().to_vec()
    } else {
        providers.to_vec()
    };

    let mut steps = Vec::new();
    for id in &providers {
        let cat = catalog_entry(*id);
        let pid = id.as_str().to_string();

        if cat.oauth_cli.is_empty() {
            steps.push(PortalStep {
                id: format!("{pid}.dashboard"),
                provider: pid.clone(),
                kind: "dashboard".into(),
                title: format!("{} dashboard / marketplace", cat.label),
                human: true,
                entry_url: Some(cat.token_url.into()),
                cli: None,
                detail: cat.oauth_hint.into(),
            });
        } else {
            steps.push(PortalStep {
                id: format!("{pid}.oauth"),
                provider: pid.clone(),
                kind: "oauth".into(),
                title: format!("{} CLI OAuth", cat.label),
                human: true,
                entry_url: None,
                cli: Some(cat.oauth_cli.iter().map(|s| (*s).to_string()).collect()),
                detail: cat.oauth_hint.into(),
            });
        }

        steps.push(PortalStep {
            id: format!("{pid}.token"),
            provider: pid.clone(),
            kind: "token_page".into(),
            title: format!("{} create/copy credentials", cat.label),
            human: true,
            entry_url: Some(cat.create_url.into()),
            cli: None,
            detail: if cat.secret_shown_once {
                format!(
                    "{} Docs: {}",
                    cat.once_hint, cat.docs_url
                )
            } else {
                format!("{} Docs: {}", cat.once_hint, cat.docs_url)
            },
        });

        if cat.secret_shown_once {
            steps.push(PortalStep {
                id: format!("{pid}.token_recover"),
                provider: pid.clone(),
                kind: "token_recover".into(),
                title: format!("{} — lost value? Roll or create new", cat.label),
                human: true,
                entry_url: Some(cat.token_url.into()),
                cli: None,
                detail: cat.once_hint.into(),
            });
        }

        steps.push(PortalStep {
            id: format!("{pid}.env"),
            provider: pid.clone(),
            kind: "env".into(),
            title: format!("{} environment / secrets", cat.label),
            human: true,
            entry_url: Some(cat.token_url.into()),
            cli: None,
            detail: cat.env_hint.into(),
        });
    }

    let notes = vec![
        "Portal navigates you to entry points — OAuth and env values stay manual.".into(),
        "Many API tokens are shown ONLY once (Create/Roll). Prefer CLI OAuth when available.".into(),
        "shipctl does not store secrets or call vendor HTTPS APIs.".into(),
        "Use --open to launch pages; --login to start CLI OAuth.".into(),
        "After auth: shipctl human --put  or  shipctl flow.".into(),
    ];

    Ok(PortalPlan {
        schema: "ship-studio/portal/v1".into(),
        project: project.display().to_string(),
        providers: providers.iter().map(|p| p.as_str().to_string()).collect(),
        steps,
        notes,
    })
}

pub fn open_urls(plan: &PortalPlan) -> Result<Vec<String>> {
    let mut opened = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for step in &plan.steps {
        let Some(url) = step.entry_url.as_deref() else {
            continue;
        };
        if !seen.insert(url.to_string()) {
            continue;
        }
        open_url(url)?;
        opened.push(url.to_string());
    }
    Ok(opened)
}

pub fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("open {url}"))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("open {url}"))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("open {url}"))?;
    }
    Ok(())
}

/// Run interactive login CLIs for providers in the plan.
pub fn run_logins(project: &Path, plan: &PortalPlan) -> Result<Vec<serde_json::Value>> {
    let mut results = Vec::new();
    let mut done = std::collections::HashSet::new();
    for step in &plan.steps {
        if step.kind != "oauth" {
            continue;
        }
        if !done.insert(step.provider.clone()) {
            continue;
        }
        let id = ProviderId::parse(&step.provider)?;
        let cat = catalog_entry(id);
        if cat.oauth_cli.is_empty() {
            results.push(serde_json::json!({
                "provider": step.provider,
                "ok": true,
                "skipped": true,
                "message": "no CLI login — open dashboard URL",
                "entry_url": cat.token_url,
            }));
            continue;
        }
        let (bin, args) = preferred_login(cat);
        let status = Command::new(&bin)
            .args(&args)
            .current_dir(project)
            .status()
            .with_context(|| format!("login via {bin}"))?;
        results.push(serde_json::json!({
            "provider": step.provider,
            "command": format!("{} {}", bin, args.join(" ")),
            "ok": status.success(),
            "exit_code": status.code().unwrap_or(-1),
        }));
    }
    Ok(results)
}

fn preferred_login(cat: &ProviderCatalog) -> (String, Vec<String>) {
    // Prefer Orbit login when available for deploy providers.
    if let Some(orbit_args) = cat.orbit_login {
        if which::which("orbit").is_ok() || which::which("orbit.exe").is_ok() {
            let bin = which::which("orbit")
                .or_else(|_| which::which("orbit.exe"))
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| "orbit".into());
            return (
                bin,
                orbit_args.iter().map(|s| (*s).to_string()).collect(),
            );
        }
    }
    let bin = cat.oauth_cli[0].to_string();
    let args = cat.oauth_cli[1..].iter().map(|s| (*s).to_string()).collect();
    (bin, args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "shipctl-portal-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn plan_detects_netlify_and_github() {
        let dir = tempfile_dir();
        fs::write(dir.join("netlify.toml"), "[build]\n").unwrap();
        fs::create_dir(dir.join(".git")).unwrap();
        let plan = plan_for(&dir, None).unwrap();
        assert!(plan.providers.iter().any(|p| p == "netlify"));
        assert!(plan.providers.iter().any(|p| p == "github"));
        assert!(plan.steps.iter().any(|s| s.id == "netlify.oauth"));
        assert!(plan
            .steps
            .iter()
            .any(|s| s.entry_url.as_deref() == Some(NETLIFY.token_url)));
    }

    #[test]
    fn filter_unknown_still_emits_catalog() {
        let dir = tempfile_dir();
        let plan = plan_for(&dir, Some(ProviderId::Vercel)).unwrap();
        assert_eq!(plan.providers, vec!["vercel".to_string()]);
        assert!(plan.steps.iter().all(|s| s.provider == "vercel"));
    }

    #[test]
    fn parse_aliases() {
        assert_eq!(ProviderId::parse("cf").unwrap(), ProviderId::Cloudflare);
        assert_eq!(ProviderId::parse("gh").unwrap(), ProviderId::Github);
    }

    #[test]
    fn plan_for_providers_multi() {
        let dir = tempfile_dir();
        let plan = plan_for_providers(
            &dir,
            &[ProviderId::Cloudflare, ProviderId::Github],
        )
        .unwrap();
        assert_eq!(plan.providers, vec!["cloudflare".to_string(), "github".to_string()]);
        assert!(plan.steps.iter().any(|s| s.id == "cloudflare.oauth"));
        assert!(plan.steps.iter().any(|s| s.id == "github.token"));
    }

    #[test]
    fn polar_dashboard_steps() {
        let dir = tempfile_dir();
        let plan = plan_for(&dir, Some(ProviderId::Polar)).unwrap();
        assert_eq!(plan.providers, vec!["polar".to_string()]);
        assert!(plan.steps.iter().any(|s| s.kind == "dashboard"));
        assert!(plan
            .steps
            .iter()
            .any(|s| s.entry_url.as_deref() == Some(POLAR.token_url)));
    }
}

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
    Neon,
    Supabase,
    D1,
    Turso,
    Container,
    Firebase,
    Appwrite,
    Convex,
    Fly,
    Railway,
    Render,
    DigitalOcean,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Vercel => "vercel",
            Self::Netlify => "netlify",
            Self::Github => "github",
            Self::Polar => "polar",
            Self::Neon => "neon",
            Self::Supabase => "supabase",
            Self::D1 => "d1",
            Self::Turso => "turso",
            Self::Container => "container",
            Self::Firebase => "firebase",
            Self::Appwrite => "appwrite",
            Self::Convex => "convex",
            Self::Fly => "fly",
            Self::Railway => "railway",
            Self::Render => "render",
            Self::DigitalOcean => "digitalocean",
        }
    }

    pub fn label(self) -> &'static str {
        catalog_entry(self).label
    }

    pub fn all() -> [ProviderId; 17] {
        [
            ProviderId::Cloudflare,
            ProviderId::Vercel,
            ProviderId::Netlify,
            ProviderId::Github,
            ProviderId::Polar,
            ProviderId::Neon,
            ProviderId::Supabase,
            ProviderId::D1,
            ProviderId::Turso,
            ProviderId::Container,
            ProviderId::Firebase,
            ProviderId::Appwrite,
            ProviderId::Convex,
            ProviderId::Fly,
            ProviderId::Railway,
            ProviderId::Render,
            ProviderId::DigitalOcean,
        ]
    }

    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "cloudflare" | "cf" | "wrangler" => Ok(Self::Cloudflare),
            "vercel" => Ok(Self::Vercel),
            "netlify" => Ok(Self::Netlify),
            "github" | "gh" => Ok(Self::Github),
            "polar" => Ok(Self::Polar),
            "neon" => Ok(Self::Neon),
            "supabase" => Ok(Self::Supabase),
            "d1" | "cloudflare-d1" => Ok(Self::D1),
            "turso" | "libsql" => Ok(Self::Turso),
            "container" | "docker" | "ghcr" => Ok(Self::Container),
            "firebase" => Ok(Self::Firebase),
            "appwrite" => Ok(Self::Appwrite),
            "convex" => Ok(Self::Convex),
            "fly" | "flyio" | "fly.io" => Ok(Self::Fly),
            "railway" => Ok(Self::Railway),
            "render" => Ok(Self::Render),
            "digitalocean" | "do" | "docean" => Ok(Self::DigitalOcean),
            other => {
                bail!(
                    "unknown provider '{other}' (cloudflare|vercel|netlify|github|polar|neon|supabase|d1|turso|container|firebase|appwrite|convex|fly|railway|render|digitalocean)"
                )
            }
        }
    }

    pub fn is_db(self) -> bool {
        matches!(
            self,
            Self::Neon | Self::Supabase | Self::D1 | Self::Turso
        )
    }

    pub fn is_baas(self) -> bool {
        matches!(self, Self::Firebase | Self::Appwrite | Self::Convex)
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
        ProviderId::Neon => &NEON,
        ProviderId::Supabase => &SUPABASE,
        ProviderId::D1 => &D1,
        ProviderId::Turso => &TURSO,
        ProviderId::Container => &CONTAINER,
        ProviderId::Firebase => &FIREBASE,
        ProviderId::Appwrite => &APPWRITE,
        ProviderId::Convex => &CONVEX,
        ProviderId::Fly => &FLY,
        ProviderId::Railway => &RAILWAY,
        ProviderId::Render => &RENDER,
        ProviderId::DigitalOcean => &DIGITALOCEAN,
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

static NEON: ProviderCatalog = ProviderCatalog {
    label: "Neon",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://console.neon.tech",
    create_url: "https://console.neon.tech",
    docs_url: "https://neon.tech/docs/get-started-with-neon/connect-neon",
    oauth_hint: "Open Neon Console — create a project and copy the connection string.",
    env_hint: "Put DATABASE_URL (or NEON_DATABASE_URL) on the deploy target via wrangler/vercel/netlify env.",
    secret_shown_once: false,
    once_hint: "Neon connection strings stay visible in the console. Rotate the password if leaked.",
};

static SUPABASE: ProviderCatalog = ProviderCatalog {
    label: "Supabase",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://supabase.com/dashboard",
    create_url: "https://supabase.com/dashboard",
    docs_url: "https://supabase.com/docs/guides/database/connecting-to-postgres",
    oauth_hint: "Open Supabase dashboard — create a project and copy URL + anon/service keys.",
    env_hint: "Put SUPABASE_URL / SUPABASE_ANON_KEY / DATABASE_URL on the deploy target.",
    secret_shown_once: true,
    once_hint: "Service role keys are sensitive — copy once from Project Settings → API. Rotate if lost.",
};

static D1: ProviderCatalog = ProviderCatalog {
    label: "Cloudflare D1",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://dash.cloudflare.com",
    create_url: "https://dash.cloudflare.com/?to=/:account/workers/d1",
    docs_url: "https://developers.cloudflare.com/d1/get-started/",
    oauth_hint: "Open Cloudflare D1 — create a database and bind it in wrangler.toml ([[d1_databases]]).",
    env_hint: "D1 uses Wrangler bindings; optional connection secrets go via wrangler secret put on the Worker.",
    secret_shown_once: false,
    once_hint: "D1 is usually binding-based. Account API tokens (if used) show once at create — prefer wrangler login.",
};

static TURSO: ProviderCatalog = ProviderCatalog {
    label: "Turso",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://turso.tech/app",
    create_url: "https://turso.tech/app",
    docs_url: "https://docs.turso.tech/sdk/ts/quickstart",
    oauth_hint: "Open Turso — create a database and copy URL + auth token.",
    env_hint: "Put TURSO_DATABASE_URL and TURSO_AUTH_TOKEN on the deploy target.",
    secret_shown_once: true,
    once_hint: "Turso auth tokens may be shown once — create a new token if lost.",
};

static CONTAINER: ProviderCatalog = ProviderCatalog {
    label: "Container registry",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://hub.docker.com/",
    create_url: "https://hub.docker.com/",
    docs_url: "https://docs.docker.com/get-started/docker-concepts/building-images/build-tag-and-publish-an-image/",
    oauth_hint: "Build/tag locally, then push to Docker Hub or GHCR. Ship Studio only opens docs — no remote build.",
    env_hint: "Registry credentials stay in docker login / gh auth — never in .ship/.",
    secret_shown_once: false,
    once_hint: "Use `docker login` or `gh auth token` for GHCR. Rotate registry tokens if leaked.",
};

static FIREBASE: ProviderCatalog = ProviderCatalog {
    label: "Firebase",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://console.firebase.google.com/",
    create_url: "https://console.firebase.google.com/",
    docs_url: "https://firebase.google.com/docs/auth",
    oauth_hint: "Open Firebase console — create project, enable Auth, download client config. Studio never calls Google APIs.",
    env_hint: "Put FIREBASE_* / google-services values on the app host — never in .ship/.",
    secret_shown_once: false,
    once_hint: "Rotate Firebase API keys / service accounts if leaked.",
};

static APPWRITE: ProviderCatalog = ProviderCatalog {
    label: "Appwrite",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://cloud.appwrite.io/",
    create_url: "https://cloud.appwrite.io/",
    docs_url: "https://appwrite.io/docs",
    oauth_hint: "Open Appwrite Cloud — create project and Auth. Studio only opens the dashboard.",
    env_hint: "Put APPWRITE_* endpoint/project/key on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Appwrite API keys may be shown once — create a new key if lost.",
};

static CONVEX: ProviderCatalog = ProviderCatalog {
    label: "Convex",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://dashboard.convex.dev/",
    create_url: "https://dashboard.convex.dev/",
    docs_url: "https://docs.convex.dev/",
    oauth_hint: "Open Convex dashboard — create deployment and Auth. Studio only opens the dashboard.",
    env_hint: "Put CONVEX_URL / CONVEX_DEPLOY_KEY on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Convex deploy keys may be shown once — rotate if leaked.",
};

static FLY: ProviderCatalog = ProviderCatalog {
    label: "Fly.io",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://fly.io/dashboard",
    create_url: "https://fly.io/dashboard",
    docs_url: "https://fly.io/docs/hands-on/launch-app/",
    oauth_hint: "Open Fly dashboard — create/launch the app with flyctl on your machine. Studio only opens the dashboard.",
    env_hint: "Put FLY_* secrets via `fly secrets set` or the dashboard — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Fly API tokens may be shown once — create a new token if lost.",
};

static RAILWAY: ProviderCatalog = ProviderCatalog {
    label: "Railway",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://railway.app/dashboard",
    create_url: "https://railway.app/dashboard",
    docs_url: "https://docs.railway.com/",
    oauth_hint: "Open Railway dashboard — create/deploy the service there or with railway CLI. Studio only opens the dashboard.",
    env_hint: "Put RAILWAY_* / project env on Railway — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Railway tokens may be shown once — rotate if leaked.",
};

static RENDER: ProviderCatalog = ProviderCatalog {
    label: "Render",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://dashboard.render.com/",
    create_url: "https://dashboard.render.com/",
    docs_url: "https://render.com/docs/deploy-an-app",
    oauth_hint: "Open Render dashboard — create/deploy the service there. Studio only opens the dashboard.",
    env_hint: "Put RENDER_* / service env on Render — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Render API keys may be shown once — rotate if leaked.",
};

static DIGITALOCEAN: ProviderCatalog = ProviderCatalog {
    label: "DigitalOcean",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://cloud.digitalocean.com/apps",
    create_url: "https://cloud.digitalocean.com/apps",
    docs_url: "https://docs.digitalocean.com/products/app-platform/",
    oauth_hint: "Open DigitalOcean App Platform — create/deploy the app there or with doctl. Studio only opens the dashboard.",
    env_hint: "Put DIGITALOCEAN_* / DO_* tokens via doctl or the control panel — never in .ship/.",
    secret_shown_once: true,
    once_hint: "DigitalOcean API tokens may be shown once — rotate if leaked.",
};

/// Where to copy the *value* for a named secret (not where to put it).
/// Prefer [`entry_url_for_secret`] when the put provider is known.
pub fn source_url_for_secret_name(name: &str) -> &'static str {
    entry_url_for_secret(name, None).unwrap_or(GITHUB.create_url)
}

/// Open URL for a secret hint: known vendor value-source, else put-provider dashboard.
pub fn entry_url_for_secret(name: &str, put_provider: Option<ProviderId>) -> Option<&'static str> {
    let upper = name.to_ascii_uppercase();
    if upper == "GITHUB_TOKEN" || upper.starts_with("GH_") {
        return Some(GITHUB.create_url);
    }
    if upper.starts_with("POLAR_") {
        return Some(POLAR.token_url);
    }
    if upper.starts_with("NEON_") || upper.contains("NEON") {
        return Some(NEON.create_url);
    }
    if upper.starts_with("SUPABASE_") {
        return Some(SUPABASE.create_url);
    }
    if upper.starts_with("FIREBASE_") || upper.starts_with("NEXT_PUBLIC_FIREBASE_") {
        return Some(FIREBASE.create_url);
    }
    if upper.starts_with("APPWRITE_") {
        return Some(APPWRITE.create_url);
    }
    if upper.starts_with("CONVEX_") {
        return Some(CONVEX.create_url);
    }
    if upper.starts_with("FLY_") {
        return Some(FLY.create_url);
    }
    if upper.starts_with("RAILWAY_") {
        return Some(RAILWAY.create_url);
    }
    if upper.starts_with("RENDER_") {
        return Some(RENDER.create_url);
    }
    if upper.starts_with("DIGITALOCEAN_") || upper.starts_with("DO_API_") {
        return Some(DIGITALOCEAN.create_url);
    }
    if upper.starts_with("TURSO_") || upper.starts_with("LIBSQL_") {
        return Some(TURSO.create_url);
    }
    if upper == "DATABASE_URL" || upper.ends_with("_DATABASE_URL") {
        return Some(NEON.create_url);
    }
    if upper.contains("CLOUDFLARE") || upper == "CF_API_TOKEN" || upper == "CF_API_KEY" {
        return Some(CLOUDFLARE.create_url);
    }
    if upper.starts_with("VERCEL_") {
        return Some(VERCEL.create_url);
    }
    if upper.starts_with("NETLIFY_") {
        return Some(NETLIFY.create_url);
    }
    if upper == "ANTHROPIC_API_KEY" || upper.starts_with("ANTHROPIC_") {
        return Some("https://console.anthropic.com/settings/keys");
    }
    if upper == "RESEND_API_KEY" || upper.starts_with("RESEND_") {
        return Some("https://resend.com/api-keys");
    }
    if upper.contains("WELLFOUND") {
        return Some("https://wellfound.com/");
    }
    if upper.starts_with("SIGNET_AZURE_")
        || upper.starts_with("WIN_CERT_")
        || upper == "SIGNET_OV_CERT"
    {
        return Some("https://learn.microsoft.com/en-us/azure/trusted-signing/");
    }
    if upper.starts_with("SIGNET_NOTARY_")
        || upper.starts_with("APPLE_API_")
        || upper.starts_with("NOTARY_")
    {
        return Some(
            "https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution",
        );
    }
    if upper.starts_with("GUMROAD_") {
        return Some("https://app.gumroad.com/");
    }
    if upper.starts_with("LEMON_") || upper.starts_with("LEMONSQUEEZY_") {
        return Some("https://app.lemonsqueezy.com/");
    }
    if upper.starts_with("STRIPE_") {
        return Some("https://dashboard.stripe.com/");
    }
    if upper.starts_with("PADDLE_") {
        return Some("https://vendors.paddle.com/");
    }
    // Self-generated (CRON_SECRET, BETTER_AUTH_SECRET, …) or unknown → put destination.
    match put_provider {
        Some(ProviderId::Cloudflare) => Some(CLOUDFLARE.create_url),
        Some(ProviderId::Vercel) => Some(VERCEL.create_url),
        Some(ProviderId::Netlify) => Some(NETLIFY.create_url),
        Some(ProviderId::Github) => Some(GITHUB.create_url),
        Some(ProviderId::Polar) => Some(POLAR.token_url),
        Some(ProviderId::Neon) => Some(NEON.create_url),
        Some(ProviderId::Supabase) => Some(SUPABASE.create_url),
        Some(ProviderId::D1) => Some(CLOUDFLARE.create_url),
        Some(ProviderId::Turso) => Some(TURSO.create_url),
        Some(ProviderId::Container) => Some(CONTAINER.create_url),
        Some(ProviderId::Firebase) => Some(FIREBASE.create_url),
        Some(ProviderId::Appwrite) => Some(APPWRITE.create_url),
        Some(ProviderId::Convex) => Some(CONVEX.create_url),
        Some(ProviderId::Fly) => Some(FLY.create_url),
        Some(ProviderId::Railway) => Some(RAILWAY.create_url),
        Some(ProviderId::Render) => Some(RENDER.create_url),
        Some(ProviderId::DigitalOcean) => Some(DIGITALOCEAN.create_url),
        None => None,
    }
}

pub fn once_hint_for_secret_name(name: &str) -> &'static str {
    let upper = name.to_ascii_uppercase();
    if upper == "GITHUB_TOKEN" || upper.starts_with("GH_") {
        return GITHUB.once_hint;
    }
    if upper.starts_with("POLAR_") {
        return POLAR.once_hint;
    }
    if upper.starts_with("NEON_") || upper == "DATABASE_URL" {
        return NEON.once_hint;
    }
    if upper.starts_with("SUPABASE_") {
        return SUPABASE.once_hint;
    }
    if upper.starts_with("TURSO_") || upper.starts_with("LIBSQL_") {
        return TURSO.once_hint;
    }
    if upper.contains("CLOUDFLARE") || upper == "CF_API_TOKEN" {
        return CLOUDFLARE.once_hint;
    }
    if upper.starts_with("SIGNET_")
        || upper.starts_with("WIN_CERT_")
        || upper.starts_with("APPLE_API_")
        || upper.starts_with("NOTARY_")
    {
        return "Store only in CI / Signet env — never commit private keys or .ship/.";
    }
    if upper.starts_with("GUMROAD_") || upper.starts_with("LEMON") {
        return "Create on the commerce dashboard; paste checkout URL into marketing CTA only.";
    }
    if upper.starts_with("STRIPE_") || upper.starts_with("PADDLE_") {
        return "Create on the commerce dashboard; put keys on the deploy target — never in .ship/.";
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
    if detected.neon {
        out.push(ProviderId::Neon);
    }
    if detected.supabase {
        out.push(ProviderId::Supabase);
    }
    if detected.d1 {
        out.push(ProviderId::D1);
    }
    if detected.turso {
        out.push(ProviderId::Turso);
    }
    if detected.container {
        out.push(ProviderId::Container);
    }
    if detected.firebase {
        out.push(ProviderId::Firebase);
    }
    if detected.appwrite {
        out.push(ProviderId::Appwrite);
    }
    if detected.convex {
        out.push(ProviderId::Convex);
    }
    if detected.fly {
        out.push(ProviderId::Fly);
    }
    if detected.railway {
        out.push(ProviderId::Railway);
    }
    if detected.render {
        out.push(ProviderId::Render);
    }
    if detected.digitalocean {
        out.push(ProviderId::DigitalOcean);
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

    // Prefer GHCR docs when the project looks GitHub-backed.
    let container_docs = config::container_docs_url(&project);
    for step in &mut steps {
        if step.provider == "container" {
            step.entry_url = Some(container_docs.into());
            if step.kind == "dashboard" {
                step.detail = format!(
                    "{} Preferred docs: {container_docs}",
                    catalog_entry(ProviderId::Container).oauth_hint
                );
            }
        }
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

    #[test]
    fn d1_fixture_includes_create_url() {
        let dir = tempfile_dir();
        std::fs::write(
            dir.join("wrangler.toml"),
            "name = \"x\"\n[[d1_databases]]\nbinding = \"DB\"\ndatabase_name = \"x\"\ndatabase_id = \"…\"\n",
        )
        .unwrap();
        let detected = config::probe(&dir);
        assert!(detected.d1);
        let plan = plan_for(&dir, None).unwrap();
        assert!(plan.providers.iter().any(|p| p == "d1"));
        assert!(plan.steps.iter().any(|s| {
            s.provider == "d1"
                && s.entry_url
                    .as_deref()
                    .is_some_and(|u| u.contains("d1") || u.contains("cloudflare"))
        }));
    }

    #[test]
    fn container_fixture_in_portal() {
        let dir = tempfile_dir();
        std::fs::write(dir.join("Dockerfile"), "FROM alpine\n").unwrap();
        let detected = config::probe(&dir);
        assert!(detected.container);
        let plan = plan_for(&dir, None).unwrap();
        assert!(plan.providers.iter().any(|p| p == "container"));
        assert!(plan.steps.iter().any(|s| s.provider == "container"));
    }

    #[test]
    fn neon_empty_database_url_in_secrets() {
        let dir = tempfile_dir();
        std::fs::write(dir.join(".env"), "DATABASE_URL=\nNEON_API_KEY=\n").unwrap();
        let detected = config::probe(&dir);
        assert!(detected.neon);
        let secrets = crate::secrets::plan_for(&dir, Some(ProviderId::Neon)).unwrap();
        assert!(secrets.hints.iter().any(|h| h.name == "DATABASE_URL"));
        assert!(secrets.hints.iter().any(|h| {
            h.entry_url
                .as_deref()
                .is_some_and(|u| u.contains("neon.tech"))
        }));
    }
}

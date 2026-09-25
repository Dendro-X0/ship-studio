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
    Gumroad,
    Lemon,
    Stripe,
    Paddle,
    Heroku,
    Amplify,
    CloudRun,
    AzureStatic,
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
            Self::Gumroad => "gumroad",
            Self::Lemon => "lemon",
            Self::Stripe => "stripe",
            Self::Paddle => "paddle",
            Self::Heroku => "heroku",
            Self::Amplify => "amplify",
            Self::CloudRun => "cloudrun",
            Self::AzureStatic => "azurestatic",
        }
    }

    pub fn label(self) -> &'static str {
        catalog_entry(self).label
    }

    pub fn all() -> [ProviderId; 25] {
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
            ProviderId::Gumroad,
            ProviderId::Lemon,
            ProviderId::Stripe,
            ProviderId::Paddle,
            ProviderId::Heroku,
            ProviderId::Amplify,
            ProviderId::CloudRun,
            ProviderId::AzureStatic,
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
            "gumroad" => Ok(Self::Gumroad),
            "lemon" | "lemonsqueezy" | "lemon_squeezy" => Ok(Self::Lemon),
            "stripe" => Ok(Self::Stripe),
            "paddle" => Ok(Self::Paddle),
            "heroku" => Ok(Self::Heroku),
            "amplify" | "aws-amplify" | "awsamplify" => Ok(Self::Amplify),
            "cloudrun" | "cloud-run" | "gcp-run" | "google-cloud-run" => Ok(Self::CloudRun),
            "azurestatic" | "azure-static" | "swa" | "static-web-apps" | "azure-swa" => {
                Ok(Self::AzureStatic)
            }
            other => {
                bail!(
                    "unknown provider '{other}' (cloudflare|vercel|netlify|github|polar|neon|supabase|d1|turso|container|firebase|appwrite|convex|fly|railway|render|digitalocean|gumroad|lemon|stripe|paddle|heroku|amplify|cloudrun|azurestatic)"
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

    pub fn is_commerce(self) -> bool {
        matches!(
            self,
            Self::Polar | Self::Gumroad | Self::Lemon | Self::Stripe | Self::Paddle
        )
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
    /// Official tutorial / docs — Desktop shows a Docs button (Open stays on settings UI).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
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
        ProviderId::Gumroad => &GUMROAD,
        ProviderId::Lemon => &LEMON,
        ProviderId::Stripe => &STRIPE,
        ProviderId::Paddle => &PADDLE,
        ProviderId::Heroku => &HEROKU,
        ProviderId::Amplify => &AMPLIFY,
        ProviderId::CloudRun => &CLOUDRUN,
        ProviderId::AzureStatic => &AZURESTATIC,
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
    /// When false, skip token_page (no distinct credentials UI — e.g. Fly tokens via CLI).
    pub emit_token_page: bool,
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
    emit_token_page: true,
};

static VERCEL: ProviderCatalog = ProviderCatalog {
    label: "Vercel",
    oauth_cli: &["vercel", "login"],
    orbit_login: Some(&["login", "vercel"]),
    token_url: "https://vercel.com/account/settings/tokens",
    create_url: "https://vercel.com/account/settings/tokens",
    docs_url: "https://vercel.com/docs/cli",
    oauth_hint: "Prefer Vercel CLI login. Browser opens Vercel; complete sign-in, then return here.",
    env_hint: "Project env: vercel env add <NAME> (or dashboard → Settings → Environment Variables).",
    secret_shown_once: true,
    once_hint: "Vercel access tokens are shown once at creation. If lost, create a new token and revoke the old one.",
    emit_token_page: true,
};

static NETLIFY: ProviderCatalog = ProviderCatalog {
    label: "Netlify",
    oauth_cli: &["netlify", "login"],
    orbit_login: Some(&["login", "netlify"]),
    token_url: "https://app.netlify.com/user/applications#personal-access-tokens",
    create_url: "https://app.netlify.com/user/applications#personal-access-tokens",
    docs_url: "https://docs.netlify.com/api-and-cli-guides/cli-guides/get-started-with-cli/",
    oauth_hint: "Prefer Netlify CLI login. Browser opens Netlify; authorize CLI, then return here.",
    env_hint: "Site env: netlify env:set <NAME> <value> (or Site configuration → Environment variables).",
    secret_shown_once: true,
    once_hint: "Netlify personal access tokens are shown once. If lost, generate a new token.",
    emit_token_page: true,
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
    emit_token_page: true,
};

static POLAR: ProviderCatalog = ProviderCatalog {
    label: "Polar",
    oauth_cli: &[],
    orbit_login: None,
    // Overview entry — deep links need POLAR_ORGANIZATION_SLUG (see polar_entry_urls).
    token_url: "https://polar.sh/dashboard",
    create_url: "https://polar.sh/docs/integrate/oat",
    docs_url: "https://docs.polar.sh/",
    oauth_hint: "Open the Polar dashboard — create products and checkout links on Products.",
    env_hint: "Put POLAR_CHECKOUT_URL and POLAR_WEBHOOK_SECRET on the deploy target (e.g. wrangler secret put).",
    secret_shown_once: false,
    once_hint: "Organization Access Tokens: Settings → Developers (docs: integrate/oat). Webhook secrets: Settings → Webhooks — regenerate if lost.",
    emit_token_page: true,
};

/// Webhook setup docs when no org slug is available for a dashboard deep link.
const POLAR_WEBHOOK_DOCS: &str = "https://polar.sh/docs/integrate/webhooks/endpoints";

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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
};

static FLY: ProviderCatalog = ProviderCatalog {
    label: "Fly.io",
    oauth_cli: &["fly", "auth", "login"],
    orbit_login: None,
    token_url: "https://fly.io/dashboard",
    create_url: "https://fly.io/dashboard",
    docs_url: "https://fly.io/docs/hands-on/launch-app/",
    oauth_hint: "Prefer `fly auth login` — browser OAuth. Deploy tokens: `fly tokens create deploy` (not the short-lived auth token).",
    env_hint: "App secrets: `fly secrets set NAME=value` (values never readable again). Or set secrets in the Fly dashboard.",
    secret_shown_once: true,
    once_hint: "Prefer scoped tokens via `fly tokens create` — see Fly access-tokens docs. Do not reuse short-lived `fly auth token` for CI.",
    emit_token_page: false,
};

static RAILWAY: ProviderCatalog = ProviderCatalog {
    label: "Railway",
    oauth_cli: &["railway", "login"],
    orbit_login: None,
    token_url: "https://railway.com/account/tokens",
    create_url: "https://railway.com/account/tokens",
    docs_url: "https://docs.railway.com/cli/login",
    oauth_hint: "Prefer `railway login` (browser or --browserless). For CI: RAILWAY_API_TOKEN (account) or RAILWAY_TOKEN (project) — set only one.",
    env_hint: "Service Variables tab on Railway (or `railway variables`). Never put values in .ship/.",
    secret_shown_once: true,
    once_hint: "Account/workspace tokens: Account Settings → Tokens (RAILWAY_API_TOKEN). Project tokens: project settings (RAILWAY_TOKEN). Do not set both env vars.",
    emit_token_page: true,
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
    emit_token_page: true,
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
    emit_token_page: true,
};

static GUMROAD: ProviderCatalog = ProviderCatalog {
    label: "Gumroad",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://app.gumroad.com/",
    create_url: "https://app.gumroad.com/",
    docs_url: "https://gumroad.com/help",
    oauth_hint: "Open Gumroad — create/edit the product and copy the checkout URL. Studio never creates products.",
    env_hint: "Paste GUMROAD_CHECKOUT_URL into the marketing CTA; API tokens stay on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Gumroad access tokens may need regeneration if lost — check Settings.",
    emit_token_page: true,
};

static LEMON: ProviderCatalog = ProviderCatalog {
    label: "Lemon Squeezy",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://app.lemonsqueezy.com/",
    create_url: "https://app.lemonsqueezy.com/",
    docs_url: "https://docs.lemonsqueezy.com/",
    oauth_hint: "Open Lemon Squeezy — create the product/variant and copy checkout + webhook secrets. Studio never creates SKUs.",
    env_hint: "Put LEMON_* / LEMONSQUEEZY_* on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Lemon API keys / webhook secrets may be shown once — rotate if leaked.",
    emit_token_page: true,
};

static STRIPE: ProviderCatalog = ProviderCatalog {
    label: "Stripe",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://dashboard.stripe.com/",
    create_url: "https://dashboard.stripe.com/",
    docs_url: "https://docs.stripe.com/",
    oauth_hint: "Open Stripe Dashboard — create products/prices/Payment Links there. Studio never creates charges.",
    env_hint: "Put STRIPE_* keys on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Stripe secret keys are shown once at create — roll a new key if lost.",
    emit_token_page: true,
};

static PADDLE: ProviderCatalog = ProviderCatalog {
    label: "Paddle",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://vendors.paddle.com/",
    create_url: "https://vendors.paddle.com/",
    docs_url: "https://developer.paddle.com/",
    oauth_hint: "Open Paddle vendor dashboard — create products/prices there. Studio never creates transactions.",
    env_hint: "Put PADDLE_* keys on the deploy target — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Paddle API keys may be shown once — rotate if leaked.",
    emit_token_page: true,
};

static HEROKU: ProviderCatalog = ProviderCatalog {
    label: "Heroku",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://dashboard.heroku.com/apps",
    create_url: "https://dashboard.heroku.com/apps",
    docs_url: "https://devcenter.heroku.com/",
    oauth_hint: "Open Heroku dashboard — create/deploy the app with heroku CLI on your machine. Studio only opens the dashboard.",
    env_hint: "Put HEROKU_* / config vars via heroku config:set — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Heroku API keys may be shown once — rotate if leaked.",
    emit_token_page: true,
};

static AMPLIFY: ProviderCatalog = ProviderCatalog {
    label: "AWS Amplify",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://console.aws.amazon.com/amplify/home",
    create_url: "https://console.aws.amazon.com/amplify/home",
    docs_url: "https://docs.aws.amazon.com/amplify/",
    oauth_hint: "Open AWS Amplify console — create/deploy the app there. Studio only opens the console.",
    env_hint: "Put AMPLIFY_* / AWS credentials via Amplify console or CLI — never in .ship/.",
    secret_shown_once: true,
    once_hint: "AWS access keys may be shown once — rotate if leaked.",
    emit_token_page: true,
};

static CLOUDRUN: ProviderCatalog = ProviderCatalog {
    label: "Google Cloud Run",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://console.cloud.google.com/run",
    create_url: "https://console.cloud.google.com/run",
    docs_url: "https://cloud.google.com/run/docs",
    oauth_hint: "Open Cloud Run console — create/deploy the service with gcloud or the console. Studio only opens the page.",
    env_hint: "Put CLOUD_RUN_* / GCP credentials via gcloud or Secret Manager — never in .ship/.",
    secret_shown_once: true,
    once_hint: "GCP service account keys may be shown once — rotate if leaked.",
    emit_token_page: true,
};

static AZURESTATIC: ProviderCatalog = ProviderCatalog {
    label: "Azure Static Web Apps",
    oauth_cli: &[],
    orbit_login: None,
    token_url: "https://portal.azure.com/#view/HubsExtension/BrowseResource/resourceType/Microsoft.Web%2FstaticSites",
    create_url: "https://portal.azure.com/#view/HubsExtension/BrowseResource/resourceType/Microsoft.Web%2FstaticSites",
    docs_url: "https://learn.microsoft.com/azure/static-web-apps/",
    oauth_hint: "Open Azure Static Web Apps — create/deploy the app there or with SWA CLI. Studio only opens the portal.",
    env_hint: "Put AZURE_STATIC_* / deployment tokens via Azure portal or SWA CLI — never in .ship/.",
    secret_shown_once: true,
    once_hint: "Azure deployment tokens may be shown once — rotate if leaked.",
    emit_token_page: true,
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
    if upper.starts_with("POLAR_WEBHOOK") {
        return Some(POLAR_WEBHOOK_DOCS);
    }
    if upper.starts_with("POLAR_") {
        return Some(POLAR.create_url);
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
    if upper.starts_with("HEROKU_") {
        return Some(HEROKU.create_url);
    }
    if upper.starts_with("AMPLIFY_") {
        return Some(AMPLIFY.create_url);
    }
    if upper.starts_with("CLOUD_RUN_") || upper == "K_SERVICE" {
        return Some(CLOUDRUN.create_url);
    }
    if upper.starts_with("AZURE_STATIC_") || upper.starts_with("STATIC_WEB_APP_") {
        return Some(AZURESTATIC.create_url);
    }
    if upper.starts_with("GUMROAD_") {
        return Some(GUMROAD.create_url);
    }
    if upper.starts_with("LEMON_") || upper.starts_with("LEMONSQUEEZY_") {
        return Some(LEMON.create_url);
    }
    if upper.starts_with("STRIPE_") {
        return Some(STRIPE.create_url);
    }
    if upper.starts_with("PADDLE_") {
        return Some(PADDLE.create_url);
    }
    // Self-generated (CRON_SECRET, BETTER_AUTH_SECRET, …) or unknown → put destination.
    match put_provider {
        Some(ProviderId::Cloudflare) => Some(CLOUDFLARE.create_url),
        Some(ProviderId::Vercel) => Some(VERCEL.create_url),
        Some(ProviderId::Netlify) => Some(NETLIFY.create_url),
        Some(ProviderId::Github) => Some(GITHUB.create_url),
        Some(ProviderId::Polar) => Some(POLAR.create_url),
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
        Some(ProviderId::Gumroad) => Some(GUMROAD.create_url),
        Some(ProviderId::Lemon) => Some(LEMON.create_url),
        Some(ProviderId::Stripe) => Some(STRIPE.create_url),
        Some(ProviderId::Paddle) => Some(PADDLE.create_url),
        Some(ProviderId::Heroku) => Some(HEROKU.create_url),
        Some(ProviderId::Amplify) => Some(AMPLIFY.create_url),
        Some(ProviderId::CloudRun) => Some(CLOUDRUN.create_url),
        Some(ProviderId::AzureStatic) => Some(AZURESTATIC.create_url),
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

fn env_entry_url(id: ProviderId, cat: &ProviderCatalog) -> &'static str {
    match id {
        // Settings / console — Docs button carries the tutorial URL.
        ProviderId::Polar => POLAR_WEBHOOK_DOCS,
        ProviderId::Cloudflare => "https://dash.cloudflare.com/?to=/:account/workers-and-pages",
        ProviderId::Vercel => "https://vercel.com/dashboard",
        ProviderId::Netlify => "https://app.netlify.com",
        ProviderId::Github => "https://github.com/settings/secrets/actions",
        ProviderId::D1 => "https://dash.cloudflare.com/?to=/:account/workers/d1",
        ProviderId::Fly => "https://fly.io/dashboard",
        ProviderId::Railway => "https://railway.app/dashboard",
        _ => cat.token_url,
    }
}

/// Tutorial URL for the Docs button (kind-aware).
fn step_docs_url(id: ProviderId, kind: &str, cat: &ProviderCatalog) -> Option<&'static str> {
    match kind {
        "env" => Some(match id {
            ProviderId::Cloudflare => {
                "https://developers.cloudflare.com/workers/configuration/secrets/"
            }
            ProviderId::Vercel => "https://vercel.com/docs/projects/environment-variables",
            ProviderId::Netlify => {
                "https://docs.netlify.com/build/environment-variables/get-started"
            }
            ProviderId::Github => {
                "https://docs.github.com/en/actions/security-guides/using-secrets-in-github-actions"
            }
            ProviderId::Polar => POLAR_WEBHOOK_DOCS,
            ProviderId::D1 => "https://developers.cloudflare.com/d1/get-started/",
            ProviderId::Fly => "https://fly.io/docs/apps/secrets/",
            ProviderId::Railway => "https://docs.railway.com/guides/variables",
            _ => cat.docs_url,
        }),
        "oauth" => Some(match id {
            ProviderId::Vercel => "https://vercel.com/docs/cli/login",
            ProviderId::Netlify => {
                "https://docs.netlify.com/api-and-cli-guides/cli-guides/get-started-with-cli/"
            }
            ProviderId::Fly => "https://fly.io/docs/flyctl/auth-login/",
            ProviderId::Railway => "https://docs.railway.com/cli/login",
            _ => cat.docs_url,
        }),
        "token_page" | "token_recover" => Some(match id {
            ProviderId::Fly => "https://fly.io/docs/security/tokens/",
            ProviderId::Railway => "https://docs.railway.com/cli/login",
            _ => cat.docs_url,
        }),
        _ => Some(cat.docs_url),
    }
}

fn portal_step(
    id: String,
    provider: String,
    kind: &str,
    title: String,
    entry_url: Option<String>,
    cli: Option<Vec<String>>,
    detail: String,
    docs: Option<&'static str>,
) -> PortalStep {
    PortalStep {
        id,
        provider,
        kind: kind.into(),
        title,
        human: true,
        entry_url,
        docs_url: docs.map(|u| u.to_string()),
        cli,
        detail,
    }
}

/// Optional org slug from local env — enables Polar dashboard deep links (no vendor HTTPS).
fn polar_organization_slug(project: &Path) -> Option<String> {
    config::env_key_value(project, "POLAR_ORGANIZATION_SLUG")
        .or_else(|| config::env_key_value(project, "POLAR_ORG_SLUG"))
        .map(|s| {
            s.trim()
                .trim_matches('/')
                .split('/')
                .next()
                .unwrap_or("")
                .to_string()
        })
        .filter(|s| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

fn apply_polar_deep_links(project: &Path, steps: &mut [PortalStep]) {
    let Some(slug) = polar_organization_slug(project) else {
        return;
    };
    let base = format!("https://polar.sh/dashboard/{slug}");
    for step in steps.iter_mut().filter(|s| s.provider == "polar") {
        let url = match step.kind.as_str() {
            "dashboard" => format!("{base}/products"),
            "token_page" => format!("{base}/settings"),
            "env" => format!("{base}/settings/webhooks"),
            _ => continue,
        };
        step.entry_url = Some(url);
        match step.kind.as_str() {
            "dashboard" => {
                step.detail =
                    "Products — create/update listing and checkout URL on polar.sh.".into();
                step.docs_url = Some(POLAR.docs_url.into());
            }
            "token_page" => {
                step.detail =
                    "Org settings — scroll to Developers for Organization Access Tokens.".into();
                step.docs_url = Some(POLAR.create_url.into());
            }
            "env" => {
                step.detail = "Webhooks — add endpoint and copy signing secret.".into();
                step.docs_url = Some(POLAR_WEBHOOK_DOCS.into());
            }
            _ => {}
        }
    }
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
    if detected.heroku {
        out.push(ProviderId::Heroku);
    }
    if detected.amplify {
        out.push(ProviderId::Amplify);
    }
    if detected.cloudrun {
        out.push(ProviderId::CloudRun);
    }
    if detected.azurestatic {
        out.push(ProviderId::AzureStatic);
    }
    if detected.gumroad {
        out.push(ProviderId::Gumroad);
    }
    if detected.lemon {
        out.push(ProviderId::Lemon);
    }
    if detected.stripe {
        out.push(ProviderId::Stripe);
    }
    if detected.paddle {
        out.push(ProviderId::Paddle);
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
            steps.push(portal_step(
                format!("{pid}.dashboard"),
                pid.clone(),
                "dashboard",
                format!("{} dashboard / marketplace", cat.label),
                Some(cat.token_url.into()),
                None,
                cat.oauth_hint.into(),
                step_docs_url(*id, "dashboard", cat),
            ));
        } else {
            steps.push(portal_step(
                format!("{pid}.oauth"),
                pid.clone(),
                "oauth",
                format!("{} CLI OAuth", cat.label),
                None,
                Some(cat.oauth_cli.iter().map(|s| (*s).to_string()).collect()),
                cat.oauth_hint.into(),
                step_docs_url(*id, "oauth", cat),
            ));
        }

        if cat.emit_token_page {
            steps.push(portal_step(
                format!("{pid}.token"),
                pid.clone(),
                "token_page",
                format!("{} create/copy credentials", cat.label),
                Some(cat.create_url.into()),
                None,
                cat.once_hint.into(),
                step_docs_url(*id, "token_page", cat),
            ));

            // Skip recover when it would open the same URL as create/copy (Cloudflare, Vercel, …).
            if cat.secret_shown_once && cat.create_url != cat.token_url {
                steps.push(portal_step(
                    format!("{pid}.token_recover"),
                    pid.clone(),
                    "token_recover",
                    format!("{} — lost value? Roll or create new", cat.label),
                    Some(cat.token_url.into()),
                    None,
                    cat.once_hint.into(),
                    step_docs_url(*id, "token_recover", cat),
                ));
            }
        }

        steps.push(portal_step(
            format!("{pid}.env"),
            pid.clone(),
            "env",
            format!("{} environment / secrets", cat.label),
            Some(env_entry_url(*id, cat).into()),
            None,
            cat.env_hint.into(),
            step_docs_url(*id, "env", cat),
        ));
    }

    apply_polar_deep_links(&project, &mut steps);

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
    fn cloudflare_steps_have_distinct_open_targets() {
        let dir = tempfile_dir();
        let plan = plan_for(&dir, Some(ProviderId::Cloudflare)).unwrap();
        let token = plan
            .steps
            .iter()
            .find(|s| s.kind == "token_page")
            .expect("token_page");
        let env = plan.steps.iter().find(|s| s.kind == "env").expect("env");
        // Same create/token URL → no duplicate recover row.
        assert!(
            !plan.steps.iter().any(|s| s.kind == "token_recover"),
            "recover must not duplicate create URL"
        );
        assert_eq!(
            token.entry_url.as_deref(),
            Some(CLOUDFLARE.create_url)
        );
        assert_eq!(
            env.entry_url.as_deref(),
            Some("https://dash.cloudflare.com/?to=/:account/workers-and-pages")
        );
        assert_ne!(token.entry_url, env.entry_url);
        assert_eq!(
            env.docs_url.as_deref(),
            Some("https://developers.cloudflare.com/workers/configuration/secrets/")
        );
        assert_eq!(token.docs_url.as_deref(), Some(CLOUDFLARE.docs_url));
    }

    #[test]
    fn netlify_and_vercel_docs_urls_are_stable() {
        assert!(NETLIFY.docs_url.contains("api-and-cli-guides"));
        assert_eq!(VERCEL.docs_url, "https://vercel.com/docs/cli");
        let dir = tempfile_dir();
        let netlify = plan_for(&dir, Some(ProviderId::Netlify)).unwrap();
        let env = netlify.steps.iter().find(|s| s.kind == "env").expect("env");
        assert_eq!(
            env.docs_url.as_deref(),
            Some("https://docs.netlify.com/build/environment-variables/get-started")
        );
        let vercel = plan_for(&dir, Some(ProviderId::Vercel)).unwrap();
        let oauth = vercel
            .steps
            .iter()
            .find(|s| s.kind == "oauth")
            .expect("oauth");
        assert_eq!(
            oauth.docs_url.as_deref(),
            Some("https://vercel.com/docs/cli/login")
        );
    }

    #[test]
    fn fly_and_railway_have_oauth_and_distinct_env_docs() {
        let dir = tempfile_dir();
        let fly = plan_for(&dir, Some(ProviderId::Fly)).unwrap();
        assert!(fly.steps.iter().any(|s| s.kind == "oauth"));
        assert!(
            !fly.steps.iter().any(|s| s.kind == "token_page"),
            "Fly has no distinct token UI — omit token_page"
        );
        let fly_env = fly.steps.iter().find(|s| s.kind == "env").expect("env");
        assert_eq!(
            fly_env.docs_url.as_deref(),
            Some("https://fly.io/docs/apps/secrets/")
        );
        let fly_oauth = fly.steps.iter().find(|s| s.kind == "oauth").unwrap();
        assert_eq!(
            fly_oauth.cli.as_ref().map(|c| c.join(" ")).as_deref(),
            Some("fly auth login")
        );

        let railway = plan_for(&dir, Some(ProviderId::Railway)).unwrap();
        assert!(railway.steps.iter().any(|s| s.kind == "oauth"));
        let token = railway
            .steps
            .iter()
            .find(|s| s.kind == "token_page")
            .expect("token_page");
        let env = railway.steps.iter().find(|s| s.kind == "env").expect("env");
        assert_eq!(
            token.entry_url.as_deref(),
            Some("https://railway.com/account/tokens")
        );
        assert_eq!(
            env.entry_url.as_deref(),
            Some("https://railway.app/dashboard")
        );
        assert_ne!(token.entry_url, env.entry_url);
        assert_eq!(
            env.docs_url.as_deref(),
            Some("https://docs.railway.com/guides/variables")
        );
    }

    #[test]
    fn polar_dashboard_steps() {
        let dir = tempfile_dir();
        let plan = plan_for(&dir, Some(ProviderId::Polar)).unwrap();
        assert_eq!(plan.providers, vec!["polar".to_string()]);
        let dash = plan
            .steps
            .iter()
            .find(|s| s.kind == "dashboard")
            .expect("dashboard");
        let token = plan
            .steps
            .iter()
            .find(|s| s.kind == "token_page")
            .expect("token_page");
        let env = plan.steps.iter().find(|s| s.kind == "env").expect("env");
        assert_eq!(dash.entry_url.as_deref(), Some(POLAR.token_url));
        assert_eq!(token.entry_url.as_deref(), Some(POLAR.create_url));
        assert_eq!(env.entry_url.as_deref(), Some(POLAR_WEBHOOK_DOCS));
        assert_ne!(dash.entry_url, token.entry_url);
        assert_ne!(token.entry_url, env.entry_url);
        assert_ne!(dash.entry_url, env.entry_url);
    }

    #[test]
    fn polar_org_slug_deep_links() {
        let dir = tempfile_dir();
        fs::write(dir.join(".env"), "POLAR_ORGANIZATION_SLUG=dendro-x0\n").unwrap();
        let plan = plan_for(&dir, Some(ProviderId::Polar)).unwrap();
        let url = |kind: &str| {
            plan.steps
                .iter()
                .find(|s| s.kind == kind)
                .and_then(|s| s.entry_url.as_deref())
                .unwrap()
                .to_string()
        };
        assert_eq!(url("dashboard"), "https://polar.sh/dashboard/dendro-x0/products");
        assert_eq!(url("token_page"), "https://polar.sh/dashboard/dendro-x0/settings");
        assert_eq!(
            url("env"),
            "https://polar.sh/dashboard/dendro-x0/settings/webhooks"
        );
    }

    #[test]
    fn commerce_portal_detects_stripe_and_gumroad() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join(".ship")).unwrap();
        fs::write(dir.join(".env"), "STRIPE_SECRET_KEY=\n").unwrap();
        fs::write(
            dir.join(".ship/markets.json"),
            r#"["gumroad","paddle"]"#,
        )
        .unwrap();
        let plan = plan_for(&dir, None).unwrap();
        assert!(plan.providers.iter().any(|p| p == "stripe"));
        assert!(plan.providers.iter().any(|p| p == "gumroad"));
        assert!(plan.providers.iter().any(|p| p == "paddle"));
        assert!(plan.steps.iter().any(|s| {
            s.provider == "stripe" && s.entry_url.as_deref() == Some(STRIPE.token_url)
        }));
        assert!(plan.steps.iter().any(|s| {
            s.provider == "gumroad" && s.entry_url.as_deref() == Some(GUMROAD.token_url)
        }));
        assert_eq!(ProviderId::parse("lemonsqueezy").unwrap(), ProviderId::Lemon);
        assert!(ProviderId::Stripe.is_commerce());
        let err = crate::secrets::put_secret(&dir, ProviderId::Stripe, "STRIPE_SECRET_KEY");
        assert!(err.is_err());
        let msg = format!("{}", err.unwrap_err());
        assert!(msg.contains("Stripe") || msg.contains("commerce") || msg.contains("dashboard"));
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

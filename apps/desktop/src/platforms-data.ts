/** Deployment catalog: On this machine · Cloud hosts · Official signing (Sign). */

import type { ProviderWizard } from "./types";

/** Local runtime / compose — not a cloud hosting vendor. */
export const LOCAL_RUNTIME_GROUP = "On this machine";
/** Vendor PaaS / Pages — CLI deploy to their cloud. */
export const CLOUD_HOST_GROUP = "Cloud hosts";

export const PLATFORM_WIZARDS: ProviderWizard[] = [
  {
    id: "selfhost",
    group: LOCAL_RUNTIME_GROUP,
    title: "Self-host",
    blurb:
      "Local auto lane — Studio streams the run and checks it (Docker cousin; not a cloud host).",
    openUrl: "https://github.com/Dendro-X0/ship-studio",
    openLabel: "Deploy",
    needsPublic: false,
    steps: [
      "Self-host runs on this machine — no Dendro datacenter, no vendor OAuth for the cut.",
      "Deploy runs a **fast local check** (detect → artifact → GET health) and finishes — typically under a second on Harbor.",
      "Need the site in a browser? **Open live** starts serve and opens loopback; **Cancel serve** stops it. Cloud hosts: **Cancel on dashboard**.",
      "Live URL is loopback only (http://127.0.0.1:…) — never a *.pages.dev / Vercel / Netlify URL. **Start over** clears Results after a check.",
    ],
  },
  {
    id: "umami",
    group: LOCAL_RUNTIME_GROUP,
    title: "Umami",
    blurb:
      "Self-hosted analytics — Deploy runs docker compose + Admin heartbeat on this machine.",
    openUrl: "http://127.0.0.1:3000",
    openLabel: "Deploy",
    needsPublic: false,
    steps: [
      "Umami Get-live: **ensure secrets** → **compose up** → Admin URL (Orbit shortens the official Docker checklist).",
      "Deploy needs Docker on PATH. Studio writes strong `.env` secrets — values never shown in the UI.",
      "**Open Admin** opens the loopback dashboard. Change the default admin password, then Confirm on Publish.",
      "Not a cloud host and not Self-host static — tracker paste on your site stays a human step.",
    ],
  },
  {
    id: "flowsint",
    group: LOCAL_RUNTIME_GROUP,
    title: "Flowsint",
    blurb:
      "Self-hosted OSINT graph — Deploy runs compose.prod + App proof on this machine.",
    openUrl: "http://127.0.0.1:5173",
    openLabel: "Deploy",
    needsPublic: false,
    steps: [
      "Flowsint Get-live: **ensure multi-env secrets** → **compose.prod up** → App URL (Orbit shortens four manual .env copies).",
      "Deploy needs Docker on PATH. Studio writes strong AUTH_SECRET / vault / DB passwords — values never shown in the UI.",
      "**Open App** opens the loopback UI. Create an account at /register, then Confirm on Publish.",
      "Enricher API keys stay in Flowsint’s vault — not Integrations theater.",
    ],
  },
  {
    id: "medusa",
    group: LOCAL_RUNTIME_GROUP,
    title: "Medusa",
    blurb:
      "Self-hosted commerce API — Deploy runs compose deps + Admin health on this machine.",
    openUrl: "http://127.0.0.1:9000/app",
    openLabel: "Deploy",
    needsPublic: false,
    steps: [
      "Medusa Get-live: **ensure secrets** → **compose up** → Admin URL (Orbit sequences detect → Put → Deploy).",
      "Deploy needs Docker on PATH for Postgres/Redis. Soft-L3 Admin needs a real Medusa clone (fixture is deps-only).",
      "**Open Admin** opens the loopback dashboard. Create/sign in, then Confirm on Publish.",
      "Payment plugins stay vendor dashboards — not Integrations theater.",
    ],
  },
  {
    id: "cloudflare",
    group: CLOUD_HOST_GROUP,
    title: "Cloudflare",
    blurb:
      "Workers / Pages — Deploy streams wrangler on this machine; Open dashboard to see the project.",
    provider: "cloudflare",
    openUrl: "https://dash.cloudflare.com/?to=/:account/workers-and-pages",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider cloudflare",
    steps: [
      "Deploy streams orbityard hostdeploy (wrangler Pages or Workers) — watch Output for phases.",
      "Optional: **Import env…** from local .dev.vars / .env — confirm names, then push via wrangler (values never saved in Studio).",
      "Not logged in? Login CLI (`wrangler login`) or Sign in (web) — Studio never creates API tokens.",
      "After you delete on the vendor: **Clear evidence** syncs Studio (drops local last-run). Hosts still bill you — Studio complements their CLI.",
    ],
  },
  {
    id: "vercel",
    group: CLOUD_HOST_GROUP,
    title: "Vercel",
    blurb:
      "Vercel project — Deploy streams vercel CLI on this machine; Open dashboard to see the project.",
    provider: "vercel",
    openUrl: "https://vercel.com/dashboard",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider vercel",
    steps: [
      "Deploy streams orbityard hostdeploy (vercel --prod) — watch Output for phases.",
      "Optional: **Import env…** from local .env — confirm names, then `vercel env add` (values never saved in Studio).",
      "Not logged in? Login CLI (`vercel login`) or Sign in (web) — Studio never stores tokens.",
      "Copy the production URL → Publish Live check → Confirm (Studio does not auto-Confirm).",
    ],
  },
  {
    id: "netlify",
    group: CLOUD_HOST_GROUP,
    title: "Netlify",
    blurb:
      "Netlify site — Deploy streams netlify CLI on this machine; Open dashboard to see the site.",
    provider: "netlify",
    openUrl: "https://app.netlify.com/projects",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider netlify",
    steps: [
      "Deploy streams orbityard hostdeploy (netlify deploy --prod) — watch Output for phases.",
      "Optional: **Import env…** from local .env — confirm names, then `netlify env:import` (ephemeral; values never saved in Studio).",
      "Not logged in? Login CLI (`netlify login`) or Sign in (web) — Studio never stores tokens.",
      "Copy the site URL → Publish Live check → Confirm (Studio does not auto-Confirm).",
    ],
  },
  {
    id: "github-pages",
    group: CLOUD_HOST_GROUP,
    title: "GitHub Pages",
    blurb: "Static docs / marketing — Settings → Pages (not PAT portal).",
    openUrl: "https://github.com/settings/pages",
    openLabel: "Open GitHub",
    docsUrl:
      "https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site",
    needsPublic: true,
    steps: [
      "Open GitHub Pages settings (or your repo → Settings → Pages when a remote exists).",
      "Publish the site yourself — Studio does not push Pages.",
      "Need `gh` login / PAT for CI? Portal view → GitHub — not this card.",
      "Learn more only if stuck on the Pages UI. Copy the Pages URL → Confirm.",
    ],
  },
  {
    id: "fly",
    group: CLOUD_HOST_GROUP,
    title: "Fly.io",
    blurb: "Deploy streams fly deploy after fly launch — Login CLI + Import env on this machine.",
    provider: "fly",
    openUrl: "https://fly.io/dashboard",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider fly",
    steps: [
      "First time: run `fly launch` yourself (creates fly.toml) — Studio never silent-launches.",
      "Deploy streams `fly deploy --yes` via hostdeploy — watch Output.",
      "Optional: **Import env…** — `fly secrets set NAME=-` (stdin; values never saved in Studio).",
      "Not logged in? Login CLI (`fly auth login`). Copy the app URL → Confirm.",
    ],
  },
  {
    id: "railway",
    group: CLOUD_HOST_GROUP,
    title: "Railway",
    blurb: "Deploy streams railway up after link — Login CLI + Import env on this machine.",
    provider: "railway",
    openUrl: "https://railway.com/dashboard",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider railway",
    steps: [
      "First time: run `railway link` or `railway init` yourself — Studio never silent-inits.",
      "Deploy streams `railway up -y` via hostdeploy — watch Output.",
      "Optional: **Import env…** — `railway variable set NAME --stdin` (values never saved in Studio).",
      "Not logged in? Login CLI (`railway login`). Copy the public URL → Confirm.",
    ],
  },
  {
    id: "orbit",
    group: CLOUD_HOST_GROUP,
    title: "Orbit",
    blurb:
      "Local Orbit CLI → your Cloudflare / Vercel / Netlify (only when this repo is Orbit-configured).",
    openUrl: "https://github.com/Dendro-X0/orbit#readme",
    openLabel: "Orbit docs",
    needsPublic: true,
    deployArgs: "status",
    steps: [
      "Orbit is a local CLI, not a datacenter product — you still need a host account.",
      "Confirm Orbit is on PATH and this repo has orbit.toml / .orbit.",
      "Run deploy from Ritual or Tools — Studio does not upload for you.",
      "Open the host dashboard for the target provider to manage live projects.",
    ],
  },
  {
    id: "apple-sign",
    group: "Official signing",
    title: "Apple",
    blurb: "Developer certificates, notarization, App Store Connect.",
    openUrl: "https://developer.apple.com/account/resources/certificates/list",
    needsPublic: false,
    steps: [
      "Open Apple Developer — certificates / profiles stay on their UI.",
      "Run Signet graduate / notarize locally when your layout needs it (Sign panel).",
      "App Store Connect submit is on Apple — then Confirm on Publish.",
    ],
  },
  {
    id: "microsoft-sign",
    group: "Official signing",
    title: "Microsoft",
    blurb: "Authenticode / MSIX / Partner Center.",
    openUrl: "https://partner.microsoft.com/dashboard/products",
    needsPublic: false,
    steps: [
      "Open Partner Center (or your cert portal) and finish signing there.",
      "Self-sign with Signet is enough for many local Windows cuts.",
      "Store submit stays on Microsoft — Confirm on Publish after.",
    ],
  },
  {
    id: "google-play",
    group: "Official signing",
    title: "Google Play",
    blurb: "Play Console listing and upload (when mobile is in scope).",
    openUrl: "https://play.google.com/console/u/0/developers",
    needsPublic: false,
    steps: [
      "Open Play Console when mobile is in Scopes.",
      "Upload / review on Google — Studio does not hold Play credentials.",
      "Confirm the listing / submit step on Publish when it is current.",
    ],
  },
  {
    id: "github-sign",
    group: "Official signing",
    title: "GitHub Release",
    blurb: "Create the repo / draft release — Studio opens the door only.",
    openUrl: "https://github.com/new",
    needsPublic: false,
    steps: [
      "Open GitHub → create the repo (or open Releases when the remote exists).",
      "Signet release / CI still run on your machine — never inside Studio.",
      "Confirm on Publish after the release cut is live.",
    ],
  },
];

/** Deployment panel section order — local runtime first (not a PaaS marketplace). */
export const PLATFORM_GROUPS = [LOCAL_RUNTIME_GROUP, CLOUD_HOST_GROUP] as const;

/** Official store / cert lanes — rendered as a grid on the Sign panel. */
export const SIGN_GROUPS = ["Official signing"] as const;

export function isDeployCatalogGroup(group: string): boolean {
  return group === LOCAL_RUNTIME_GROUP || group === CLOUD_HOST_GROUP;
}

/** Prefer the section that owns the selected card (local first when unset). */
export function deployPreferGroup(selectId?: string | null): string {
  if (!selectId) return LOCAL_RUNTIME_GROUP;
  const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectId);
  return wiz?.group === CLOUD_HOST_GROUP ? CLOUD_HOST_GROUP : LOCAL_RUNTIME_GROUP;
}

export function signingCatalogEntries() {
  return PLATFORM_WIZARDS.filter((w) => w.group === "Official signing");
}

/** Deployment cards (local + cloud); Orbit only when the bound project is Orbit-configured. */
export function hostingCatalogEntries(opts?: { orbitConfigured?: boolean }) {
  return PLATFORM_WIZARDS.filter((w) => {
    if (!isDeployCatalogGroup(w.group)) return false;
    if (w.id === "orbit") return Boolean(opts?.orbitConfigured);
    return true;
  });
}

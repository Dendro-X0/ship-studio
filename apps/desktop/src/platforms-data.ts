/** Hosting (Deployment) + Official signing (Sign) catalog entries. */

import type { ProviderWizard } from "./types";

export const PLATFORM_WIZARDS: ProviderWizard[] = [
  {
    id: "selfhost",
    group: "Hosting",
    title: "Self-host",
    blurb:
      "Local auto lane — Studio streams the run and checks it (Docker cousin; not a cloud host).",
    openUrl: "https://github.com/Dendro-X0/ship-studio",
    openLabel: "Deploy",
    needsPublic: false,
    steps: [
      "Self-host runs on this machine — no Dendro datacenter, no vendor OAuth for the cut.",
      "Deploy streams shipctl selfhost: detect → artifact → local GET health.",
      "Done when health returns 200 — writes .ship/last-run.json; no Confirm→Next.",
      "Optional long serve: Ritual/terminal `shipctl selfhost --serve`. SaaS cutover uses cards below.",
    ],
  },
  {
    id: "cloudflare",
    group: "Hosting",
    title: "Cloudflare",
    blurb: "Workers / Pages — Put secrets in Studio’s terminal; Open only for the dashboard.",
    provider: "cloudflare",
    openUrl: "https://dash.cloudflare.com",
    needsPublic: true,
    deployArgs: "deploy --provider cloudflare",
    steps: [
      "Put secrets here (terminal) — never paste values into Studio.",
      "Portal steps → Login CLI when you need wrangler/Orbit OAuth.",
      "Open dashboard only to create/copy values or pick the Worker/Pages project.",
      "Deploy on your machine; copy the live URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "vercel",
    group: "Hosting",
    title: "Vercel",
    blurb: "Vercel project — Put env in Studio’s terminal; Open for the dashboard.",
    provider: "vercel",
    openUrl: "https://vercel.com/dashboard",
    needsPublic: true,
    deployArgs: "deploy --provider vercel",
    steps: [
      "Put env vars here in the terminal — never paste into Studio.",
      "Portal steps → Login CLI when Publish asks for Vercel OAuth.",
      "Open dashboard to create/copy values or import the project.",
      "Deploy yourself; copy the production URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "netlify",
    group: "Hosting",
    title: "Netlify",
    blurb: "Netlify site — Put site env in Studio’s terminal.",
    provider: "netlify",
    openUrl: "https://app.netlify.com",
    needsPublic: true,
    deployArgs: "deploy --provider netlify",
    steps: [
      "Put site env here in the terminal — never paste into Studio.",
      "Portal steps → Login CLI when the plan asks for Netlify login.",
      "Open dashboard to create/copy values or create the site.",
      "Deploy yourself; copy the site URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "github-pages",
    group: "Hosting",
    title: "GitHub Pages",
    blurb: "Static docs / marketing — Settings → Pages (not PAT portal).",
    openUrl: "https://github.com",
    openLabel: "Open GitHub",
    docsUrl:
      "https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site",
    needsPublic: true,
    steps: [
      "Open your repo → Settings → Pages (branch or Actions source).",
      "Publish the site yourself — Studio does not push Pages.",
      "Need `gh` login / PAT for CI? Portal view → GitHub — not this card.",
      "Learn more only if stuck on the Pages UI. Copy the Pages URL → Confirm.",
    ],
  },
  {
    id: "fly",
    group: "Hosting",
    title: "Fly.io",
    blurb: "flyctl auth + secrets — Portal Login CLI, then dashboard.",
    provider: "fly",
    openUrl: "https://fly.io/dashboard",
    needsPublic: true,
    steps: [
      "Portal steps → Login CLI (`fly auth login`).",
      "Deploy with flyctl on your machine — Studio does not upload.",
      "Put app secrets with flyctl or the dashboard — never in Studio.",
      "Copy the app URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "railway",
    group: "Hosting",
    title: "Railway",
    blurb: "railway login + Variables — Portal Login CLI, then dashboard.",
    provider: "railway",
    openUrl: "https://railway.app/dashboard",
    needsPublic: true,
    steps: [
      "Portal steps → Login CLI (`railway login`).",
      "Put service variables on Railway — never in Studio.",
      "Account tokens only when CI needs them (not both TOKEN env vars).",
      "Copy the public URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "orbit",
    group: "Hosting",
    title: "Orbit",
    blurb:
      "Local Orbit CLI → your Cloudflare / Vercel / Netlify (only when this repo is Orbit-configured).",
    openUrl: "https://github.com/Dendro-X0/orbit",
    openLabel: "Orbit docs",
    needsPublic: true,
    deployArgs: "status",
    steps: [
      "Orbit is a local CLI, not a datacenter product — you still need a host account.",
      "Confirm Orbit is on PATH and this repo has orbit.toml / .orbit.",
      "Run deploy from Ritual or Tools — Studio does not upload for you.",
      "Copy the live URL → Publish Live check → Confirm.",
    ],
  },
  {
    id: "apple-sign",
    group: "Official signing",
    title: "Apple",
    blurb: "Developer certificates, notarization, App Store Connect.",
    openUrl: "https://developer.apple.com/account",
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
    openUrl: "https://partner.microsoft.com/dashboard",
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
    openUrl: "https://play.google.com/console",
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

export const PLATFORM_GROUPS = ["Hosting"] as const;

/** Official store / cert lanes — rendered as a grid on the Sign panel. */
export const SIGN_GROUPS = ["Official signing"] as const;

export function signingCatalogEntries() {
  return PLATFORM_WIZARDS.filter((w) => w.group === "Official signing");
}

/** Hosting cards; Orbit only when the bound project is Orbit-configured. */
export function hostingCatalogEntries(opts?: { orbitConfigured?: boolean }) {
  return PLATFORM_WIZARDS.filter((w) => {
    if (w.group !== "Hosting") return false;
    if (w.id === "orbit") return Boolean(opts?.orbitConfigured);
    return true;
  });
}

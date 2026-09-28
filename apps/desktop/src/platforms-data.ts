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
      "Deploy runs a **fast local check** (detect → artifact → GET health) and finishes — typically under a second on Harbor.",
      "Need the site in a browser? **Open live** starts serve and opens loopback; **Cancel serve** stops it. Cloud hosts: **Cancel on dashboard**.",
      "Live URL is loopback only (http://127.0.0.1:…) — never a *.pages.dev / Vercel / Netlify URL. **Start over** clears Results after a check.",
    ],
  },
  {
    id: "cloudflare",
    group: "Hosting",
    title: "Cloudflare",
    blurb:
      "Workers / Pages — Deploy streams wrangler on this machine; Open dashboard to see the project.",
    provider: "cloudflare",
    openUrl: "https://dash.cloudflare.com/?to=/:account/workers-and-pages",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider cloudflare",
    steps: [
      "Deploy streams shipctl hostdeploy (wrangler Pages or Workers) — watch Output for phases.",
      "Success URL is Cloudflare-hosted (*.pages.dev / workers.dev) — not Self-host loopback.",
      "Not logged in? Login CLI (`wrangler login`) or Sign in (web) — Studio never creates API tokens.",
      "After you delete on the vendor: **Clear evidence** syncs Studio (drops local last-run). **Cancel on dashboard** opens Workers & Pages — Studio never undeploys for you.",
    ],
  },
  {
    id: "vercel",
    group: "Hosting",
    title: "Vercel",
    blurb:
      "Vercel project — Deploy streams vercel CLI on this machine; Open dashboard to see the project.",
    provider: "vercel",
    openUrl: "https://vercel.com/dashboard",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider vercel",
    steps: [
      "Deploy streams shipctl hostdeploy (vercel --prod) — watch Output for phases.",
      "Not logged in? Login CLI (`vercel login`) or Sign in (web) — Studio never stores tokens.",
      "Open dashboard → Projects list (or the project when linked). Stop/delete only on Vercel.",
      "Copy the production URL → Publish Live check → Confirm (Studio does not auto-Confirm).",
    ],
  },
  {
    id: "netlify",
    group: "Hosting",
    title: "Netlify",
    blurb:
      "Netlify site — Deploy streams netlify CLI on this machine; Open dashboard to see the site.",
    provider: "netlify",
    openUrl: "https://app.netlify.com/projects",
    openLabel: "Deploy",
    needsPublic: true,
    deployArgs: "deploy --provider netlify",
    steps: [
      "Deploy streams shipctl hostdeploy (netlify deploy --prod) — watch Output for phases.",
      "Not logged in? Login CLI (`netlify login`) or Sign in (web) — Studio never stores tokens.",
      "Open dashboard → Projects (site deep-link when known). Stop/delete only on Netlify.",
      "Copy the site URL → Publish Live check → Confirm (Studio does not auto-Confirm).",
    ],
  },
  {
    id: "github-pages",
    group: "Hosting",
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
    group: "Hosting",
    title: "Fly.io",
    blurb: "flyctl auth + secrets — Portal Login CLI, then dashboard.",
    provider: "fly",
    openUrl: "https://fly.io/dashboard",
    needsPublic: true,
    steps: [
      "Portal steps → Login CLI (`fly auth login`).",
      "Open dashboard → Apps list to manage / destroy apps.",
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
    openUrl: "https://railway.com/dashboard",
    needsPublic: true,
    steps: [
      "Portal steps → Login CLI (`railway login`).",
      "Open dashboard → projects to manage services / variables.",
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

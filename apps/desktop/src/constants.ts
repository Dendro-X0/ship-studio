/** Desktop shell constants (nav · storage · action ids). */

export const LAST_PROJECT_KEY = "orbit-yard.last-project";
export const RECENT_KEY = "orbit-yard.recent-projects";
export const OFFLINE_KEY = "orbit-yard.offline";
export const DEPLOY_KEY = "orbit-yard.include-deploy";
export const MODE_KEY = "orbit-yard.mode";
export const INTENT_KEY = "orbit-yard.intent";
/** "1" = show bottom output dock; unset/0 = hidden (default). */
export const OUTPUT_DOCK_KEY = "orbit-yard.output-dock";
/** stages | list — Publish UI presentation */
export const PUBLISH_UI_KEY = "orbit-yard.publish-ui";
/** Last dashboard workflow card id */
export const WORKFLOW_KEY = "orbit-yard.workflow";
/** Collapsed/open state for sidebar nav sections */
export const NAV_SECTIONS_KEY = "orbit-yard.nav-sections";
export const MAX_RECENT = 6;

export const ACTION_IDS = [
  "btn-doctor",
  "btn-wizard",
  "btn-ship",
  "btn-human",
  "btn-human-open",
  "btn-human-put",
  "btn-launch",
  "btn-launch-open",
  "btn-launch-verify",
  "btn-launch-confirm",
  "btn-launch-next",
  "btn-publish",
  "btn-publish-related",
  "btn-publish-open",
  "btn-publish-verify",
  "btn-publish-continue",
  "btn-publish-confirm",
  "btn-publish-next",
  "btn-guide",
  "btn-configure",
  "btn-portal",
  "btn-portal-open",
  "btn-portal-refresh",
  "btn-portal-open-all",
  "btn-secrets",
  "btn-vault",
  "btn-vault-export",
  "btn-sign",
  "btn-deploy",
  "btn-flow-dry",
  "btn-flow",
  "btn-status",
  "btn-cancel",
  "btn-copy",
  "btn-clear",
  "btn-reveal",
  "btn-save-args",
  "btn-assist",
  "btn-assist-start",
  "btn-scopes",
  "btn-scopes-save",
  "btn-env",
  "btn-sign-paths",
] as const;

export const VIEW_META: Record<string, { title: string; desc: string }> = {
  dashboard: {
    title: "Dashboard",
    desc: "The repo you’re shipping, and the next human action.",
  },
  assist: {
    title: "Assist",
    desc: "Checklist overview — Start publishing for the live workflow.",
  },
  publish: {
    title: "Publish",
    desc: "Your publish checklist — Open/Run → Confirm → Next; Related opens detail panels.",
  },
  scopes: {
    title: "Targets",
    desc: "Active publish surfaces in this repo — Web / API / Desktop / Mobile / Container / Docs.",
  },
  env: {
    title: "ENV & tokens",
    desc: "Detail panel — configure, retrieve, create on official dashboards (incl. DB hosts).",
  },
  sign: {
    title: "Sign",
    desc: "Official signing grid + Signet Check status — Open dashboard, finish on the vendor site.",
  },
  launch: {
    title: "Launch",
    desc: "Optional choice board — open Sign / Deployment / Integrations, then Confirm. Prefer Publish for the checklist.",
  },
  portal: {
    title: "Portal",
    desc: "Detail panel — human paste sprint, provider entry, markets & container docs.",
  },
  integrations: {
    title: "Integrations",
    desc: "Payment and email wizards — open the vendor, then confirm here.",
  },
  platforms: {
    title: "Deployment",
    desc: "Hosting hosts — Put / Login CLI / Open dashboard. Official signing is under Sign.",
  },
  ritual: {
    title: "Ritual",
    desc: "Detail panel — sign_args / deploy_args in .ship/studio.json.",
  },
  tools: {
    title: "Tools",
    desc: "Pass-through doctor / sign / deploy / flow — not the primary start.",
  },
  output: {
    title: "Output",
    desc: "Full console for the active orbityard stream.",
  },
};

export const RELATED_VIEW_LABELS: Record<string, string> = {
  scopes: "Open Targets",
  env: "Open Env",
  sign: "Open Sign",
  portal: "Open Portal",
  ritual: "Open Ritual",
  tools: "Open Tools",
  launch: "Open Launch",
  dashboard: "Open Dashboard",
  platforms: "Open Deployment",
  integrations: "Open Integrations",
};

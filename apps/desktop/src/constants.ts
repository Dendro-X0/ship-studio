/** Desktop shell constants (nav · storage · action ids). */

export const LAST_PROJECT_KEY = "ship-studio.last-project";
export const RECENT_KEY = "ship-studio.recent-projects";
export const OFFLINE_KEY = "ship-studio.offline";
export const DEPLOY_KEY = "ship-studio.include-deploy";
export const MODE_KEY = "ship-studio.mode";
export const INTENT_KEY = "ship-studio.intent";
/** "1" = show bottom output dock; unset/0 = hidden (default). */
export const OUTPUT_DOCK_KEY = "ship-studio.output-dock";
/** stages | list — Publish UI presentation */
export const PUBLISH_UI_KEY = "ship-studio.publish-ui";
/** Last dashboard workflow card id */
export const WORKFLOW_KEY = "ship-studio.workflow";
/** Collapsed/open state for sidebar nav sections */
export const NAV_SECTIONS_KEY = "ship-studio.nav-sections";
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
    desc: "Checklist overview — Start publishing for the live spine.",
  },
  publish: {
    title: "Publish",
    desc: "Minute spine — Open/Run → Confirm → Next; Related opens detail panels.",
  },
  scopes: {
    title: "Scopes",
    desc: "Detail panel — Web / API / Desktop / Mobile / Container directories for the current publish step.",
  },
  env: {
    title: "ENV & tokens",
    desc: "Detail panel — configure, retrieve, create on official dashboards (incl. DB hosts).",
  },
  sign: {
    title: "Sign",
    desc: "Detail panel — self-sign, official certificates, or store submit portals for this step.",
  },
  launch: {
    title: "Launch",
    desc: "Companion stepper — prefer Publish for the full minute path.",
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
    title: "Platforms",
    desc: "Hosting and official signing — pick a provider, open their UI, then Confirm on Publish.",
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
    desc: "Full console for the active shipctl stream.",
  },
};

export const RELATED_VIEW_LABELS: Record<string, string> = {
  scopes: "Open Scopes",
  env: "Open Env",
  sign: "Open Sign",
  portal: "Open Portal",
  ritual: "Open Ritual",
  tools: "Open Tools",
  launch: "Open Launch",
  dashboard: "Open Dashboard",
  platforms: "Open Platforms",
  integrations: "Open Integrations",
};

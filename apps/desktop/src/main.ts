import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";

import {
  ACTION_IDS,
  DEPLOY_KEY,
  INTENT_KEY,
  LAST_PROJECT_KEY,
  MAX_RECENT,
  MODE_KEY,
  OFFLINE_KEY,
  OUTPUT_DOCK_KEY,
  PUBLISH_UI_KEY,
  RECENT_KEY,
  RECENT_PILL_MAX,
  SHIPPED_AT_KEY,
  WORKFLOW_KEY,
  NAV_SECTIONS_KEY,
  RELATED_VIEW_LABELS,
  VIEW_META,
} from "./constants";
import {
  integrationIconHtml,
  providerIconHtml,
} from "./icons";
import {
  INTEGRATION_WIZARDS,
  INTEGRATION_DONE_CRITERIA,
  INTEGRATION_HOST_PUT,
  INTEGRATION_PUBLISH_STEP,
} from "./integrations-data";
import { SUITE_CARDS } from "./suite-data";
import {
  CLOUD_HOST_GROUP,
  PLATFORM_GROUPS,
  PLATFORM_WIZARDS,
  SIGN_GROUPS,
  deployPreferGroup,
  hostingCatalogEntries,
  isDeployCatalogGroup,
  signingCatalogEntries,
} from "./platforms-data";
import {
  evaluateStructureGate,
  flowsintFixtureGuessPath,
  isHardStructureLane,
  medusaFixtureGuessPath,
  umamiFixtureGuessPath,
  type StructureGate,
} from "./structure-gate";
import {
  paintProviderWizard,
  renderProviderCatalogGrid,
} from "./provider-catalog";
import type {
  AssistPlan,
  CmdItem,
  CmdResult,
  Detected,
  DoctorReport,
  EnvPortal,
  HumanSprint,
  LaunchView,
  PortalPlan,
  ProjectPulse,
  ProviderWizard,
  PublishView,
  ScopePlan,
  SecretsPlan,
  ShipIntent,
  ShipState,
  SignPortal,
  StreamLine,
  StudioMode,
  ToastKind,
} from "./types";
import {
  fetchUpdateCheck,
  loadSnoozedTag,
  openLatestRelease,
  snoozeUpdateTag,
} from "./update-check";
import {
  escapeHtml,
  joinArgs,
  parentPath,
  prettyMaybe,
  projectName,
  splitArgs,
} from "./util";

let lastLaunch: LaunchView | null = null;
let lastPublish: PublishView | null = null;
let lastPulse: ProjectPulse | null = null;
/** S1.15 — skip dirty Confirm toast until tree cleans or project rebinds. */
let dirtyConfirmArmed = false;
let lastRunState: ShipState["last_run"] = null;
let lastDetected: Detected | undefined;
let lastHuman: HumanSprint | null = null;
let lastPortal: PortalPlan | null = null;
let lastSecrets: SecretsPlan | null = null;
let portalFilter: string | null = null;

function studioMode(): StudioMode {
  return localStorage.getItem(MODE_KEY) === "advanced" ? "advanced" : "general";
}

function shipIntent(): ShipIntent {
  // First run / unset → Local (shortest path). Explicit "public" stays Public.
  return localStorage.getItem(INTENT_KEY) === "public" ? "public" : "local";
}

function publishArgs(extra: string[] = []): string[] {
  return [
    "publish",
    "--mode",
    studioMode(),
    "--intent",
    shipIntent(),
    "--project",
    projectPath(),
    ...extra,
  ];
}

function applyShipIntent(intent: ShipIntent, opts?: { rebuild?: boolean }) {
  localStorage.setItem(INTENT_KEY, intent);
  document.body.dataset.intent = intent;
  document.querySelectorAll<HTMLButtonElement>(".intent-btn").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.intent === intent);
  });
  syncIntentCue();
  syncNowQuick();
  if (activeViewId === "platforms") {
    renderPlatforms();
  }
  if (activeViewId === "integrations") {
    renderIntegrations();
  }
  if (opts?.rebuild && projectPath()) {
    void (async () => {
      const result = await run(publishArgs(["reset"]), { quietHeader: true });
      if (result?.stdout) {
        try {
          applyPublishView(JSON.parse(result.stdout) as PublishView);
        } catch {
          /* ignore */
        }
      }
      toast(
        intent === "local"
          ? "Local — done on this PC (no public URL required)"
          : "Public — done when you have a public URL on your host",
        "ok",
      );
      await refreshSessionNow();
    })();
  }
}

/** S1.14 — one-sentence Local / Public consequences on Dashboard. */
function syncIntentCue() {
  const el = document.querySelector<HTMLElement>("#intent-cue");
  if (!el) return;
  if (!projectPath()) {
    el.hidden = true;
    el.textContent = "";
    return;
  }
  el.hidden = false;
  el.textContent =
    shipIntent() === "local"
      ? "Local — done = signed/shipped on this machine (no public URL)."
      : "Public — done = public URL on your host (or the named gate you Confirm).";
}

function applyStudioMode(mode: StudioMode, opts?: { rebuild?: boolean }) {
  localStorage.setItem(MODE_KEY, mode);
  document.body.dataset.mode = mode;
  document.querySelectorAll<HTMLButtonElement>(".mode-btn").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.mode === mode);
  });
  syncDeployToggle();
  syncNowQuick();
  const advancedViews = new Set(["assist", "launch", "portal", "ritual", "tools"]);
  if (mode === "general" && advancedViews.has(activeViewId)) {
    setView("dashboard");
  }
  if (opts?.rebuild && projectPath()) {
    void (async () => {
      const result = await run(publishArgs(["reset"]), { quietHeader: true });
      if (result?.stdout) {
        try {
          applyPublishView(JSON.parse(result.stdout) as PublishView);
        } catch {
          /* ignore */
        }
      }
      toast(
        mode === "general"
          ? "General — one loud next gate (done = public URL)"
          : "Advanced — full path and peer tools",
        "ok",
      );
      await refreshSessionNow();
    })();
  }
}

const pathEl = () => document.querySelector<HTMLInputElement>("#project-path");
const outputEl = () => document.querySelector<HTMLPreElement>("#output");
const stateEl = () => document.querySelector<HTMLElement>("#run-state");
const offlineEl = () => document.querySelector<HTMLInputElement>("#opt-offline");
const deployEl = () => document.querySelector<HTMLInputElement>("#opt-deploy");
const signArgsEl = () => document.querySelector<HTMLInputElement>("#sign-args");
const deployArgsEl = () => document.querySelector<HTMLInputElement>("#deploy-args");


let running = false;
let lastBusyToastAt = 0;
let busyWatchdog: number | null = null;
let runTickTimer: number | null = null;
let runStartedAt = 0;
let runBaseLabel = "Ready";
let streamBuf = "";

function projectPath(): string {
  return pathEl()?.value.trim() ?? "";
}

function offline(): boolean {
  return offlineEl()?.checked ?? true;
}

/** S0.8 — notice newer GitHub Release; never auto-install. */
async function runUpdateCheck(opts?: { quiet?: boolean }): Promise<void> {
  const quiet = Boolean(opts?.quiet);
  const result = await fetchUpdateCheck({ offline: offline() });
  if (result.kind === "offline") {
    if (!quiet) toast("Offline — turn off Offline to check for updates", "info");
    return;
  }
  if (result.kind === "error") {
    if (!quiet) toast(`Update check failed — ${result.detail}`, "err", 5000);
    return;
  }
  if (result.kind === "current") {
    if (!quiet) toast(`Up to date · ${result.local}`, "ok");
    return;
  }
  if (quiet && loadSnoozedTag() === result.remote) return;
  toast(`Update available · ${result.remote} (you have ${result.local})`, "info", 12000, [
    {
      id: "open-release",
      label: "Open download",
      icon: "open",
      run: () => {
        void openLatestRelease();
      },
    },
    {
      id: "later",
      label: "Later",
      run: () => snoozeUpdateTag(result.remote),
    },
  ]);
}

function includeDeploy(): boolean {
  return deployEl()?.checked ?? false;
}

function clearBusyWatchdog() {
  if (busyWatchdog !== null) {
    window.clearTimeout(busyWatchdog);
    busyWatchdog = null;
  }
}

function clearRunTick() {
  if (runTickTimer !== null) {
    window.clearInterval(runTickTimer);
    runTickTimer = null;
  }
}

function paintRunStateLabel(text: string) {
  for (const el of [
    stateEl(),
    document.querySelector<HTMLElement>("#run-state-bar"),
  ]) {
    if (!el) continue;
    const label = el.querySelector<HTMLElement>(".run-state-label");
    if (label) label.textContent = text;
    else el.textContent = text;
  }
}

/** Update busy chrome mid-run (e.g. Self-host check → Serving) without unlocking UI. */
function noteRunPhase(label: string, opts?: { resetElapsed?: boolean }) {
  if (!running) return;
  runBaseLabel = label;
  if (opts?.resetElapsed !== false) runStartedAt = Date.now();
  paintRunStateLabel(label);
}

function paintOutputCancelButtons() {
  const dock = document.querySelector<HTMLButtonElement>("#btn-output-cancel");
  const bar = document.querySelector<HTMLButtonElement>("#btn-statusbar-cancel");
  const show = running || selfhostServing;
  const label = selfhostServing ? "Cancel serve" : "Cancel";
  for (const btn of [dock, bar]) {
    if (!btn) continue;
    btn.hidden = !show;
    btn.disabled = !show;
    btn.textContent = label;
  }
}

function setBusy(busy: boolean, label = "Ready", failed = false) {
  running = busy;
  runBaseLabel = label;
  const className = busy ? "busy" : failed ? "failed" : "ready";
  for (const el of [
    stateEl(),
    document.querySelector<HTMLElement>("#run-state-bar"),
  ]) {
    if (!el) continue;
    const labelEl = el.querySelector<HTMLElement>(".run-state-label");
    if (labelEl) labelEl.textContent = label;
    else el.textContent = label;
    el.className =
      el.id === "run-state-bar" ? `${className} statusbar-run` : className;
    if (el.id === "run-state-bar") {
      el.hidden = !busy && !failed && label === "Ready";
    }
  }
  const dock = document.querySelector<HTMLElement>("#output-dock");
  if (dock) dock.dataset.running = busy ? "1" : "";
  document.documentElement.dataset.shipRunning = busy ? "1" : "";
  // Always re-sync disabled state — Cancel / Clear / errors must unlock Refresh.
  setProjectUi(Boolean(projectPath()));
  syncNowQuick();
  clearBusyWatchdog();
  clearRunTick();
  if (busy) {
    runStartedAt = Date.now();
    runTickTimer = window.setInterval(() => {
      if (!running) return;
      const sec = Math.floor((Date.now() - runStartedAt) / 1000);
      const base = runBaseLabel.replace(/\s*[·•]\s*\d+s\s*$/i, "").trim() || "Running…";
      paintRunStateLabel(`${base.replace(/…$/, "")} · ${sec}s`);
    }, 1000);
    busyWatchdog = window.setTimeout(() => {
      if (!running) return;
      toast("Still running — Cancel serve (dock) or Esc to stop", "info", 7000);
      const cancel = document.querySelector<HTMLButtonElement>("#btn-cancel");
      if (cancel) cancel.disabled = false;
      paintOutputCancelButtons();
    }, 90_000);
  }
  paintOutputCancelButtons();
  if (activeViewId === "platforms") {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform) ?? null;
    paintPlatCancelButtons(
      wiz,
      !(document.querySelector<HTMLElement>("#plat-results")?.hidden ?? true),
    );
  }
}

function forceUnlockUi(reason = "Unlocked") {
  clearBusyWatchdog();
  clearRunTick();
  running = false;
  selfhostServing = false;
  pendingSelfhostOpenLive = false;
  document.documentElement.dataset.shipRunning = "";
  const dock = document.querySelector<HTMLElement>("#output-dock");
  if (dock) dock.dataset.running = "";
  for (const el of [
    stateEl(),
    document.querySelector<HTMLElement>("#run-state-bar"),
  ]) {
    if (!el) continue;
    const labelEl = el.querySelector<HTMLElement>(".run-state-label");
    if (labelEl) labelEl.textContent = reason;
    else el.textContent = reason;
    el.className = el.id === "run-state-bar" ? "ready statusbar-run" : "ready";
    if (el.id === "run-state-bar") el.hidden = true;
  }
  setProjectUi(Boolean(projectPath()));
  syncNowQuick();
  paintOutputCancelButtons();
  if (activeViewId === "platforms") {
    paintPlatformWizard();
  }
}

function setProjectUi(on: boolean) {
  for (const id of ACTION_IDS) {
    const btn = document.querySelector<HTMLButtonElement>(`#${id}`);
    if (!btn) continue;
    if (id === "btn-cancel") {
      btn.disabled = !running;
      continue;
    }
    if (id === "btn-deploy") {
      btn.disabled = !on || offline() || running;
      continue;
    }
    if (id === "btn-clear" || id === "btn-copy") {
      btn.disabled = !on;
      continue;
    }
    // Confirm / Next / Continue: syncPublishGateButtons owns enable + primary.
    if (
      id === "btn-publish-confirm" ||
      id === "btn-publish-next" ||
      id === "btn-publish-continue"
    ) {
      continue;
    }
    btn.disabled = !on || running;
  }
  const sa = signArgsEl();
  const da = deployArgsEl();
  if (sa) sa.disabled = !on || running;
  if (da) da.disabled = !on || running;
  document.querySelectorAll<HTMLButtonElement>(".preset").forEach((btn) => {
    btn.disabled = !on || running;
  });
  const nowP = document.querySelector<HTMLButtonElement>("#now-primary");
  if (nowP) nowP.disabled = running;
  const dashOpen = document.querySelector<HTMLButtonElement>("#dash-open");
  if (dashOpen) dashOpen.disabled = running;
  syncDeployToggle();
  syncPublishGateButtons();
  syncPublishRelated();
  syncIntentCue();
  syncBackToPublish();
}

/** Continue is the fast path; Confirm/Next stay for explicit control. S2: one loud primary. */
function syncPublishGateButtons() {
  const continueBtn = document.querySelector<HTMLButtonElement>("#btn-publish-continue");
  const confirmBtn = document.querySelector<HTMLButtonElement>("#btn-publish-confirm");
  const nextBtn = document.querySelector<HTMLButtonElement>("#btn-publish-next");
  if (!confirmBtn || !nextBtn) return;
  const on = Boolean(projectPath());
  const cur = lastPublish?.current;
  const finished = Boolean(lastPublish?.finished);
  const status = (cur?.status ?? "").toLowerCase();
  const kind = (cur?.kind ?? "").toLowerCase();
  const pending = !finished && Boolean(lastPublish?.steps?.length) && status === "pending";
  const done =
    !finished && Boolean(lastPublish?.steps?.length) && (status === "done" || status === "skipped");
  const humanGate =
    pending &&
    (kind === "human" ||
      kind === "oauth" ||
      kind === "deploy" ||
      kind === "list" ||
      kind === "check" ||
      (kind === "sign" && (cur?.id ?? "").includes("release") && !(cur?.id ?? "").includes("dry")));
  const confirmReady = confirmBtn.classList.contains("watch-ready");
  confirmBtn.disabled = !on || running || !pending;
  nextBtn.disabled = !on || running || !done;
  if (continueBtn) {
    continueBtn.disabled = !on || running || finished || (!pending && !done);
    continueBtn.textContent = humanGate ? "Open this step" : "Continue";
    continueBtn.title = humanGate
      ? "Open this step first, then Confirm"
      : "Continue automatic checks (stops when you need to act)";
  }
  // S2 — only one green button: Confirm when the gate is ready; else Continue/Open.
  const loudConfirm = humanGate && pending && confirmReady;
  if (continueBtn) {
    continueBtn.classList.toggle("primary", !loudConfirm);
    continueBtn.classList.toggle("gate-secondary", loudConfirm);
  }
  confirmBtn.classList.toggle("primary", loudConfirm);
  confirmBtn.classList.toggle("gate-secondary", !loudConfirm);
  nextBtn.classList.toggle("primary", false);
  nextBtn.classList.add("gate-secondary");
  if (!pending) confirmBtn.classList.remove("watch-ready");
}

type PublishStepRow = NonNullable<PublishView["steps"]>[number];

/** Honesty / irreversible gates — must Open/Confirm; not auto-Continue. */
function isHonestyGateStep(step: { kind?: string; id?: string }): boolean {
  const kind = (step.kind ?? "").toLowerCase();
  const id = step.id ?? "";
  if (kind === "human" || kind === "oauth" || kind === "deploy" || kind === "list" || kind === "check") {
    return true;
  }
  return kind === "sign" && id.includes("release") && !id.includes("dry");
}

function partitionPublishProgress(view: PublishView): {
  done: Array<{ step: PublishStepRow; index: number }>;
  required: Array<{ step: PublishStepRow; index: number }>;
  later: Array<{ step: PublishStepRow; index: number }>;
} {
  const steps = view.steps ?? [];
  const curIdx = view.current_index ?? 0;
  const done: Array<{ step: PublishStepRow; index: number }> = [];
  const required: Array<{ step: PublishStepRow; index: number }> = [];
  const later: Array<{ step: PublishStepRow; index: number }> = [];
  steps.forEach((step, index) => {
    const status = (step.status ?? "").toLowerCase();
    if (status === "done" || status === "skipped") {
      if (index === curIdx && !view.finished) {
        required.push({ step, index });
      } else {
        done.push({ step, index });
      }
      return;
    }
    if (status !== "pending") return;
    if (index === curIdx || isHonestyGateStep(step)) {
      required.push({ step, index });
    } else {
      later.push({ step, index });
    }
  });
  return { done, required, later };
}

function publishProgressSummary(view: PublishView | null | undefined): string {
  if (!view?.steps?.length) return "";
  if (view.finished) return "All required gates done";
  const { done, required, later } = partitionPublishProgress(view);
  const mins = view.minutes_remaining ?? 0;
  return `${done.length} done · ${required.length} required · ${later.length} later · ~${mins} min`;
}

/** Status chip with distinct colors (pending / done / skipped / …). */
function statusKindHtml(raw: string | undefined | null): string {
  const label = (raw ?? "").trim() || "—";
  const key = label.toLowerCase().replace(/\s+/g, "_");
  return `<span class="kind" data-status="${escapeHtml(key)}">${escapeHtml(label)}</span>`;
}

function outputDockVisible(): boolean {
  const dock = document.querySelector<HTMLElement>("#output-dock");
  return Boolean(dock && !dock.hidden);
}

function applyOutputDock(visible: boolean) {
  const dock = document.querySelector<HTMLElement>("#output-dock");
  const toggle = document.querySelector<HTMLButtonElement>("#btn-statusbar-dock");
  if (dock) dock.hidden = !visible;
  localStorage.setItem(OUTPUT_DOCK_KEY, visible ? "1" : "0");
  if (toggle) {
    toggle.textContent = visible ? "Hide dock" : "Dock";
    toggle.title = visible ? "Hide the output dock" : "Show the output dock";
    toggle.setAttribute("aria-pressed", visible ? "true" : "false");
  }
  document.body.dataset.outputDock = visible ? "on" : "off";
}

function renderPublishStepItem(
  step: PublishStepRow,
  index: number,
  currentIndex: number | undefined,
): string {
  const active = index === currentIndex ? " active-step" : "";
  const related = (step.desktop_view ?? "").trim();
  const nav = related && RELATED_VIEW_LABELS[related];
  const clickable = nav ? " portal-step-nav" : "";
  const attrs = nav
    ? ` role="button" tabindex="0" data-step-index="${index}" data-desktop-view="${escapeHtml(related)}" title="Open ${escapeHtml(RELATED_VIEW_LABELS[related] ?? related)}"`
    : ` data-step-index="${index}"`;
  return `<li class="portal-step${active}${clickable}"${attrs}>
        <div class="meta">
          <div class="title">${statusKindHtml(step.status)}${escapeHtml(step.title ?? step.id ?? "")}${
            nav
              ? `<span class="step-nav-hint">${escapeHtml(RELATED_VIEW_LABELS[related] ?? related)}</span>`
              : ""
          }</div>
        </div>
      </li>`;
}

function renderPublishStepBands(view: PublishView): string {
  const { done, required, later } = partitionPublishProgress(view);
  const cur = view.current_index;
  const laterLabel =
    (view.mode ?? "").toLowerCase() === "advanced" ? "Advanced lanes" : "Later";
  const band = (
    id: string,
    label: string,
    rows: Array<{ step: PublishStepRow; index: number }>,
    open: boolean,
  ) => {
    if (!rows.length) return "";
    return `<details class="step-band" data-band="${id}"${open ? " open" : ""}>
      <summary>${escapeHtml(label)} (${rows.length})</summary>
      <ul>${rows.map((r) => renderPublishStepItem(r.step, r.index, cur)).join("")}</ul>
    </details>`;
  };
  return [
    band("required", "Required", required, true),
    band("later", laterLabel, later, later.length > 0 && later.length <= 6),
    band("done", "Done", done, false),
  ]
    .filter(Boolean)
    .join("");
}

type WorkflowId = "sign_only" | "sign_deploy" | "publish_platform" | "deploy_only";

const WORKFLOW_PRESETS: Record<
  WorkflowId,
  {
    mode: StudioMode;
    intent: ShipIntent;
    label: string;
    toast: string;
    /** S1.2c E — preview before reset (approx; ±2 steps ok). */
    preview: string;
  }
> = {
  sign_only: {
    mode: "general",
    intent: "local",
    label: "Sign only",
    toast: "Sign only — done on this PC (no public URL).",
    preview: "~6 steps · local signed build · no public URL",
  },
  sign_deploy: {
    mode: "general",
    intent: "public",
    label: "Sign and get live",
    toast: "Sign and get live — done when you have a public URL.",
    preview: "~8 steps · sign, then live URL on your host",
  },
  publish_platform: {
    mode: "advanced",
    intent: "public",
    label: "Publish to stores",
    toast: "Publish to stores — Advanced · Confirm each listing/store gate.",
    preview: "~15 steps · listings and store review",
  },
  deploy_only: {
    mode: "general",
    intent: "public",
    label: "Get live",
    toast: "Get live — Open → Host → Login → Deploy → Open live.",
    preview: "Primary · host → Deploy → public URL → Confirm",
  },
};

function getLiveToastForDetected(): string {
  if (lastDetected?.medusa) {
    return "Get live — Medusa: Deploy compose → Open Admin → Confirm login.";
  }
  if (lastDetected?.flowsint) {
    return "Get live — Flowsint: Deploy compose.prod → Open App → Confirm register.";
  }
  if (lastDetected?.umami) {
    return "Get live — Umami: Deploy compose → Open Admin → Confirm password.";
  }
  return WORKFLOW_PRESETS.deploy_only.toast;
}

let stageFocusIndex = 0;
/** True while Continue walks Auto gates one-at-a-time with dwell. */
let publishPacing = false;

function paceDwellMs(): number {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches ? 200 : 1800;
}

function sleepMs(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

function setStagePacingUi(on: boolean) {
  const panel = document.querySelector<HTMLElement>("#stage-panel");
  if (!panel) return;
  if (on) panel.dataset.pacing = "1";
  else delete panel.dataset.pacing;
}

function isWorkflowId(raw: string | null | undefined): raw is WorkflowId {
  return Boolean(raw && raw in WORKFLOW_PRESETS);
}

function savedWorkflow(): WorkflowId | null {
  const raw = localStorage.getItem(WORKFLOW_KEY);
  return isWorkflowId(raw) ? raw : null;
}

function publishUiMode(): "stages" | "list" {
  return localStorage.getItem(PUBLISH_UI_KEY) === "list" ? "list" : "stages";
}

function applyPublishUi(mode: "stages" | "list") {
  localStorage.setItem(PUBLISH_UI_KEY, mode);
  document.body.dataset.publishUi = mode;
  const stagesBtn = document.querySelector<HTMLButtonElement>("#btn-publish-stages");
  const listBtn = document.querySelector<HTMLButtonElement>("#btn-publish-list");
  stagesBtn?.setAttribute("aria-pressed", mode === "stages" ? "true" : "false");
  listBtn?.setAttribute("aria-pressed", mode === "list" ? "true" : "false");
  const stageEl = document.querySelector<HTMLElement>("#publish-stage");
  if (stageEl) {
    stageEl.hidden = mode !== "stages" || !lastPublish?.steps?.length;
  }
}

function syncWorkflowCards() {
  const active = savedWorkflow();
  document.querySelectorAll<HTMLButtonElement>(".workflow-card").forEach((btn) => {
    const id = btn.dataset.workflow;
    btn.setAttribute("aria-current", id && id === active ? "true" : "false");
    if (isWorkflowId(id)) {
      let preview = btn.querySelector<HTMLElement>(".workflow-card-preview");
      if (!preview) {
        preview = document.createElement("span");
        preview.className = "workflow-card-preview";
        btn.appendChild(preview);
      }
      preview.textContent = WORKFLOW_PRESETS[id].preview;
    }
  });
  const pick = document.querySelector<HTMLElement>("#workflow-pick");
  if (pick) {
    // Mid-flight: only Continue — starting another workflow mid-pass is noise.
    const mid =
      Boolean(lastPublish?.steps?.length && !lastPublish.finished) ||
      Boolean(lastLaunch?.steps?.length && !lastLaunch.finished);
    pick.hidden = !projectPath() || mid;
  }
}

function verifyStatusLabel(raw?: string | null): string {
  switch ((raw ?? "").toLowerCase()) {
    case "disk":
      return "files on disk";
    case "local_cli":
      return "local tools";
    case "operator_cli":
      return "official CLI";
    case "human_attest":
      return "your confirmation";
    default:
      return "";
  }
}

function stageGuideline(step: {
  kind?: string;
  id?: string;
  title?: string;
  status?: string | null;
  verify_status?: string | null;
}): string {
  const status = (step.status ?? "").toLowerCase();
  const layer = verifyStatusLabel(step.verify_status);
  const layerPrefix = layer ? `We'll check ${layer}. ` : "";
  if (status === "done" || status === "skipped") {
    return `${layerPrefix}This checkpoint is done. Press Continue to advance.`;
  }
  const kind = (step.kind ?? "").toLowerCase();
  const id = step.id ?? "";
  if ((step.verify_status ?? "").toLowerCase() === "human_attest") {
    return `${layerPrefix}Use Login CLI or Open portal for auth, then Verify → Confirm. Studio does not hold tokens.`;
  }
  if (isScopesStep(step)) {
    return `${layerPrefix}Choose what you’re shipping below, then Confirm & continue.`;
  }
  if (kind === "human" || id.includes("scope")) {
    return `${layerPrefix}Save the selection below, then Confirm & continue.`;
  }
  if (kind === "oauth" || kind === "list" || kind === "deploy" || kind === "check") {
    if ((step.id ?? "") === "live_check") {
      return `${layerPrefix}Outcome: open your public URL (or Deploy first). Desktop-only? Switch to Local — Live check is omitted.`;
    }
    if (kind === "deploy") {
      return `${layerPrefix}Get live on your host — Deploy, then Open live. Confirm when the URL works.`;
    }
    if (kind === "oauth") {
      return `${layerPrefix}Sign in at the host (web), or Login CLI for local credentials. Verify, then Confirm.`;
    }
    return `${layerPrefix}Finish this step on the vendor site if needed, then Confirm.`;
  }
  if (kind === "sign" && id.includes("release") && !id.includes("dry")) {
    return `${layerPrefix}Run local Signet for this cut, then Confirm when the artifact is ready.`;
  }
  if (kind === "auto" || kind === "doctor" || id === "configure" || id === "dry_run" || !kind) {
    return `${layerPrefix}Press Continue — Studio checks this locally and advances.`;
  }
  if (isHonestyGateStep(step)) {
    return `${layerPrefix}Open / Run if needed, then Confirm.`;
  }
  return `${layerPrefix}Follow the detail below, then use the green button.`;
}

function stagePrimaryLabel(view: PublishView): {
  label: string;
  action: "continue" | "open" | "confirm" | "review";
} {
  if (view.finished) return { label: "Back to Dashboard", action: "review" };
  const cur = view.current;
  const status = (cur?.status ?? "").toLowerCase();
  const kind = (cur?.kind ?? "").toLowerCase();
  // Done checkpoint — one advance verb (never “Next” beside another Next).
  if (status === "done" || status === "skipped") return { label: "Continue", action: "continue" };
  // Inline Scopes: Confirm stays on the card (no bounce to detail panel).
  if (isScopesStep(cur)) {
    return { label: "Confirm & continue", action: "confirm" };
  }
  // Local Auto (configure / doctor / dry-run) — never “Open Ritual”.
  if (kind === "auto" || gateToastKind(cur) === "continue") {
    return { label: "Continue", action: "continue" };
  }
  if (gateToastKind(cur) === "open") {
    const related = (cur?.desktop_view ?? "").trim();
    // Dashboard is not a work panel — Confirm stays on Publish (no Open bounce).
    if (related === "dashboard" || !shouldLeavePublishForRelated(related)) {
      return { label: "Confirm", action: "confirm" };
    }
    if ((cur?.kind ?? "").toLowerCase() === "oauth") return { label: "Sign in", action: "open" };
    if (related && RELATED_VIEW_LABELS[related]) return { label: "Open", action: "open" };
    if (cur?.entry_url || cur?.run?.length) return { label: "Open", action: "open" };
    return { label: "Confirm", action: "confirm" };
  }
  return { label: "Confirm", action: "confirm" };
}

function isScopesStep(step: { id?: string; desktop_view?: string | null } | null | undefined): boolean {
  if (!step) return false;
  const id = (step.id ?? "").toLowerCase();
  const view = (step.desktop_view ?? "").trim().toLowerCase();
  return id === "scopes" || view === "scopes";
}

function scopeKindIcon(kind: string): string {
  const k = kind.toLowerCase();
  // Small 20×20 glyphs — match desktop nav weight, not emoji.
  if (k === "desktop") {
    return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M4 5.5h16A1.5 1.5 0 0 1 21.5 7v8A1.5 1.5 0 0 1 20 16.5H4A1.5 1.5 0 0 1 2.5 15V7A1.5 1.5 0 0 1 4 5.5zm0 11h16V18H4zm6.5 1.5h3v1.5h-3z"/></svg>`;
  }
  if (k === "web" || k === "docs") {
    return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12 2.5a9.5 9.5 0 1 0 0 19 9.5 9.5 0 0 0 0-19zm0 1.6c1.6 0 3.1.5 4.3 1.4H7.7A7.8 7.8 0 0 1 12 4.1zm-5.6 3h11.2c.4.7.7 1.5.9 2.4H5.5c.2-.9.5-1.7.9-2.4zM4.7 12c0-.5 0-1 .1-1.5h14.4c.1.5.1 1 .1 1.5s0 1-.1 1.5H4.8c-.1-.5-.1-1-.1-1.5zm.8 4.5h13c-.2.9-.5 1.7-.9 2.4H6.4c-.4-.7-.7-1.5-.9-2.4zM7.7 19.5h8.6A7.8 7.8 0 0 1 12 20.9a7.8 7.8 0 0 1-4.3-1.4z"/></svg>`;
  }
  if (k === "api") {
    return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M8.2 7.2 4.5 12l3.7 4.8 1.5-1.2L7.2 12l2.5-3.6zm7.6 0-1.5 1.2L16.8 12l-2.5 3.6 1.5 1.2L19.5 12zM10.4 16.2l3.2-8.4h1.7l-3.2 8.4z"/></svg>`;
  }
  if (k === "mobile") {
    return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M8 2.5h8A1.5 1.5 0 0 1 17.5 4v16a1.5 1.5 0 0 1-1.5 1.5H8A1.5 1.5 0 0 1 6.5 20V4A1.5 1.5 0 0 1 8 2.5zm0 2v12h8V4.5zm4 14.2a1 1 0 1 0 0-2 1 1 0 0 0 0 2z"/></svg>`;
  }
  if (k === "container") {
    return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3.5 7.2 12 3.5l8.5 3.7v9.6L12 20.5l-8.5-3.7zm1.6 1.5v6.8L12 18.7l6.9-3.2V8.7L12 5.5z"/></svg>`;
  }
  return `<svg class="scope-card-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M4 6.5h16v2H4zm0 4.5h16v2H4zm0 4.5h10v2H4z"/></svg>`;
}

function scopesGridHtml(plan: ScopePlan | null): string {
  const scopes = plan?.scopes ?? [];
  const active = new Set(plan?.active ?? []);
  if (!scopes.length) {
    return `<p class="detail empty-hint">No targets detected — press Detect.</p>`;
  }
  return scopes
    .map((s) => {
      const id = s.id ?? "";
      const kind = (s.kind ?? "root").toLowerCase();
      const relative = s.relative ?? ".";
      const on = active.has(id) ? "checked" : "";
      const chips: string[] = [];
      if (s.provider) chips.push(s.provider);
      for (const sig of s.signals ?? []) {
        if (chips.length >= 3) break;
        const t = String(sig).trim();
        if (!t || chips.includes(t)) continue;
        chips.push(t);
      }
      const chipHtml = chips.length
        ? `<div class="scope-card-chips">${chips
            .map((c) => `<span class="scope-chip">${escapeHtml(c)}</span>`)
            .join("")}</div>`
        : "";
      const provider = (s.provider ?? "").trim();
      return `<div class="scope-card" data-kind="${escapeHtml(kind)}" data-scope-id="${escapeHtml(id)}">
            <label class="scope-card-select">
              <input type="checkbox" data-scope-id="${escapeHtml(id)}" ${on} />
              <span class="scope-card-icon" aria-hidden="true">${scopeKindIcon(kind)}</span>
              <div class="scope-card-body">
                <strong>${escapeHtml(s.label ?? id)}</strong>
                <span class="scope-card-meta">${escapeHtml(kind)} · ${escapeHtml(relative)}</span>
                ${chipHtml}
              </div>
            </label>
            <div class="scope-card-actions">
              <button type="button" class="ghost scope-open-folder" data-relative="${escapeHtml(relative)}" title="Open this target folder">Folder</button>
              <button type="button" class="ghost scope-open-deploy" data-provider="${escapeHtml(provider)}" data-relative="${escapeHtml(relative)}" title="Open Deployment for this target">Deploy</button>
            </div>
          </div>`;
    })
    .join("");
}

function scopeAbsolutePath(relative: string | undefined): string | null {
  const project = projectPath();
  if (!project) return null;
  const rel = (relative ?? ".").replace(/\\/g, "/").replace(/^\/+/, "");
  if (!rel || rel === ".") return project;
  const sep = project.includes("\\") ? "\\" : "/";
  return `${project.replace(/[\\/]+$/, "")}${sep}${rel.split("/").join(sep)}`;
}

function platformIdForScopeProvider(provider: string | null | undefined): string {
  const p = (provider ?? "").trim().toLowerCase().replace(/_/g, "-");
  if (
    p === "cloudflare" ||
    p === "vercel" ||
    p === "netlify" ||
    p === "fly" ||
    p === "railway" ||
    p === "orbit" ||
    p === "selfhost"
  ) {
    return p;
  }
  if (p === "github-pages" || p === "github" || p === "pages") return "github-pages";
  return preferredHostingPlatformId(lastDetected) ?? "selfhost";
}

async function openScopeFolder(relative: string | undefined) {
  const abs = scopeAbsolutePath(relative);
  if (!abs) {
    toast("Bind a project first", "info");
    return;
  }
  try {
    await openPath(abs);
    toast(`Opened ${relative && relative !== "." ? relative : "project"}`, "ok", 2500);
  } catch (err) {
    toast(String(err), "err");
  }
}

function openScopeDeployment(
  provider: string | null | undefined,
  relative?: string | null,
) {
  // Hosted targets need Public + network; Targets Deploy should unlock that path.
  const platformId = platformIdForScopeProvider(provider);
  const needsHost =
    platformId === "cloudflare" ||
    platformId === "vercel" ||
    platformId === "netlify" ||
    platformId === "fly" ||
    platformId === "railway";
  if (needsHost) {
    ensureHostingReady({ toastOk: true });
  }
  const focusAbs = relative ? scopeAbsolutePath(relative) : null;
  if (focusAbs && needsHost && isHostedCliDeployCard(platformId)) {
    // Remember subdirectory for the upcoming Deploy stream (Website Astro → apps/website).
    lastDeployFocusProject = focusAbs;
  } else {
    lastDeployFocusProject = null;
  }
  openPlatformsCatalog({
    selectId: platformId,
  });
}

function wireScopeGridActions(root: ParentNode | null) {
  if (!root) return;
  root.querySelectorAll<HTMLButtonElement>(".scope-open-folder").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      void openScopeFolder(btn.dataset.relative);
    });
  });
  root.querySelectorAll<HTMLButtonElement>(".scope-open-deploy").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      openScopeDeployment(btn.dataset.provider, btn.dataset.relative);
    });
  });
}

function fillScopeGrids(plan: ScopePlan | null) {
  const html = scopesGridHtml(plan);
  const main = document.querySelector("#scope-grid");
  const stage = document.querySelector("#stage-scope-grid");
  if (main) {
    main.innerHTML = html;
    wireScopeGridActions(main);
  }
  if (stage) {
    stage.innerHTML = html;
    wireScopeGridActions(stage);
  }
}

function stageScopesRoot(): HTMLElement | null {
  return document.querySelector<HTMLElement>("#stage-inline-scopes");
}

let stageScopesDetectInFlight = false;

function syncStageInlineScopes(view: PublishView, focusIndex: number) {
  const root = stageScopesRoot();
  if (!root) return;
  const steps = view.steps ?? [];
  const curIdx = view.current_index ?? 0;
  const focus = steps[focusIndex];
  const show =
    publishUiMode() === "stages" &&
    focusIndex === curIdx &&
    isScopesStep(focus) &&
    (focus?.status ?? "").toLowerCase() === "pending";
  root.hidden = !show;
  if (!show) return;
  fillScopeGrids(lastScopes);
  if (
    !lastScopes?.scopes?.length &&
    projectPath() &&
    !running &&
    !stageScopesDetectInFlight
  ) {
    stageScopesDetectInFlight = true;
    void detectScopes({ quiet: true }).finally(() => {
      stageScopesDetectInFlight = false;
    });
  }
}

function isLoopbackDeployUrl(u: string): boolean {
  const l = u.toLowerCase();
  return (
    l.includes("127.0.0.1") ||
    l.includes("localhost") ||
    l.includes("[::1]") ||
    l.startsWith("file:")
  );
}

function hostedDeployUrls(dep: { urls?: string[] | null } | null | undefined): string[] {
  return (dep?.urls ?? []).filter((u) => !isLoopbackDeployUrl(u));
}

/** Local-auto self-host health — distinct from cloud / Orbit live. */
function isSelfhostLocalEvidence(pulse: ProjectPulse | null | undefined): boolean {
  return pulse?.deploy?.signal === "selfhost_ok";
}

/** Hosted deploy evidence only (never self-host loopback). */
function deployEvidenceFromPulse(pulse: ProjectPulse | null | undefined): boolean {
  const dep = pulse?.deploy;
  if (!dep || dep.signal === "selfhost_ok") return false;
  if (dep.signal === "yard_deployed") return true;
  if (dep.signal === "last_run_ok") {
    const urls = dep.urls ?? [];
    if (urls.length > 0 && urls.every(isLoopbackDeployUrl)) return false;
    return true;
  }
  return hostedDeployUrls(dep).length > 0;
}

function shortStageLabel(raw: string): string {
  const t = raw.trim();
  // "Scopes — pick Web / …" → "Scopes"
  const em = t.split(/\s*[—–-]\s*/)[0]?.trim();
  if (em && em.length <= 28) return em;
  return t.length > 28 ? `${t.slice(0, 26)}…` : t;
}

function renderPublishStage(view: PublishView | null) {
  const stageRoot = document.querySelector<HTMLElement>("#publish-stage");
  const toggle = document.querySelector<HTMLElement>("#publish-ui-toggle");
  const scrub = document.querySelector<HTMLElement>("#stage-scrub");
  const scrubTrack = document.querySelector<HTMLElement>("#stage-scrub-track");
  const scrubLabel = document.querySelector<HTMLElement>("#stage-scrub-label");
  const rail = document.querySelector<HTMLElement>("#stage-rail");
  const kicker = document.querySelector<HTMLElement>("#stage-kicker");
  const title = document.querySelector<HTMLElement>("#stage-title");
  const indexEl = document.querySelector<HTMLElement>("#stage-index");
  const guide = document.querySelector<HTMLElement>("#stage-guide");
  const detail = document.querySelector<HTMLElement>("#stage-detail");
  const prevBtn = document.querySelector<HTMLButtonElement>("#btn-stage-prev");
  const nextBtn = document.querySelector<HTMLButtonElement>("#btn-stage-next");
  const primaryBtn = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
  if (!stageRoot || !rail) return;

  const steps = view?.steps ?? [];
  const hasPlan = steps.length > 0;
  if (toggle) toggle.hidden = !hasPlan;
  applyPublishUi(publishUiMode());
  if (!hasPlan || !view) {
    stageRoot.hidden = true;
    rail.innerHTML = "";
    if (scrub) scrub.hidden = true;
    if (scrubTrack) scrubTrack.innerHTML = "";
    return;
  }

  const curIdx = view.current_index ?? 0;
  if (stageFocusIndex < 0 || stageFocusIndex >= steps.length) {
    stageFocusIndex = curIdx;
  }
  // Keep focus on the live gate when plan advances past a stale focus.
  if (stageFocusIndex !== curIdx && (steps[stageFocusIndex]?.status ?? "").toLowerCase() === "pending") {
    stageFocusIndex = curIdx;
  }

  const { required, later } = partitionPublishProgress(view);
  // Slice B — required-now only; later collapsed behind "+N later" → List.
  const railRows: Array<{
    step: (typeof steps)[number];
    index: number;
    band: "required" | "later";
  }> = required.map((r) => ({ ...r, band: "required" as const }));
  const ensure = (index: number) => {
    if (!railRows.some((r) => r.index === index) && steps[index]) {
      railRows.push({
        step: steps[index],
        index,
        band: index === curIdx ? "required" : "later",
      });
    }
  };
  ensure(curIdx);
  ensure(stageFocusIndex);
  // Cap chips so the rail stays a pager, not the whole Adaptive plan.
  railRows.sort((a, b) => a.index - b.index);
  const capped = railRows.slice(0, 6);
  const laterCount = later.filter((r) => !capped.some((c) => c.index === r.index)).length;

  rail.innerHTML =
    capped
      .map(({ step, index, band }) => {
        const status = (step.status ?? "").toLowerCase();
        const state =
          publishPacing && index === curIdx
            ? "checking"
            : index === stageFocusIndex
              ? "current"
              : status === "done" || status === "skipped"
                ? "done"
                : band === "later"
                  ? "later"
                  : "current";
        const label = shortStageLabel(step.title ?? step.id ?? `Step ${index + 1}`);
        return `<button type="button" class="stage-dot" data-stage-index="${index}" data-state="${state}" title="${escapeHtml(step.title ?? step.id ?? "")}">${escapeHtml(label)}</button>`;
      })
      .join("") +
    (laterCount > 0
      ? `<button type="button" class="stage-dot stage-dot-more" data-stage-more="1" data-state="later" title="Show full plan in List">+${laterCount} later</button>`
      : "");

  if (scrub && scrubTrack) {
    scrub.hidden = false;
    scrubTrack.innerHTML = steps
      .map((step, index) => {
        const status = (step.status ?? "").toLowerCase();
        const state =
          publishPacing && index === curIdx
            ? "checking"
            : index === stageFocusIndex || index === curIdx
              ? "current"
              : status === "done" || status === "skipped"
                ? "done"
                : "later";
        const label = step.title ?? step.id ?? `Step ${index + 1}`;
        return `<button type="button" class="stage-scrub-seg" data-stage-index="${index}" data-state="${state}" title="${escapeHtml(`${index + 1}. ${label}`)}" aria-label="${escapeHtml(`Review step ${index + 1}: ${label}`)}"></button>`;
      })
      .join("");
    if (scrubLabel) {
      const focusTitle = steps[stageFocusIndex]?.title ?? steps[curIdx]?.title ?? "Checkpoint";
      scrubLabel.textContent = publishPacing
        ? `Checking ${stageFocusIndex + 1} / ${steps.length} — ${shortStageLabel(focusTitle)}`
        : `Step ${stageFocusIndex + 1} / ${steps.length} — click a segment to review`;
    }
  }

  const focus = steps[stageFocusIndex] ?? steps[curIdx];
  const focusStatus = (focus?.status ?? "").toLowerCase();
  const wf = savedWorkflow();
  if (kicker) {
    kicker.textContent = wf
      ? `${WORKFLOW_PRESETS[wf].label} · checkpoint`
      : stageFocusIndex === curIdx
        ? "Current checkpoint"
        : "Checkpoint";
  }
  if (title) title.textContent = focus?.title ?? focus?.id ?? "—";
  if (indexEl) {
    indexEl.textContent = `${stageFocusIndex + 1} / ${steps.length}${
      focus?.minutes ? ` · ~${focus.minutes}m` : ""
    }`;
  }
  if (guide) {
    if (view.finished) {
      const live = deployEvidenceFromPulse(lastPulse);
      const selfhost = isSelfhostLocalEvidence(lastPulse);
      guide.textContent = live
        ? "This pass’s required gates are done. Open the live URL when you want to smoke it again."
        : selfhost
          ? "Required gates for this pass are done. Self-host local check is ok — that is not a cloud host. Choose host when you want Public SaaS."
          : "Required gates for this pass are done. Deploy may still show no signal — that means no hosted URL yet (use Local for desktop-only cuts).";
    } else {
      guide.textContent = focus ? stageGuideline(focus) : "";
    }
  }
  if (detail) {
    const related = (focus?.desktop_view ?? "").trim();
    const showNav =
      related &&
      RELATED_VIEW_LABELS[related] &&
      !isScopesStep(focus) &&
      gateToastKind(focus) === "open";
    const layer = verifyStatusLabel(focus?.verify_status);
    detail.textContent = [
      focusStatus ? focusStatus.toUpperCase() : "",
      focus?.kind ? String(focus.kind) : "",
      layer ? `Status · ${layer}` : "",
      isScopesStep(focus) ? "Edit below" : "",
      showNav ? `Opens ${RELATED_VIEW_LABELS[related] ?? related}` : "",
    ]
      .filter(Boolean)
      .join(" · ");
  }

  syncStageInlineScopes(view, stageFocusIndex);

  const busy = running || publishPacing;
  const primary = stagePrimaryLabel(view);
  if (primaryBtn) {
    if (view.finished) {
      primaryBtn.textContent = "Back to Dashboard";
      primaryBtn.dataset.stageAction = "review";
      primaryBtn.disabled = busy;
    } else if (stageFocusIndex !== curIdx) {
      primaryBtn.textContent = "Go to current";
      primaryBtn.dataset.stageAction = "focus_current";
      primaryBtn.disabled = busy;
    } else if (
      (focus?.id ?? "") === "live_check" &&
      focusStatus === "pending" &&
      shipIntent() === "public" &&
      !deployEvidenceFromPulse(lastPulse)
    ) {
      primaryBtn.textContent = "Switch to Local";
      primaryBtn.dataset.stageAction = "intent_local";
      primaryBtn.disabled = busy;
      primaryBtn.title =
        "Public: done needs a public URL. Local omits Live check for desktop-only cuts.";
    } else if (isScopesStep(focus) && primary.action === "confirm") {
      const checked = document.querySelectorAll("#stage-scope-grid [data-scope-id]:checked").length;
      const hasActive = checked > 0 || (lastScopes?.active?.length ?? 0) > 0;
      primaryBtn.textContent = "Confirm & continue";
      primaryBtn.dataset.stageAction = "confirm";
      primaryBtn.disabled = busy || !hasActive;
      primaryBtn.title = hasActive
        ? "Save selection and mark this checkpoint done"
        : "Select at least one scope first";
    } else {
      primaryBtn.textContent = publishPacing ? "Checking…" : primary.label;
      primaryBtn.dataset.stageAction = primary.action;
      primaryBtn.disabled = busy;
      primaryBtn.removeAttribute("title");
    }
  }
  if (prevBtn) prevBtn.disabled = busy || stageFocusIndex <= 0;
  if (nextBtn) {
    // On the live gate the green primary owns advance — hide the second Next.
    const browsing = stageFocusIndex !== curIdx && !view.finished;
    nextBtn.hidden = !browsing;
    if (browsing) {
      const focusDone = focusStatus === "done" || focusStatus === "skipped";
      const canAdvanceFocus =
        stageFocusIndex < steps.length - 1 && (focusDone || stageFocusIndex < curIdx);
      nextBtn.disabled = busy || !canAdvanceFocus;
      nextBtn.textContent = "Peek next";
      nextBtn.title = "Browse the next checkpoint without advancing the plan";
    } else {
      nextBtn.disabled = true;
      nextBtn.textContent = "Next";
      nextBtn.removeAttribute("title");
    }
  }

  const panel = document.querySelector<HTMLElement>("#stage-panel");
  if (panel) {
    if (publishPacing) panel.dataset.pacing = "1";
    else delete panel.dataset.pacing;
    if (!publishPacing) {
      panel.style.animation = "none";
      void panel.offsetWidth;
      panel.style.animation = "";
    }
  }
}

async function startWorkflow(id: WorkflowId) {
  if (!projectPath()) {
    toast("Open a folder first", "err");
    return;
  }
  const preset = WORKFLOW_PRESETS[id];
  localStorage.setItem(WORKFLOW_KEY, id);
  applyStudioMode(preset.mode);
  applyShipIntent(preset.intent);
  applyPublishUi("stages");
  syncWorkflowCards();
  stageFocusIndex = 0;
  const result = await run(publishArgs(["reset"]), { quietHeader: true });
  if (result?.stdout) {
    try {
      const view = JSON.parse(result.stdout) as PublishView;
      stageFocusIndex = view.current_index ?? 0;
      applyPublishView(view, { reveal: true });
    } catch {
      setView("publish");
      await refreshPublish();
    }
  } else {
    setView("publish");
    await refreshPublish();
  }
  toast(id === "deploy_only" ? getLiveToastForDetected() : preset.toast, "ok", 4500);
  await refreshSessionNow();
}

function onStagePrimary() {
  const btn = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
  const action = btn?.dataset.stageAction ?? "continue";
  if (action === "review") {
    setView("dashboard");
    return;
  }
  if (action === "focus_current") {
    stageFocusIndex = lastPublish?.current_index ?? 0;
    if (lastPublish) renderPublishStage(lastPublish);
    return;
  }
  if (action === "open") {
    document.querySelector<HTMLButtonElement>("#btn-publish-open")?.click();
    return;
  }
  if (action === "confirm") {
    void (async () => {
      if (isScopesStep(lastPublish?.current)) {
        const inline = stageScopesRoot();
        if (inline && !inline.hidden) {
          const ok = await saveScopes({ silentToast: true });
          if (!ok) return;
        }
      }
      document.querySelector<HTMLButtonElement>("#btn-publish-confirm")?.click();
    })();
    return;
  }
  if (action === "intent_local") {
    applyShipIntent("local", { rebuild: true });
    toast("Local intent — hosted deploy / live check omitted. Continue the short path.", "ok", 5000);
    return;
  }
  const status = (lastPublish?.current?.status ?? "").toLowerCase();
  if (status === "done" || status === "skipped") {
    document.querySelector<HTMLButtonElement>("#btn-publish-next")?.click();
    return;
  }
  document.querySelector<HTMLButtonElement>("#btn-publish-continue")?.click();
}

function syncDeployToggle() {
  const dep = deployEl();
  const offlineOn = offline();
  if (dep) {
    if (offlineOn) {
      dep.checked = false;
      dep.disabled = true;
    } else {
      dep.disabled = !projectPath() || running;
    }
  }
  const deployBtn = document.querySelector<HTMLButtonElement>("#btn-deploy");
  if (deployBtn && projectPath()) {
    deployBtn.disabled = offlineOn || running;
  }
  const flowBtn = document.querySelector<HTMLButtonElement>("#btn-flow");
  if (flowBtn) {
    flowBtn.textContent =
      includeDeploy() && !offlineOn ? "Run flow" : "Run flow (skip deploy)";
  }
  localStorage.setItem(OFFLINE_KEY, offlineOn ? "1" : "0");
  localStorage.setItem(DEPLOY_KEY, includeDeploy() && !offlineOn ? "1" : "0");
}

async function refreshShipctlPath() {
  const el = document.querySelector("#orbityard-path");
  if (!el) return;
  try {
    const path = await invoke<string>("resolve_orbityard_path");
    el.textContent = `orbityard: ${path}`;
    el.setAttribute("title", path);
  } catch (err) {
    el.textContent = `orbityard: ${String(err)}`;
  }
}

function isTypingTarget(t: EventTarget | null): boolean {
  if (!(t instanceof HTMLElement)) return false;
  const tag = t.tagName;
  return tag === "INPUT" || tag === "TEXTAREA" || t.isContentEditable;
}

const OUTPUT_MIRROR_MAX = 120_000;

function syncOutputMirror() {
  const mirror = document.querySelector<HTMLPreElement>("#output-focus");
  const out = outputEl();
  if (mirror && out) {
    const full = out.textContent ?? "";
    // Keep the live Output pane full; mirror only the tail so opening Output stays snappy.
    mirror.textContent =
      full.length > OUTPUT_MIRROR_MAX ? full.slice(full.length - OUTPUT_MIRROR_MAX) : full;
    mirror.scrollTop = mirror.scrollHeight;
  }
  syncOutputPreview();
}

function outputPreviewVisible(): boolean {
  const root = document.querySelector<HTMLElement>("#output-preview");
  return Boolean(root && !root.hidden);
}

let previewSearchQuery = "";
let previewMatchIndex = 0;
let previewMatchCount = 0;

function previewSourceText(): string {
  return streamBuf || outputEl()?.textContent || "";
}

function renderOutputPreviewBody(opts?: { stickBottom?: boolean }) {
  const body = document.querySelector<HTMLPreElement>("#output-preview-body");
  if (!body) return;
  const text = previewSourceText();
  const atBottom =
    opts?.stickBottom ??
    body.scrollHeight - body.scrollTop - body.clientHeight < 48;
  const q = previewSearchQuery.trim();
  previewMatchCount = 0;
  if (!q) {
    body.textContent = text;
    previewMatchIndex = 0;
    updatePreviewSearchChrome();
    if (atBottom) body.scrollTop = body.scrollHeight;
    return;
  }
  const lower = text.toLowerCase();
  const needle = q.toLowerCase();
  const ranges: Array<{ start: number; end: number }> = [];
  let from = 0;
  while (from < text.length) {
    const found = lower.indexOf(needle, from);
    if (found === -1) break;
    ranges.push({ start: found, end: found + needle.length });
    from = found + needle.length;
  }
  previewMatchCount = ranges.length;
  if (previewMatchCount === 0) previewMatchIndex = 0;
  else if (previewMatchIndex >= previewMatchCount) {
    previewMatchIndex = previewMatchCount - 1;
  }
  let html = "";
  let cursor = 0;
  ranges.forEach((range, idx) => {
    html += escapeHtml(text.slice(cursor, range.start));
    const chunk = text.slice(range.start, range.end);
    const current = idx === previewMatchIndex ? " current" : "";
    html += `<mark class="preview-hit${current}" data-hit="${idx}">${escapeHtml(chunk)}</mark>`;
    cursor = range.end;
  });
  html += escapeHtml(text.slice(cursor));
  body.innerHTML = html;
  updatePreviewSearchChrome();
  const current = body.querySelector<HTMLElement>(
    `mark.preview-hit[data-hit="${previewMatchIndex}"]`,
  );
  if (current) {
    current.scrollIntoView({ block: "center", behavior: "smooth" });
  } else if (atBottom) {
    body.scrollTop = body.scrollHeight;
  }
}

function updatePreviewSearchChrome() {
  const count = document.querySelector("#output-preview-search-count");
  const prev = document.querySelector<HTMLButtonElement>("#btn-output-preview-prev");
  const next = document.querySelector<HTMLButtonElement>("#btn-output-preview-next");
  const q = previewSearchQuery.trim();
  if (count) {
    if (!q) count.textContent = "";
    else if (previewMatchCount === 0) count.textContent = "0 matches";
    else count.textContent = `${previewMatchIndex + 1} / ${previewMatchCount}`;
  }
  const enabled = previewMatchCount > 0;
  if (prev) prev.disabled = !enabled;
  if (next) next.disabled = !enabled;
}

function stepPreviewMatch(delta: number) {
  if (previewMatchCount <= 0) return;
  previewMatchIndex =
    (previewMatchIndex + delta + previewMatchCount) % previewMatchCount;
  renderOutputPreviewBody({ stickBottom: false });
}

function syncOutputPreview() {
  if (!outputPreviewVisible()) return;
  renderOutputPreviewBody();
}

function openOutputPreview() {
  const root = document.querySelector<HTMLElement>("#output-preview");
  const search = document.querySelector<HTMLInputElement>("#output-preview-search");
  if (!root) return;
  if (cmdkVisible()) cmdkClose();
  root.hidden = false;
  renderOutputPreviewBody({ stickBottom: true });
  search?.focus();
  search?.select();
}

function closeOutputPreview() {
  const root = document.querySelector<HTMLElement>("#output-preview");
  if (root) root.hidden = true;
}

function show(text: string) {
  streamBuf = text;
  const out = outputEl();
  if (out) {
    out.textContent = text;
    out.scrollTop = out.scrollHeight;
  }
  syncOutputMirror();
}


type ToastAction = {
  id: string;
  label: string;
  /** Visual cue — confirm · verify · open (auth / vendor) */
  icon?: "confirm" | "verify" | "open" | "continue";
  run: () => void;
};

function toastActionIcon(kind: ToastAction["icon"]): string {
  if (kind === "confirm") {
    return `<svg class="toast-action-ico" viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M6.5 11.2 3.2 7.9l1.1-1.1 2.2 2.2 4.4-4.4 1.1 1.1z"/></svg>`;
  }
  if (kind === "continue") {
    return `<svg class="toast-action-ico" viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M5.2 3.2v9.6L12.8 8z"/></svg>`;
  }
  if (kind === "verify") {
    return `<svg class="toast-action-ico" viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zm0 1.8a4.7 4.7 0 1 1 0 9.4 4.7 4.7 0 0 1 0-9.4zm-.7 2.2h1.4v3.2H7.3zm0 4.2h1.4V11H7.3z"/></svg>`;
  }
  if (kind === "open") {
    return `<svg class="toast-action-ico" viewBox="0 0 16 16" aria-hidden="true"><path fill="currentColor" d="M9.2 2.5h4.3v4.3h-1.5V5.1L8.5 8.6 7.4 7.5l3.5-3.5H9.2zm-5.5 1.2h4v1.5h-3.2v6.6h6.6V9.6H12.6v4.2H3.7z"/></svg>`;
  }
  return "";
}

function toast(
  message: string,
  kind: ToastKind = "info",
  ms = 3200,
  actions?: ToastAction[],
) {
  const host = document.querySelector<HTMLElement>("#toast-host");
  if (!host || !message.trim()) return;
  const el = document.createElement("div");
  el.className = `toast ${kind}${actions?.length ? " toast-actions" : ""}`;
  el.setAttribute("role", "status");
  const actionsHtml = actions?.length
    ? `<div class="toast-actions-row">${actions
        .map(
          (a) =>
            `<button type="button" class="toast-action" data-toast-action="${escapeHtml(a.id)}">${toastActionIcon(a.icon)}<span>${escapeHtml(a.label)}</span></button>`,
        )
        .join("")}</div>`
    : "";
  el.innerHTML = `<span class="toast-mark" aria-hidden="true"></span><div class="toast-body"><p class="toast-msg">${escapeHtml(message)}</p>${actionsHtml}</div>`;
  const dismiss = () => {
    if (el.dataset.leaving === "1") return;
    el.dataset.leaving = "1";
    window.setTimeout(() => el.remove(), 170);
  };
  el.querySelectorAll<HTMLButtonElement>("[data-toast-action]").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.stopPropagation();
      const id = btn.dataset.toastAction;
      const action = actions?.find((a) => a.id === id);
      dismiss();
      action?.run();
    });
  });
  el.addEventListener("click", (ev) => {
    if ((ev.target as HTMLElement).closest("[data-toast-action]")) return;
    dismiss();
  });
  host.appendChild(el);
  window.setTimeout(dismiss, actions?.length ? Math.max(ms, 7200) : ms);
}

/** Classify current gate for actionable toasts (avoid “official path” for local Auto). */
function gateToastKind(cur: PublishView["current"]): "continue" | "open" | "confirm" {
  if (!cur) return "confirm";
  const kind = (cur.kind ?? "").toLowerCase();
  const id = (cur.id ?? "").toLowerCase();
  if (kind === "auto" || id === "configure" || id === "doctor" || id === "dry_run") {
    return "continue";
  }
  if (isScopesStep(cur)) return "confirm";
  if (
    kind === "oauth" ||
    kind === "list" ||
    kind === "deploy" ||
    kind === "check" ||
    Boolean(cur.entry_url)
  ) {
    return "open";
  }
  if (kind === "sign" && id.includes("release") && !id.includes("dry")) return "open";
  if (kind === "human") {
    // Local file / pack gates — Confirm after Verify, not vendor Open.
    if (id.includes("legal") || id.includes("trust") || id.includes("scope")) return "confirm";
    return "open";
  }
  if (cur.run?.length && kind !== "human") return "continue";
  return "confirm";
}

function polishShipctlUserMessage(raw: string): string | null {
  const t = raw.trim();
  if (!t) return null;
  if (/after Open\/Run succeeds.*publish confirm/i.test(t)) {
    return "This step still needs Confirm after you finish Open / Run.";
  }
  if (/use confirm for this step/i.test(t)) {
    return "Use Confirm on Publish when this step is ready.";
  }
  if (/Confirm or Verify|still pending/i.test(t)) {
    return null; // handled by toastPublishGatePending
  }
  if (/unknown provider/i.test(t)) {
    return "That provider has no Portal steps — use Open dashboard on Deployment instead.";
  }
  if (/has no secret put CLI/i.test(t)) {
    return "This provider has no put CLI — open the vendor dashboard (or Learn more).";
  }
  if (
    /program not found|cannot find|No such file|is not recognized as an internal or external command|The system cannot find the file/i.test(
      t,
    )
  ) {
    return "CLI missing on PATH — install wrangler/vercel/netlify/orbit/signet or use Learn more / Open dashboard.";
  }
  // Drop raw CLI invocations from operator-facing toasts.
  if (/^orbityard\b/i.test(t) || /\borbityard publish\b/i.test(t)) {
    return "Finish the current checkpoint on Publish, then Confirm.";
  }
  return t;
}

/** Providers accepted by `orbityard portal --provider`. */
const PORTAL_PROVIDER_IDS = new Set([
  "cloudflare",
  "vercel",
  "netlify",
  "github",
  "polar",
  "neon",
  "supabase",
  "d1",
  "turso",
  "container",
  "firebase",
  "appwrite",
  "convex",
  "fly",
  "railway",
  "render",
  "digitalocean",
  "gumroad",
  "lemon",
  "stripe",
  "paddle",
  "creem",
  "waffo",
  "heroku",
  "amplify",
  "cloudrun",
  "azurestatic",
]);

function isPortalProvider(id: string | null | undefined): boolean {
  return Boolean(id && PORTAL_PROVIDER_IDS.has(id.toLowerCase()));
}

function cmdFailDetail(result: CmdResult): string {
  const raw = `${result.stderr}\n${result.stdout}`.trim();
  const polished = polishShipctlUserMessage(raw);
  if (polished) return polished.slice(0, 160);
  const line =
    raw
      .split(/\r?\n/)
      .map((l) => l.trim())
      .filter(Boolean)
      .find((l) => !l.startsWith("{") && !l.startsWith("[")) || raw;
  return (line || `exit ${result.code}`).slice(0, 160);
}

function isSoftCmdFailure(result: CmdResult): boolean {
  const err = `${result.stderr}\n${result.stdout}`.trim();
  return /Confirm or Verify|still pending|unknown provider|not a directory|has no secret put CLI|program not found|cannot find|No such file|is not recognized as an internal or external command|The system cannot find the file|auth expired|not logged in|please log in|login required|authentication required|re-?auth|unauthorized|token.*(expired|invalid)|orbit.*login|wrangler.*login|vercel.*login|netlify.*login/i.test(
    err,
  );
}

/** Class T — interactive orbityard in a real terminal (never headless stdin). */
async function openShipctlTerminal(
  args: string[],
  opts?: { title?: string; okToast?: string; meta?: string },
): Promise<boolean> {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return false;
  }
  if (!args.length) {
    toast("No orbityard args for terminal", "err");
    return false;
  }
  try {
    await invoke("open_orbityard_terminal", {
      project,
      args,
      title: opts?.title ?? "Orbit Yard",
    });
    if (opts?.meta) {
      appendStream({ stream: "meta", text: opts.meta });
    }
    if (opts?.okToast) {
      toast(opts.okToast, "ok", 5500);
    }
    return true;
  } catch (err) {
    appendStream({ stream: "stderr", text: String(err) });
    toast(`Terminal failed — ${String(err).slice(0, 120)}`, "err", 7000, [
      { id: "preview", label: "Preview log", icon: "open", run: () => openOutputPreview() },
    ]);
    return false;
  }
}

async function openPortalLoginTerminal(provider: string) {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return;
  }
  if (!isPortalProvider(provider)) {
    toast("No CLI login for this provider — use Open on the step URL", "info");
    return;
  }
  await openShipctlTerminal(
    ["portal", "--project", project, "--provider", provider, "--login"],
    {
      title: "Orbit Yard portal login",
      meta: `Launched terminal: orbityard portal --provider ${provider} --login — complete OAuth there.`,
      okToast: `Login CLI opened for ${provider} — finish in the terminal`,
    },
  );
}

/** Provider id for Launch/Publish Login CLI from step id / GitHub URL. */
function loginProviderForStep(step: {
  id?: string | null;
  kind?: string | null;
  entry_url?: string | null;
} | null | undefined): string | null {
  if (!step) return null;
  const id = (step.id ?? "").toLowerCase();
  // Collapsed host gate opens Deployment — not a single-provider Login CLI.
  if (id === "oauth.hosts" || id === "deploy.hosts") return null;
  const oauth = /^oauth\.(.+)$/.exec(id);
  if (oauth) return oauth[1];
  const url = (step.entry_url ?? "").toLowerCase();
  if (
    id.includes("github") ||
    id === "signet.release" ||
    id === "ship.desktop_cut" ||
    url.includes("github.com")
  ) {
    return "github";
  }
  return null;
}

/** Header Open / card Run — interactive terminal when the gate needs a TTY. */
async function openLaunchCurrentGate() {
  const project = projectPath();
  if (!project) return;
  const cur = lastLaunch?.current;
  const related = (cur?.desktop_view ?? "").trim();
  if (related && RELATED_VIEW_LABELS[related]) {
    if (related === "platforms") {
      openPlatformsCatalog();
    } else if (related === "integrations") {
      setView("integrations");
      renderIntegrations();
    } else {
      setView(related);
    }
    toast(`${RELATED_VIEW_LABELS[related]} — finish there, then Verify → Confirm`, "info", 5000);
    return;
  }
  const kind = (cur?.kind ?? "").toLowerCase();
  const loginProvider = loginProviderForStep(cur);

  if (kind === "oauth" && loginProvider) {
    await openPortalLoginTerminal(loginProvider);
    window.setTimeout(() => {
      void refreshLaunch();
    }, 1500);
    return;
  }

  const needsTerminal =
    Boolean(cur?.run?.length) ||
    kind === "oauth" ||
    kind === "paste" ||
    kind === "sign" ||
    kind === "deploy";
  if (needsTerminal) {
    const opened = await openShipctlTerminal(
      ["launch", "--project", project, "open"],
      {
        title: "Orbit Yard launch",
        meta: "Launched terminal: orbityard launch open — complete auth / run there, then Verify/Confirm here.",
        okToast: "Terminal opened for this step",
      },
    );
    if (opened) {
      window.setTimeout(() => {
        void refreshLaunch();
      }, 1500);
    } else {
      void launchAction(["open"]);
    }
    return;
  }

  if (cur?.entry_url) {
    await openUrl(cur.entry_url);
    return;
  }
  void launchAction(["open"]);
}

/** Pending Next / pause — offer the right next action, not a Verify red herring on Auto steps. */
function toastPublishGatePending() {
  const cur = lastPublish?.current;
  const gate = gateToastKind(cur);
  const related = (cur?.desktop_view ?? "").trim();
  const title = cur?.title
    ? cur.title.replace(/\s*—\s*.*$/, "").trim() || cur.title
    : "this step";
  const actions: ToastAction[] = [];

  if (gate === "continue") {
    actions.push({
      id: "continue",
      label: "Continue",
      icon: "continue",
      run: () => {
        setView("publish");
        if (publishUiMode() === "stages") {
          const primary = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
          if (primary && !primary.disabled) {
            primary.click();
            return;
          }
        }
        document.querySelector<HTMLButtonElement>("#btn-publish-continue")?.click();
      },
    });
    toast(
      `«${title}» is automatic — press Continue. Studio checks locally and advances; you don’t need Verify.`,
      "info",
      7500,
      actions,
    );
    return;
  }

  if (gate === "open") {
    const openLabel =
      (cur?.kind ?? "").toLowerCase() === "oauth"
        ? "Sign in"
        : related && RELATED_VIEW_LABELS[related]
          ? RELATED_VIEW_LABELS[related]
          : "Open step";
    actions.push({
      id: "open",
      label: openLabel,
      icon: "open",
      run: () => {
        setView("publish");
        if (publishUiMode() === "stages") {
          const primary = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
          if (primary?.dataset.stageAction === "open") {
            primary.click();
            return;
          }
        }
        document.querySelector<HTMLButtonElement>("#btn-publish-open")?.click();
      },
    });
  }

  if (gate === "confirm" || gate === "open") {
    actions.push({
      id: "confirm",
      label: "Confirm",
      icon: "confirm",
      run: () => {
        setView("publish");
        if (publishUiMode() === "stages") {
          const primary = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
          if (primary?.dataset.stageAction === "confirm" && !primary.disabled) {
            primary.click();
            return;
          }
        }
        document.querySelector<HTMLButtonElement>("#btn-publish-confirm")?.click();
      },
    });
  }

  // Verify only when Confirm/Open is the honesty path — not for Auto Continue.
  if (gate === "confirm") {
    actions.unshift({
      id: "verify",
      label: "Verify",
      icon: "verify",
      run: () => {
        setView("publish");
        void publishAction(["verify"]);
      },
    });
  }

  const message =
    gate === "open"
      ? `«${title}» — Login CLI or Open portal for auth, then Verify → Confirm.`
      : `Confirm «${title}» when ready (Verify checks local evidence first).`;

  toast(message, "err", 8000, actions);
}

function appendStream(line: StreamLine) {
  const prefix =
    line.stream === "stderr" ? "[err] " : line.stream === "meta" ? "[meta] " : "";
  streamBuf = `${streamBuf}${streamBuf && !streamBuf.endsWith("\n") ? "\n" : ""}${prefix}${line.text}\n`;
  const out = outputEl();
  if (out) {
    out.textContent = streamBuf;
    out.scrollTop = out.scrollHeight;
  }
  syncOutputMirror();

  // Self-host: setup ends at health check; remaining busy is intentional serve.
  if (selfhostServing && /serving until Cancel/i.test(line.text)) {
    noteRunPhase("Serving", { resetElapsed: true });
    void refreshShipState().then(() => paintDeployResultsBay());
  } else if (selfhostServing && /selfhost · check ok/i.test(line.text)) {
    noteRunPhase("Check ok", { resetElapsed: false });
    const m = line.text.match(/https?:\/\/127\.0\.0\.1:\d+\/?/);
    if (pendingSelfhostOpenLive && m?.[0]) {
      pendingSelfhostOpenLive = false;
      void openUrl(m[0]).then(() => toast("Opened live URL", "ok", 3500));
    }
    void refreshShipState().then(() => paintDeployResultsBay());
  }
}

type StatusProbeState = "checking" | "ok" | "missing" | "guide";

type StatusProbeRow = {
  id: "self-sign" | "official-sign" | "deploy";
  label: string;
  state: StatusProbeState;
  detail: string;
  suggestion: string;
  badge: string;
  action?: { id: string; label: string };
  actions?: Array<{ id: string; label: string }>;
};

function statusProbeIcon(id: StatusProbeRow["id"]): string {
  if (id === "self-sign") {
    return `<svg class="status-probe-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M12.2 2.4a3.2 3.2 0 0 0-2.3 5.4l-6.7 6.7v4.2h4.2l1.1-1.1v-1.6h1.6v-1.6h1.6l2.4-2.4a3.2 3.2 0 1 0-1.9-9.6zm0 1.8a1.4 1.4 0 1 1 0 2.8 1.4 1.4 0 0 1 0-2.8z"/></svg>`;
  }
  if (id === "official-sign") {
    return `<svg class="status-probe-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M4.5 4.2h15v2.2H4.5zm1.8 3.8h11.4v11.2c0 .7-.5 1.2-1.2 1.2H7.5c-.7 0-1.2-.5-1.2-1.2zm3.2 2.4v1.8h4.8V10.4zm0 3.4v1.8h3.2v-1.8z"/></svg>`;
  }
  return `<svg class="status-probe-ico" viewBox="0 0 24 24" aria-hidden="true"><path fill="currentColor" d="M3.4 12.1 19.8 4.6l-3.4 15.2-4.1-4.7-3.2 3.1v-4.4l-5.7-1.7zm8.3 1.1 2.2 2.5 1.8-8.1-9.1 4.2z"/></svg>`;
}

function bindStatusProbeActions(host: HTMLElement) {
  host.querySelectorAll<HTMLButtonElement>("[data-probe-action]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const action = btn.dataset.probeAction;
      if (action === "open-sign") {
        setView("sign");
        return;
      }
      if (action === "view-paths" || action === "choose-platform") {
        openSignCatalog({ selectId: "apple-sign" });
        return;
      }
      if (action === "choose-host") {
        openPlatformsCatalog({
          selectId: preferredHostingPlatformId(lastDetected) ?? "selfhost",
        });
        return;
      }
      if (action === "intent-local") {
        applyShipIntent("local", { rebuild: true });
        toast("Local intent — hosted deploy / live check omitted", "ok", 4500);
        return;
      }
      if (action === "recheck") {
        void refreshStatusProbes({
          animate: true,
          views: host.id === "status-probe-publish" ? ["publish"] : ["sign"],
        });
      }
    });
  });
}

function renderStatusProbe(
  host: HTMLElement | null,
  opts: { title: string; phase: "checking" | "ready"; rows: StatusProbeRow[]; compact?: boolean },
) {
  if (!host) return;
  host.hidden = false;
  host.classList.toggle("status-probe-compact", Boolean(opts.compact));
  host.dataset.phase = opts.phase;
  const rowsHtml = opts.rows
    .map((r) => {
      const acts = r.actions ?? (r.action ? [r.action] : []);
      const actionHtml = acts
        .map(
          (a) =>
            `<button type="button" class="status-probe-cta" data-probe-action="${escapeHtml(a.id)}">${escapeHtml(a.label)}</button>`,
        )
        .join("");
      return `<article class="status-probe-card" data-state="${escapeHtml(r.state)}" data-probe-id="${escapeHtml(r.id)}">
        <div class="status-probe-card-top">
          <span class="status-probe-glyph" aria-hidden="true">${statusProbeIcon(r.id)}</span>
          <span class="status-probe-badge">${escapeHtml(r.badge)}</span>
        </div>
        <h4 class="status-probe-label">${escapeHtml(r.label)}</h4>
        <p class="status-probe-detail">${escapeHtml(r.detail)}</p>
        <p class="status-probe-suggest">${escapeHtml(r.suggestion)}</p>
        ${actionHtml ? `<div class="status-probe-ctas">${actionHtml}</div>` : ""}
      </article>`;
    })
    .join("");
  const headLabel =
    opts.phase === "checking" ? "Running local probe…" : escapeHtml(opts.title);
  const scriptHint =
    opts.phase === "checking"
      ? `<span class="status-probe-script" aria-hidden="true">orbityard pulse · sign-paths</span>`
      : `<button type="button" class="status-probe-recheck" data-probe-action="recheck">Check again</button>`;
  host.innerHTML = `<div class="status-probe-head" data-phase="${opts.phase}">
      <div class="status-probe-head-main">
        <span class="status-probe-spinner" aria-hidden="true"></span>
        <div>
          <span class="status-probe-kicker">Local check</span>
          <span class="status-probe-title">${headLabel}</span>
        </div>
      </div>
      ${scriptHint}
    </div>
    <div class="status-probe-lanes">${rowsHtml}</div>`;
  bindStatusProbeActions(host);
}

function buildSignDeployProbeRows(
  plan: SignPortal | null,
  pulse: ProjectPulse | null,
): StatusProbeRow[] {
  const signetOk = Boolean(pulse?.tools?.signet_found);
  const signetLoc = pulse?.tools?.signet_version;
  const selfPaths = (plan?.paths ?? []).filter((p) => (p.kind ?? "") === "self");
  const official = (plan?.paths ?? []).filter((p) => (p.kind ?? "") === "official");
  const rows: StatusProbeRow[] = [
    {
      id: "self-sign",
      label: "Self-sign",
      state: signetOk ? "ok" : "missing",
      badge: signetOk ? "Ready" : "Needs setup",
      detail: signetOk
        ? `Signet ready${signetLoc ? ` · ${signetLoc}` : ""}${
            selfPaths.length ? ` · ${selfPaths.length} path(s)` : ""
          }`
        : "Signet not on PATH — needed for local desktop cuts",
      suggestion: signetOk
        ? "Ready for local desktop cuts"
        : "Install Signet, then Check again",
      action: signetOk
        ? { id: "open-sign", label: "Open Sign" }
        : { id: "recheck", label: "Check again" },
    },
  ];
  if (official.length) {
    const names = official
      .map((p) => p.title ?? p.id ?? "")
      .filter(Boolean)
      .slice(0, 4)
      .join(" · ");
    rows.push({
      id: "official-sign",
      label: "Official signing",
      state: "guide",
      badge: "Guide",
      detail: `${names}${official.length > 4 ? "…" : ""} — finish on vendor UIs`,
      suggestion: "Apple / Microsoft / store — Studio guides; never auto-Done",
      action: { id: "choose-platform", label: "Choose platform" },
    });
  } else {
    rows.push({
      id: "official-sign",
      label: "Official signing",
      state: "guide",
      badge: "Optional",
      detail: "No store lanes for this layout",
      suggestion: "Self-sign is enough unless you ship App Store / MSIX",
      action: { id: "choose-platform", label: "Choose platform" },
    });
  }

  const dep = pulse?.deploy;
  const deployOk = deployEvidenceFromPulse(pulse);
  const selfhostOk = isSelfhostLocalEvidence(pulse);
  const linked =
    dep?.signal === "vercel_linked" || dep?.signal === "yard_configured";
  const deployActions: Array<{ id: string; label: string }> = [
    { id: "choose-host", label: "Choose host" },
  ];
  if (!deployOk && !linked && !selfhostOk) {
    deployActions.push({ id: "intent-local", label: "Use Local" });
  }
  rows.push({
    id: "deploy",
    label: "Deploy",
    state: deployOk ? "ok" : selfhostOk || linked ? "guide" : "missing",
    badge: deployOk ? "Ready" : selfhostOk ? "Self-host" : linked ? "Linked" : "No signal",
    detail: deployOk
      ? hostedDeployUrls(dep)[0] || dep?.detail || "Prior hosted deploy evidence found"
      : selfhostOk
        ? dep?.urls?.[0] || dep?.detail || "Local self-host check ok"
        : linked
          ? dep?.detail || "Host linked — deploy when releasing"
          : dep?.detail || "No Orbit / last-run / host link yet",
    suggestion: deployOk
      ? "Prior hosted evidence found — redeploy when you cut again"
      : selfhostOk
        ? "Self-host is local-auto only — distinct from cloud. Choose host for Cloudflare / Vercel / …"
        : linked
          ? "Configured locally — run deploy on release"
          : "No hosted URL yet — pick a host, or Use Local for desktop-only",
    actions: deployActions,
  });
  return rows;
}

function checkingProbeRows(): StatusProbeRow[] {
  return [
    {
      id: "self-sign",
      label: "Self-sign",
      state: "checking",
      badge: "Probing",
      detail: "Looking for Signet…",
      suggestion: "Reading local tools",
    },
    {
      id: "official-sign",
      label: "Official signing",
      state: "checking",
      badge: "Probing",
      detail: "Scanning Apple / Windows / store lanes…",
      suggestion: "Layout only — no vendor login",
    },
    {
      id: "deploy",
      label: "Deploy",
      state: "checking",
      badge: "Probing",
      detail: "Checking last-run / host signals…",
      suggestion: "Pulse deploy evidence",
    },
  ];
}

async function refreshStatusProbes(opts?: { animate?: boolean; views?: Array<"sign" | "publish"> }) {
  const views = opts?.views ?? ["sign", "publish"];
  const animate = opts?.animate !== false;
  const signHost = document.querySelector<HTMLElement>("#status-probe-sign");
  const pubHost = document.querySelector<HTMLElement>("#status-probe-publish");

  if (animate) {
    if (views.includes("sign") && signHost) {
      renderStatusProbe(signHost, {
        title: "Sign & deploy",
        phase: "checking",
        rows: checkingProbeRows(),
      });
    }
    if (views.includes("publish") && pubHost && projectPath()) {
      renderStatusProbe(pubHost, {
        title: "Sign & deploy",
        phase: "checking",
        rows: checkingProbeRows(),
        compact: true,
      });
    }
    await new Promise((r) => window.setTimeout(r, 420));
  }

  if (!projectPath()) {
    if (signHost) signHost.hidden = true;
    if (pubHost) pubHost.hidden = true;
    return;
  }

  let plan = (await loadJsonCmd(["sign-paths", "--project", projectPath()])) as SignPortal | null;
  if (!plan && views.includes("sign")) {
    plan = null;
  }
  if (views.includes("sign") && plan) applySignPaths(plan);

  // Pulse may already be warm from Dashboard; refresh lightly if missing tools.
  if (!lastPulse?.tools && projectPath()) {
    const pulse = (await loadJsonCmd(["pulse", "--project", projectPath()])) as ProjectPulse | null;
    if (pulse) {
      lastPulse = pulse;
      applyPulse(pulse);
    }
  }

  const rows = buildSignDeployProbeRows(plan, lastPulse);
  if (views.includes("sign") && signHost) {
    renderStatusProbe(signHost, {
      title: "Sign & deploy",
      phase: "ready",
      rows,
    });
  }
  if (views.includes("publish") && pubHost) {
    renderStatusProbe(pubHost, {
      title: "Sign & deploy",
      phase: "ready",
      rows,
      compact: true,
    });
  }
}



let activeViewId = "dashboard";
/** Stack of prior views for ← Back (not including the current view). */
const viewHistory: string[] = [];
const VIEW_HISTORY_MAX = 24;
let setViewFromBack = false;

function publishMidFlight(): boolean {
  return Boolean(lastPublish?.steps?.length && !lastPublish.finished);
}

function syncBackToPublish() {
  const btn = document.querySelector<HTMLButtonElement>("#btn-back-publish");
  if (!btn) return;
  const show = publishMidFlight() && activeViewId !== "publish" && Boolean(projectPath());
  btn.hidden = !show;
  btn.disabled = !show;
}

function syncBackView() {
  const btn = document.querySelector<HTMLButtonElement>("#btn-back-view");
  if (!btn) return;
  const prev = viewHistory[viewHistory.length - 1];
  const show = Boolean(prev && prev !== activeViewId);
  btn.hidden = !show;
  btn.disabled = !show;
  if (show && prev) {
    const label = VIEW_META[prev]?.title ?? prev;
    btn.title = `Back to ${label} (Alt+←)`;
    btn.textContent = `← ${label}`;
  } else {
    btn.textContent = "← Back";
    btn.title = "Previous page (Alt+←)";
  }
}

/** Return to the previous view, if any. */
function goBackView(): boolean {
  while (viewHistory.length) {
    const prev = viewHistory.pop()!;
    if (prev === activeViewId || !VIEW_META[prev]) continue;
    setViewFromBack = true;
    setView(prev);
    setViewFromBack = false;
    return true;
  }
  syncBackView();
  return false;
}

function afterPaint(fn: () => void) {
  requestAnimationFrame(() => {
    requestAnimationFrame(fn);
  });
}

const NAV_SECTION_DEFAULTS: Record<string, boolean> = {
  ship: true,
  more: true,
  run: true,
};

const VIEW_TO_NAV_SECTION: Record<string, string> = {
  dashboard: "ship",
  publish: "ship",
  scopes: "ship",
  sign: "ship",
  env: "ship",
  platforms: "ship",
  integrations: "ship",
  assist: "more",
  launch: "more",
  portal: "more",
  ritual: "more",
  tools: "more",
  output: "run",
};

function loadNavSectionState(): Record<string, boolean> {
  try {
    const raw = localStorage.getItem(NAV_SECTIONS_KEY);
    if (!raw) return { ...NAV_SECTION_DEFAULTS };
    const parsed = JSON.parse(raw) as Record<string, boolean>;
    return { ...NAV_SECTION_DEFAULTS, ...parsed };
  } catch {
    return { ...NAV_SECTION_DEFAULTS };
  }
}

function saveNavSectionState(state: Record<string, boolean>) {
  localStorage.setItem(NAV_SECTIONS_KEY, JSON.stringify(state));
}

function wireNavSections() {
  const state = loadNavSectionState();
  document.querySelectorAll<HTMLDetailsElement>("details.nav-section[data-nav-section]").forEach((el) => {
    const id = el.dataset.navSection;
    if (!id) return;
    el.open = state[id] !== false;
    el.addEventListener("toggle", () => {
      const next = loadNavSectionState();
      next[id] = el.open;
      saveNavSectionState(next);
    });
  });
}

function ensureNavSectionOpen(viewId: string) {
  const sectionId = VIEW_TO_NAV_SECTION[viewId];
  if (!sectionId) return;
  const el = document.querySelector<HTMLDetailsElement>(
    `details.nav-section[data-nav-section="${sectionId}"]`,
  );
  if (!el || el.open) return;
  el.open = true;
  const next = loadNavSectionState();
  next[sectionId] = true;
  saveNavSectionState(next);
}

function setView(id: string) {
  if (!VIEW_META[id]) return;
  const prev = activeViewId;
  if (prev !== id && !setViewFromBack && VIEW_META[prev]) {
    if (viewHistory[viewHistory.length - 1] !== prev) {
      viewHistory.push(prev);
      if (viewHistory.length > VIEW_HISTORY_MAX) viewHistory.shift();
    }
  }
  activeViewId = id;
  ensureNavSectionOpen(id);
  document.querySelectorAll<HTMLElement>(".view").forEach((el) => {
    const on = el.dataset.view === id;
    el.classList.toggle("active", on);
    el.hidden = !on;
  });
  document.querySelectorAll<HTMLButtonElement>(".nav-item").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.nav === id);
  });
  const meta = VIEW_META[id];
  const title = document.querySelector("#view-title");
  const desc = document.querySelector("#view-desc");
  if (title) title.textContent = meta.title;
  if (desc) {
    const path = projectPath();
    desc.textContent = path ? `${projectName(path)} · ${meta.desc}` : meta.desc;
  }
  // Avoid full identity + sidebar rebuild on every nav (multi-second freeze on large projects).
  syncBackToPublish();
  syncBackView();
  if (id === "output") syncOutputMirror();
  if (id === "integrations") renderIntegrations();
  if (id === "platforms") renderPlatforms();
  if (id === "sign") renderSignCatalog();
  if (id === "sign" && projectPath()) {
    afterPaint(() => {
      void refreshStatusProbes({ views: ["sign"], animate: true });
    });
  }
  if (id === "publish" && projectPath()) {
    afterPaint(() => {
      void refreshStatusProbes({ views: ["publish"], animate: !lastPulse?.tools });
    });
  }
}

function commandItems(): CmdItem[] {
  return [
    {
      id: "nav-back",
      title: "Go back",
      keywords: "previous page back history",
      group: "Navigate",
      run: () => {
        if (!goBackView()) toast("No previous page", "info", 2000);
      },
    },
    {
      id: "nav-dashboard",
      title: "Go to Dashboard",
      keywords: "home overview health",
      group: "Navigate",
      run: () => setView("dashboard"),
    },
    {
      id: "nav-publish",
      title: "Go to Publish",
      keywords: "publish wizard portal minute ship",
      group: "Navigate",
      run: () => setView("publish"),
    },
    {
      id: "nav-assist",
      title: "Go to Assist",
      keywords: "wizard fullstack deploy help checklist",
      group: "Navigate",
      run: () => setView("assist"),
    },
    {
      id: "nav-scopes",
      title: "Go to Targets",
      keywords: "targets scopes web api desktop directory",
      group: "Navigate",
      run: () => setView("scopes"),
    },
    {
      id: "nav-env",
      title: "Go to ENV / tokens",
      keywords: "secrets env put create retrieve",
      group: "Navigate",
      run: () => setView("env"),
    },
    {
      id: "nav-sign",
      title: "Go to Sign",
      keywords: "self-sign official apple windows play",
      group: "Navigate",
      run: () => setView("sign"),
    },
    {
      id: "nav-launch",
      title: "Go to Launch",
      keywords: "guided ship release deploy",
      group: "Navigate",
      run: () => setView("launch"),
    },
    {
      id: "nav-portal",
      title: "Go to Portal",
      keywords: "human secrets oauth paste",
      group: "Navigate",
      run: () => setView("portal"),
    },
    {
      id: "nav-integrations",
      title: "Go to Suite",
      keywords: "suite strata velocity truss scaffold template payment email polar",
      group: "Navigate",
      run: () => setView("integrations"),
    },
    {
      id: "nav-platforms",
      title: "Go to Deployment",
      keywords: "deploy host vercel cloudflare netlify selfhost orbit fly railway pages hosting",
      group: "Navigate",
      run: () => openPlatformsCatalog(),
    },
    {
      id: "nav-selfhost",
      title: "Self-host (Deployment)",
      keywords: "selfhost local auto stream docker cousin",
      group: "Navigate",
      run: () => openPlatformsCatalog({ selectId: "selfhost" }),
    },
    {
      id: "nav-ritual",
      title: "Go to Ritual",
      keywords: "sign_args deploy_args configure",
      group: "Navigate",
      run: () => setView("ritual"),
    },
    {
      id: "nav-tools",
      title: "Go to Tools",
      keywords: "doctor sign deploy flow vault",
      group: "Navigate",
      run: () => setView("tools"),
    },
    {
      id: "nav-output",
      title: "Go to Output",
      keywords: "console log",
      group: "Navigate",
      run: () => setView("output"),
    },
    {
      id: "act-update-check",
      title: "Check for updates",
      keywords: "update release github download version",
      group: "Run",
      run: () => {
        void runUpdateCheck();
      },
    },
    {
      id: "act-output-preview",
      title: "Preview output",
      keywords: "console preview enlarge json",
      group: "Run",
      run: () => openOutputPreview(),
    },
    {
      id: "act-open",
      title: "Open folder",
      keywords: "project bind",
      group: "Project",
      run: () => document.querySelector<HTMLButtonElement>("#btn-open")?.click(),
    },
    {
      id: "act-switch",
      title: "Switch project",
      keywords: "recent directory bind",
      group: "Project",
      run: () => toggleProjectSwitcher(true),
    },
    {
      id: "act-publish",
      title: "Refresh Publish portal",
      keywords: "publish wizard minute",
      group: "Ship",
      run: () => {
        setView("publish");
        document.querySelector<HTMLButtonElement>("#btn-publish")?.click();
      },
    },
    {
      id: "act-assist",
      title: "Refresh deploy assist",
      keywords: "wizard fullstack",
      group: "Ship",
      run: () => {
        setView("assist");
        document.querySelector<HTMLButtonElement>("#btn-assist")?.click();
      },
    },
    {
      id: "act-launch",
      title: "Refresh Launch plan",
      keywords: "guided launch",
      group: "Ship",
      run: () => {
        setView("launch");
        document.querySelector<HTMLButtonElement>("#btn-launch")?.click();
      },
    },
    {
      id: "act-human",
      title: "Start Human portal",
      keywords: "paste polar github secrets",
      group: "Ship",
      run: () => {
        setView("portal");
        document.querySelector<HTMLButtonElement>("#btn-human")?.click();
      },
    },
    {
      id: "act-polar",
      title: "Set up Polar",
      keywords: "commerce checkout refund listing polar dashboard",
      group: "Ship",
      run: () => setupPolarPortal(),
    },
    {
      id: "act-doctor",
      title: "Run Doctor",
      keywords: "signet orbit health",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-doctor")?.click();
      },
    },
    {
      id: "act-wizard",
      title: "Run Wizard",
      keywords: "offline prep",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-wizard")?.click();
      },
    },
    {
      id: "act-configure",
      title: "Configure studio.json",
      keywords: "configure ritual",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-configure")?.click();
      },
    },
    {
      id: "act-sign",
      title: "Sign",
      keywords: "signet build",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-sign")?.click();
      },
    },
    {
      id: "act-deploy",
      title: "Deploy",
      keywords: "orbit ship network",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-deploy")?.click();
      },
    },
    {
      id: "act-flow-dry",
      title: "Flow dry-run",
      keywords: "offline plan",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-flow-dry")?.click();
      },
    },
    {
      id: "act-flow",
      title: "Run flow",
      keywords: "sign deploy pipeline",
      group: "Tools",
      run: () => {
        setView("tools");
        document.querySelector<HTMLButtonElement>("#btn-flow")?.click();
      },
    },
    {
      id: "act-vault",
      title: "Export vault",
      keywords: "km clavis secrets backup",
      group: "Tools",
      run: () => {
        setView("portal");
        document.querySelector<HTMLButtonElement>("#btn-vault")?.click();
      },
    },
    {
      id: "act-portal",
      title: "Load portal steps",
      keywords: "providers oauth",
      group: "Ship",
      run: () => {
        setView("portal");
        document.querySelector<HTMLButtonElement>("#btn-portal")?.click();
      },
    },
    {
      id: "act-secrets",
      title: "Load secret hints",
      keywords: "paste hints",
      group: "Ship",
      run: () => {
        setView("portal");
        document.querySelector<HTMLButtonElement>("#btn-secrets")?.click();
      },
    },
  ];
}

let cmdkIndex = 0;
let cmdkFiltered: CmdItem[] = [];

function cmdkOpen() {
  const root = document.querySelector<HTMLElement>("#cmdk");
  const input = document.querySelector<HTMLInputElement>("#cmdk-input");
  if (!root || !input) return;
  root.hidden = false;
  input.value = "";
  cmdkIndex = 0;
  renderCmdk("");
  queueMicrotask(() => input.focus());
}

function cmdkClose() {
  const root = document.querySelector<HTMLElement>("#cmdk");
  if (root) root.hidden = true;
}

function cmdkVisible(): boolean {
  const root = document.querySelector<HTMLElement>("#cmdk");
  return !!root && !root.hidden;
}

function renderCmdk(query: string) {
  const list = document.querySelector<HTMLElement>("#cmdk-list");
  if (!list) return;
  const q = query.trim().toLowerCase();
  cmdkFiltered = commandItems().filter((item) => {
    if (!q) return true;
    const hay = `${item.title} ${item.keywords} ${item.group}`.toLowerCase();
    return hay.includes(q);
  });
  if (cmdkIndex >= cmdkFiltered.length) cmdkIndex = Math.max(0, cmdkFiltered.length - 1);
  list.innerHTML = cmdkFiltered.length
    ? cmdkFiltered
        .map(
          (item, i) => `<li>
      <button type="button" class="cmdk-item${i === cmdkIndex ? " active" : ""}" data-cmd-idx="${i}">
        <span>${escapeHtml(item.title)}</span>
        <span class="meta">${escapeHtml(item.group)}</span>
      </button>
    </li>`,
        )
        .join("")
    : `<li><button type="button" class="cmdk-item" disabled><span>No matches</span></button></li>`;
  list.querySelectorAll<HTMLButtonElement>(".cmdk-item[data-cmd-idx]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const idx = Number(btn.dataset.cmdIdx);
      runCmdk(idx);
    });
  });
}

function runCmdk(idx: number) {
  const item = cmdkFiltered[idx];
  if (!item) return;
  cmdkClose();
  item.run();
}


function setPill(id: string, kind: "ok" | "bad" | "muted", label: string) {
  const el = document.querySelector<HTMLElement>(`#${id}`);
  if (!el) return;
  el.className = `pill ${kind}`;
  el.textContent = label;
}

function setStep(id: string, state: "idle" | "active" | "done" | "fail") {
  const el = document.querySelector<HTMLElement>(`.step[data-step="${id}"]`);
  if (!el) return;
  el.classList.remove("active", "done", "fail");
  if (state !== "idle") el.classList.add(state);
}

function resetSteps() {
  for (const id of ["doctor", "portal", "sign", "deploy"]) {
    setStep(id, "idle");
  }
}






function describeKind(detected?: Detected): string {
  if (!detected) return "";
  const bits: string[] = [];
  if (detected.tauri) bits.push("Desktop (Tauri)");
  else if (detected.wrangler) bits.push("API / Cloudflare Worker");
  else if (detected.vercel) bits.push("Web (Vercel)");
  else if (detected.netlify) bits.push("Web (Netlify)");
  else if (detected.package_json) bits.push("Node project");
  if (detected.polar) bits.push("Polar listing");
  if (detected.github) bits.push("GitHub");
  if (detected.signet_toml && !detected.tauri) bits.push("Signet");
  return bits.join(" · ");
}

function syncProjectIdentity() {
  const path = projectPath();
  const bound = Boolean(path);
  const name = bound ? projectName(path) : "Choose a project";
  const parent = bound ? parentPath(path) : "Open a folder to start shipping";
  document.body.classList.toggle("bound", bound);

  const chromeName = document.querySelector("#chrome-name");
  const chromePath = document.querySelector("#chrome-path");
  if (chromeName) chromeName.textContent = name;
  if (chromePath) chromePath.textContent = parent;
  const chrome = document.querySelector<HTMLButtonElement>("#chrome-project");
  if (chrome) {
    chrome.title = bound ? `Switch project · ${path}` : "Open or switch project";
  }

  const sideName = document.querySelector("#sidebar-project-name");
  if (sideName) sideName.textContent = bound ? name : "No project";
  const sideSession = document.querySelector("#sidebar-session");
  if (sideSession) sideSession.textContent = bound ? name : "No project bound";

  const crumb = document.querySelector<HTMLElement>("#session-crumb");
  const crumbName = document.querySelector("#crumb-name");
  const crumbPath = document.querySelector("#crumb-path");
  if (crumb) crumb.hidden = !bound;
  if (crumbName) crumbName.textContent = name;
  if (crumbPath) crumbPath.textContent = path;

  const empty = document.querySelector<HTMLElement>("#session-empty");
  const boundEl = document.querySelector<HTMLElement>("#session-bound");
  if (empty) empty.hidden = bound;
  if (boundEl) boundEl.hidden = !bound;
  syncWorkflowCards();
  const sessionName = document.querySelector("#session-name");
  const sessionPath = document.querySelector("#session-path");
  const sessionKind = document.querySelector("#session-kind");
  if (sessionName) sessionName.textContent = name;
  if (sessionPath) sessionPath.textContent = path;
  if (sessionKind) sessionKind.textContent = describeKind(lastDetected);

  const health = document.querySelector<HTMLElement>("#health");
  if (health) health.hidden = !bound;

  const nowPrimary = document.querySelector<HTMLButtonElement>("#now-primary");
  const nowSwitch = document.querySelector<HTMLButtonElement>("#now-switch");
  if (nowSwitch) nowSwitch.disabled = false;
  if (nowPrimary) {
    nowPrimary.disabled = running;
    if (!bound) {
      setCtaLabel(nowPrimary, "Open folder…");
      nowPrimary.dataset.pulseId = "open";
      nowPrimary.dataset.pulseView = "";
      setNowCtaState("open");
    } else if (lastPulse?.now?.primary?.label) {
      setCtaLabel(nowPrimary, polishCtaLabel(lastPulse.now.primary.label));
      nowPrimary.dataset.pulseId = lastPulse.now.primary.id ?? "publish_start";
      nowPrimary.dataset.pulseView = lastPulse.now.primary.view || "publish";
      setNowCtaState(ctaStateFromPulseId(lastPulse.now.primary.id, lastPulse.now.primary.label));
    } else {
      const label = lastPublish?.finished
        ? "Review publish"
        : lastPublish?.current
          ? "Continue publishing"
          : "Start publishing";
      setCtaLabel(nowPrimary, label);
      nowPrimary.dataset.pulseId = "publish_start";
      nowPrimary.dataset.pulseView = "publish";
      setNowCtaState(
        lastPublish?.finished ? "review" : lastPublish?.current ? "continue" : "start",
      );
    }
  }
  syncNowQuick();
  paintNextShip();
}

function setCtaLabel(btn: HTMLButtonElement, label: string) {
  const span = btn.querySelector<HTMLElement>(".cta-label");
  if (span) span.textContent = label;
  else btn.textContent = label;
}

function canShowPolarSetup(): boolean {
  return Boolean(projectPath()) && studioMode() === "advanced" && shipIntent() === "public";
}

function syncNowQuick() {
  const quick = document.querySelector<HTMLElement>("#now-quick");
  const polarBtn = document.querySelector<HTMLButtonElement>("#now-polar");
  const bound = Boolean(projectPath());
  // S2 — General: peer Portal/Env strip stays Advanced-only (CSS + hidden).
  const general = studioMode() === "general";
  if (quick) quick.hidden = !bound || general;
  if (polarBtn) polarBtn.hidden = !canShowPolarSetup() || general;
  for (const id of ["now-human", "now-portal", "now-env", "now-polar"] as const) {
    const btn = document.querySelector<HTMLButtonElement>(`#${id}`);
    if (btn) btn.disabled = !bound || running;
  }
}

/** Load portal plan for one provider and open Portal view. */
async function openPortalProvider(provider: string) {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return;
  }
  if (!isPortalProvider(provider)) {
    toast(
      `No Portal plan for «${provider}» — use Open dashboard on Deployment`,
      "info",
      5500,
    );
    return;
  }
  portalFilter = provider;
  setView("portal");
  const result = await run(["portal", "--project", project, "--provider", provider], {
    step: "portal",
    quietToast: true,
  });
  if (!result) {
    toast("Portal busy — Cancel to unlock, then retry", "err");
    return;
  }
  if (!result.ok || !result.stdout.trim()) {
    toast(`Portal · ${provider}: ${cmdFailDetail(result)}`, "err", 8000);
    return;
  }
  try {
    const plan = JSON.parse(result.stdout) as PortalPlan;
    applyPortalPlan(plan);
    setStep("portal", "done");
    toast(`Portal · ${provider}`, "ok");
  } catch {
    toast(`Portal · ${provider}: could not parse plan — open Preview`, "err", 7000, [
      { id: "preview", label: "Preview log", icon: "open", run: () => openOutputPreview() },
    ]);
  }
}

function setupPolarPortal() {
  if (!canShowPolarSetup()) {
    toast("Set up Polar needs Advanced mode + Public intent", "info");
    return;
  }
  setView("integrations");
  selectIntegration("polar");
}


let selectedIntegration = "polar";
let selectedPlatform = "selfhost";
let selectedSignLane = "apple-sign";

async function openIntegrationVendor(url: string, label: string): Promise<void> {
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  await openUrl(url);
  toast(`Opened ${label}`, "ok");
}

function platformLocked(wiz: (typeof PLATFORM_WIZARDS)[number]): boolean {
  return Boolean(wiz.needsPublic && shipIntent() !== "public");
}

/** Clear Offline + switch to Public so hosted Deploy can start. */
function ensureHostingReady(opts?: { toastOk?: boolean }): boolean {
  let changed = false;
  if (offline()) {
    const el = offlineEl();
    if (el) {
      el.checked = false;
      syncDeployToggle();
      changed = true;
    }
  }
  if (shipIntent() !== "public") {
    applyShipIntent("public");
    changed = true;
  }
  if (changed && opts?.toastOk !== false) {
    toast("Public intent · Online — hosted Deploy unlocked", "ok", 4000);
  }
  return true;
}

function toastHostingNeedsPublic(onReady: () => void) {
  toast("Hosting providers need Public intent — or Use Local for desktop-only", "info", 10000, [
    {
      id: "switch-public",
      label: "Switch to Public",
      icon: "continue",
      run: () => {
        ensureHostingReady();
        onReady();
      },
    },
  ]);
}

function renderSuiteShelf() {
  const host = document.querySelector<HTMLElement>("#suite-catalog");
  if (!host) return;
  host.innerHTML = SUITE_CARDS.map((card) => {
    const primary = card.openUrl
      ? `<button type="button" class="primary suite-open" data-suite-url="${escapeHtml(card.openUrl)}">${escapeHtml(card.openLabel ?? "Open")}</button>`
      : `<button type="button" class="ghost suite-why" data-suite-id="${escapeHtml(card.id)}">How it fits</button>`;
    const secondary =
      card.secondaryUrl && card.secondaryLabel
        ? `<button type="button" class="ghost suite-open" data-suite-url="${escapeHtml(card.secondaryUrl)}">${escapeHtml(card.secondaryLabel)}</button>`
        : "";
    const note = card.note ? `<p class="suite-card-note">${escapeHtml(card.note)}</p>` : "";
    return `<article class="suite-card" data-suite="${escapeHtml(card.id)}">
      <h3 class="suite-card-title">${escapeHtml(card.title)}</h3>
      <p class="suite-card-blurb">${escapeHtml(card.blurb)}</p>
      ${note}
      <div class="hero-cta suite-card-actions">${primary}${secondary}</div>
    </article>`;
  }).join("");
}

function renderIntegrations() {
  renderSuiteShelf();
  const host = document.querySelector<HTMLElement>("#integrations-catalog");
  if (!host) return;
  renderProviderCatalogGrid({
    host,
    entries: INTEGRATION_WIZARDS,
    groups: ["Payments", "Email"],
    selectedId: selectedIntegration,
    iconHtml: integrationIconHtml,
    isLocked: (w) => w.needsPublic && shipIntent() !== "public",
    onSelect: (id, locked) => {
      if (locked) {
        toast("Payment wizards need Public intent", "info");
        return;
      }
      selectIntegration(id);
    },
  });
  paintIntegrationWizard();
  syncIntegrationsOrientation();
}

/** S3.3 — Suite is not the current Publish gate unless listing/marketing/env.sprint. */
function publishIntegrationsOk(stepId: string | undefined): boolean {
  const id = stepId ?? "";
  return id.startsWith("listing.") || id.startsWith("marketing.") || id === "env.sprint";
}

function syncIntegrationsOrientation() {
  const banner = document.querySelector<HTMLElement>("#int-gate-orient");
  const text = document.querySelector("#int-gate-orient-text");
  if (!banner || !text) return;
  const cur = lastPublish?.current;
  const id = cur?.id ?? "";
  const show =
    Boolean(cur) &&
    !lastPublish?.finished &&
    (cur?.status ?? "").toLowerCase() !== "done" &&
    !publishIntegrationsOk(id);
  banner.hidden = !show;
  if (show) {
    text.textContent = `Current Publish gate: ${cur?.title ?? id}. Suite is optional — use Publish / Deployment for this step.`;
  }
}

function goToCurrentPublishGate() {
  const related = (lastPublish?.current?.desktop_view ?? "").trim();
  if (related === "scopes") {
    setView("scopes");
    const project = projectPath();
    if (project) {
      void loadJsonCmd(["scopes", "--project", project]).then((plan) => {
        applyScopes(plan as ScopePlan | null);
      });
    }
    toast("Current gate is Scopes — save Targets, then Back to Publish → Confirm", "info", 6000);
    return;
  }
  if (related === "platforms") {
    openPlatformsCatalog();
    toast("Current gate is host login / deploy — finish on Deployment, then Confirm on Publish", "info", 6000);
    return;
  }
  setView("publish");
  toast("Current gate is on Publish — Open / Confirm there", "info", 5000);
}

function selectIntegration(id: string) {
  const wiz = INTEGRATION_WIZARDS.find((w) => w.id === id);
  if (!wiz) return;
  if (wiz.needsPublic && shipIntent() !== "public") {
    toast("Payment wizards need Public intent", "info");
    return;
  }
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  selectedIntegration = id;
  if (activeViewId !== "integrations") setView("integrations");
  else renderIntegrations();
}

function paintIntegrationWizard() {
  const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration) ?? null;
  const putName = INTEGRATION_HOST_PUT[selectedIntegration] ?? null;
  const putHost = putName ? preferredEnvPutHost() : null;
  paintProviderWizard({
    entry: wiz,
    panel: document.querySelector<HTMLElement>("#integrations-wizard"),
    titleEl: document.querySelector("#int-wizard-title"),
    blurbEl: document.querySelector("#int-wizard-blurb"),
    stepsEl: document.querySelector("#int-wizard-steps"),
    openBtn: document.querySelector<HTMLButtonElement>("#int-open"),
    secondaryBtn: document.querySelector<HTMLButtonElement>("#int-portal-steps"),
    secondaryVisible: Boolean(wiz?.provider && isPortalProvider(wiz.provider)),
    docsBtn: document.querySelector<HTMLButtonElement>("#int-docs"),
    putBtn: document.querySelector<HTMLButtonElement>("#int-put"),
    putVisible: Boolean(putName),
    openLinksEl: document.querySelector<HTMLElement>("#int-open-links"),
  });
  const putBtn = document.querySelector<HTMLButtonElement>("#int-put");
  const openBtn = document.querySelector<HTMLButtonElement>("#int-open");
  if (putBtn && putName) {
    putBtn.textContent = putHost ? `Put ${putName}` : "Put key (need host)";
    putBtn.title = putHost
      ? `Put ${putName} on ${putHost} — paste only in the terminal`
      : "Detect Cloudflare, Vercel, or Netlify first — Put targets the deploy host";
    putBtn.disabled = !putHost || !projectPath();
    putBtn.classList.add("primary");
    openBtn?.classList.remove("primary");
  } else {
    putBtn?.classList.remove("primary");
    openBtn?.classList.add("primary");
  }
  const doneEl = document.querySelector<HTMLElement>("#int-wizard-done");
  if (doneEl) {
    const cue = wiz ? INTEGRATION_DONE_CRITERIA[wiz.id] : "";
    doneEl.hidden = !cue;
    doneEl.textContent = cue || "";
  }
}

/** Tier A host for Integrations Env Put (Resend key lives on the deploy host). */
function preferredEnvPutHost(detected?: Detected | null): string | null {
  const d = detected ?? lastDetected ?? null;
  if (!d) return null;
  if (d.wrangler) return "cloudflare";
  if (d.vercel) return "vercel";
  if (d.netlify) return "netlify";
  return null;
}

function preferredHostingPlatformId(detected?: Detected | null): string | null {
  if (!detected) return null;
  // Orbit-deploy hosts first (Tier A), then CLI hosts, Pages, Orbit, Umami, Self-host.
  if (detected.wrangler) return "cloudflare";
  if (detected.vercel) return "vercel";
  if (detected.netlify) return "netlify";
  if (detected.fly) return "fly";
  if (detected.railway) return "railway";
  if (detected.marketing_site && (detected.marketing_host === "pages" || !detected.marketing_host)) {
    return "github-pages";
  }
  if (detected.marketing_host === "pages") return "github-pages";
  if (detected.yard_configured) return "orbit";
  if (detected.umami) return "umami";
  return "selfhost";
}

function openPlatformsCatalog(opts?: { preferGroup?: string; selectId?: string }) {
  if (opts?.preferGroup === "Official signing") {
    openSignCatalog({ selectId: opts.selectId ?? "apple-sign" });
    return;
  }
  const preferredHost = preferredHostingPlatformId(lastDetected);
  if (opts?.selectId) selectedPlatform = opts.selectId;
  else if (savedPrimaryHost()) selectedPlatform = savedPrimaryHost()!;
  else selectedPlatform = preferredHost ?? "selfhost";
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  // Explicit pick (chip / Targets Deploy / command) persists primary_host.
  // Nav / soft-open only highlights — Detect suggests; human chooses (Launch L4).
  if (opts?.selectId) {
    selectPlatform(selectedPlatform);
  } else {
    setView("platforms");
  }
}

let rememberedPrimaryHost: string | null = null;

function savedPrimaryHost(): string | null {
  return rememberedPrimaryHost;
}

function openSignCatalog(opts?: { selectId?: string }) {
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  if (opts?.selectId) selectedSignLane = opts.selectId;
  setView("sign");
  selectSignLane(selectedSignLane);
}

function signCatalogEntries() {
  return signingCatalogEntries();
}

function renderSignCatalog() {
  const host = document.querySelector<HTMLElement>("#sign-catalog");
  if (!host) return;
  const entries = signCatalogEntries();
  if (!entries.some((e) => e.id === selectedSignLane)) {
    selectedSignLane = entries[0]?.id ?? "apple-sign";
  }
  renderProviderCatalogGrid({
    host,
    entries,
    groups: SIGN_GROUPS,
    selectedId: selectedSignLane,
    preferGroup: "Official signing",
    iconHtml: providerIconHtml,
    onSelect: (id) => {
      selectSignLane(id);
    },
  });
  paintSignWizard();
}

function selectSignLane(id: string) {
  const wiz = signCatalogEntries().find((w) => w.id === id);
  if (!wiz) return;
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  selectedSignLane = id;
  if (activeViewId !== "sign") setView("sign");
  else renderSignCatalog();
}

function paintSignWizard() {
  const wiz = signCatalogEntries().find((w) => w.id === selectedSignLane) ?? null;
  paintProviderWizard({
    entry: wiz,
    panel: document.querySelector<HTMLElement>("#sign-wizard"),
    titleEl: document.querySelector("#sign-wizard-title"),
    blurbEl: document.querySelector("#sign-wizard-blurb"),
    stepsEl: document.querySelector("#sign-wizard-steps"),
    openBtn: document.querySelector<HTMLButtonElement>("#sign-open"),
  });
  paintSignStructureGate(wiz?.id);
}

function deployCatalogEntries() {
  return hostingCatalogEntries({ orbitConfigured: Boolean(lastDetected?.yard_configured) });
}

function renderPlatforms() {
  const host = document.querySelector<HTMLElement>("#platforms-catalog");
  if (!host) return;
  const entries = deployCatalogEntries();
  if (!entries.some((e) => e.id === selectedPlatform)) {
    const saved = savedPrimaryHost();
    selectedPlatform =
      (saved && entries.some((e) => e.id === saved) ? saved : null) ??
      preferredHostingPlatformId(lastDetected) ??
      entries[0]?.id ??
      "selfhost";
  }
  renderProviderCatalogGrid({
    host,
    entries,
    groups: PLATFORM_GROUPS,
    selectedId: selectedPlatform,
    preferGroup: deployPreferGroup(selectedPlatform),
    iconHtml: providerIconHtml,
    onSelect: (id) => {
      selectPlatform(id);
    },
  });
  paintPlatformWizard();
}

function selectPlatform(id: string) {
  const wiz = deployCatalogEntries().find((w) => w.id === id);
  if (!wiz) return;
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  selectedPlatform = id;
  rememberedPrimaryHost = id;
  const project = projectPath();
  void invoke("set_primary_host", { project, hostId: id }).catch((err) => {
    console.warn(err);
  });
  if (activeViewId !== "platforms") setView("platforms");
  else renderPlatforms();
  // Same auth vetting for every CLI host — silent when already signed in.
  void vetProviderOnSelect(wiz);
}

/** Hosts that require vendor CLI login before Deploy / Portal work. */
function providerNeedsCliAuthVetting(wiz: ProviderWizard | null | undefined): boolean {
  if (!wiz) return false;
  const p = (wiz.provider ?? wiz.id).toLowerCase();
  return (
    p === "cloudflare" ||
    p === "vercel" ||
    p === "netlify" ||
    p === "fly" ||
    p === "railway"
  );
}

async function vetProviderOnSelect(wiz: ProviderWizard): Promise<void> {
  if (!providerNeedsCliAuthVetting(wiz)) return;
  await ensureProviderCliAuth(wiz, { context: "select" });
}


function hostedResultUrls(): string[] {
  const fromPulse = hostedDeployUrls(lastPulse?.deploy);
  if (fromPulse.length) return fromPulse;
  return (lastRunState?.urls ?? []).filter((u) => !isLoopbackDeployUrl(u));
}

/** Which host wrote `.ship/last-run.json` — never share Live URL across cards. */
function lastRunHostProvider(): string | null {
  const tagged = (lastRunState?.host_provider ?? "").trim().toLowerCase();
  if (tagged) return tagged;
  const steps = lastRunState?.steps ?? [];
  if (steps.some((s) => (s.id ?? "").startsWith("selfhost"))) return "selfhost";
  if (steps.some((s) => (s.id ?? "").startsWith("medusa"))) return "medusa";
  if (steps.some((s) => (s.id ?? "").startsWith("flowsint"))) return "flowsint";
  if (steps.some((s) => (s.id ?? "").startsWith("umami"))) return "umami";
  const blob = `${lastRunState?.message ?? ""}\n${steps.map((s) => s.detail ?? "").join("\n")}`.toLowerCase();
  if (blob.includes("selfhost")) return "selfhost";
  if (blob.includes("medusa")) return "medusa";
  if (blob.includes("flowsint")) return "flowsint";
  if (blob.includes("umami")) return "umami";
  if (blob.includes("cloudflare") || blob.includes("wrangler") || blob.includes("pages.dev")) {
    return "cloudflare";
  }
  if (blob.includes("vercel")) return "vercel";
  if (blob.includes("netlify")) return "netlify";
  if (blob.includes("fly.dev") || blob.includes("fly.io") || /\bfly\b/.test(blob)) {
    return "fly";
  }
  if (blob.includes("railway")) return "railway";
  if (blob.includes("orbit") || (steps.some((s) => s.id === "deploy") && blob.includes("orbit"))) {
    return "orbit";
  }
  return null;
}

function resultsBayMatchesCard(wiz: ProviderWizard | null): boolean {
  if (!wiz) return false;
  const hp = lastRunHostProvider();
  if (!hp) return false;
  if (wiz.id === "selfhost") return hp === "selfhost";
  if (wiz.id === "medusa") return hp === "medusa";
  if (wiz.id === "flowsint") return hp === "flowsint";
  if (wiz.id === "umami") return hp === "umami";
  if (wiz.id === "orbit") return hp === "orbit";
  const id = (wiz.provider ?? wiz.id).toLowerCase();
  return hp === id || hp === wiz.id;
}

function defaultHostProjectName(): string {
  const p = projectPath();
  if (!p) return "harbor";
  const base = p.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || "project";
  return base.toLowerCase().replace(/[^a-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "") || "project";
}

/** Best-effort Pages/site name from last hostdeploy run or remembered dialog. */
function lastHostProjectName(): string {
  if (lastHostDeployName.trim()) return lastHostDeployName.trim();
  const blob = `${lastRunState?.message ?? ""}\n${(lastRunState?.steps ?? [])
    .map((s) => s.detail ?? "")
    .join("\n")}`;
  const m =
    /--project-name[=\s]+([a-z0-9][a-z0-9_-]*)/i.exec(blob) ||
    /pages project create\s+([a-z0-9][a-z0-9_-]*)/i.exec(blob) ||
    /project-name[=:\s]+([a-z0-9][a-z0-9_-]*)/i.exec(blob);
  if (m?.[1]) return m[1].toLowerCase();
  return defaultHostProjectName();
}

/**
 * Open dashboard URL that lands on the list/project the operator needs —
 * not the vendor marketing home.
 */
function hostDashboardUrl(wiz: ProviderWizard | null | undefined): string {
  if (!wiz?.openUrl) return "";
  const id = wiz.id;
  const name = lastHostProjectName();
  const enc = encodeURIComponent(name);
  if (id === "cloudflare") {
    // Prefer project deep-link when this card's last deploy succeeded; else Workers & Pages list.
    if (
      name &&
      lastRunHostProvider() === "cloudflare" &&
      lastRunState?.ok === true
    ) {
      return `https://dash.cloudflare.com/?to=/:account/pages/view/${enc}`;
    }
    return "https://dash.cloudflare.com/?to=/:account/workers-and-pages";
  }
  if (id === "vercel") {
    return "https://vercel.com/dashboard";
  }
  if (id === "netlify") {
    if (name && lastRunHostProvider() === "netlify" && lastRunState?.ok === true) {
      return `https://app.netlify.com/projects/${enc}`;
    }
    return "https://app.netlify.com/projects";
  }
  if (id === "fly") return "https://fly.io/dashboard";
  if (id === "railway") return "https://railway.com/dashboard";
  if (id === "github-pages") return "https://github.com/settings/pages";
  return wiz.openUrl;
}

async function openHostDashboard(wiz: ProviderWizard): Promise<void> {
  const url = hostDashboardUrl(wiz);
  if (!url) {
    toast("No dashboard URL for this host", "info");
    return;
  }
  await openUrl(url);
  const where =
    wiz.id === "cloudflare"
      ? lastRunHostProvider() === "cloudflare"
        ? `Cloudflare Pages · ${lastHostProjectName()}`
        : "Cloudflare Workers & Pages"
      : wiz.id === "netlify"
        ? "Netlify projects"
        : wiz.id === "vercel"
          ? "Vercel dashboard"
          : wiz.title;
  toast(`Opened ${where}`, "ok", 4500);
}

/** True while `orbityard selfhost --serve` is the active Desktop command. */
let selfhostServing = false;
/** Open live asked for serve — open loopback when health ok streams. */
let pendingSelfhostOpenLive = false;
let lastHostDeployName = "";
/** Absolute path for Targets → Deploy (e.g. apps/website) — used as hostdeploy project cwd. */
let lastDeployFocusProject: string | null = null;
let pendingHostDeployWiz: ProviderWizard | null = null;
/** Relative scope dir for Deploy dialog (e.g. apps/website). */
let pendingHostDeployTargetRel = "";
let pendingHostDeployEnvPackId = "";
let pendingHostDeployPacks: EnvPack[] = [];
/** Restore target/pack after Switch-to-hint Deploy. */
let pendingHostDeployRestore: { targetRel: string; packId: string } | null = null;
let pendingEnvImportWiz: ProviderWizard | null = null;
let pendingEnvImportFile = "";
let pendingEnvImportPackId = "";
let pendingEnvImportPackDir = "";

type EnvImportCandidate = { name?: string; empty?: boolean; denied?: boolean };
type EnvPack = {
  id?: string;
  label?: string;
  kind?: string;
  rel_dir?: string;
  file?: string;
  host_hint?: string | null;
  key_count?: number;
  nonempty_count?: number;
};
type EnvImportReport = {
  schema?: string;
  provider?: string;
  file?: string;
  packs?: EnvPack[];
  candidates?: EnvImportCandidate[];
  put?: { name?: string; ok?: boolean; code?: number; error?: string | null }[];
  skipped?: { name?: string; reason?: string }[];
  dry_run?: boolean;
  notes?: string[];
};
type EnvPacksReport = {
  schema?: string;
  packs?: EnvPack[];
};

function isCloudHostedUrl(u: string): boolean {
  const l = u.toLowerCase();
  return (
    l.includes(".pages.dev") ||
    l.includes(".workers.dev") ||
    l.includes("vercel.app") ||
    l.includes("netlify.app") ||
    l.includes("netlify.com") ||
    l.includes(".fly.dev") ||
    l.includes(".fly.io") ||
    l.includes("railway.app") ||
    l.includes("up.railway.app")
  );
}

function cancelSelfhostServe(): void {
  pendingSelfhostOpenLive = false;
  selfhostServing = false;
  // Do not rely on #btn-cancel.click() — disabled Tools buttons swallow programmatic clicks.
  void (async () => {
    try {
      const killed = await invoke<boolean>("cancel_orbityard");
      appendStream({
        stream: "meta",
        text: killed ? "cancel signal sent" : "nothing to cancel — unlocking UI",
      });
    } catch (err) {
      appendStream({ stream: "stderr", text: String(err) });
    } finally {
      forceUnlockUi("Cancelled");
      toast("Self-host serve stopped", "ok", 3500);
    }
  })();
}

async function startSelfhostServe(opts?: { openLive?: boolean }): Promise<void> {
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  if (opts?.openLive) pendingSelfhostOpenLive = true;
  if (selfhostServing || running) {
    if (opts?.openLive) {
      const fromBtn = document
        .querySelector<HTMLButtonElement>("#plat-open-live")
        ?.dataset.url?.trim();
      const fromRun = (lastRunState?.urls ?? [])
        .map((u) => String(u ?? ""))
        .find((u) => /^https?:\/\/127\.0\.0\.1/i.test(u));
      const url = fromBtn || fromRun || "";
      if (url) {
        pendingSelfhostOpenLive = false;
        await openUrl(url);
        toast("Opened live URL", "ok", 3500);
        return;
      }
      toast("Serve still starting — try Open live again in a moment", "info", 4000);
    } else {
      toast("Self-host already busy — Cancel serve first", "info", 4000);
    }
    return;
  }
  selfhostResultsDismissed = false;
  applyOutputDock(true);
  selfhostServing = true;
  paintPlatformWizard();
  toast(
    opts?.openLive
      ? "Serving locally — Cancel serve or Esc when the GIF / demo is done"
      : "Self-host serving until Cancel serve (or Esc)",
    "ok",
    7000,
    [
      {
        id: "cancel-serve",
        label: "Cancel serve",
        icon: "continue",
        run: () => cancelSelfhostServe(),
      },
    ],
  );
  const result = await run(["selfhost", "--serve"], {
    quietToast: true,
    busyLabel: "Checking…",
  });
  pendingSelfhostOpenLive = false;
  selfhostServing = false;
  paintPlatformWizard();
  if (!result) return;
  if (result.cancelled) toast("Self-host serve cancelled", "ok", 4000);
  else if (result.ok) toast("Self-host serve ended", "ok", 4000);
  else toast("Self-host serve failed — see Output", "err", 5000);
}

/** Hide stale Self-host Results until the next Deploy. */
let selfhostResultsDismissed = false;
/** Cleared cloud last-run for this provider id until next Deploy. */
let cloudEvidenceClearedProvider: string | null = null;

function hostProviderKey(wiz: ProviderWizard | null): string {
  if (!wiz) return "";
  return (wiz.provider ?? wiz.id).toLowerCase();
}

function hasSelfhostDeployEvidence(): boolean {
  if (selfhostResultsDismissed) return false;
  if (lastRunHostProvider() === "selfhost") return true;
  if (isSelfhostLocalEvidence(lastPulse)) return true;
  const steps = lastRunState?.steps ?? [];
  return steps.some((s) => (s.id ?? "").startsWith("selfhost"));
}

/** Prior cloud deploy for this card — gates Cancel on dashboard / Clear evidence. */
function hasCloudHostDeployEvidence(wiz: ProviderWizard | null): boolean {
  if (!wiz || wiz.id === "selfhost") return false;
  if (cloudEvidenceClearedProvider === hostProviderKey(wiz)) return false;
  if (!resultsBayMatchesCard(wiz)) return false;
  if (lastRunState?.ok === true) return true;
  const urls = (lastRunState?.urls ?? []).filter((u) => !isLoopbackDeployUrl(String(u)));
  if (urls.length > 0) return true;
  return (
    deployEvidenceFromPulse(lastPulse) && hostedDeployUrls(lastPulse?.deploy).length > 0
  );
}

/** Drop local last-run after the operator deleted the cloud project. */
async function clearHostDeployEvidence(wiz: ProviderWizard): Promise<void> {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return;
  }
  cloudEvidenceClearedProvider = hostProviderKey(wiz);
  const sep = project.includes("\\") ? "\\" : "/";
  const lastPath = `${project.replace(/[\\/]+$/, "")}${sep}.ship${sep}last-run.json`;
  try {
    if (resultsBayMatchesCard(wiz)) {
      await invoke("delete_path", { path: lastPath });
    }
  } catch (err) {
    console.warn(err);
  }
  lastRunState = null;
  await refreshShipState();
  await refreshSessionNow();
  paintDeployResultsBay();
  paintPlatformWizard();
  toast("Studio evidence cleared — Deploy again when the project exists", "ok", 5000);
}

async function resetSelfhostSession(): Promise<void> {
  if (running || selfhostServing) {
    cancelSelfhostServe();
    await new Promise((r) => window.setTimeout(r, 400));
  }
  selfhostServing = false;
  selfhostResultsDismissed = true;
  paintDeployResultsBay();
  paintPlatformWizard();
  await refreshShipState();
  await refreshSessionNow();
  toast("Self-host cleared — Deploy again for a fresh run", "ok", 4500);
}

/** Self-host: kill local serve / Start over. Cloud: Clear evidence or open dashboard. */
function runPlatCancelDeploy(wiz: ProviderWizard, opts?: { mode?: "clear" | "dashboard" }): void {
  if (wiz.id === "selfhost") {
    if (selfhostServing || running) {
      cancelSelfhostServe();
      toast("Cancelling Self-host serve…", "ok", 3500);
      return;
    }
    if (hasSelfhostDeployEvidence()) {
      void resetSelfhostSession();
      return;
    }
    toast("Deploy Self-host first, then Cancel serve or Start over", "info", 5000);
    return;
  }
  if (isHostedCliDeployCard(wiz.id) || wiz.id === "fly" || wiz.id === "railway" || wiz.id === "github-pages") {
    if (!hasCloudHostDeployEvidence(wiz) && opts?.mode !== "clear") {
      toast("Deploy this host first — cancel/clear appears after a successful run", "info", 5500);
      return;
    }
    const mode = opts?.mode ?? "clear";
    if (mode === "clear") {
      void clearHostDeployEvidence(wiz);
      return;
    }
    void openHostDashboard(wiz);
    toast(
      `Delete on ${wiz.title} dashboard, then Clear evidence here to sync Studio`,
      "info",
      8000,
      [
        {
          id: "clear-evidence",
          label: "Clear evidence",
          icon: "continue",
          run: () => void clearHostDeployEvidence(wiz),
        },
      ],
    );
    return;
  }
  toast("No cancel action for this card", "info");
}

function paintPlatCancelButtons(wiz: ProviderWizard | null, resultsVisible: boolean) {
  const main = document.querySelector<HTMLButtonElement>("#plat-cancel-deploy");
  const bay = document.querySelector<HTMLButtonElement>("#plat-cancel-deploy-bay");
  const cloud =
    Boolean(wiz) &&
    (isHostedCliDeployCard(wiz!.id) ||
      wiz!.id === "fly" ||
      wiz!.id === "railway" ||
      wiz!.id === "github-pages");
  const self = wiz?.id === "selfhost";
  const selfActive = self && (selfhostServing || running);
  const selfStartOver = self && !selfActive && hasSelfhostDeployEvidence();
  const cloudEvidence = cloud && hasCloudHostDeployEvidence(wiz);

  if (main) {
    if (selfActive) {
      main.hidden = false;
      main.disabled = false;
      main.textContent = "Cancel serve";
      main.title = "Stop the local selfhost process";
    } else if (selfStartOver) {
      main.hidden = false;
      main.disabled = false;
      main.textContent = "Start over";
      main.title = "Clear this Self-host result and Deploy again";
    } else if (cloudEvidence) {
      main.hidden = false;
      main.disabled = false;
      main.textContent = "Clear evidence";
      main.title =
        "Remove local last-run after you deleted the cloud project — Studio does not poll the vendor";
    } else {
      main.hidden = true;
      main.disabled = true;
    }
  }

  if (bay) {
    if (selfActive) {
      bay.hidden = false;
      bay.textContent = "Cancel serve";
    } else if (selfStartOver) {
      bay.hidden = false;
      bay.textContent = "Start over";
    } else if (cloudEvidence && resultsVisible) {
      bay.hidden = false;
      bay.textContent = "Cancel on dashboard";
    } else {
      bay.hidden = true;
    }
  }
}

/** H3 — Deployment aside: phases · live URL · Open live / Open dashboard. */
function paintDeployResultsBay() {
  const bay = document.querySelector<HTMLElement>("#plat-results");
  if (!bay) return;
  const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform) ?? null;
  const statusEl = document.querySelector<HTMLElement>("#plat-results-status");
  const phasesEl = document.querySelector<HTMLElement>("#plat-results-phases");
  const urlEl = document.querySelector<HTMLElement>("#plat-results-url");
  const urlLabelEl = document.querySelector<HTMLElement>("#plat-results-url-label");
  const kickerEl = document.querySelector<HTMLElement>("#plat-results-kicker");
  const liveBtn = document.querySelector<HTMLButtonElement>("#plat-open-live");
  const dashBtn = document.querySelector<HTMLButtonElement>("#plat-results-dashboard");

  const match = resultsBayMatchesCard(wiz);
  const selfhost = isSelfhostLocalEvidence(lastPulse) && wiz?.id === "selfhost";
  const umamiOk = match && wiz?.id === "umami" && lastRunHostProvider() === "umami";
  const flowsintOk = match && wiz?.id === "flowsint" && lastRunHostProvider() === "flowsint";
  const medusaOk = match && wiz?.id === "medusa" && lastRunHostProvider() === "medusa";
  const hostedOk =
    deployEvidenceFromPulse(lastPulse) &&
    match &&
    wiz?.id !== "selfhost" &&
    wiz?.id !== "umami" &&
    wiz?.id !== "flowsint" &&
    wiz?.id !== "medusa";
  const detail = match
    ? (lastPulse?.deploy?.detail ?? lastRunState?.message ?? "")
    : "";
  // Hard isolate: Self-host / Umami / Flowsint / Medusa = loopback only; cloud cards never show loopback.
  const urls = !match
    ? []
    : wiz?.id === "selfhost" ||
        wiz?.id === "umami" ||
        wiz?.id === "flowsint" ||
        wiz?.id === "medusa"
      ? (lastPulse?.deploy?.urls ?? lastRunState?.urls ?? [])
          .filter((u) => isLoopbackDeployUrl(u) && !isCloudHostedUrl(u))
          .slice(0, 1)
      : hostedResultUrls().filter((u) => !isLoopbackDeployUrl(u));
  const allSteps = match ? (lastRunState?.steps ?? []) : [];
  const hostSteps = allSteps.filter((s) => (s.id ?? "").startsWith("host."));
  const selfhostSteps = allSteps.filter((s) => (s.id ?? "").startsWith("selfhost"));
  const umamiSteps = allSteps.filter((s) => (s.id ?? "").startsWith("umami"));
  const flowsintSteps = allSteps.filter((s) => (s.id ?? "").startsWith("flowsint"));
  const medusaSteps = allSteps.filter((s) => (s.id ?? "").startsWith("medusa"));
  const steps =
    wiz?.id === "selfhost"
      ? selfhostSteps
      : wiz?.id === "umami"
        ? umamiSteps
        : wiz?.id === "flowsint"
          ? flowsintSteps
          : wiz?.id === "medusa"
            ? medusaSteps
            : hostSteps.length
              ? hostSteps
              : allSteps.filter((s) => s.id === "deploy");

  // Prefer card-relevant evidence only (never cross-host Live URL).
  let show = false;
  if (wiz && isDeployCatalogGroup(wiz.group) && match) {
    if (wiz.id === "selfhost") {
      if (selfhostResultsDismissed) {
        show = selfhostServing;
      } else {
        show =
          selfhost ||
          selfhostSteps.length > 0 ||
          Boolean(urls.length) ||
          selfhostServing;
      }
    } else if (wiz.id === "umami") {
      show = umamiOk || umamiSteps.length > 0 || Boolean(urls.length);
    } else if (wiz.id === "flowsint") {
      show = flowsintOk || flowsintSteps.length > 0 || Boolean(urls.length);
    } else if (wiz.id === "medusa") {
      show = medusaOk || medusaSteps.length > 0 || Boolean(urls.length);
    } else if (
      wiz.id === "cloudflare" ||
      wiz.id === "vercel" ||
      wiz.id === "netlify" ||
      wiz.id === "fly" ||
      wiz.id === "railway"
    ) {
      if (cloudEvidenceClearedProvider === hostProviderKey(wiz)) {
        show = false;
      } else {
        show =
          hostedOk ||
          hostSteps.length > 0 ||
          urls.length > 0 ||
          (lastRunState?.ok === false &&
            /hostdeploy|wrangler|vercel|netlify|fly|railway/i.test(
              lastRunState.message ?? "",
            ));
      }
    } else {
      show = hostedOk || urls.length > 0;
    }
  }
  // Self-host Cancel surface even before last-run match when serve is active.
  if (!show && wiz?.id === "selfhost" && selfhostServing) {
    show = true;
  }
  bay.hidden = !show;
  if (!show) {
    bay.dataset.serving = "";
    paintPlatCancelButtons(wiz, false);
    // Fall through only for cancel paint; hide rest of bay content below via early return.
    return;
  }
  bay.dataset.serving = wiz?.id === "selfhost" && selfhostServing ? "1" : "";
  paintPlatCancelButtons(wiz, true);
  if (kickerEl) {
    kickerEl.textContent =
      wiz?.id === "selfhost"
        ? "Last self-host (local)"
        : wiz?.id === "umami"
          ? "Last Umami deploy (local)"
          : wiz?.id === "flowsint"
            ? "Last Flowsint deploy (local)"
            : wiz?.id === "medusa"
              ? "Last Medusa deploy (local)"
              : wiz?.id === "cloudflare"
              ? "Last Cloudflare deploy"
              : wiz?.id === "vercel"
                ? "Last Vercel deploy"
                : wiz?.id === "netlify"
                  ? "Last Netlify deploy"
                  : wiz?.id === "fly"
                    ? "Last Fly deploy"
                    : wiz?.id === "railway"
                      ? "Last Railway deploy"
                      : "Last deploy";
  }

  let status = "No deploy evidence for this host yet.";
  if (wiz?.id === "selfhost" && selfhostServing) {
    status = "Self-host serving on this machine — Cancel serve to stop.";
  } else if (wiz?.id === "umami" && (umamiOk || urls.length)) {
    status =
      detail ||
      "Umami Admin heartbeat ok — Open Admin, change the default password, Confirm on Publish.";
  } else if (wiz?.id === "flowsint" && (flowsintOk || urls.length)) {
    status =
      detail ||
      "Flowsint App proof ok — Open App, register at /register, Confirm on Publish.";
  } else if (wiz?.id === "medusa" && (medusaOk || urls.length)) {
    status =
      detail ||
      "Medusa Admin health ok — Open Admin, create/sign in, Confirm on Publish.";
  } else if (hostedOk) {
    status = detail || "Public URL evidence — Confirm Live check on Publish when ready.";
  } else if (selfhost) {
    status = detail || "Self-host local check ok — loopback only (not a cloud host).";
  } else if (lastRunState && lastRunState.ok === false && match) {
    status = lastRunState.message || "Last deploy run failed — see Output / Troubleshoot next.";
  } else if (lastRunState?.ok && match) {
    status = detail || lastRunState.message || "Last deploy succeeded.";
  } else if (detail) {
    status = detail;
  }
  if (statusEl) statusEl.textContent = status;

  if (phasesEl) {
    if (steps.length) {
      phasesEl.innerHTML = steps
        .map((s) => {
          const ok = s.ok !== false;
          const mark = ok ? "✓" : "✗";
          const cls = ok ? "is-ok" : "is-fail";
          const id = escapeHtml(s.id ?? "step");
          const d = escapeHtml(s.detail ?? "");
          return `<li class="${cls}"><span>${mark}</span> ${id}${d ? ` — ${d}` : ""}</li>`;
        })
        .join("");
      phasesEl.hidden = false;
    } else {
      phasesEl.innerHTML = "";
      phasesEl.hidden = true;
    }
  }

  const primaryUrl = urls[0] ?? "";
  if (urlLabelEl) {
    if (primaryUrl && wiz?.id === "selfhost") {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = "Local loopback URL (Self-host — not Cloudflare)";
    } else if (primaryUrl && wiz?.id === "umami") {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = "Umami Admin (local Docker — not a cloud host)";
    } else if (primaryUrl && wiz?.id === "flowsint") {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = "Flowsint App (local Docker — not a cloud host)";
    } else if (primaryUrl && wiz?.id === "medusa") {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = "Medusa Admin (local Docker — not a cloud host)";
    } else if (primaryUrl && wiz?.id === "cloudflare") {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = "Cloudflare Pages / Workers URL (not Self-host)";
    } else if (
      primaryUrl &&
      (wiz?.id === "vercel" ||
        wiz?.id === "netlify" ||
        wiz?.id === "fly" ||
        wiz?.id === "railway")
    ) {
      urlLabelEl.hidden = false;
      urlLabelEl.textContent = `${wiz.title} production URL (not Self-host)`;
    } else {
      urlLabelEl.hidden = true;
      urlLabelEl.textContent = "";
    }
  }
  if (urlEl) {
    if (primaryUrl) {
      urlEl.hidden = false;
      urlEl.textContent = primaryUrl;
    } else {
      urlEl.hidden = true;
      urlEl.textContent = "";
    }
  }
  if (liveBtn) {
    liveBtn.hidden = !primaryUrl;
    liveBtn.dataset.url = primaryUrl;
    liveBtn.textContent =
      wiz?.id === "umami" || wiz?.id === "medusa"
        ? "Open Admin"
        : wiz?.id === "flowsint"
          ? "Open App"
          : "Open live";
  }
  if (dashBtn) {
    const dash = hostDashboardUrl(wiz);
    const showDash =
      Boolean(dash) &&
      (wiz?.id === "cloudflare" ||
        wiz?.id === "vercel" ||
        wiz?.id === "netlify" ||
        wiz?.id === "orbit" ||
        wiz?.id === "fly" ||
        wiz?.id === "railway" ||
        wiz?.id === "github-pages");
    dashBtn.hidden = !showDash;
    dashBtn.dataset.url = dash;
  }
  paintPlatCancelButtons(wiz, true);

  const troubleBtn = document.querySelector<HTMLButtonElement>("#plat-troubleshoot");
  const retryBtn = document.querySelector<HTMLButtonElement>("#plat-retry-deploy");
  const hintEl = document.querySelector<HTMLElement>("#plat-results-hint");
  const failed =
    lastRunState?.ok === false ||
    /auth required|exited|failed|detect/i.test(detail);
  if (
    troubleBtn &&
    hintEl &&
    wiz &&
    failed &&
    !selfhostServing &&
    (wiz.id === "cloudflare" ||
      wiz.id === "vercel" ||
      wiz.id === "netlify" ||
      wiz.id === "fly" ||
      wiz.id === "railway" ||
      wiz.id === "selfhost")
  ) {
    const recovery = classifyHostDeployFailure(
      `${detail}\n${lastRunState?.message ?? ""}`,
      wiz.provider ?? wiz.id,
    );
    troubleBtn.hidden = false;
    troubleBtn.textContent = recovery.label;
    troubleBtn.dataset.action = recovery.action;
    troubleBtn.dataset.kind = recovery.kind;
    hintEl.hidden = false;
    // F2 — one recovery: hint names Next; no peer Retry (Troubleshoot is the CTA).
    const cloudCancelNote =
      wiz.id === "cloudflare" ||
      wiz.id === "vercel" ||
      wiz.id === "netlify" ||
      wiz.id === "fly" ||
      wiz.id === "railway"
        ? " Cancel/delete stays on the vendor dashboard."
        : "";
    hintEl.textContent = `Next: ${recovery.hint}${cloudCancelNote}`;
    if (retryBtn) retryBtn.hidden = true;
  } else if (troubleBtn && hintEl) {
    troubleBtn.hidden = true;
    troubleBtn.dataset.action = "";
    if (wiz?.id === "selfhost" && selfhostServing) {
      hintEl.hidden = false;
      hintEl.textContent =
        "Serving locally — Cancel serve stops the process. Deploy itself is a fast check (no long wait).";
    } else if (wiz?.id === "selfhost" && hasSelfhostDeployEvidence()) {
      hintEl.hidden = false;
      hintEl.textContent =
        "Last check finished. Open live starts a local serve; Deploy alone does not keep the server up.";
    } else {
      hintEl.hidden = true;
      hintEl.textContent = "";
    }
    if (retryBtn) retryBtn.hidden = true;
  } else if (retryBtn) {
    retryBtn.hidden = true;
  }
}

/** Spec: structure-proceed-gate-design — wire module actions to Desktop bind/pick. */
function structureGateToastActions(gate: StructureGate): ToastAction[] {
  if (gate.ok) return [];
  return gate.actions.map((a) => ({
    id: a.id,
    label: a.label,
    icon: a.icon,
    run: () => {
      if (a.id === "structure-open-project") {
        void invoke<string | null>("pick_project").then((p) => {
          if (p) void bindProject(p, true);
        });
        return;
      }
      if (a.id === "structure-bind-fixture") {
        const fixture =
          selectedPlatform === "medusa"
            ? medusaFixtureGuessPath(projectPath())
            : selectedPlatform === "flowsint"
              ? flowsintFixtureGuessPath(projectPath())
              : umamiFixtureGuessPath(projectPath());
        if (fixture) void bindProject(fixture, true);
      }
    },
  }));
}

function paintStructureGateBanner(
  hostId: string,
  kickerId: string,
  reasonId: string,
  actionsId: string,
  laneId: string | null | undefined,
) {
  const host = document.querySelector<HTMLElement>(`#${hostId}`);
  const kicker = document.querySelector(`#${kickerId}`);
  const reasonEl = document.querySelector(`#${reasonId}`);
  const actionsEl = document.querySelector<HTMLElement>(`#${actionsId}`);
  if (!host || !reasonEl || !actionsEl) return;
  if (!laneId) {
    host.hidden = true;
    return;
  }
  const gate = evaluateStructureGate(laneId, lastDetected, { projectPath: projectPath() });
  if (!gate) {
    host.hidden = true;
    host.removeAttribute("data-state");
    reasonEl.textContent = "";
    actionsEl.innerHTML = "";
    return;
  }
  host.hidden = false;
  host.dataset.state = gate.ok ? "ok" : gate.mode === "soft" ? "soft" : "blocked";
  if (kicker) {
    kicker.textContent = gate.ok
      ? "Structure · ok"
      : gate.mode === "soft"
        ? "Structure · note"
        : "Structure · blocked";
  }
  reasonEl.textContent = gate.ok ? gate.summary : gate.reason;
  actionsEl.innerHTML = "";
  if (!gate.ok) {
    for (const a of structureGateToastActions(gate)) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = a.label;
      btn.addEventListener("click", () => a.run());
      actionsEl.appendChild(btn);
    }
  }
}

function paintDeployStructureGate(wizId: string | null | undefined) {
  paintStructureGateBanner(
    "plat-structure-gate",
    "plat-structure-kicker",
    "plat-structure-reason",
    "plat-structure-actions",
    wizId,
  );
}

function paintSignStructureGate(laneId: string | null | undefined) {
  paintStructureGateBanner(
    "sign-structure-gate",
    "sign-structure-kicker",
    "sign-structure-reason",
    "sign-structure-actions",
    laneId,
  );
}

/** Quiet doctor refresh, then hard-block spawn CTAs when structure ≠ lane. */
async function assertStructureProceed(
  laneId: string,
  opts?: { paintDeploy?: boolean; paintSign?: boolean },
): Promise<boolean> {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return false;
  }
  const result = await run(["doctor", "--project", project], {
    quietToast: true,
    quietHeader: true,
    busyLabel: "Checking structure…",
  });
  if (result?.stdout) {
    const report = parseDoctor(result.stdout);
    if (report) applyDoctor(report);
  }
  if (opts?.paintDeploy !== false) paintDeployStructureGate(laneId);
  if (opts?.paintSign) paintSignStructureGate(laneId);
  const gate = evaluateStructureGate(laneId, lastDetected, { projectPath: project });
  if (!gate) return true;
  if (gate.ok) return true;
  if (gate.mode === "soft") {
    toast(gate.reason, "info", 10000, structureGateToastActions(gate));
    return true;
  }
  toast(gate.reason, "err", 14000, structureGateToastActions(gate));
  return false;
}

function paintPlatformWizard() {
  const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform) ?? null;
  const showLocal =
    wiz?.group === CLOUD_HOST_GROUP && shipIntent() === "public";
  const showPut = Boolean(wiz?.provider && providerHasEnvPut(wiz.provider));
  const cliDeploy =
    wiz?.id === "selfhost" ||
    wiz?.id === "umami" ||
    wiz?.id === "flowsint" ||
    wiz?.id === "medusa" ||
    wiz?.id === "cloudflare" ||
    wiz?.id === "vercel" ||
    wiz?.id === "netlify" ||
    wiz?.id === "fly" ||
    wiz?.id === "railway";
  const general = studioMode() === "general";
  paintProviderWizard({
    entry: wiz,
    panel: document.querySelector<HTMLElement>("#platforms-wizard"),
    titleEl: document.querySelector("#plat-wizard-title"),
    blurbEl: document.querySelector("#plat-wizard-blurb"),
    stepsEl: document.querySelector("#plat-wizard-steps"),
    openBtn: document.querySelector<HTMLButtonElement>("#plat-open"),
    secondaryBtn: document.querySelector<HTMLButtonElement>("#plat-portal-steps"),
    secondaryVisible:
      !general && Boolean(wiz?.provider && isPortalProvider(wiz.provider)),
    docsBtn: document.querySelector<HTMLButtonElement>("#plat-docs"),
    putBtn: document.querySelector<HTMLButtonElement>("#plat-put"),
    // Put as wizard primary only when there is no Deploy CTA (non–CLI-host cards).
    putVisible: showPut && !cliDeploy,
  });
  paintDeployStructureGate(wiz?.id);
  const structureGate = wiz
    ? evaluateStructureGate(wiz.id, lastDetected, { projectPath: projectPath() })
    : null;
  const structureBlocked = Boolean(
    structureGate && !structureGate.ok && structureGate.mode === "hard",
  );
  const putBtn = document.querySelector<HTMLButtonElement>("#plat-put");
  const importBtn = document.querySelector<HTMLButtonElement>("#plat-import-env");
  const openBtn = document.querySelector<HTMLButtonElement>("#plat-open");
  const dashBtn = document.querySelector<HTMLButtonElement>("#plat-dashboard");
  const liveBtn = document.querySelector<HTMLButtonElement>("#plat-open-live");
  const troubleBtn = document.querySelector<HTMLButtonElement>("#plat-troubleshoot");
  // Clear peer primaries — S2 assigns exactly one loud CTA.
  for (const b of [putBtn, openBtn, importBtn, dashBtn, liveBtn, troubleBtn]) {
    b?.classList.remove("primary");
  }
  if (openBtn && wiz && isHardStructureLane(wiz.id)) {
    openBtn.disabled = structureBlocked;
    openBtn.title = structureBlocked
      ? "Fix structure (see banner) before Deploy"
      : "";
  } else if (openBtn) {
    openBtn.disabled = false;
  }
  if (putBtn) {
    putBtn.hidden = !showPut;
  }
  if (importBtn) {
    importBtn.hidden = !showPut;
    importBtn.classList.add("gate-secondary");
  }
  if (putBtn) putBtn.classList.add("gate-secondary");
  if (dashBtn) {
    dashBtn.hidden = !(
      wiz?.id === "cloudflare" ||
      wiz?.id === "vercel" ||
      wiz?.id === "netlify" ||
      wiz?.id === "fly" ||
      wiz?.id === "railway" ||
      wiz?.id === "github-pages"
    );
    dashBtn.classList.add("gate-secondary");
  }
  const cancelServeBtn = document.querySelector<HTMLButtonElement>("#plat-cancel-deploy");
  if (cancelServeBtn) {
    // Visibility set in paintPlatCancelButtons
  }
  const localBtn = document.querySelector<HTMLButtonElement>("#plat-use-local");
  if (localBtn) {
    localBtn.hidden = !showLocal;
    localBtn.classList.add("gate-secondary");
  }
  paintDeployResultsBay();
  paintPlatCancelButtons(wiz, !(document.querySelector<HTMLElement>("#plat-results")?.hidden ?? true));
  // S2/F2 loudness: one recovery on fail → Open live (URL) → Deploy → Put.
  const resultsHidden =
    document.querySelector<HTMLElement>("#plat-results")?.hidden ?? true;
  const troubleLoud = Boolean(troubleBtn && !troubleBtn.hidden && !resultsHidden);
  const liveLoud = Boolean(liveBtn && !liveBtn.hidden && liveBtn.dataset.url);
  const retryBtnLoud = document.querySelector<HTMLButtonElement>("#plat-retry-deploy");
  if (retryBtnLoud) retryBtnLoud.classList.add("gate-secondary");
  if (troubleLoud && troubleBtn) {
    troubleBtn.classList.add("primary");
    troubleBtn.classList.remove("gate-secondary");
  } else if (liveLoud && liveBtn) {
    liveBtn.classList.add("primary");
  } else if (openBtn && (cliDeploy || !showPut)) {
    openBtn.classList.add("primary");
  } else if (putBtn && showPut && !cliDeploy) {
    putBtn.classList.remove("gate-secondary");
    putBtn.classList.add("primary");
  } else if (openBtn) {
    openBtn.classList.add("primary");
  }
}

function routeDetectChip(label: string) {
  const key = label.trim().toLowerCase();
  switch (key) {
    case "polar":
      setupPolarPortal();
      return;
    case "wrangler":
      openPlatformsCatalog({ selectId: "cloudflare" });
      return;
    case "vercel":
      openPlatformsCatalog({ selectId: "vercel" });
      return;
    case "netlify":
      openPlatformsCatalog({ selectId: "netlify" });
      return;
    case "fly":
      openPlatformsCatalog({ selectId: "fly" });
      return;
    case "railway":
      openPlatformsCatalog({ selectId: "railway" });
      return;
    case "pages":
    case "github pages":
      openPlatformsCatalog({ selectId: "github-pages" });
      return;
    case "github":
      // Auth/CI — Portal GitHub, not Pages card.
      void openPortalProvider("github");
      return;
    case "package.json":
    case "signet.toml":
      setView("publish");
      void refreshPublish();
      return;
    case "tauri":
      setView("sign");
      return;
    case "orbit":
      if (lastDetected?.yard_configured) {
        openPlatformsCatalog({ selectId: "orbit" });
      } else {
        openPlatformsCatalog({ selectId: "selfhost" });
        toast("Orbit only when this repo is Orbit-configured — Self-host is the local lane", "info", 4500);
      }
      return;
    case "umami":
      openPlatformsCatalog({ selectId: "umami" });
      return;
    case "flowsint":
      openPlatformsCatalog({ selectId: "selfhost" });
      toast("Flowsint lane is on hold — use Self-host or bind the compose yourself", "info", 5000);
      return;
    case "medusa":
      openPlatformsCatalog({ selectId: "selfhost" });
      toast("Medusa Desktop lane is on hold — new kits use Velocity", "info", 5000);
      return;
    default:
      setView("portal");
      void loadPortal(false);
  }
}


type HostFailClass =
  | "auth"
  | "missing_cli"
  | "build"
  | "network"
  | "account"
  | "detect"
  | "link"
  | "unknown";

type HostRecovery = {
  kind: HostFailClass;
  action: "login_cli" | "install_cli" | "preview_log" | "retry_deploy" | "open_dashboard";
  label: string;
  hint: string;
};

/** H4 — mirror orbityard hostdeploy::classify_host_failure (one primary recovery). */
function classifyHostDeployFailure(text: string, provider = "cloudflare"): HostRecovery {
  const l = text.toLowerCase();
  const loginCmd =
    provider === "vercel"
      ? "vercel login"
      : provider === "netlify"
        ? "netlify login"
        : provider === "fly"
          ? "fly auth login"
          : provider === "railway"
            ? "railway login"
            : "wrangler login";
  const install =
    provider === "vercel"
      ? "npm i -g vercel"
      : provider === "netlify"
        ? "npm i -g netlify-cli"
        : provider === "fly"
          ? "install flyctl"
          : provider === "railway"
            ? "npm i -g @railway/cli"
            : "npm i -g wrangler";

  if (
    l.includes("[link]") ||
    l.includes("fly launch") ||
    l.includes("railway link") ||
    l.includes("railway init") ||
    (l.includes("no fly.toml") && l.includes("launch"))
  ) {
    const hint =
      provider === "fly"
        ? "No fly.toml yet — run `fly launch` in the app folder (human), then Retry Deploy."
        : provider === "railway"
          ? "Not linked — run `railway link` or `railway init` (human), then Retry Deploy."
          : "Host project not linked — finish vendor link/init (human), then Retry Deploy.";
    return {
      kind: "link",
      action: "preview_log",
      label: "Preview log",
      hint,
    };
  }
  if (
    l.includes("not on path") ||
    ((l.includes("not found") || l.includes("cannot find") || l.includes("no such file")) &&
      (l.includes("wrangler") ||
        l.includes("vercel") ||
        l.includes("netlify") ||
        l.includes("fly") ||
        l.includes("railway"))) ||
    l.includes("is not recognized") ||
    l.includes("program not found")
  ) {
    return {
      kind: "missing_cli",
      action: "install_cli",
      label: "Preview log",
      hint: `Install the CLI on PATH (${install}), then retry Deploy.`,
    };
  }
  if (
    l.includes("no wrangler.toml") ||
    l.includes("no static index") ||
    l.includes("no vercel.json") ||
    l.includes("detect failed") ||
    /hostdeploy (cloudflare|vercel|netlify|fly|railway): no /.test(l)
  ) {
    return {
      kind: "detect",
      action: "open_dashboard",
      label: "Open dashboard",
      hint: "No deployable surface detected — add host config or a static index, or Open dashboard.",
    };
  }
  if (
    /not logged in|please log in|login required|wrangler login|vercel login|netlify login|fly auth login|railway login|authentication error|unauthorized|missing credentials|auth required|re-authenticate|no existing credentials|not authenticated|\[auth\]|sign in \(web\)|run login cli/i.test(
      l,
    )
  ) {
    return {
      kind: "auth",
      action: "login_cli",
      label: "Login CLI",
      hint: `Sign in with Login CLI (${loginCmd}) or Sign in (web). Studio never creates API tokens.`,
    };
  }
  if (
    /econnrefused|etimedout|enotfound|network error|network request failed|could not resolve|getaddrinfo|tls handshake|connection reset|timed out/.test(
      l,
    )
  ) {
    return {
      kind: "network",
      action: "retry_deploy",
      label: "Retry Deploy",
      hint: "Network error — check connectivity, then retry Deploy.",
    };
  }
  if (
    /permission denied|access denied|forbidden|status code 403|http 403|wrong account|not a member|insufficient permission|does not exist|project not found|couldn't find project|could not find project|no such project|pages project create|project doesn't exist/.test(
      l,
    ) ||
    (l.includes("project") && l.includes("not found") && !l.includes("module not found"))
  ) {
    const hint =
      provider === "vercel"
        ? "Host project missing or wrong account — Open dashboard to create/select the project, then Retry Deploy."
        : provider === "netlify"
          ? "Host project missing or wrong account — Open dashboard (or `netlify sites:create`), then Retry Deploy."
          : "Pages/Workers still blocked after create attempt — Open dashboard to confirm account/name, then Retry Deploy.";
    return {
      kind: "account",
      action: "open_dashboard",
      label: "Open dashboard",
      hint,
    };
  }
  if (
    /build failed|compile error|syntax error|typescript error|failed to compile|error ts|module not found|cannot find module|npm err/.test(
      l,
    )
  ) {
    return {
      kind: "build",
      action: "preview_log",
      label: "Preview log",
      hint: "Build failed — open Output, fix the project, then retry Deploy.",
    };
  }
  // Tag from last-run message: [...kind]
  const tag = /\[(auth|missing_cli|build|network|account|detect|link|unknown)\]/.exec(l);
  if (tag?.[1] === "auth") {
    return classifyHostDeployFailure("auth required wrangler login", provider);
  }
  if (tag?.[1] === "missing_cli") {
    return classifyHostDeployFailure("wrangler not on PATH", provider);
  }
  if (tag?.[1] === "network") {
    return classifyHostDeployFailure("getaddrinfo ENOTFOUND", provider);
  }
  if (tag?.[1] === "build") {
    return classifyHostDeployFailure("build failed module not found", provider);
  }
  if (tag?.[1] === "account") {
    return classifyHostDeployFailure("HTTP 403 Forbidden insufficient permission", provider);
  }
  if (tag?.[1] === "detect") {
    return classifyHostDeployFailure("hostdeploy cloudflare: no wrangler.toml", provider);
  }
  if (tag?.[1] === "link") {
    return classifyHostDeployFailure(
      "hostdeploy fly: no fly.toml — run `fly launch` [link]",
      provider,
    );
  }
  return {
    kind: "unknown",
    action: "preview_log",
    label: "Preview log",
    hint: "Deploy failed — open Output, fix if needed, then Deploy again from the host card.",
  };
}

function runHostRecoveryAction(
  recovery: HostRecovery,
  wiz: ProviderWizard,
): void {
  const provider = wiz.provider ?? wiz.id;
  switch (recovery.action) {
    case "login_cli":
      void openPortalLoginTerminal(provider);
      break;
    case "open_dashboard":
      void openHostDashboard(wiz);
      break;
    case "retry_deploy":
      void runHostedCliDeploy(wiz, {
        name: lastHostDeployName || defaultHostProjectName(),
        skipConfirm: true,
      });
      break;
    case "install_cli":
    case "preview_log":
    default:
      openOutputPreview();
      break;
  }
}

function isHostedCliDeployCard(id: string): boolean {
  return (
    id === "cloudflare" ||
    id === "vercel" ||
    id === "netlify" ||
    id === "fly" ||
    id === "railway"
  );
}

function showHostDeployStep(step: "setup" | "names") {
  const setup = document.querySelector<HTMLElement>("#host-deploy-step-setup");
  const names = document.querySelector<HTMLElement>("#host-deploy-step-names");
  if (setup) setup.hidden = step !== "setup";
  if (names) names.hidden = step !== "names";
}

function closeHostDeployDialog() {
  const dlg = document.querySelector<HTMLElement>("#host-deploy-dialog");
  if (dlg) dlg.hidden = true;
  pendingHostDeployWiz = null;
  pendingHostDeployTargetRel = "";
  pendingHostDeployEnvPackId = "";
  pendingHostDeployPacks = [];
  const targets = document.querySelector("#host-deploy-targets");
  if (targets) targets.innerHTML = "";
  const packs = document.querySelector("#host-deploy-env-packs");
  if (packs) packs.innerHTML = "";
  const warnEl = document.querySelector<HTMLElement>("#host-deploy-import-mismatch");
  if (warnEl) {
    warnEl.hidden = true;
    warnEl.textContent = "";
  }
  const switchBtn = document.querySelector<HTMLButtonElement>("#host-deploy-switch-hint");
  if (switchBtn) switchBtn.hidden = true;
  const importList = document.querySelector("#host-deploy-import-list");
  if (importList) importList.innerHTML = "";
  showHostDeployStep("setup");
}

function staleOrbityardCliHint(stderr: string, stdout: string): boolean {
  const blob = `${stderr}\n${stdout}`;
  return /unexpected argument ['`]?--list-packs/i.test(blob);
}

function hostDeployPackHint(): string {
  const pack = pendingHostDeployPacks.find(
    (p) => (p.id || p.file) === pendingHostDeployEnvPackId,
  );
  return (pack?.host_hint || "").toString().toLowerCase();
}

function hostDeployImportMismatch(provider: string): boolean {
  const hint = hostDeployPackHint();
  return (
    Boolean(pendingHostDeployEnvPackId) &&
    Boolean(hint) &&
    providerHasEnvPut(hint) &&
    hint !== provider
  );
}

function bindHostDeploySwitchHint(hint: string) {
  const switchBtn = document.querySelector<HTMLButtonElement>("#host-deploy-switch-hint");
  if (!switchBtn) return;
  const title =
    PLATFORM_WIZARDS.find((w) => (w.provider || w.id).toLowerCase() === hint)?.title || hint;
  switchBtn.textContent = `Switch to ${title} Deploy`;
  switchBtn.onclick = () => {
    const targetRel = pendingHostDeployTargetRel;
    const packId = pendingHostDeployEnvPackId;
    const wiz = PLATFORM_WIZARDS.find((w) => (w.provider || w.id).toLowerCase() === hint);
    closeHostDeployDialog();
    if (!wiz || !isHostedCliDeployCard(wiz.id)) {
      toast(`No Deploy dialog for ${hint} — use Import env… on that card`, "info", 6000);
      if (wiz) selectPlatform(wiz.id);
      return;
    }
    pendingHostDeployRestore = { targetRel, packId };
    selectPlatform(wiz.id);
    void openHostDeployDialog(wiz);
  };
}

/** Pack host_hint ≠ Deploy card → warn + default Import off + Switch CTA (slice 6–7). */
function syncHostDeployImportMismatch(provider: string) {
  const importCb = document.querySelector<HTMLInputElement>("#host-deploy-import-env");
  const warnEl = document.querySelector<HTMLElement>("#host-deploy-import-mismatch");
  const switchBtn = document.querySelector<HTMLButtonElement>("#host-deploy-switch-hint");
  const pack = pendingHostDeployPacks.find(
    (p) => (p.id || p.file) === pendingHostDeployEnvPackId,
  );
  const hint = hostDeployPackHint();
  const mismatch = hostDeployImportMismatch(provider);
  if (warnEl) {
    warnEl.hidden = !mismatch;
    warnEl.textContent = mismatch
      ? `Pack hints ${hint}; import uses ${provider} (this Deploy card). Leave Import off unless you mean that.`
      : "";
  }
  if (switchBtn) {
    switchBtn.hidden = !mismatch;
    if (mismatch) bindHostDeploySwitchHint(hint);
  }
  if (!importCb || importCb.disabled) return;
  if (mismatch) {
    importCb.checked = false;
  } else if ((pack?.nonempty_count ?? 0) > 0) {
    importCb.checked = true;
  } else {
    importCb.checked = false;
  }
}

function firstImportPutError(report: EnvImportReport | null): string {
  if (!report?.put?.length) return "";
  for (const p of report.put) {
    if (p && p.ok === false && p.error) return polishImportErrorText(String(p.error));
  }
  return "";
}

/** Strip leftover ANSI / trim for toast (CLI also scrubs; belt-and-suspenders). */
function polishImportErrorText(raw: string): string {
  let t = raw
    .replace(/\u001b\[[0-9;]*[A-Za-z]/g, "")
    .replace(/\[[0-9;]*m/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (/logs were written/i.test(t)) {
    t = t.replace(/\s*Logs were written to[^.]*\.?/gi, "").trim();
  }
  return t.slice(0, 160);
}

function importErrorIsMissingProject(text: string, stderr = "", stdout = ""): boolean {
  const blob = `${text}\n${stderr}\n${stdout}`;
  return (
    /\[missing_project\]/i.test(blob) ||
    (/does not exist/i.test(blob) && /project/i.test(blob))
  );
}

function parseEnvImportReport(stdout: string): EnvImportReport | null {
  try {
    return JSON.parse(stdout.trim() || "{}") as EnvImportReport;
  } catch {
    return null;
  }
}

type HostDeployReportJson = {
  ok?: boolean;
  provider?: string;
  lane?: string;
  urls?: string[];
  message?: string;
};

type UmamiDeployReportJson = {
  ok?: boolean;
  admin_url?: string | null;
  heartbeat_ok?: boolean;
  message?: string;
};

type FlowsintDeployReportJson = {
  ok?: boolean;
  app_url?: string | null;
  api_health_ok?: boolean;
  message?: string;
};

type MedusaDeployReportJson = {
  ok?: boolean;
  admin_url?: string | null;
  heartbeat_ok?: boolean;
  message?: string;
};

/** Last JSON object in umami deploy stdout (stream lines precede the report). */
function parseUmamiDeployReport(stdout: string): UmamiDeployReportJson | null {
  const trimmed = (stdout || "").trim();
  if (!trimmed) return null;
  const tryParse = (s: string): UmamiDeployReportJson | null => {
    try {
      const obj = JSON.parse(s) as UmamiDeployReportJson;
      if (obj && typeof obj === "object" && ("admin_url" in obj || "heartbeat_ok" in obj)) {
        return obj;
      }
    } catch {
      /* not json */
    }
    return null;
  };
  const direct = tryParse(trimmed);
  if (direct) return direct;
  const start = trimmed.lastIndexOf("{");
  if (start >= 0) return tryParse(trimmed.slice(start));
  return null;
}

/** Last JSON object in flowsint deploy stdout (stream lines precede the report). */
function parseFlowsintDeployReport(stdout: string): FlowsintDeployReportJson | null {
  const trimmed = (stdout || "").trim();
  if (!trimmed) return null;
  const tryParse = (s: string): FlowsintDeployReportJson | null => {
    try {
      const obj = JSON.parse(s) as FlowsintDeployReportJson;
      if (obj && typeof obj === "object" && ("app_url" in obj || "api_health_ok" in obj)) {
        return obj;
      }
    } catch {
      /* not json */
    }
    return null;
  };
  const direct = tryParse(trimmed);
  if (direct) return direct;
  const start = trimmed.lastIndexOf("{");
  if (start >= 0) return tryParse(trimmed.slice(start));
  return null;
}

/** Last JSON object in medusa deploy stdout (stream lines precede the report). */
function parseMedusaDeployReport(stdout: string): MedusaDeployReportJson | null {
  const trimmed = (stdout || "").trim();
  if (!trimmed) return null;
  const tryParse = (s: string): MedusaDeployReportJson | null => {
    try {
      const obj = JSON.parse(s) as MedusaDeployReportJson;
      if (obj && typeof obj === "object" && ("admin_url" in obj || "heartbeat_ok" in obj)) {
        return obj;
      }
    } catch {
      /* not json */
    }
    return null;
  };
  const direct = tryParse(trimmed);
  if (direct) return direct;
  const start = trimmed.lastIndexOf("{");
  if (start >= 0) return tryParse(trimmed.slice(start));
  return null;
}

/** Last JSON object in hostdeploy stdout (stream lines precede the report). */
function parseHostDeployReport(stdout: string): HostDeployReportJson | null {
  const trimmed = (stdout || "").trim();
  if (!trimmed) return null;
  const tryParse = (s: string): HostDeployReportJson | null => {
    try {
      const obj = JSON.parse(s) as HostDeployReportJson;
      if (obj && typeof obj === "object" && Array.isArray(obj.urls)) return obj;
    } catch {
      /* not json */
    }
    return null;
  };
  const whole = tryParse(trimmed);
  if (whole) return whole;
  const idx = trimmed.lastIndexOf("\n{");
  const start = idx >= 0 ? idx + 1 : trimmed.lastIndexOf("{");
  if (start < 0) return null;
  return tryParse(trimmed.slice(start));
}

function paintHostDeployEnvPacks(provider: string) {
  const grid = document.querySelector("#host-deploy-env-packs");
  const emptyEl = document.querySelector<HTMLElement>("#host-deploy-env-empty");
  const importCb = document.querySelector<HTMLInputElement>("#host-deploy-import-env");
  const warnEl = document.querySelector<HTMLElement>("#host-deploy-import-mismatch");
  if (!grid) return;
  const rel = (pendingHostDeployTargetRel || ".").replace(/\\/g, "/");
  const filtered = pendingHostDeployPacks.filter((p) => {
    const dir = (p.rel_dir || ".").replace(/\\/g, "/");
    if (rel === "." || rel === "") return true;
    return dir === rel || (p.file || "").replace(/\\/g, "/").startsWith(`${rel}/`);
  });
  if (!filtered.length) {
    grid.innerHTML = "";
    if (emptyEl) emptyEl.hidden = false;
    pendingHostDeployEnvPackId = "";
    if (importCb) {
      importCb.checked = false;
      importCb.disabled = true;
    }
    if (warnEl) {
      warnEl.hidden = true;
      warnEl.textContent = "";
    }
    return;
  }
  if (emptyEl) emptyEl.hidden = true;
  if (importCb) importCb.disabled = false;
  const restorePack = pendingHostDeployRestore?.packId || "";
  const preferred =
    (restorePack
      ? filtered.find((p) => (p.id || p.file) === restorePack)
      : undefined) ||
    filtered.find((p) => (p.host_hint || "").toLowerCase() === provider) ||
    filtered[0];
  pendingHostDeployEnvPackId = preferred?.id || preferred?.file || "";
  grid.innerHTML = filtered
    .map((p) => {
      const id = p.id || p.file || "";
      const selected = id === pendingHostDeployEnvPackId;
      const hint = p.host_hint ? String(p.host_hint) : "local";
      const keys = `${p.nonempty_count ?? 0}/${p.key_count ?? 0} keys`;
      return `<button type="button" class="env-pack-card" role="option" data-pack-id="${escapeHtml(
        id,
      )}" aria-selected="${selected}">
        <span class="title">${escapeHtml(p.label || "Env")}</span>
        <span class="path">${escapeHtml(p.file || id)}</span>
        <span class="meta">${escapeHtml(hint)} · ${escapeHtml(keys)}</span>
      </button>`;
    })
    .join("");
  grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((btn) => {
    btn.addEventListener("click", () => {
      pendingHostDeployEnvPackId = btn.getAttribute("data-pack-id") || "";
      grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((b) => {
        b.setAttribute("aria-selected", String(b === btn));
      });
      syncHostDeployImportMismatch(provider);
    });
  });
  syncHostDeployImportMismatch(provider);
}

function paintHostDeployTargets(
  scopes: NonNullable<ScopePlan["scopes"]>,
  provider: string,
) {
  const grid = document.querySelector("#host-deploy-targets");
  const emptyEl = document.querySelector<HTMLElement>("#host-deploy-targets-empty");
  if (!grid) return;
  const deployable = scopes.filter((s) => {
    const k = (s.kind || "").toLowerCase();
    const rel = (s.relative || ".").replace(/\\/g, "/");
    if (rel === "." || rel === "") return false;
    return ["web", "api", "docs", "desktop"].includes(k) || Boolean(s.provider);
  });
  if (!deployable.length) {
    grid.innerHTML = `<button type="button" class="env-pack-card" role="option" data-rel="." aria-selected="true">
      <span class="title">Project root</span>
      <span class="path">.</span>
      <span class="meta">whole repo</span>
    </button>`;
    pendingHostDeployTargetRel = ".";
    if (emptyEl) emptyEl.hidden = true;
    paintHostDeployEnvPacks(provider);
    return;
  }
  if (emptyEl) emptyEl.hidden = true;
  const focus = (lastDeployFocusProject || "").replace(/\\/g, "/").toLowerCase();
  const restoreRel = (pendingHostDeployRestore?.targetRel || "").replace(/\\/g, "/");
  // A1: provider affinity first; focus only when that scope matches the Deploy card.
  let preferred =
    (restoreRel
      ? deployable.find((s) => (s.relative || "").replace(/\\/g, "/") === restoreRel)
      : undefined) ||
    deployable.find((s) => (s.provider || "").toLowerCase() === provider) ||
    deployable.find((s) => {
      const rel = (s.relative || "").replace(/\\/g, "/").toLowerCase();
      const matchesFocus = focus.endsWith(`/${rel}`) || focus.endsWith(rel);
      if (!matchesFocus) return false;
      const scopeProv = (s.provider || "").toLowerCase();
      return !scopeProv || scopeProv === provider;
    }) ||
    deployable[0];
  pendingHostDeployTargetRel = (preferred?.relative || ".").replace(/\\/g, "/");
  grid.innerHTML = deployable
    .map((s) => {
      const rel = (s.relative || ".").replace(/\\/g, "/");
      const selected = rel === pendingHostDeployTargetRel;
      const host = s.provider || s.kind || "app";
      return `<button type="button" class="env-pack-card" role="option" data-rel="${escapeHtml(
        rel,
      )}" aria-selected="${selected}">
        <span class="title">${escapeHtml(s.label || s.kind || rel)}</span>
        <span class="path">${escapeHtml(rel)}</span>
        <span class="meta">${escapeHtml(String(host))}</span>
      </button>`;
    })
    .join("");
  grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((btn) => {
    btn.addEventListener("click", () => {
      pendingHostDeployTargetRel = btn.getAttribute("data-rel") || ".";
      grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((b) => {
        b.setAttribute("aria-selected", String(b === btn));
      });
      paintHostDeployEnvPacks(provider);
    });
  });
  paintHostDeployEnvPacks(provider);
  if (pendingHostDeployRestore) {
    const wantPack = pendingHostDeployRestore.packId;
    pendingHostDeployRestore = null;
    if (wantPack && pendingHostDeployPacks.some((p) => (p.id || p.file) === wantPack)) {
      pendingHostDeployEnvPackId = wantPack;
      const packGrid = document.querySelector("#host-deploy-env-packs");
      packGrid?.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((b) => {
        b.setAttribute(
          "aria-selected",
          String((b.getAttribute("data-pack-id") || "") === wantPack),
        );
      });
      syncHostDeployImportMismatch(provider);
    }
  }
}

function closeEnvImportDialog() {
  const dlg = document.querySelector<HTMLElement>("#env-import-dialog");
  if (dlg) dlg.hidden = true;
  pendingEnvImportWiz = null;
  pendingEnvImportFile = "";
  pendingEnvImportPackId = "";
  pendingEnvImportPackDir = "";
  const list = document.querySelector("#env-import-list");
  if (list) list.innerHTML = "";
  const grid = document.querySelector("#env-pack-grid");
  if (grid) grid.innerHTML = "";
}

function showEnvImportStep(step: "packs" | "names") {
  const packs = document.querySelector<HTMLElement>("#env-import-step-packs");
  const names = document.querySelector<HTMLElement>("#env-import-step-names");
  if (packs) packs.hidden = step !== "packs";
  if (names) names.hidden = step !== "names";
}

function preferEnvPack(packs: EnvPack[], provider: string): EnvPack | null {
  if (!packs.length) return null;
  const focus = (lastDeployFocusProject || "").replace(/\\/g, "/").toLowerCase();
  if (focus) {
    const byFocus = packs.find((p) => {
      const dir = (p.rel_dir || "").replace(/\\/g, "/").toLowerCase();
      const file = (p.file || "").replace(/\\/g, "/").toLowerCase();
      return focus.endsWith(`/${dir}`) || focus.endsWith(dir) || file.includes(dir);
    });
    if (byFocus) return byFocus;
  }
  return (
    packs.find((p) => (p.host_hint || "").toLowerCase() === provider) || packs[0] || null
  );
}

function paintEnvPackGrid(packs: EnvPack[], provider: string) {
  const grid = document.querySelector("#env-pack-grid");
  const emptyEl = document.querySelector<HTMLElement>("#env-pack-empty");
  const cont = document.querySelector<HTMLButtonElement>("#env-pack-continue");
  if (!grid) return;
  if (!packs.length) {
    grid.innerHTML = "";
    if (emptyEl) emptyEl.hidden = false;
    if (cont) cont.disabled = true;
    return;
  }
  if (emptyEl) emptyEl.hidden = true;
  const preferred = preferEnvPack(packs, provider);
  pendingEnvImportPackId = preferred?.id || preferred?.file || "";
  pendingEnvImportPackDir = preferred?.rel_dir || "";
  grid.innerHTML = packs
    .map((p) => {
      const id = p.id || p.file || "";
      const selected = id === pendingEnvImportPackId;
      const hint = p.host_hint ? String(p.host_hint) : "local";
      const keys = `${p.nonempty_count ?? 0}/${p.key_count ?? 0} keys`;
      return `<button type="button" class="env-pack-card" role="option" data-pack-id="${escapeHtml(
        id,
      )}" data-rel-dir="${escapeHtml(p.rel_dir || "")}" aria-selected="${selected}">
        <span class="title">${escapeHtml(p.label || p.kind || "App")}</span>
        <span class="path">${escapeHtml(p.file || id)}</span>
        <span class="meta">${escapeHtml(hint)} · ${escapeHtml(keys)}</span>
      </button>`;
    })
    .join("");
  const sync = () => {
    if (cont) cont.disabled = !pendingEnvImportPackId;
  };
  grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((btn) => {
    btn.addEventListener("click", () => {
      pendingEnvImportPackId = btn.getAttribute("data-pack-id") || "";
      pendingEnvImportPackDir = btn.getAttribute("data-rel-dir") || "";
      grid.querySelectorAll<HTMLButtonElement>(".env-pack-card").forEach((b) => {
        b.setAttribute("aria-selected", String(b === btn));
      });
      sync();
    });
  });
  sync();
}

function envImportCandidateRowsHtml(
  candidates: EnvImportCandidate[],
  checkboxClass: string,
): string {
  return candidates
    .map((c) => {
      const name = c.name ?? "";
      const denied = Boolean(c.denied);
      const empty = Boolean(c.empty);
      const disabled = denied || empty;
      const cls = [denied ? "denied" : "", empty ? "empty" : ""].filter(Boolean).join(" ");
      const tip = denied
        ? "skipped — shell/local name"
        : empty
          ? "empty in .env — fill a value locally, then reopen Import"
          : "value masked (stays on this PC)";
      const why = denied ? "blocked" : empty ? "empty in .env" : "••••";
      return `<li class="${cls}">
        <input type="checkbox" class="${checkboxClass}" data-name="${escapeHtml(name)}" ${
          disabled ? "disabled" : "checked"
        } aria-label="${escapeHtml(name)}" title="${escapeHtml(tip)}" />
        <span title="${escapeHtml(tip)}">${escapeHtml(name)}</span>
        <span class="mask" title="${escapeHtml(tip)}">${escapeHtml(why)}</span>
      </li>`;
    })
    .join("");
}

function envImportReadySummary(candidates: EnvImportCandidate[]): string {
  const ready = candidates.filter((c) => c.name && !c.denied && !c.empty).length;
  const empty = candidates.filter((c) => c.name && c.empty && !c.denied).length;
  const denied = candidates.filter((c) => c.name && c.denied).length;
  const parts = [`${ready} ready to push`];
  if (empty) parts.push(`${empty} empty in .env (not selectable)`);
  if (denied) parts.push(`${denied} blocked`);
  return parts.join(" · ");
}

function paintEnvImportCandidates(report: EnvImportReport) {
  const list = document.querySelector("#env-import-list");
  const emptyEl = document.querySelector<HTMLElement>("#env-import-empty");
  const fileEl = document.querySelector("#env-import-file");
  const confirmBtn = document.querySelector<HTMLButtonElement>("#env-import-confirm");
  const candidates = (report.candidates ?? []).filter((c) => c.name);
  if (fileEl) {
    const src = report.file
      ? `Source: ${report.file}`
      : "Source: local .dev.vars / .env (auto)";
    fileEl.textContent = candidates.length
      ? `${src} — ${envImportReadySummary(candidates)}`
      : src;
  }
  if (!list) return;
  if (!candidates.length) {
    list.innerHTML = "";
    if (emptyEl) emptyEl.hidden = false;
    if (confirmBtn) confirmBtn.disabled = true;
    return;
  }
  if (emptyEl) emptyEl.hidden = true;
  list.innerHTML = envImportCandidateRowsHtml(candidates, "env-import-name");
  const syncConfirm = () => {
    if (!confirmBtn) return;
    confirmBtn.disabled = !list.querySelector<HTMLInputElement>(
      ".env-import-name:checked:not(:disabled)",
    );
  };
  list.querySelectorAll<HTMLInputElement>(".env-import-name").forEach((cb) => {
    cb.addEventListener("change", syncConfirm);
  });
  syncConfirm();
}

async function loadEnvImportNames(provider: string, packId: string) {
  // Root-relative pack ids (apps/website/.env) — never run under Deploy focus cwd.
  const args = ["secrets", "--import-env", "--provider", provider, "--dry-run"];
  if (packId) args.push("--file", packId);
  const dry = await run(args, {
    silent: true,
    quietToast: true,
    quietHeader: true,
    busyLabel: "Reading env pack…",
  });
  if (!dry) return null;
  if (!dry.ok) {
    toast(
      dry.stderr.trim().slice(0, 180) || "Env dry-run failed — pick another pack",
      "err",
      7000,
    );
    return null;
  }
  try {
    return JSON.parse(dry.stdout.trim() || "{}") as EnvImportReport;
  } catch {
    toast("Could not read env import plan — see Output", "err");
    return null;
  }
}

async function openEnvImportDialog(wiz: ProviderWizard) {
  const provider = (wiz.provider ?? "").trim().toLowerCase();
  if (!providerHasEnvPut(provider)) {
    toast("Import env is for Cloudflare / Vercel / Netlify / Fly / Railway", "info");
    return;
  }
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  const gate = evaluateStructureGate(wiz.id, lastDetected, { projectPath: projectPath() });
  if (gate && !gate.ok) {
    toast(gate.reason, "info", 9000, structureGateToastActions(gate));
    // Soft — Import may still help after rebind; dialog continues.
  }
  pendingEnvImportWiz = wiz;
  const dlg = document.querySelector<HTMLElement>("#env-import-dialog");
  const titleEl = document.querySelector("#env-import-dialog-title");
  const blurbEl = document.querySelector("#env-import-dialog-blurb");
  const confirmBtn = document.querySelector<HTMLButtonElement>("#env-import-confirm");
  if (titleEl) titleEl.textContent = `Import env · ${wiz.title}`;
  if (blurbEl) {
    blurbEl.textContent =
      "Pick the app pack, then confirm names. Values stay on this PC → host CLI only. Not saved in Orbit Yard.";
  }
  if (confirmBtn) {
    confirmBtn.disabled = true;
    confirmBtn.textContent = "Push to host";
  }
  const thenDeploy = document.querySelector<HTMLInputElement>("#env-import-then-deploy");
  if (thenDeploy) {
    thenDeploy.checked = false;
    // Tier B (Fly/Railway): Import only — no Orbit hostdeploy stream.
    thenDeploy.disabled = !isHostedCliDeployCard(wiz.id);
    const thenLabel = thenDeploy.closest("label");
    if (thenLabel) thenLabel.hidden = !isHostedCliDeployCard(wiz.id);
  }

  // List packs from monorepo root (not focus-only) so Website + API both appear.
  const listed = await run(["secrets", "--list-packs"], {
    silent: true,
    quietToast: true,
    quietHeader: true,
    busyLabel: "Finding env packs…",
  });
  if (!listed) {
    closeEnvImportDialog();
    return;
  }
  if (staleOrbityardCliHint(listed.stderr, listed.stdout)) {
    toast(
      "Old orbityard — rebuild with cargo build -p orbityard (and --release), restart Desktop, then retry",
      "err",
      12000,
    );
    closeEnvImportDialog();
    return;
  }
  let packsReport: EnvPacksReport = {};
  try {
    packsReport = JSON.parse(listed.stdout.trim() || "{}") as EnvPacksReport;
  } catch {
    toast("Could not list env packs", "err");
    closeEnvImportDialog();
    return;
  }
  const packs = packsReport.packs ?? [];
  if (!listed.ok || !packs.length) {
    toast(
      "No .env / .dev.vars found under scopes — e.g. apps/website/.env (not only the repo root)",
      "err",
      8000,
    );
    closeEnvImportDialog();
    return;
  }

  if (dlg) dlg.hidden = false;

  if (packs.length === 1) {
    const only = packs[0];
    pendingEnvImportPackId = only?.id || only?.file || "";
    pendingEnvImportPackDir = only?.rel_dir || "";
    const report = await loadEnvImportNames(provider, pendingEnvImportPackId);
    if (!report) {
      closeEnvImportDialog();
      return;
    }
    pendingEnvImportFile = report.file ?? pendingEnvImportPackId;
    paintEnvImportCandidates(report);
    showEnvImportStep("names");
    return;
  }

  paintEnvPackGrid(packs, provider);
  showEnvImportStep("packs");
}

async function continueEnvImportFromPack() {
  const wiz = pendingEnvImportWiz;
  if (!wiz?.provider || !pendingEnvImportPackId) {
    toast("Select an env pack first", "info");
    return;
  }
  const provider = wiz.provider.trim().toLowerCase();
  // Align Deploy-after with the same ship unit directory.
  if (pendingEnvImportPackDir && pendingEnvImportPackDir !== ".") {
    const abs = scopeAbsolutePath(pendingEnvImportPackDir);
    if (abs) lastDeployFocusProject = abs;
  }
  const report = await loadEnvImportNames(provider, pendingEnvImportPackId);
  if (!report) return;
  pendingEnvImportFile = report.file ?? pendingEnvImportPackId;
  paintEnvImportCandidates(report);
  showEnvImportStep("names");
}

async function confirmEnvImport() {
  const wiz = pendingEnvImportWiz;
  if (!wiz?.provider) {
    closeEnvImportDialog();
    return;
  }
  const provider = wiz.provider.trim().toLowerCase();
  const selected = Array.from(
    document.querySelectorAll<HTMLInputElement>(".env-import-name:checked:not(:disabled)"),
  )
    .map((el) => el.getAttribute("data-name") ?? "")
    .filter(Boolean);
  if (!selected.length) {
    toast("Select at least one name to push", "info");
    return;
  }
  const thenDeploy = Boolean(
    document.querySelector<HTMLInputElement>("#env-import-then-deploy")?.checked,
  );
  const file = pendingEnvImportFile;
  const packId = pendingEnvImportPackId;
  const focusOverride =
    pendingEnvImportPackDir && pendingEnvImportPackDir !== "."
      ? scopeAbsolutePath(pendingEnvImportPackDir)
      : lastDeployFocusProject;
  closeEnvImportDialog();

  const args = ["secrets", "--import-env", "--provider", provider, "--yes"];
  // Prefer --file from monorepo root so pack ids like apps/website/.env resolve.
  if (file) args.push("--file", file);
  else if (packId) args.push("--file", packId);
  const hostName = (lastHostDeployName || defaultHostProjectName()).trim();
  if (hostName) args.push("--host-name", hostName);
  for (const name of selected) {
    args.push("--name", name);
  }
  applyOutputDock(true);
  const result = await run(args, {
    quietToast: true,
    busyLabel: "Pushing secrets…",
  });
  if (!result) return;
  if (result.cancelled) {
    toast("Env import cancelled", "ok", 3500);
    return;
  }
  const report = parseEnvImportReport(result.stdout) ?? {};
  const okCount = (report.put ?? []).filter((p) => p.ok).length;
  const failCount = (report.put ?? []).filter((p) => !p.ok).length;
  if (result.ok && failCount === 0) {
    toast(
      okCount
        ? `Pushed ${okCount} secret(s) to ${wiz.title} — values not stored in Studio`
        : "Import finished — nothing to push",
      "ok",
      5500,
    );
  } else {
    const detail = firstImportPutError(report);
    const missingProject = importErrorIsMissingProject(
      detail,
      result.stderr,
      result.stdout,
    );
    toast(
      missingProject
        ? `Env import: host project missing — Deploy once to create it, then Import again`
        : detail
          ? `Env import: ${okCount} ok, ${failCount || "?"} failed — ${detail}`
          : `Env import: ${okCount} ok, ${failCount || "?"} failed — names only in Output`,
      "err",
      9000,
    );
  }
  if (thenDeploy && result.ok && failCount === 0 && isHostedCliDeployCard(wiz.id)) {
    if (focusOverride) lastDeployFocusProject = focusOverride;
    void runHostedCliDeploy(wiz);
  }
}

async function openHostDeployDialog(wiz: ProviderWizard) {
  pendingHostDeployWiz = wiz;
  const dlg = document.querySelector<HTMLElement>("#host-deploy-dialog");
  const titleEl = document.querySelector("#host-deploy-dialog-title");
  const blurbEl = document.querySelector("#host-deploy-dialog-blurb");
  const nameInput = document.querySelector<HTMLInputElement>("#host-deploy-name");
  const metaEl = document.querySelector("#host-deploy-meta");
  if (!dlg || !nameInput) {
    void runHostedCliDeploy(wiz, { skipConfirm: true });
    return;
  }
  const provider = (wiz.provider ?? wiz.id).toLowerCase();
  if (titleEl) titleEl.textContent = `Deploy · ${wiz.title}`;
  if (blurbEl) {
    blurbEl.textContent =
      "Choose the app target and .env pack, then Deploy. Secrets stay on this PC → host CLI; hosts still bill you.";
  }
  const remembered =
    lastHostDeployName ||
    (typeof localStorage !== "undefined"
      ? localStorage.getItem(`ship.hostdeploy.name.${wiz.id}`) || ""
      : "");
  nameInput.value = remembered || defaultHostProjectName();
  // Fly/Railway app identity comes from fly.toml / railway link — name is display-only.
  nameInput.readOnly = wiz.id === "fly" || wiz.id === "railway";
  if (metaEl) {
    metaEl.textContent =
      wiz.id === "cloudflare"
        ? "Lane: Cloudflare · create-if-missing · then deploy"
        : wiz.id === "vercel"
          ? "Lane: Vercel production · project name → vercel --project"
          : wiz.id === "fly"
            ? "Lane: Fly · needs fly.toml from `fly launch` (human) · then fly deploy"
            : wiz.id === "railway"
              ? "Lane: Railway · needs `railway link` (human) · then railway up"
              : "Lane: Netlify production · editable site name";
  }

  dlg.hidden = false;
  showHostDeployStep("setup");

  // Load scopes + packs (bound monorepo root — not focus-only).
  let scopes = lastScopes?.scopes ?? [];
  if (!scopes.length) {
    const plan = (await loadJsonCmd(["scopes", "--project", projectPath()], {
      silent: true,
    })) as ScopePlan | null;
    if (plan) {
      applyScopes(plan);
      scopes = plan.scopes ?? [];
    }
  }
  const listed = await run(["secrets", "--list-packs"], {
    silent: true,
    quietToast: true,
    quietHeader: true,
    busyLabel: "Finding targets…",
  });
  pendingHostDeployPacks = [];
  if (listed && staleOrbityardCliHint(listed.stderr, listed.stdout)) {
    toast(
      "Desktop is using an old orbityard — rebuild: cargo build -p orbityard (debug+release), then restart Desktop",
      "err",
      12000,
    );
  } else if (listed?.ok) {
    try {
      const report = JSON.parse(listed.stdout.trim() || "{}") as EnvPacksReport;
      pendingHostDeployPacks = report.packs ?? [];
    } catch {
      /* ignore */
    }
  }
  paintHostDeployTargets(scopes, provider);
  nameInput.focus();
  nameInput.select();
}

/** Opens confirm dialog for hosted CLI Deploy (Self-host stays one-click). */
function promptHostedCliDeploy(wiz: ProviderWizard): void {
  void openHostDeployDialog(wiz);
}

function offerDeployWithoutImport(
  wiz: ProviderWizard,
  name: string,
  focusAbs: string | null,
  message: string,
  opts?: { switchHint?: string; targetRel?: string; packId?: string },
) {
  // F2 — one recovery only (mismatch → Switch; else Deploy without import).
  const hint = (opts?.switchHint || "").toLowerCase();
  const actions: ToastAction[] = [];
  if (hint && providerHasEnvPut(hint)) {
    const title =
      PLATFORM_WIZARDS.find((w) => (w.provider || w.id).toLowerCase() === hint)?.title || hint;
    const restoreTarget = opts?.targetRel || "";
    const restorePack = opts?.packId || "";
    actions.push({
      id: "switch_hint_deploy",
      label: `Switch to ${title}`,
      icon: "continue",
      run: () => {
        const target = PLATFORM_WIZARDS.find(
          (w) => (w.provider || w.id).toLowerCase() === hint,
        );
        if (!target || !isHostedCliDeployCard(target.id)) {
          toast(`No Deploy dialog for ${hint}`, "info", 5000);
          if (target) selectPlatform(target.id);
          return;
        }
        pendingHostDeployRestore = { targetRel: restoreTarget, packId: restorePack };
        selectPlatform(target.id);
        void openHostDeployDialog(target);
      },
    });
  } else {
    actions.push({
      id: "deploy_without_import",
      label: "Deploy without import",
      icon: "continue",
      run: () => {
        if (focusAbs) lastDeployFocusProject = focusAbs;
        void runHostedCliDeploy(wiz, { name, skipConfirm: true });
      },
    });
  }
  const msg =
    hint && providerHasEnvPut(hint)
      ? `${message} — Next: switch host card (Output has details).`
      : `${message} — Next: Deploy without import (Output has details).`;
  toast(msg, "err", 14000, actions);
}

function paintHostDeployImportCandidates(report: EnvImportReport) {
  const list = document.querySelector("#host-deploy-import-list");
  const emptyEl = document.querySelector<HTMLElement>("#host-deploy-import-empty");
  const fileEl = document.querySelector("#host-deploy-import-file");
  const confirmBtn = document.querySelector<HTMLButtonElement>("#host-deploy-confirm-import");
  const candidates = (report.candidates ?? []).filter((c) => c.name);
  if (fileEl) {
    const base = report.file
      ? `Confirm names to push → ${report.file}`
      : "Confirm names to push";
    fileEl.textContent = candidates.length
      ? `${base} — ${envImportReadySummary(candidates)}`
      : base;
  }
  if (!list) return;
  if (!candidates.length) {
    list.innerHTML = "";
    if (emptyEl) emptyEl.hidden = false;
    if (confirmBtn) confirmBtn.disabled = true;
    return;
  }
  if (emptyEl) emptyEl.hidden = true;
  list.innerHTML = envImportCandidateRowsHtml(candidates, "host-deploy-import-name");
  const syncConfirm = () => {
    if (!confirmBtn) return;
    confirmBtn.disabled = !list.querySelector<HTMLInputElement>(
      ".host-deploy-import-name:checked:not(:disabled)",
    );
  };
  list.querySelectorAll<HTMLInputElement>(".host-deploy-import-name").forEach((cb) => {
    cb.addEventListener("change", syncConfirm);
  });
  syncConfirm();
}

async function runHostDeployImportThenDeploy(
  wiz: ProviderWizard,
  provider: string,
  packId: string,
  selectedNames: string[],
  deployName: string,
  focusAbs: string | null,
  packHint = "",
) {
  const args = [
    "secrets",
    "--import-env",
    "--provider",
    provider,
    "--file",
    packId,
    "--yes",
  ];
  const hostName = deployName.trim();
  if (hostName) args.push("--host-name", hostName);
  for (const n of selectedNames) args.push("--name", n);
  applyOutputDock(true);
  const putResult = await run(args, {
    quietToast: true,
    busyLabel: "Importing env…",
  });
  if (!putResult) return;
  const packDir = packId.includes("/")
    ? packId.replace(/\\/g, "/").replace(/\/[^/]+$/, "") || "."
    : ".";
  const mismatchOpts =
    packHint && packHint !== provider
      ? { switchHint: packHint, targetRel: packDir, packId }
      : undefined;
  if (putResult.cancelled) {
    offerDeployWithoutImport(
      wiz,
      deployName,
      focusAbs,
      "Env import cancelled — Deploy without import if you want",
      mismatchOpts,
    );
    return;
  }
  if (staleOrbityardCliHint(putResult.stderr, putResult.stdout)) {
    toast("Old orbityard binary — rebuild CLI, then retry Deploy", "err", 10000);
    return;
  }
  if (!putResult.ok) {
    const blob = `${putResult.stderr}\n${putResult.stdout}`;
    const missingCli = /\[missing_cli\]|not on PATH|program not found/i.test(blob);
    const report = parseEnvImportReport(putResult.stdout);
    const detail = firstImportPutError(report);
    const missingProject = importErrorIsMissingProject(
      detail,
      putResult.stderr,
      putResult.stdout,
    );
    const base = missingCli
      ? `Env import: ${wiz.title} CLI missing [missing_cli] — Deploy without import, or install CLI`
      : missingProject
        ? `Env import: Pages project “${deployName}” missing — Deploy without import creates it, then Import again`
        : detail
          ? `Env import failed — ${detail}`
          : "Env import failed — see Output; Deploy without import if you want";
    offerDeployWithoutImport(wiz, deployName, focusAbs, base, mismatchOpts);
    return;
  }
  toast("Env pushed to host — starting Deploy", "ok", 3500);
  if (focusAbs) lastDeployFocusProject = focusAbs;
  await runHostedCliDeploy(wiz, { name: deployName, skipConfirm: true });
}

/** Setup step → optional name confirm (A3) → put → Deploy. */
async function confirmHostDeployAllInOne() {
  const wiz = pendingHostDeployWiz;
  const nameInput = document.querySelector<HTMLInputElement>("#host-deploy-name");
  if (!wiz) {
    closeHostDeployDialog();
    return;
  }
  const name = (nameInput?.value ?? "").trim() || defaultHostProjectName();
  const provider = (wiz.provider ?? wiz.id).toLowerCase();
  const importEnv = Boolean(
    document.querySelector<HTMLInputElement>("#host-deploy-import-env")?.checked,
  );
  const packId = pendingHostDeployEnvPackId;
  const targetRel = pendingHostDeployTargetRel || ".";
  const focusAbs =
    targetRel && targetRel !== "." ? scopeAbsolutePath(targetRel) : null;
  if (focusAbs) lastDeployFocusProject = focusAbs;

  if (importEnv && packId && providerHasEnvPut(provider)) {
    if (hostDeployImportMismatch(provider)) {
      const hint = hostDeployPackHint();
      const title =
        PLATFORM_WIZARDS.find((w) => (w.provider || w.id).toLowerCase() === hint)?.title ||
        hint;
      toast(
        `Import will push to ${provider}, not ${hint}. Switch Deploy card or leave Import off.`,
        "info",
        10000,
        [
          {
            id: "switch_before_import",
            label: `Switch to ${title}`,
            icon: "continue",
            run: () => {
              const targetRel = pendingHostDeployTargetRel;
              const restorePack = pendingHostDeployEnvPackId;
              const target = PLATFORM_WIZARDS.find(
                (w) => (w.provider || w.id).toLowerCase() === hint,
              );
              closeHostDeployDialog();
              if (!target || !isHostedCliDeployCard(target.id)) {
                if (target) selectPlatform(target.id);
                return;
              }
              pendingHostDeployRestore = { targetRel, packId: restorePack };
              selectPlatform(target.id);
              void openHostDeployDialog(target);
            },
          },
        ],
      );
      // Still allow names step if they insist — warn stays visible on file line.
    }
    const report = await loadEnvImportNames(provider, packId);
    if (!report) return;
    paintHostDeployImportCandidates(report);
    showHostDeployStep("names");
    const fileEl = document.querySelector("#host-deploy-import-file");
    if (fileEl && hostDeployImportMismatch(provider)) {
      const hint = hostDeployPackHint();
      const summary = envImportReadySummary(
        (report.candidates ?? []).filter((c) => c.name),
      );
      fileEl.textContent = `Confirm names to push → ${report.file || packId} (to ${provider}, pack hints ${hint}) — ${summary}`;
    }
    const pushBtn = document.querySelector<HTMLButtonElement>("#host-deploy-confirm-import");
    if (pushBtn) {
      pushBtn.onclick = () => {
        const selected = Array.from(
          document.querySelectorAll<HTMLInputElement>(
            ".host-deploy-import-name:checked:not(:disabled)",
          ),
        )
          .map((el) => el.getAttribute("data-name") ?? "")
          .filter(Boolean);
        if (!selected.length) {
          toast("Select at least one name to push", "info");
          return;
        }
        const hintAtConfirm = hostDeployPackHint();
        closeHostDeployDialog();
        void runHostDeployImportThenDeploy(
          wiz,
          provider,
          packId,
          selected,
          name,
          focusAbs,
          hintAtConfirm,
        );
      };
    }
    return;
  }

  closeHostDeployDialog();
  await runHostedCliDeploy(wiz, { name, skipConfirm: true });
}

async function ensureProviderCliAuth(
  wiz: ProviderWizard,
  opts?: { context?: "select" | "deploy" },
): Promise<boolean> {
  const provider = wiz.provider ?? wiz.id;
  const title = wiz.title;
  const context = opts?.context ?? "deploy";
  const check = await run(["hostdeploy", "--provider", provider, "--auth-check"], {
    silent: true,
    quietToast: true,
    quietHeader: true,
  });
  if (!check) return false;
  const blob = `${check.stdout}\n${check.stderr}`;
  let authRequired = false;
  let missingCli = false;
  let message = "";
  try {
    const j = JSON.parse(check.stdout.trim() || "{}") as {
      ok?: boolean;
      auth_required?: boolean;
      message?: string;
    };
    if (j.ok) return true;
    authRequired = Boolean(j.auth_required) || /\[auth\]/i.test(j.message ?? "");
    missingCli = /\[missing_cli\]/i.test(j.message ?? "");
    message = j.message ?? "";
  } catch {
    authRequired =
      !check.ok &&
      /not logged in|login required|auth_required|\[auth\]|please run login/i.test(blob);
    missingCli = /\[missing_cli\]|not on path/i.test(blob);
    message = blob.slice(0, 240);
  }
  if (check.ok) return true;

  if (missingCli) {
    toast(
      `${title}: install the vendor CLI on PATH, then Login CLI — ${message.slice(0, 140)}`,
      "err",
      10000,
      [
        {
          id: "preview",
          label: "Preview log",
          icon: "open",
          run: () => openOutputPreview(),
        },
      ],
    );
    return false;
  }

  if (authRequired || /not logged in|login required|\[auth\]/i.test(blob)) {
    // Never auto-spawn a terminal on card select — flashy and repeated.
    // Deploy path: toast CTA only; human opens Login CLI when ready.
    const actions: ToastAction[] = [
      {
        id: "login",
        label: "Login CLI",
        icon: "open",
        run: () => {
          void openPortalLoginTerminal(provider);
        },
      },
      {
        id: "recheck",
        label: context === "deploy" ? "Retry Deploy" : "Check again",
        icon: "continue",
        run: () => {
          if (context === "deploy") {
            void runHostedCliDeploy(wiz, {
              name: lastHostDeployName || defaultHostProjectName(),
              skipConfirm: true,
            });
          } else {
            void ensureProviderCliAuth(wiz, { context: "select" });
          }
        },
      },
    ];
    toast(
      context === "deploy"
        ? `${title}: sign in required — tap Login CLI, finish auth, then Retry Deploy`
        : `${title}: sign in required — tap Login CLI when ready (no auto terminal)`,
      "info",
      12000,
      actions,
    );
    return false;
  }
  toast(`${title}: ${message || "auth check failed"}`, "err", 8000);
  return false;
}

async function runHostedCliDeploy(
  wiz: ProviderWizard,
  opts?: { name?: string; skipConfirm?: boolean },
): Promise<void> {
  if (!opts?.skipConfirm && !opts?.name) {
    promptHostedCliDeploy(wiz);
    return;
  }
  if (isHardStructureLane(wiz.id)) {
    const structureOk = await assertStructureProceed(wiz.id);
    paintPlatformWizard();
    if (!structureOk) return;
  }
  const provider = wiz.provider ?? wiz.id;
  const title = wiz.title;
  const name = (opts?.name ?? (lastHostDeployName || defaultHostProjectName())).trim();
  if (name) {
    lastHostDeployName = name;
    try {
      localStorage.setItem(`ship.hostdeploy.name.${wiz.id}`, name);
    } catch {
      /* ignore quota */
    }
  }

  // Redirect unauthenticated operators to Login CLI before streaming deploy.
  const authed = await ensureProviderCliAuth(wiz, { context: "deploy" });
  if (!authed) return;

  applyOutputDock(true);
  cloudEvidenceClearedProvider = null;
  toast(`${title} Deploy — streaming CLI in Output`, "ok", 3500);
  const args = ["hostdeploy", "--provider", provider];
  if (
    name &&
    (wiz.id === "cloudflare" || wiz.id === "netlify" || wiz.id === "vercel")
  ) {
    args.push("--name", name);
  }
  const focus =
    lastDeployFocusProject && lastDeployFocusProject.length > 0
      ? lastDeployFocusProject
      : undefined;
  const result = await run(args, {
    quietToast: true,
    projectOverride: focus,
  });
  // One-shot focus — next Deploy from root uses the bound monorepo again.
  lastDeployFocusProject = null;
  if (!result) return;
  await refreshShipState();
  await refreshSessionNow();
  void refreshStatusProbes({ views: ["publish", "sign"], animate: false });
  const errText = `${result.stderr}\n${result.stdout}`;
  if (result.cancelled) {
    toast("hostdeploy cancelled", "err");
    return;
  }
  if (result.ok) {
    const report = parseHostDeployReport(result.stdout);
    const liveUrl =
      (report?.urls ?? []).find((u) => /^https:\/\//i.test(u) && !isLoopbackDeployUrl(u)) ||
      (lastRunState?.urls ?? []).find(
        (u) => /^https:\/\//i.test(String(u)) && !isLoopbackDeployUrl(String(u)),
      ) ||
      "";
    markProjectShipped(projectPath());
    const actions: ToastAction[] = [];
    if (liveUrl) {
      actions.push({
        id: "open_live",
        label: "Open live",
        icon: "open",
        run: () => {
          void openUrl(liveUrl).then(() => toast("Opened live URL", "ok", 2800));
        },
      });
    }
    // F5 — one secondary: next local repo, else vendor dashboard.
    const next = nextShipCandidate(projectPath());
    if (next) {
      const name = projectName(next);
      actions.push({
        id: "next_ship",
        label: `Next ship: ${name}`,
        icon: "continue",
        run: () => {
          void bindProject(next, true);
        },
      });
    } else {
      actions.push({
        id: "dash",
        label: "Open dashboard",
        icon: "open",
        run: () => {
          void openHostDashboard(wiz);
        },
      });
    }
    toast(
      liveUrl
        ? `${title} deploy ok — ${liveUrl}`
        : `${title} deploy ok — no public URL in CLI output; Open dashboard to confirm`,
      "ok",
      liveUrl ? 10000 : 8000,
      actions,
    );
    paintDeployResultsBay();
    paintNextShip();
    return;
  }
  const recovery = classifyHostDeployFailure(errText, provider);
  // Auth mid-deploy: toast CTAs only — do not auto-pop a terminal (unobtrusive UX).
  const toastActions: ToastAction[] = [];
  if (recovery.kind === "auth") {
    toastActions.push({
      id: "login",
      label: "Login CLI",
      icon: "open",
      run: () => {
        void openPortalLoginTerminal(provider);
      },
    });
  }
  toastActions.push({
    id: "primary",
    label: recovery.kind === "auth" ? "Retry Deploy" : recovery.label,
    icon:
      recovery.kind === "auth" ||
      recovery.action === "login_cli" ||
      recovery.action === "retry_deploy"
        ? "continue"
        : "open",
    run: () => {
      if (recovery.kind === "auth") {
        void runHostedCliDeploy(wiz, { name, skipConfirm: true });
      } else {
        runHostRecoveryAction(recovery, wiz);
      }
    },
  });
  toast(
    recovery.kind === "auth"
      ? `${title}: Next — Login CLI, then Retry Deploy`
      : `${title}: Next — ${recovery.hint}`,
    "err",
    11000,
    toastActions,
  );
}

function polishCtaLabel(raw: string): string {
  const t = raw.trim();
  if (t === "Start publish") return "Start publishing";
  if (t === "Continue publish") return "Continue publishing";
  return t;
}

function ctaStateFromPulseId(id: string | undefined, label: string): string {
  if (id === "open") return "open";
  if (id === "publish_continue" || /continue/i.test(label)) return "continue";
  if (/review/i.test(label)) return "review";
  if (id === "publish_start" || /start publish/i.test(label)) return "start";
  return "other";
}

function setNowCtaState(state: string) {
  const now = document.querySelector<HTMLElement>("#now");
  const primary = document.querySelector<HTMLButtonElement>("#now-primary");
  const hint = document.querySelector("#now-cta-hint");
  if (now) now.dataset.ready = state === "start" || state === "continue" ? state : "";
  if (primary) primary.dataset.cta = state;
  if (!hint) return;
  const mins = lastPublish?.minutes_remaining;
  const minsNote = mins != null ? ` · ~${mins} min left` : "";
  switch (state) {
    case "open":
      hint.textContent = "Bind a repo — then one click starts the publish workflow.";
      break;
    case "start":
      hint.textContent =
        "One click — we open the right portals; you confirm each step.";
      break;
    case "continue":
      hint.textContent = (() => {
        const summary = publishProgressSummary(lastPublish);
        return summary
          ? `${summary}. Continue automatic checks; Open/Confirm for steps you finish.`
          : `Pick up the current step${minsNote}. Confirm when the vendor UI is done.`;
      })();
      break;
    case "review":
      hint.textContent = "Open Publish to scan the completed pass or start another.";
      break;
    default:
      hint.textContent = "Follow the primary action — vendor UIs stay in your browser.";
  }
}

function applyNow(view: PublishView | null) {
  // Prefer pulse when available; fall back to publish view titles.
  if (lastPulse?.now) {
    applyPulseNow(lastPulse);
    return;
  }
  const title = document.querySelector("#now-title");
  const detail = document.querySelector("#now-detail");
  if (!title || !detail) return;
  const path = projectPath();
  if (!path) {
    title.textContent = "Open a project to see what’s next";
    detail.textContent =
      "Bind the folder you’re shipping. Then we sequence vendor UIs — you paste, sign, list, and deploy.";
    setNowCtaState("open");
    return;
  }
  if (view?.finished) {
    title.textContent = "Required gates done for this pass";
    detail.textContent = `${projectName(path)} — ${publishProgressSummary(view) || "All required gates done"}. Switch project or start another pass from Publish.`;
    setNowCtaState("review");
    return;
  }
  const cur = view?.current;
  if (cur?.title) {
    const summary = publishProgressSummary(view);
    title.textContent = cur.title;
    detail.textContent = `${cur.detail ?? "Open/Run on the official platform, then Confirm."}${summary ? ` (${summary})` : ""}`;
    setNowCtaState("continue");
    return;
  }
  title.textContent = `Pick up ${projectName(path)}`;
  detail.textContent =
    "Start the publish portal — doctor, env, sign, listing, deploy. You finish the vendor UIs; Orbit Yard keeps the sequence.";
  setNowCtaState("start");
}

function applyPulseNow(pulse: ProjectPulse) {
  const title = document.querySelector("#now-title");
  const detail = document.querySelector("#now-detail");
  const primary = document.querySelector<HTMLButtonElement>("#now-primary");
  const extra = document.querySelector<HTMLElement>("#now-extra");
  if (title) title.textContent = pulse.now?.title ?? "Ready";
  if (detail) {
    const base = pulse.now?.detail ?? "";
    const summary =
      lastPublish?.steps?.length && !lastPublish.finished
        ? publishProgressSummary(lastPublish)
        : lastPublish?.finished
          ? publishProgressSummary(lastPublish)
          : "";
    detail.textContent = summary && base ? `${base} · ${summary}` : base || summary;
  }
  if (primary) {
    const raw = pulse.now?.primary?.label ?? "Start publishing";
    setCtaLabel(primary, polishCtaLabel(raw));
    primary.dataset.pulseId = pulse.now?.primary?.id ?? "publish_start";
    primary.dataset.pulseView = pulse.now?.primary?.view || "publish";
    primary.disabled = running;
    setNowCtaState(ctaStateFromPulseId(pulse.now?.primary?.id, raw));
  }
  if (extra) {
    const acts = (pulse.now?.actions ?? []).filter((a) => a.id && a.id !== pulse.now?.primary?.id);
    if (!acts.length) {
      extra.hidden = true;
      extra.innerHTML = "";
    } else {
      extra.hidden = false;
      extra.innerHTML = acts
        .slice(0, 6)
        .map(
          (a) =>
            `<button type="button" data-pulse-action="${escapeHtml(a.id ?? "")}" data-pulse-view="${escapeHtml(a.view ?? "")}">${escapeHtml(a.label ?? a.id ?? "")}</button>`,
        )
        .join("");
      extra.querySelectorAll<HTMLButtonElement>("[data-pulse-action]").forEach((btn) => {
        btn.addEventListener("click", () => {
          void runPulseAction(btn.dataset.pulseAction ?? "", btn.dataset.pulseView ?? "");
        });
      });
    }
  }
}

type StatusItem = {
  id: string;
  state: "done" | "active" | "warn" | "blocked" | "idle";
  icon: string;
  title: string;
  detail: string;
};

function classifyOverall(pulse: ProjectPulse): {
  state: string;
  badge: string;
  title: string;
  detail: string;
} {
  const wantsSignet = (pulse.kind ?? "").toLowerCase().includes("desktop")
    || (pulse.kind ?? "").toLowerCase().includes("tauri");
  const signet = !!pulse.tools?.signet_found;
  const deployOk = deployEvidenceFromPulse(pulse);
  const selfhostOk = isSelfhostLocalEvidence(pulse);
  const linked =
    pulse.deploy?.signal === "vercel_linked" ||
    pulse.deploy?.signal === "yard_configured";
  const localOnly = pulse.deploy?.signal === "wrangler_local";
  const pub = pulse.publish;
  const launch = pulse.launch;
  const stepId = (pub?.current_id || launch?.current_title || "").toLowerCase();
  const midWizard =
    (pub?.present && !pub.finished) || (launch?.present && !launch.finished);

  // Hard-block only when Signet is required and missing. Orbit is a soft cue.
  const toolsBlocked = wantsSignet && !signet;

  if (toolsBlocked) {
    return {
      state: "blocked",
      badge: "Blocked",
      title: "Signet missing",
      detail: "Signet not on PATH — run Doctor before a desktop cut.",
    };
  }

  // Mid-flight wins over prior deploy evidence (GIF / Dashboard honesty).
  if (
    midWizard &&
    (stepId.includes("list") ||
      stepId.includes("polar") ||
      stepId.includes("paste") ||
      stepId.includes("env") ||
      stepId.includes("secret"))
  ) {
    return {
      state: "pending",
      badge: "Pending submission",
      title: pub?.current_title || launch?.current_title || "Human gate open",
      detail: "Finish this step on the vendor UI, then Confirm → Next.",
    };
  }
  if (midWizard) {
    const idx = pub?.present
      ? `${(pub.current_index ?? 0) + 1}/${pub.total ?? 0}`
      : `${(launch?.current_index ?? 0) + 1}/${launch?.total ?? 0}`;
    return {
      state: "progress",
      badge: "In progress",
      title: pub?.current_title || launch?.current_title || "Wizard in flight",
      detail: pub?.present
        ? `Publish ${idx} — Continue runs automatic checks`
        : `Launch ${idx}`,
    };
  }

  if (deployOk) {
    const dirtyNote = pulse.git?.dirty
      ? ` · ${pulse.git.dirty_count ?? "?"} uncommitted — not “all shipped”`
      : "";
    const url = hostedDeployUrls(pulse.deploy)[0];
    return {
      state: "deployed",
      badge: "Live",
      title: url ? "Public URL ready" : "Already live",
      detail:
        (url ||
          pulse.deploy?.detail ||
          "Prior successful deploy — Open live when you want to verify.") + dirtyNote,
    };
  }
  if (selfhostOk) {
    return {
      state: "ready",
      badge: "Self-host",
      title: "Local loopback ready",
      detail:
        pulse.deploy?.urls?.[0] ||
        pulse.deploy?.detail ||
        "Serving on this machine — not a public host URL.",
    };
  }
  if (linked) {
    return {
      state: "ready",
      badge: "Ready",
      title: "Host linked — get live when ready",
      detail: pulse.deploy?.detail || "Configured locally — Deploy for a public URL.",
    };
  }
  if (localOnly) {
    return {
      state: "ready",
      badge: "Local only",
      title: "Dev cache only",
      detail: "Local wrangler state — not proof of a public URL.",
    };
  }
  if (pulse.git?.dirty) {
    return {
      state: "warn",
      badge: "Dirty tree",
      title: `${pulse.git.dirty_count ?? "?"} uncommitted change(s)`,
      detail: "Commit or stash when you care about provenance before live deploy.",
    };
  }
  return {
    state: "ready",
    badge: "Ready",
    title: "Ready to start",
    detail: "Open a workflow or Start publishing — nothing here claims the cut is done.",
  };
}

function buildStatusChecklist(pulse: ProjectPulse): StatusItem[] {
  const items: StatusItem[] = [];
  const signet = !!pulse.tools?.signet_found;
  const orbit = !!pulse.tools?.orbit_found;
  const linked =
    pulse.deploy?.signal === "yard_deployed" ||
    pulse.deploy?.signal === "last_run_ok" ||
    pulse.deploy?.signal === "vercel_linked" ||
    pulse.deploy?.signal === "yard_configured" ||
    hostedDeployUrls(pulse.deploy).length > 0;
  const wantsSignet = (pulse.kind ?? "").toLowerCase().includes("desktop")
    || (pulse.kind ?? "").toLowerCase().includes("tauri");
  const localIntent = shipIntent() === "local";
  const pub = pulse.publish;
  const launch = pulse.launch;
  const midWizard =
    (pub?.present && !pub.finished) || (launch?.present && !launch.finished);

  // Tools — always name Signet / Orbit so General Dashboard is honest.
  if (wantsSignet && !signet) {
    items.push({
      id: "tools",
      state: "blocked",
      icon: "!",
      title: "Signet missing",
      detail: "Install Signet for local desktop cuts",
    });
  } else if (signet && orbit) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Signet ok · Orbit ok",
      detail: "Both on PATH",
    });
  } else if (signet && !orbit) {
    items.push({
      id: "tools",
      state: localIntent ? "done" : "warn",
      icon: localIntent ? "✓" : "!",
      title: localIntent ? "Signet ok · Orbit optional" : "Signet ok · Orbit missing",
      detail: localIntent
        ? "Orbit optional for Local cuts"
        : "Orbit not on PATH — needed for Public Orbit host deploys",
    });
  } else if (!wantsSignet && linked) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Worker tooling OK",
      detail: "Prior live deploy evidence — Orbit optional for this stack",
    });
  } else if (!signet && !orbit) {
    items.push({
      id: "tools",
      state: "idle",
      icon: "·",
      title: "No Signet / Orbit on PATH",
      detail: localIntent
        ? "Fine for docs-only / Local without desktop cut"
        : "Install tools before Signet release or Orbit deploy",
    });
  } else if (orbit && !signet) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Orbit ok · Signet not required",
      detail: "Orbit on PATH",
    });
  }

  const git = pulse.git;
  if (git?.is_repo && git.dirty) {
    items.push({
      id: "git",
      state: "warn",
      icon: "!",
      title: "Uncommitted changes",
      detail: `${git.dirty_count ?? "?"} dirty on ${git.branch ?? "branch"}`,
    });
  }

  if (pub?.present && !pub.finished) {
    items.push({
      id: "ship",
      state: "active",
      icon: "→",
      title: "Publish in progress",
      detail: `${pub.current_title ?? "Step"} · ${(pub.current_index ?? 0) + 1}/${pub.total ?? 0}`,
    });
  } else if (launch?.present && !launch.finished) {
    items.push({
      id: "ship",
      state: "active",
      icon: "→",
      title: "Launch in progress",
      detail: `${launch.current_title ?? "Step"} · ${(launch.current_index ?? 0) + 1}/${launch.total ?? 0}`,
    });
  } else if (pub?.finished) {
    items.push({
      id: "ship",
      state: "done",
      icon: "✓",
      title: "Publish pass finished",
      detail: "This pass marked finished — not a claim that every surface shipped",
    });
  } else if (!midWizard) {
    const dep = pulse.deploy;
    const live = deployEvidenceFromPulse(pulse);
    const selfhost = isSelfhostLocalEvidence(pulse);
    if (live) {
      items.push({
        id: "deploy",
        state: "done",
        icon: "✓",
        title: "Prior hosted deploy",
        detail: hostedDeployUrls(dep)[0] || dep?.detail || "Last orbityard hosted run succeeded",
      });
    } else if (selfhost) {
      items.push({
        id: "deploy",
        state: "idle",
        icon: "·",
        title: "Self-host local only",
        detail: dep?.urls?.[0] || dep?.detail || "Not a cloud host",
      });
    }
  }

  return items.slice(0, 4);
}

function applyStatusBar(pulse: ProjectPulse | null) {
  const bar = document.querySelector<HTMLElement>("#status-bar");
  const badge = document.querySelector<HTMLElement>("#status-badge");
  const overall = document.querySelector("#status-overall");
  const detail = document.querySelector("#status-overall-detail");
  const list = document.querySelector("#status-check");
  if (!bar || !list) return;
  if (!pulse || !projectPath()) {
    bar.hidden = true;
    list.innerHTML = "";
    return;
  }
  bar.hidden = false;
  const head = classifyOverall(pulse);
  if (badge) {
    badge.textContent = head.badge;
    badge.dataset.state = head.state === "warn" ? "pending" : head.state;
  }
  if (overall) overall.textContent = head.title;
  if (detail) detail.textContent = head.detail;
  const items = buildStatusChecklist(pulse);
  list.innerHTML = items
    .map(
      (it) => `<li data-state="${it.state}">
        <span class="ico" aria-hidden="true">${escapeHtml(it.icon)}</span>
        <div class="check-body"><strong>${escapeHtml(it.title)}</strong><span>${escapeHtml(it.detail)}</span></div>
      </li>`,
    )
    .join("");
}

function applyPulseHealth(pulse: ProjectPulse) {
  const signetOk = !!pulse.tools?.signet_found;
  const orbitOk = !!pulse.tools?.orbit_found;
  const localIntent = shipIntent() === "local";
  const wantsSignet = (pulse.kind ?? "").toLowerCase().includes("desktop")
    || (pulse.kind ?? "").toLowerCase().includes("tauri");
  setPill(
    "pill-signet",
    signetOk ? "ok" : wantsSignet ? "bad" : "muted",
    signetOk ? "Found" : "Missing",
  );
  setPill(
    "pill-orbit",
    orbitOk ? "ok" : localIntent ? "muted" : "bad",
    orbitOk ? "Found" : "Missing",
  );
  const metaSignet = document.querySelector("#meta-signet");
  const metaOrbit = document.querySelector("#meta-orbit");
  if (metaSignet) {
    metaSignet.textContent = signetOk
      ? pulse.tools?.signet_version ?? "on PATH"
      : "Not on PATH (SIGNET_PATH)";
  }
  if (metaOrbit) {
    metaOrbit.textContent = orbitOk
      ? pulse.tools?.orbit_version ?? "on PATH"
      : "Not on PATH (ORBIT_PATH)";
  }

  const git = pulse.git;
  if (!git?.is_repo) {
    setPill("pill-git", "muted", "No repo");
    const meta = document.querySelector("#meta-git");
    if (meta) meta.textContent = "Not a git repository";
  } else if (git.dirty) {
    setPill("pill-git", "bad", `${git.dirty_count ?? "?"} dirty`);
    const meta = document.querySelector("#meta-git");
    const branch = git.branch ? `${git.branch} · ` : "";
    const last = git.last_commit
      ? `${git.last_commit.hash} — ${git.last_commit.subject}`
      : "uncommitted changes";
    if (meta) meta.textContent = `${branch}${last}`;
  } else {
    setPill("pill-git", "ok", "Clean");
    const meta = document.querySelector("#meta-git");
    const branch = git.branch ? `${git.branch} · ` : "";
    const last = git.last_commit
      ? `${git.last_commit.hash} — ${git.last_commit.subject}`
      : "clean tree";
    const ab =
      git.ahead != null || git.behind != null
        ? ` · ↑${git.ahead ?? 0} ↓${git.behind ?? 0}`
        : "";
    if (meta) meta.textContent = `${branch}${last}${ab}`;
  }

  const dep = pulse.deploy;
  const signal = dep?.signal ?? "unknown";
  if (deployEvidenceFromPulse(pulse)) {
    setPill("pill-deploy", "ok", "Live");
  } else if (signal === "selfhost_ok") {
    setPill("pill-deploy", "muted", "Self-host");
  } else if (signal === "vercel_linked" || signal === "yard_configured") {
    setPill("pill-deploy", "ok", "Linked");
  } else if (signal === "wrangler_local") {
    setPill("pill-deploy", "muted", "Local");
  } else {
    setPill("pill-deploy", "muted", "None");
  }
  const metaDep = document.querySelector("#meta-deploy");
  if (metaDep) metaDep.textContent = dep?.detail ?? "No local deploy signal";
}

async function openRelatedStudioView(view: string): Promise<boolean> {
  const id = view.trim();
  if (!id || !VIEW_META[id] || id === "publish") return false;
  setView(id);
  const project = projectPath();
  if (!project) return true;
  // Paint the page first; then load orbityard JSON so chrome does not hitch with the console work.
  await new Promise<void>((resolve) => afterPaint(() => resolve()));
  if (id === "scopes") {
    applyScopes((await loadJsonCmd(["scopes", "--project", project])) as ScopePlan | null);
  } else if (id === "env") {
    applyEnv((await loadJsonCmd(["env", "--project", project])) as EnvPortal | null);
  } else if (id === "sign") {
    applySignPaths((await loadJsonCmd(["sign-paths", "--project", project])) as SignPortal | null);
  } else if (id === "portal") {
    document.querySelector<HTMLButtonElement>("#btn-portal")?.click();
  } else if (id === "launch") {
    await refreshLaunch();
  } else if (id === "platforms") {
    openPlatformsCatalog();
  } else if (id === "tools") {
    /* stay — doctor available on Tools */
  } else if (id === "dashboard") {
    await refreshSessionNow();
  }
  return true;
}

function syncPublishRelated() {
  const btn = document.querySelector<HTMLButtonElement>("#btn-publish-related");
  if (!btn) return;
  const related = (lastPublish?.current?.desktop_view ?? "").trim();
  const label = RELATED_VIEW_LABELS[related];
  const show = Boolean(label) && !lastPublish?.finished;
  btn.hidden = !show;
  btn.disabled = !show || running || !projectPath();
  if (label) {
    btn.textContent = label;
    btn.dataset.relatedView = related;
  } else {
    delete btn.dataset.relatedView;
  }
}

async function runPulseAction(id: string, view: string) {
  if (id === "git_status") {
    await showGitStatus();
    return;
  }
  if (id === "open") {
    document.querySelector<HTMLButtonElement>("#btn-open")?.click();
    return;
  }
  if (id === "doctor") {
    void run(["doctor", "--project", projectPath()], { step: "doctor" });
    return;
  }
  if (id === "publish_continue") {
    const cur = lastPublish?.current;
    const pending = (cur?.status ?? "").toLowerCase() === "pending";
    const kind = (cur?.kind ?? "").toLowerCase();
    const pausedHuman =
      pending &&
      (kind === "human" ||
        kind === "oauth" ||
        kind === "deploy" ||
        kind === "list" ||
        kind === "check");
    setView("publish");
    if (!lastPublish?.steps?.length) await refreshPublish();
    // Already paused at a honesty gate — open the checkpoint; do not re-Continue
    // (that flashed Publish → related panel → yank).
    if (pausedHuman) {
      stageFocusIndex = lastPublish?.current_index ?? 0;
      if (lastPublish) renderPublishStage(lastPublish);
      toast("Finish this checkpoint on Publish, then Confirm", "info", 4500);
      return;
    }
    void publishContinuePaced();
    return;
  }
  const target = view || "publish";
  setView(target);
  if (target === "publish") {
    await refreshPublish();
    return;
  }
  if (target === "launch") {
    await refreshLaunch();
    return;
  }
  if (target === "env") {
    applyEnv((await loadJsonCmd(["env", "--project", projectPath()])) as EnvPortal | null);
    return;
  }
  if (target === "scopes") {
    applyScopes((await loadJsonCmd(["scopes", "--project", projectPath()])) as ScopePlan | null);
    return;
  }
  if (target === "tools") {
    void run(["doctor", "--project", projectPath()], { step: "doctor" });
  }
}

async function showGitStatus() {
  const project = projectPath();
  if (!project) return;
  setView("output");
  try {
    const result = await invoke<CmdResult>("run_git", {
      project,
      args: ["status", "-sb"],
    });
    const log = await invoke<CmdResult>("run_git", {
      project,
      args: ["log", "-3", "--oneline"],
    });
    show(
      `git status -sb · exit ${result.code}\n\n${result.stdout || result.stderr}\n\ngit log -3 --oneline\n\n${log.stdout || log.stderr}`,
    );
  } catch (e) {
    show(String(e));
  }
}

/** S1.15 — quiet Publish strip when pulse reports a dirty tree. */
function syncPublishDirtyCue() {
  const el = document.querySelector<HTMLElement>("#publish-dirty-cue");
  if (!el) return;
  const git = lastPulse?.git;
  if (!projectPath() || !git?.is_repo || !git.dirty) {
    el.hidden = true;
    el.textContent = "";
    return;
  }
  const n = git.dirty_count ?? "?";
  el.hidden = false;
  el.textContent = `${n} uncommitted — Confirm on release/deploy only if you intend this tree`;
}

function isDirtySensitivePublishStep(step: {
  id?: string;
  kind?: string;
} | null | undefined): boolean {
  if (!step) return false;
  const id = (step.id ?? "").toLowerCase();
  const kind = (step.kind ?? "").toLowerCase();
  if (kind === "deploy" || kind === "list") return true;
  if (id === "live_check" || id === "ship.desktop_cut") return true;
  if (
    id.startsWith("deploy.") ||
    id.startsWith("listing.") ||
    id.startsWith("submit.")
  ) {
    return true;
  }
  return id.includes("release") || id.includes("desktop_cut");
}

/** Soft gate: dirty tree + release/deploy/listing Confirm → warn once, then arm. */
async function publishConfirmWithDirtySoftGate() {
  const git = lastPulse?.git;
  const dirty = Boolean(git?.is_repo && git.dirty);
  const sensitive = isDirtySensitivePublishStep(lastPublish?.current);
  if (dirty && sensitive && !dirtyConfirmArmed) {
    const n = git?.dirty_count ?? "?";
    toast(
      `${n} uncommitted change(s) — Confirm only if this dirty tree is intentional`,
      "info",
      10_000,
      [
        {
          id: "confirm-anyway",
          label: "Confirm anyway",
          icon: "confirm",
          run: () => {
            dirtyConfirmArmed = true;
            void publishAction(["confirm"]);
          },
        },
        {
          id: "git-status",
          label: "Git status",
          icon: "open",
          run: () => {
            void showGitStatus();
          },
        },
      ],
    );
    return;
  }
  await publishAction(["confirm"]);
}

function applyPulse(pulse: ProjectPulse | null) {
  lastPulse = pulse;
  if (!pulse) {
    applyNow(null);
    return;
  }
  if (pulse.kind) {
    const sessionKind = document.querySelector("#session-kind");
    if (sessionKind) sessionKind.textContent = pulse.kind;
  }
  applyPulseHealth(pulse);
  applyStatusBar(pulse);
  applyPulseNow(pulse);
  syncProjectIdentity();
  paintDeployResultsBay();
  syncPublishDirtyCue();
  if (!pulse.git?.dirty) dirtyConfirmArmed = false;
}

async function refreshSessionNow() {
  const path = projectPath();
  if (!path) {
    lastPublish = null;
    lastPulse = null;
    applyNow(null);
    applyStatusBar(null);
    syncProjectIdentity();
    return;
  }
  const pulse = (await loadJsonCmd(["pulse", "--project", path])) as ProjectPulse | null;
  if (pulse) {
    applyPulse(pulse);
    return;
  }
  // Fallback if pulse unavailable
  const view = (await loadJsonCmd(publishArgs())) as PublishView | null;
  if (view?.steps?.length) lastPublish = view;
  applyNow(lastPublish);
  syncProjectIdentity();
}

async function setTitle(path: string | null) {
  const name = path ? projectName(path) : "No project";
  syncProjectIdentity();
  try {
    const win = getCurrentWindow();
    await win.setTitle(path ? `Orbit Yard — ${name}` : "Orbit Yard");
  } catch {
    /* ignore in browser preview */
  }
}

async function refreshMaxIcon() {
  const btn = document.querySelector<HTMLButtonElement>("#win-max");
  const icon = document.querySelector("#win-max-icon");
  if (!btn || !icon) return;
  try {
    const maxed = await getCurrentWindow().isMaximized();
    btn.title = maxed ? "Restore" : "Maximize";
    btn.setAttribute("aria-label", btn.title);
    icon.innerHTML = maxed
      ? '<path d="M3.2 4.2h5.2v5.2H3.2z" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M4.6 2.6h5.2v5.2" stroke="currentColor" stroke-width="1.2" fill="none"/>'
      : '<rect x="2.2" y="2.2" width="7.6" height="7.6" rx="1.1" stroke="currentColor" stroke-width="1.2" fill="none"/>';
  } catch {
    /* ignore */
  }
}

function wireWindowChrome() {
  document.querySelector("#win-min")?.addEventListener("click", () => {
    void getCurrentWindow().minimize();
  });
  document.querySelector("#win-max")?.addEventListener("click", () => {
    void getCurrentWindow()
      .toggleMaximize()
      .then(() => refreshMaxIcon());
  });
  document.querySelector("#win-close")?.addEventListener("click", () => {
    void getCurrentWindow().close();
  });
  document.querySelector(".titlebar")?.addEventListener("dblclick", (ev) => {
    if ((ev.target as HTMLElement).closest(".window-controls")) return;
    void getCurrentWindow()
      .toggleMaximize()
      .then(() => refreshMaxIcon());
  });
  void refreshMaxIcon();
  void getCurrentWindow().onResized(() => {
    void refreshMaxIcon();
  });
}

function normalizeRecentPath(path: string): string {
  return path.trim().replace(/[/\\]+$/, "");
}

function recentPathKey(path: string): string {
  return normalizeRecentPath(path).replace(/\\/g, "/").toLowerCase();
}

function loadRecent(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    const parsed = raw ? (JSON.parse(raw) as string[]) : [];
    if (!Array.isArray(parsed)) return [];
    const out: string[] = [];
    const seen = new Set<string>();
    for (const p of parsed) {
      if (typeof p !== "string" || !p.trim()) continue;
      const norm = normalizeRecentPath(p);
      const key = recentPathKey(norm);
      if (seen.has(key)) continue;
      seen.add(key);
      out.push(norm);
    }
    return out;
  } catch {
    return [];
  }
}

function persistRecent(list: string[]) {
  const capped = list.slice(0, MAX_RECENT);
  localStorage.setItem(RECENT_KEY, JSON.stringify(capped));
  renderRecent(capped);
}

async function pathIsDir(path: string): Promise<boolean> {
  try {
    return await invoke<boolean>("path_is_dir", { path });
  } catch {
    return false;
  }
}

/** Drop folders that were renamed, moved, or deleted. */
async function pruneMissingRecents(): Promise<string[]> {
  const list = loadRecent();
  const kept: string[] = [];
  for (const p of list) {
    if (await pathIsDir(p)) kept.push(p);
  }
  if (kept.length !== list.length) {
    persistRecent(kept);
    const last = localStorage.getItem(LAST_PROJECT_KEY);
    if (last && !kept.some((p) => recentPathKey(p) === recentPathKey(last))) {
      localStorage.removeItem(LAST_PROJECT_KEY);
    }
  } else {
    renderRecent(kept);
  }
  return kept;
}

function removeRecent(path: string) {
  const key = recentPathKey(path);
  const next = loadRecent().filter((p) => recentPathKey(p) !== key);
  persistRecent(next);
  const last = localStorage.getItem(LAST_PROJECT_KEY);
  if (last && recentPathKey(last) === key) {
    localStorage.removeItem(LAST_PROJECT_KEY);
  }
  toast(`Removed ${projectName(path)} from recents`, "ok", 2500);
}

async function openRecentProject(path: string) {
  if (!(await pathIsDir(path))) {
    removeRecent(path);
    toast(
      `${projectName(path)} is no longer available — removed from recents`,
      "info",
      6000,
    );
    return;
  }
  await bindProject(path, true);
}

function loadShippedAt(): Record<string, string> {
  try {
    const raw = localStorage.getItem(SHIPPED_AT_KEY);
    const parsed = raw ? (JSON.parse(raw) as Record<string, string>) : {};
    return parsed && typeof parsed === "object" ? parsed : {};
  } catch {
    return {};
  }
}

/** F5 — mark current folder as shipped this pass (local map only). */
function markProjectShipped(path: string | null | undefined) {
  const p = (path ?? "").trim();
  if (!p) return;
  const map = loadShippedAt();
  map[p] = new Date().toISOString();
  try {
    localStorage.setItem(SHIPPED_AT_KEY, JSON.stringify(map));
  } catch {
    /* ignore quota */
  }
  paintNextShip();
}

/**
 * F5 — next local recent to ship: prefer not-yet-shipped (or older ship), skip current.
 * No cloud portfolio hub.
 */
function nextShipCandidate(current?: string | null): string | null {
  const cur = (current ?? projectPath() ?? "").trim();
  const shipped = loadShippedAt();
  const others = loadRecent().filter((p) => p && p !== cur);
  if (!others.length) return null;
  const unshipped = others.filter((p) => !shipped[p]);
  if (unshipped.length) return unshipped[0] ?? null;
  // All shipped recently — MRU among others (oldest ship first if timestamps differ).
  const ranked = [...others].sort((a, b) => {
    const ta = Date.parse(shipped[a] ?? "") || 0;
    const tb = Date.parse(shipped[b] ?? "") || 0;
    return ta - tb;
  });
  return ranked[0] ?? null;
}

function paintNextShip() {
  const card = document.querySelector<HTMLElement>("#next-ship");
  const body = document.querySelector("#next-ship-body");
  const btn = document.querySelector<HTMLButtonElement>("#next-ship-open");
  if (!card || !body || !btn) return;
  const current = projectPath();
  const next = nextShipCandidate(current);
  const showCue =
    Boolean(next) &&
    (!current ||
      Boolean(lastPublish?.finished) ||
      Boolean(lastPulse?.publish?.finished) ||
      Boolean(
        (lastPulse?.deploy?.urls ?? lastRunState?.urls ?? []).some(
          (u) => /^https:\/\//i.test(String(u)) && !isLoopbackDeployUrl(String(u)),
        ),
      ));
  if (!next || !showCue) {
    card.hidden = true;
    body.textContent = "";
    btn.dataset.path = "";
    return;
  }
  const name = projectName(next);
  card.hidden = false;
  body.textContent = current
    ? `This folder’s cut can wait — open ${name} next (local recents only · no cloud hub).`
    : `Continue with ${name} from local recents — Get live spine, same machine.`;
  btn.dataset.path = next;
  btn.textContent = `Open ${name}`;
}

function saveRecent(path: string) {
  const norm = normalizeRecentPath(path);
  const key = recentPathKey(norm);
  const next = [norm, ...loadRecent().filter((p) => recentPathKey(p) !== key)].slice(
    0,
    MAX_RECENT,
  );
  localStorage.setItem(LAST_PROJECT_KEY, norm);
  persistRecent(next);
}

function recentChipHtml(p: string): string {
  return `<span class="recent-chip" title="${escapeHtml(p)}">
    <button type="button" class="recent-open" data-recent="${escapeHtml(p)}">${escapeHtml(projectName(p))}</button>
    <button type="button" class="recent-remove" data-remove-recent="${escapeHtml(p)}" title="Remove from history" aria-label="Remove ${escapeHtml(projectName(p))} from history">×</button>
  </span>`;
}

function wireRecentChips(root: ParentNode) {
  root.querySelectorAll<HTMLButtonElement>("button[data-recent]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const p = btn.getAttribute("data-recent");
      if (p) void openRecentProject(p);
    });
  });
  root.querySelectorAll<HTMLButtonElement>("button[data-remove-recent]").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      const p = btn.getAttribute("data-remove-recent");
      if (p) removeRecent(p);
    });
  });
}

function renderRecent(list: string[]) {
  const host = document.querySelector<HTMLElement>("#recent-list");
  const dash = document.querySelector<HTMLElement>("#dash-recents");
  const dashKicker = document.querySelector<HTMLElement>("#dash-recents-kicker");
  const historyBtn = document.querySelector<HTMLButtonElement>("#btn-recent-history");
  const current = projectPath();
  const curKey = current ? recentPathKey(current) : "";
  const others = list.filter((p) => recentPathKey(p) !== curKey);
  const pills = others.slice(0, RECENT_PILL_MAX);
  const html = pills.map((p) => recentChipHtml(p)).join("");
  const showHistory = list.length > 0;
  if (historyBtn) {
    historyBtn.hidden = !showHistory;
    historyBtn.textContent =
      others.length > RECENT_PILL_MAX
        ? `All projects… (${list.length})`
        : "All projects…";
  }
  if (host) {
    if (pills.length === 0) {
      host.hidden = true;
      host.innerHTML = "";
    } else {
      host.hidden = false;
      host.innerHTML = html;
      wireRecentChips(host);
    }
  }
  if (dash) {
    if (pills.length === 0) {
      dash.hidden = true;
      dash.innerHTML = "";
      if (dashKicker) dashKicker.hidden = true;
    } else {
      dash.hidden = false;
      if (dashKicker) dashKicker.hidden = false;
      dash.innerHTML = html;
      wireRecentChips(dash);
    }
  }
  renderSwitcher(list);
  if (!document.querySelector<HTMLElement>("#recent-history-dialog")?.hidden) {
    renderRecentHistoryList();
  }
  paintNextShip();
}

function toggleProjectSwitcher(force?: boolean) {
  const el = document.querySelector<HTMLElement>("#project-switcher");
  if (!el) return;
  if (typeof force === "boolean") el.hidden = !force;
  else el.hidden = !el.hidden;
  if (!el.hidden) renderSwitcher(loadRecent());
}

function renderSwitcher(list: string[]) {
  const host = document.querySelector<HTMLElement>("#switcher-list");
  if (!host) return;
  const current = projectPath();
  const curKey = current ? recentPathKey(current) : "";
  const items = list.slice(0, RECENT_PILL_MAX);
  host.innerHTML = items.length
    ? items
        .map((p) => {
          const on = recentPathKey(p) === curKey ? " active" : "";
          return `<div class="switcher-row${on}">
            <button type="button" class="switcher-item" data-switch="${escapeHtml(p)}">${escapeHtml(projectName(p))}<span>${escapeHtml(p)}</span></button>
            <button type="button" class="switcher-remove" data-remove-recent="${escapeHtml(p)}" title="Remove from history" aria-label="Remove ${escapeHtml(projectName(p))}">×</button>
          </div>`;
        })
        .join("")
    : `<p class="detail empty-hint">No recents yet — Open folder. Use All projects… to search history.</p>`;
  host.querySelectorAll<HTMLButtonElement>("[data-switch]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const p = btn.getAttribute("data-switch");
      toggleProjectSwitcher(false);
      if (p) void openRecentProject(p);
    });
  });
  host.querySelectorAll<HTMLButtonElement>("[data-remove-recent]").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      const p = btn.getAttribute("data-remove-recent");
      if (p) removeRecent(p);
    });
  });
}

function recentHistoryQuery(): string {
  return (
    document.querySelector<HTMLInputElement>("#recent-history-search")?.value.trim() ?? ""
  );
}

function filterRecentHistory(list: string[], query: string): string[] {
  const q = query.trim().toLowerCase();
  if (!q) return list;
  return list.filter((p) => {
    const name = projectName(p).toLowerCase();
    const path = p.replace(/\\/g, "/").toLowerCase();
    return name.includes(q) || path.includes(q);
  });
}

function renderRecentHistoryList() {
  const listEl = document.querySelector<HTMLElement>("#recent-history-list");
  const emptyEl = document.querySelector<HTMLElement>("#recent-history-empty");
  const metaEl = document.querySelector("#recent-history-meta");
  const capEl = document.querySelector("#recent-history-cap");
  if (capEl) capEl.textContent = String(MAX_RECENT);
  if (!listEl) return;
  const all = loadRecent();
  const matched = filterRecentHistory(all, recentHistoryQuery());
  const current = projectPath();
  const curKey = current ? recentPathKey(current) : "";
  if (metaEl) {
    metaEl.textContent = recentHistoryQuery()
      ? `${matched.length} of ${all.length} match · cap ${MAX_RECENT}`
      : `${all.length} remembered · cap ${MAX_RECENT}`;
  }
  listEl.innerHTML = matched
    .map((p) => {
      const on = recentPathKey(p) === curKey ? " is-current" : "";
      return `<div class="recent-history-row${on}" role="option">
        <button type="button" class="recent-history-open" data-history-open="${escapeHtml(p)}">
          <span class="recent-history-name">${escapeHtml(projectName(p))}</span>
          <span class="recent-history-path">${escapeHtml(p)}</span>
        </button>
        <button type="button" class="recent-history-remove" data-remove-recent="${escapeHtml(p)}" title="Remove from history" aria-label="Remove ${escapeHtml(projectName(p))}">×</button>
      </div>`;
    })
    .join("");
  if (emptyEl) emptyEl.hidden = matched.length > 0;
  listEl.querySelectorAll<HTMLButtonElement>("[data-history-open]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const p = btn.getAttribute("data-history-open");
      closeRecentHistoryDialog();
      if (p) void openRecentProject(p);
    });
  });
  listEl.querySelectorAll<HTMLButtonElement>("[data-remove-recent]").forEach((btn) => {
    btn.addEventListener("click", (ev) => {
      ev.preventDefault();
      ev.stopPropagation();
      const p = btn.getAttribute("data-remove-recent");
      if (p) removeRecent(p);
    });
  });
}

function openRecentHistoryDialog() {
  const dlg = document.querySelector<HTMLElement>("#recent-history-dialog");
  if (!dlg) return;
  toggleProjectSwitcher(false);
  dlg.hidden = false;
  const search = document.querySelector<HTMLInputElement>("#recent-history-search");
  if (search) {
    search.value = "";
    search.focus();
  }
  renderRecentHistoryList();
}

function closeRecentHistoryDialog() {
  const dlg = document.querySelector<HTMLElement>("#recent-history-dialog");
  if (dlg) dlg.hidden = true;
}





async function loadJsonCmd(
  args: string[],
  opts?: { silent?: boolean; user?: boolean; label?: string },
): Promise<unknown | null> {
  const user = Boolean(opts?.user);
  const result = await run(args, {
    quietHeader: true,
    // User path: show dock output so Preview is useful; quietToast so we own fail toast.
    silent: user ? false : opts?.silent !== false,
    quietToast: user,
  });
  const label = opts?.label ?? args[0] ?? "orbityard";
  const preview: ToastAction[] = [
    { id: "preview", label: "Preview log", icon: "open", run: () => openOutputPreview() },
  ];
  if (!result) {
    if (user) toast(`${label} busy — Cancel to unlock, then retry`, "err", 5000);
    return null;
  }
  if (!result.ok || !result.stdout?.trim()) {
    if (user) {
      toast(`${label}: ${cmdFailDetail(result)}`, "err", 8000, preview);
    }
    return null;
  }
  try {
    return JSON.parse(result.stdout);
  } catch {
    if (user) {
      toast(`${label}: output was not JSON — open Preview`, "err", 7000, preview);
    }
    return null;
  }
}

function applyAssist(plan: AssistPlan | null) {
  const list = document.querySelector("#assist-steps");
  const hint = document.querySelector("#assist-hint");
  if (!list) return;
  if (hint && plan?.sign_path) {
    hint.textContent = `Signing path: ${plan.sign_path}. Prefer Start publishing — detail Open jumps are optional panels.`;
  }
  const steps = plan?.steps ?? [];
  list.innerHTML = steps.length
    ? steps
        .map(
          (s) => `<li class="portal-step">
        <div class="meta">
          <div class="title"><span class="kind">${s.ready ? "ready" : "todo"}</span>${escapeHtml(s.title ?? "")}</div>
          <p class="detail">${escapeHtml(s.detail ?? "")}</p>
        </div>
        <div class="btns">
          <button type="button" class="assist-go" data-view="${escapeHtml(s.view ?? "dashboard")}">Detail</button>
        </div>
      </li>`,
        )
        .join("")
    : `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Refresh assist after binding a project — or Start publishing.</p></div></li>`;
  list.querySelectorAll<HTMLButtonElement>(".assist-go").forEach((btn) => {
    btn.addEventListener("click", () => {
      const view = btn.getAttribute("data-view");
      if (view) setView(view);
      if (view === "publish") document.querySelector<HTMLButtonElement>("#btn-publish")?.click();
      if (view === "launch") document.querySelector<HTMLButtonElement>("#btn-launch")?.click();
      if (view === "env") document.querySelector<HTMLButtonElement>("#btn-env")?.click();
      if (view === "sign") document.querySelector<HTMLButtonElement>("#btn-sign-paths")?.click();
      if (view === "scopes") document.querySelector<HTMLButtonElement>("#btn-scopes")?.click();
      if (view === "portal") document.querySelector<HTMLButtonElement>("#btn-portal")?.click();
      if (publishMidFlight() && view && view !== "publish") {
        toast("Detail panel — use Back to Publish when done", "info");
      }
    });
  });
}

let lastScopes: ScopePlan | null = null;

function selectedScopeIdsFromDom(): string[] {
  const inline = stageScopesRoot();
  const useStage = Boolean(inline && !inline.hidden);
  const root = useStage
    ? document.querySelector("#stage-scope-grid")
    : document.querySelector("#scope-grid");
  return Array.from(
    (root ?? document).querySelectorAll<HTMLInputElement>("[data-scope-id]:checked"),
  )
    .map((el) => el.dataset.scopeId ?? "")
    .filter(Boolean)
    .sort();
}

function savedScopeIds(): string[] {
  return [...(lastScopes?.active ?? [])].filter(Boolean).sort();
}

function scopesSelectionIsDirty(): boolean {
  if (!lastScopes?.scopes?.length) return false;
  const a = selectedScopeIdsFromDom();
  const b = savedScopeIds();
  if (a.length !== b.length) return true;
  return a.some((id, i) => id !== b[i]);
}

/** S1.16 S3 — Save button + cue when selection ≠ persisted active. */
function syncScopesDirtyCue() {
  const dirty = scopesSelectionIsDirty();
  for (const id of ["#btn-scopes-save", "#btn-stage-scopes-save"]) {
    const btn = document.querySelector<HTMLButtonElement>(id);
    if (!btn) continue;
    btn.classList.toggle("is-dirty", dirty);
    btn.textContent = dirty ? "Save changes" : "Save selection";
    if (dirty) btn.title = "Selection differs from last Save";
    else btn.removeAttribute("title");
  }
  const cue = document.querySelector<HTMLElement>("#scopes-dirty-cue");
  if (cue) {
    cue.hidden = !dirty;
    cue.textContent = dirty
      ? "Selection changed — Save before Confirm on Publish"
      : "";
  }
}

function applyScopes(plan: ScopePlan | null) {
  lastScopes = plan;
  fillScopeGrids(plan);
  syncStageScopesPrimary();
  syncScopesDirtyCue();
}

/** Enable Confirm & continue when inline Scopes has an active selection. */
function syncStageScopesPrimary() {
  if (publishUiMode() !== "stages" || !lastPublish) return;
  const primaryBtn = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
  if (!primaryBtn || primaryBtn.dataset.stageAction !== "confirm") return;
  if (!isScopesStep(lastPublish.current)) return;
  const hasActive = (lastScopes?.active?.length ?? 0) > 0;
  primaryBtn.disabled = running || !hasActive;
  primaryBtn.title = hasActive
    ? "Mark scopes done and advance"
    : "Save at least one scope first";
}

async function detectScopes(opts?: { quiet?: boolean }) {
  if (!projectPath()) return;
  const plan = (await loadJsonCmd(["scopes", "--project", projectPath()], {
    user: !opts?.quiet,
    label: "Targets",
  })) as ScopePlan | null;
  applyScopes(plan);
  // Fail path: loadJsonCmd already toasted when user-initiated.
  if (!opts?.quiet && plan) {
    toast(plan.scopes?.length ? "Targets detected" : "No targets found", "ok");
  }
}

async function saveScopes(opts?: { silentToast?: boolean }): Promise<boolean> {
  const inline = stageScopesRoot();
  const root =
    inline && !inline.hidden
      ? document.querySelector("#stage-scope-grid")
      : document.querySelector("#scope-grid");
  const ids = Array.from(
    (root ?? document).querySelectorAll<HTMLInputElement>("[data-scope-id]:checked"),
  ).map((el) => el.dataset.scopeId ?? "");
  if (!ids.length) {
    show("Select at least one target.");
    toast("Select at least one target", "err");
    return false;
  }
  const result = await run(
    ["scopes", "--project", projectPath(), "set", "--ids", ids.join(",")],
    { quietHeader: true },
  );
  if (result?.stdout) {
    try {
      applyScopes(JSON.parse(result.stdout) as ScopePlan);
    } catch {
      /* ignore */
    }
  }
  if (!result?.ok) return false;
  const onStage = Boolean(inline && !inline.hidden);
  if (!opts?.silentToast) {
    if (onStage) {
      toast("Saved — Confirm & continue", "ok");
    } else if (publishMidFlight()) {
      setView("publish");
      toast("Targets saved — Confirm on Publish", "ok");
    } else {
      toast("Targets saved", "ok");
    }
  }
  syncScopesDirtyCue();
  return true;
}

async function openEnvPutTerminal(provider: string, name: string) {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return;
  }
  if (!provider.trim() || !name.trim() || name === "<NAME>") {
    toast("Env Put needs a provider and secret name", "info");
    return;
  }
  await openShipctlTerminal(
    ["env", "--project", project, "--provider", provider, "--put", name],
    {
      title: "Orbit Yard env put",
      meta: `Launched terminal: orbityard env --provider ${provider} --put ${name} — paste when the CLI prompts.`,
      okToast: `Env Put opened for ${name} — finish in the terminal`,
    },
  );
}

/** Resolve Tier A host for Put — commerce hints Open vendor, then Put lands on CF/Vercel/Netlify. */
function hostForSecretPut(hintProvider?: string | null): string | null {
  const p = (hintProvider ?? "").toLowerCase();
  if (providerHasEnvPut(p)) return p;
  return preferredEnvPutHost();
}

async function putNamedSecretOnHost(name: string, hintProvider?: string | null): Promise<void> {
  const host = hostForSecretPut(hintProvider);
  if (!host) {
    toast("Put needs Cloudflare, Vercel, or Netlify detected — bind a hosted project", "info", 5500);
    return;
  }
  if (!name.trim() || name === "<NAME>") {
    toast("Add empty NAME= lines to .env / .dev.vars first — then Put", "info", 5500);
    return;
  }
  await openEnvPutTerminal(host, name);
}

/** Tier A + B hosts with a real put CLI — O1 Portal env primary CTA. */
const ENV_PUT_PROVIDERS = new Set([
  "cloudflare",
  "vercel",
  "netlify",
  "fly",
  "railway",
]);

function providerHasEnvPut(provider?: string | null): boolean {
  return Boolean(provider && ENV_PUT_PROVIDERS.has(provider.toLowerCase()));
}

/** Portal env Put — named Env Put when one hint; else Env list or Human --put. */
async function openPortalEnvPut(provider: string) {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return;
  }
  const pid = provider.trim().toLowerCase();
  if (!providerHasEnvPut(pid)) {
    toast("No Put CLI for this provider — Open dashboard, then put on the host", "info", 5500);
    return;
  }
  const hints =
    lastSecrets?.hints?.filter((h) => (h.provider ?? "").toLowerCase() === pid && h.name) ?? [];
  if (hints.length === 1 && hints[0]?.name) {
    await openEnvPutTerminal(pid, hints[0].name);
    return;
  }
  if (hints.length > 1) {
    setView("env");
    const plan = (await loadJsonCmd(["env", "--project", project], {
      user: true,
      label: "Env",
    })) as EnvPortal | null;
    applyEnv(plan);
    toast("Pick Put on a named secret — never paste the value into Studio", "info", 5500);
    return;
  }
  await openShipctlTerminal(["human", "--project", project, "--no-open", "--put"], {
    title: "Orbit Yard paste",
    meta: "Launched terminal: orbityard human --no-open --put — paste each value when prompted.",
    okToast: "Put terminal opened — finish secrets there, then Confirm in Publish",
  });
}

function applyEnv(plan: EnvPortal | null) {
  const list = document.querySelector("#env-actions");
  if (!list) return;
  const actions = plan?.actions ?? [];
  list.innerHTML = actions.length
    ? actions
        .map((a) => {
          const url = a.entry_url ?? "";
          const cmd = (a.put_cli ?? []).join(" ");
          const put =
            a.kind === "retrieve" && a.provider && a.name
              ? `<button type="button" class="env-put" data-provider="${escapeHtml(a.provider)}" data-name="${escapeHtml(a.name)}">Put</button>`
              : "";
          return `<li class="portal-step">
            <div class="meta">
              <div class="title"><span class="kind">${escapeHtml(a.kind ?? "")}</span>${escapeHtml(a.title ?? "")}</div>
              <p class="detail">${escapeHtml(a.detail ?? "")}${cmd ? ` · ${escapeHtml(cmd)}` : ""}</p>
            </div>
            <div class="btns">
              <button type="button" class="env-open" data-url="${escapeHtml(url)}" ${url ? "" : "disabled"}>Open</button>
              ${put}
            </div>
          </li>`;
        })
        .join("")
    : `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Load env portal — or add empty NAME= lines to .env / .dev.vars (or wrangler # Secrets:) so Put rows appear.</p></div></li>`;
  list.querySelectorAll<HTMLButtonElement>(".env-open").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const url = btn.getAttribute("data-url");
      if (url) await openUrl(url);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".env-put").forEach((btn) => {
    btn.addEventListener("click", () => {
      const provider = btn.getAttribute("data-provider");
      const name = btn.getAttribute("data-name");
      if (!provider || !name) return;
      void openEnvPutTerminal(provider, name);
    });
  });
}

function applySignPaths(plan: SignPortal | null) {
  const list = document.querySelector("#sign-paths");
  const hint = document.querySelector("#sign-hint");
  if (!list) return;
  if (hint && plan?.recommended) {
    const hasSubmit = (plan.paths ?? []).some((p) => p.kind === "submit");
    hint.textContent = hasSubmit
      ? `Recommended: ${plan.recommended.split("_").join(" ")}. Official = certificates; Submit = store review — confirm each separately.`
      : `Recommended: ${plan.recommended.split("_").join(" ")}. Self-sign is local; official stays on vendor UIs.`;
  }
  const paths = plan?.paths ?? [];
  const signetOk = Boolean(lastPulse?.tools?.signet_found);
  // Show submit paths after official certs for clearer dogfood order.
  const ordered = [...paths].sort((a, b) => {
    const rank = (k: string | undefined) =>
      k === "self" ? 0 : k === "official" ? 1 : k === "submit" ? 2 : 3;
    return rank(a.kind) - rank(b.kind);
  });
  list.innerHTML = ordered.length
    ? ordered
        .map((p) => {
          const url = p.entry_url ?? "";
          const runCmd = (p.run ?? []).join(" ");
          const kind = p.kind ?? "";
          const status =
            kind === "self"
              ? signetOk
                ? `<span class="kind kind-ok">ready</span>`
                : `<span class="kind kind-missing">check</span>`
              : kind === "official" || kind === "submit"
                ? `<span class="kind kind-guide">guide</span>`
                : "";
          return `<li class="portal-step">
            <div class="meta">
              <div class="title"><span class="kind kind-${escapeHtml(kind)}">${escapeHtml(kind)}</span>${status}${escapeHtml(p.title ?? "")}</div>
              <p class="detail">${escapeHtml(p.detail ?? "")}${runCmd ? ` · ${escapeHtml(runCmd)}` : ""}</p>
            </div>
            <div class="btns">
              <button type="button" class="sign-open" data-url="${escapeHtml(url)}" ${url ? "" : "disabled"}>Open vendor</button>
            </div>
          </li>`;
        })
        .join("")
    : `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Check status to load signing paths.</p></div></li>`;
  list.querySelectorAll<HTMLButtonElement>(".sign-open").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const url = btn.getAttribute("data-url");
      if (url) await openUrl(url);
    });
  });
}

function applyPortalPlan(plan: PortalPlan | null) {
  lastPortal = plan;
  const list = document.querySelector<HTMLElement>("#portal-steps");
  const filters = document.querySelector<HTMLElement>("#provider-filters");
  if (!list || !filters) return;
  if (!plan?.steps?.length) {
    list.innerHTML = `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Load portal to see provider entry steps.</p></div></li>`;
    filters.hidden = true;
    filters.innerHTML = "";
    return;
  }
  setView("portal");
  const providers = plan.providers ?? [];
  filters.hidden = providers.length <= 1;
  filters.innerHTML = [
    `<button type="button" data-filter="" class="${portalFilter ? "" : "active"}">All</button>`,
    ...providers.map(
      (p) =>
        `<button type="button" data-filter="${escapeHtml(p)}" class="${
          portalFilter === p ? "active" : ""
        }">${escapeHtml(p)}</button>`,
    ),
  ].join("");
  filters.querySelectorAll("button").forEach((btn) => {
    btn.addEventListener("click", () => {
      const f = btn.getAttribute("data-filter") || null;
      portalFilter = f || null;
      applyPortalPlan(lastPortal);
    });
  });

  const steps = (plan.steps ?? []).filter(
    (s) => !portalFilter || s.provider === portalFilter,
  );
  list.innerHTML = steps
    .map((s, idx) => {
      const url = s.entry_url ?? "";
      const docs = s.docs_url ?? "";
      const cli = (s.cli ?? []).join(" ");
      const openDisabled = url ? "" : "disabled";
      const kind = (s.kind ?? "").toLowerCase();
      const isEnv = kind === "env";
      const isOauth = kind === "oauth";
      const canPut = isEnv && providerHasEnvPut(s.provider);
      const canLogin = isOauth || (s.cli && s.cli.length > 0);
      const openLabel = isEnv
        ? "Open dashboard"
        : isOauth
          ? "Sign in (web)"
          : "Open";
      const docsLabel = isEnv ? "Learn more" : "Docs";
      const putBtn = canPut
        ? `<button type="button" class="portal-put primary" data-provider="${escapeHtml(
            s.provider ?? "",
          )}">Put</button>`
        : "";
      // Web Open is primary for OAuth when a dashboard/login URL exists; Login CLI is secondary.
      const openIsPrimary = Boolean(url) && !canPut && (isOauth || !canLogin);
      const loginIsPrimary = Boolean(canLogin && isOauth && !url);
      const openBtn = `<button type="button" class="portal-open${
        openIsPrimary ? " primary" : ""
      }" data-url="${escapeHtml(
        url,
      )}" ${openDisabled} title="${escapeHtml(url || "No settings URL for this step")}">${openLabel}</button>`;
      const docsBtn = docs
        ? `<button type="button" class="portal-docs" data-url="${escapeHtml(
            docs,
          )}" title="${escapeHtml(docs)}">${docsLabel}</button>`
        : "";
      const loginBtn = canLogin
        ? `<button type="button" class="portal-login${
            loginIsPrimary ? " primary" : ""
          }" data-provider="${escapeHtml(
            s.provider ?? "",
          )}">Login CLI</button>`
        : "";
      // Env: Put → Open dashboard → Learn more. OAuth: Sign in (web) → Docs → Login CLI.
      const btns = isEnv
        ? `${putBtn}${openBtn}${docsBtn}${loginBtn}`
        : `${openBtn}${docsBtn}${loginBtn}`;
      return `<li class="portal-step" data-idx="${idx}">
        <div class="meta">
          <div class="title"><span class="kind">${escapeHtml(
            s.kind ?? "",
          )}</span>${escapeHtml(s.title ?? s.id ?? "step")}</div>
          <p class="detail">${escapeHtml(s.detail ?? "")}${
            cli && !url ? ` · ${escapeHtml(cli)}` : ""
          }</p>
        </div>
        <div class="btns">${btns}</div>
      </li>`;
    })
    .join("");

  list.querySelectorAll<HTMLButtonElement>(".portal-put").forEach((btn) => {
    btn.addEventListener("click", () => {
      const provider = btn.getAttribute("data-provider");
      if (!provider) return;
      void openPortalEnvPut(provider);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".portal-open").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const url = btn.getAttribute("data-url");
      if (url) await openUrl(url);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".portal-docs").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const url = btn.getAttribute("data-url");
      if (url) await openUrl(url);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".portal-login").forEach((btn) => {
    btn.addEventListener("click", () => {
      const provider = btn.getAttribute("data-provider");
      if (!provider) return;
      void openPortalLoginTerminal(provider);
    });
  });
}

async function loadSecrets() {
  const result = await run(["secrets", "--project", projectPath()]);
  if (!result?.ok || !result.stdout) return;
  try {
    const plan = JSON.parse(result.stdout) as SecretsPlan;
    lastSecrets = plan;
    const block = document.querySelector<HTMLElement>("#secrets-block");
    const list = document.querySelector<HTMLElement>("#secrets-steps");
    if (!block || !list) return;
    setView("portal");
    if (!plan.hints?.length) {
      block.hidden = true;
      list.innerHTML = "";
      return;
    }
    block.hidden = false;
    list.innerHTML = plan.hints
      .map((h) => {
        const cmd = (h.put_cli ?? []).join(" ");
        const url = h.entry_url ?? "";
        const name = h.name ?? "";
        const canPut = Boolean(name && name !== "<NAME>" && hostForSecretPut(h.provider));
        return `<li class="portal-step">
          <div class="meta">
            <div class="title"><span class="kind">${escapeHtml(
              h.provider ?? "",
            )}</span>${escapeHtml(name)}</div>
            <p class="detail">${escapeHtml(h.detail ?? "")}${
              cmd ? ` · ${escapeHtml(cmd)}` : ""
            }</p>
          </div>
          <div class="btns">
            <button type="button" class="secret-open" data-url="${escapeHtml(
              url,
            )}" ${url ? "" : "disabled"}>Open</button>
            <button type="button" class="secret-put${canPut ? " primary" : ""}" data-provider="${escapeHtml(
              h.provider ?? "",
            )}" data-name="${escapeHtml(name)}" ${canPut ? "" : "disabled"} title="Paste in host CLI terminal — never into Studio">Put</button>
            <button type="button" class="secret-copy" data-cmd="${escapeHtml(
              cmd,
            )}" ${cmd ? "" : "disabled"}>Copy CLI</button>
          </div>
        </li>`;
      })
      .join("");
    list.querySelectorAll<HTMLButtonElement>(".secret-open").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const url = btn.getAttribute("data-url");
        if (url) await openUrl(url);
      });
    });
    list.querySelectorAll<HTMLButtonElement>(".secret-put").forEach((btn) => {
      btn.addEventListener("click", () => {
        const name = btn.getAttribute("data-name");
        const provider = btn.getAttribute("data-provider");
        if (!name) return;
        void putNamedSecretOnHost(name, provider);
      });
    });
    list.querySelectorAll<HTMLButtonElement>(".secret-copy").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const cmd = btn.getAttribute("data-cmd") ?? "";
        if (!cmd) return;
        await navigator.clipboard.writeText(cmd);
        appendStream({ stream: "meta", text: `copied: ${cmd}` });
      });
    });
  } catch {
    /* shown in output */
  }
}

function applyHumanSprint(sprint: HumanSprint | null) {
  lastHuman = sprint;
  const list = document.querySelector<HTMLElement>("#human-queue");
  const hint = document.querySelector<HTMLElement>("#human-hint");
  if (!list) return;
  if (!sprint?.put_queue?.length && !sprint?.open_order?.length) {
    list.innerHTML = `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Start a human sprint to build the paste queue.</p></div></li>`;
    return;
  }
  setView("portal");
  if (hint && sprint.minutes_hint) hint.textContent = sprint.minutes_hint;
  const queue = sprint.put_queue ?? [];
  list.innerHTML = queue
    .map((h, i) => {
      const cmd = (h.put_cli ?? []).join(" ");
      const url = h.entry_url ?? "";
      const name = h.name ?? "";
      const canPut = Boolean(name && name !== "<NAME>" && hostForSecretPut(h.provider));
      return `<li class="portal-step">
        <div class="meta">
          <div class="title"><span class="kind">${i + 1}</span>${escapeHtml(
            h.provider ?? "",
          )} · ${escapeHtml(name)}</div>
          <p class="detail">${escapeHtml(url || "no source url")}${
            cmd ? ` · ${escapeHtml(cmd)}` : ""
          }</p>
        </div>
        <div class="btns">
          <button type="button" class="human-open" data-url="${escapeHtml(
            url,
          )}" ${url ? "" : "disabled"}>Open source</button>
          <button type="button" class="human-put${canPut ? " primary" : ""}" data-provider="${escapeHtml(
            h.provider ?? "",
          )}" data-name="${escapeHtml(name)}" ${canPut ? "" : "disabled"} title="Paste in host CLI terminal">Put</button>
          <button type="button" class="human-copy" data-cmd="${escapeHtml(
            cmd,
          )}" ${cmd ? "" : "disabled"}>Copy CLI</button>
        </div>
      </li>`;
    })
    .join("");
  list.querySelectorAll<HTMLButtonElement>(".human-open").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const url = btn.getAttribute("data-url");
      if (url) await openUrl(url);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".human-put").forEach((btn) => {
    btn.addEventListener("click", () => {
      const name = btn.getAttribute("data-name");
      const provider = btn.getAttribute("data-provider");
      if (!name) return;
      void putNamedSecretOnHost(name, provider);
    });
  });
  list.querySelectorAll<HTMLButtonElement>(".human-copy").forEach((btn) => {
    btn.addEventListener("click", async () => {
      const cmd = btn.getAttribute("data-cmd") ?? "";
      if (!cmd) return;
      await navigator.clipboard.writeText(cmd);
      appendStream({ stream: "meta", text: `copied: ${cmd}` });
    });
  });
}

async function runHumanPortal(opts?: { openSources?: boolean }) {
  setStep("paste", "active");
  const args = ["human", "--project", projectPath(), "--no-open"];
  const result = await run(args, { step: "paste" });
  if (!result?.ok || !result.stdout) {
    setStep("paste", "fail");
    return;
  }
  try {
    const sprint = JSON.parse(result.stdout) as HumanSprint;
    applyHumanSprint(sprint);
    if (opts?.openSources) {
      const urls = [...new Set(sprint.open_order ?? [])];
      for (const url of urls) await openUrl(url);
      appendStream({
        stream: "meta",
        text: `Opened ${urls.length} paste-source page(s). Copy values, then Paste in terminal.`,
      });
    }
    setStep("paste", "done");
  } catch {
    setStep("paste", "fail");
  }
}

/** Studio panels opened from a publish step — prefer these over staying on Publish. */
const PANEL_FIRST_VIEWS = new Set([
  "scopes",
  "env",
  "sign",
  "portal",
  "ritual",
  "tools",
  "launch",
  "dashboard",
  "integrations",
  "platforms",
]);

/** Related views worth leaving Publish for. Dashboard is not — it caused a bounce. */
function shouldLeavePublishForRelated(related: string): boolean {
  const id = related.trim();
  if (!id || !RELATED_VIEW_LABELS[id] || id === "dashboard") return false;
  return PANEL_FIRST_VIEWS.has(id);
}

async function navigatePublishStep(step: {
  desktop_view?: string | null;
  title?: string;
  id?: string;
}): Promise<boolean> {
  const related = (step.desktop_view ?? "").trim();
  if (!shouldLeavePublishForRelated(related)) return false;
  const ok = await openRelatedStudioView(related);
  if (ok) {
    toast(
      `${RELATED_VIEW_LABELS[related] ?? related} — finish, then Back to Publish → Confirm`,
      "info",
      4500,
    );
  }
  return ok;
}

function applyPublishView(view: PublishView | null, opts?: { reveal?: boolean }) {
  lastPublish = view;
  const currentEl = document.querySelector<HTMLElement>("#publish-current");
  const list = document.querySelector<HTMLElement>("#publish-steps");
  const hint = document.querySelector<HTMLElement>("#publish-hint");
  const mins = document.querySelector<HTMLElement>("#publish-minutes");
  const progress = document.querySelector<HTMLElement>("#publish-progress");
  if (!currentEl || !list) return;
  if (!view?.steps?.length) {
    currentEl.innerHTML =
      '<p class="detail empty-hint">Open a folder, then pick a workflow on the Dashboard — or Continue here.</p>';
    list.innerHTML = "";
    if (mins) mins.hidden = true;
    if (progress) {
      progress.hidden = true;
      progress.textContent = "";
      progress.removeAttribute("data-finished");
    }
    renderPublishStage(null);
    syncWorkflowCards();
    syncPublishRelated();
    syncIntegrationsOrientation();
    syncBackToPublish();
    syncPublishGateButtons();
    applyNow(view);
    paintNextShip();
    return;
  }
  // Do not yank the operator off Scopes/Sign/Env after Open — only jump when asked or already on Publish.
  const active =
    document.querySelector<HTMLElement>(".view:not([hidden])")?.dataset.view ?? "";
  if (opts?.reveal || active === "publish" || active === "") {
    setView("publish");
  }
  const cur = view.current;
  if (hint) {
    const modeLabel =
      (view.mode ?? "").toLowerCase() === "general"
        ? "General"
        : (view.mode ?? "").toLowerCase() === "advanced"
          ? "Advanced"
          : studioMode() === "general"
            ? "General"
            : "Advanced";
    const intentLabel =
      (view.intent ?? "").toLowerCase() === "local"
        ? "Local"
        : (view.intent ?? "").toLowerCase() === "public"
          ? "Public"
          : shipIntent() === "local"
            ? "Local"
            : "Public";
    hint.textContent = view.finished
      ? "Publish workflow finished — required gates for this pass are done."
      : `${modeLabel} · ${intentLabel} — Continue automatic checks; Open/Confirm for steps you finish. Verify checks ${
          verifyStatusLabel(cur?.verify_status) || "local files & tools"
        } (no secrets).`;
  }
  if (mins) {
    mins.hidden = false;
    mins.textContent = `~${view.minutes_remaining ?? 0} min left · ${view.minutes_total ?? 0} min total`;
  }
  if (progress) {
    const summary = publishProgressSummary(view);
    progress.hidden = !summary;
    progress.textContent = summary;
    progress.dataset.finished = view.finished ? "1" : "0";
  }
  syncPublishDirtyCue();
  const curRelated = (cur?.desktop_view ?? "").trim();
  const curNav = curRelated && RELATED_VIEW_LABELS[curRelated];
  if (curNav) {
    currentEl.classList.add("launch-current-nav");
    currentEl.dataset.desktopView = curRelated;
    currentEl.setAttribute("role", "button");
    currentEl.tabIndex = 0;
    currentEl.title = `Open ${RELATED_VIEW_LABELS[curRelated] ?? curRelated}`;
  } else {
    currentEl.classList.remove("launch-current-nav");
    delete currentEl.dataset.desktopView;
    currentEl.removeAttribute("role");
    currentEl.removeAttribute("tabindex");
    currentEl.removeAttribute("title");
  }
  currentEl.innerHTML = cur
    ? `<div class="title">${statusKindHtml(cur.status ?? cur.kind)}${escapeHtml(cur.title ?? "")}${
        cur.minutes ? ` · ~${cur.minutes}m` : ""
      }</div>
       <p class="detail">${escapeHtml(cur.detail ?? "")}${
         cur.run?.length ? ` · run: ${escapeHtml(cur.run.join(" "))}` : ""
       }${
         curNav
           ? ` · <span class="step-nav-hint">${escapeHtml(RELATED_VIEW_LABELS[curRelated] ?? curRelated)} — click to open</span>`
           : ""
       }</p>`
    : "<p class=\"detail\">No current step</p>";
  list.innerHTML = renderPublishStepBands(view);
  stageFocusIndex = view.current_index ?? 0;
  renderPublishStage(view);
  syncWorkflowCards();
  syncPublishRelated();
  syncIntegrationsOrientation();
  syncBackToPublish();
  syncPublishGateButtons();
  applyNow(view);
  if (view.finished) {
    markProjectShipped(projectPath());
  } else {
    paintNextShip();
  }
}

async function refreshPublish() {
  const result = await run(publishArgs(), {
    step: "paste",
    quietHeader: true,
  });
  if (!result?.ok || !result.stdout) {
    toast(result?.cancelled ? "Publish cancelled" : "Could not load publish plan", "err");
    return;
  }
  try {
    applyPublishView(JSON.parse(result.stdout) as PublishView, { reveal: true });
    toast("Publish plan ready", "ok");
  } catch {
    /* shown in output */
    toast("Publish output was not JSON", "err");
  }
}

function focusPublishStepIndex(index: number) {
  if (!lastPublish?.steps?.length) return;
  const i = Math.max(0, Math.min(index, lastPublish.steps.length - 1));
  stageFocusIndex = i;
  if (publishUiMode() === "stages") renderPublishStage(lastPublish);
  afterPaint(() => {
    document
      .querySelectorAll("#publish-steps .portal-step.is-handoff")
      .forEach((el) => el.classList.remove("is-handoff"));
    const li = document.querySelector<HTMLElement>(
      `#publish-steps [data-step-index="${i}"]`,
    );
    if (li) {
      li.classList.add("is-handoff");
      li.scrollIntoView({ block: "nearest", behavior: "smooth" });
    }
  });
}

/**
 * S1.10 — Continue publishing from a catalog wizard: open Publish, name/focus the
 * matching gate when present (no auto-Confirm).
 */
async function continuePublishingHandoff(opts: {
  preferredStepId?: string | null;
  preferFromView?: (view: PublishView) => string | null;
  missingHint: string;
  genericHint: string;
}): Promise<void> {
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  setView("publish");
  const result = await run(publishArgs(), {
    step: "paste",
    quietHeader: true,
    quietToast: true,
  });
  if (!result?.ok || !result.stdout) {
    toast(result?.cancelled ? "Publish cancelled" : "Could not load publish plan", "err");
    return;
  }
  let view: PublishView;
  try {
    view = JSON.parse(result.stdout) as PublishView;
  } catch {
    toast("Publish output was not JSON", "err");
    return;
  }
  applyPublishView(view, { reveal: true });

  const preferred = (
    opts.preferredStepId ??
    opts.preferFromView?.(view) ??
    ""
  ).trim();
  if (!preferred) {
    toast(opts.genericHint, "ok", 5000);
    return;
  }
  const steps = view.steps ?? [];
  const idx = steps.findIndex((s) => (s.id ?? "") === preferred);
  if (idx < 0) {
    toast(opts.missingHint, "info", 6500);
    return;
  }
  focusPublishStepIndex(idx);
  const step = steps[idx];
  const title =
    (step.title ?? preferred).replace(/\s*—\s*.*$/, "").trim() || preferred;
  const status = (step.status ?? "").toLowerCase();
  const isCurrent =
    (view.current_index ?? -1) === idx || (view.current?.id ?? "") === preferred;

  if (status === "done" || status === "skipped") {
    toast(`«${title}» already done — continue Publish`, "ok", 4500);
    return;
  }
  if (isCurrent) {
    toast(`Confirm «${title}» when the vendor side is ready`, "ok", 9000, [
      {
        id: "confirm",
        label: "Confirm",
        icon: "confirm",
        run: () => {
          void publishConfirmWithDirtySoftGate();
        },
      },
    ]);
    return;
  }
  toast(
    `Publish includes «${title}» — Continue until that gate, then Confirm`,
    "ok",
    7000,
  );
}

/**
 * S1.11 — Confirm the matching Publish gate from a catalog wizard (human intent only).
 * Refuses when the preferred step is missing or not current.
 */
async function confirmWizardPublishGate(opts: {
  preferredStepId?: string | null;
  preferFromView?: (view: PublishView) => string | null;
  missingHint: string;
  handoffMissingHint: string;
  handoffGenericHint: string;
}): Promise<void> {
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  const result = await run(publishArgs(), {
    step: "paste",
    quietHeader: true,
    quietToast: true,
  });
  if (!result?.ok || !result.stdout) {
    toast(result?.cancelled ? "Publish cancelled" : "Could not load publish plan", "err");
    return;
  }
  let view: PublishView;
  try {
    view = JSON.parse(result.stdout) as PublishView;
  } catch {
    toast("Publish output was not JSON", "err");
    return;
  }
  // Refresh plan state without yanking the operator off the wizard yet.
  applyPublishView(view, { reveal: false });

  const preferred = (
    opts.preferredStepId ??
    opts.preferFromView?.(view) ??
    ""
  ).trim();
  if (!preferred) {
    toast(opts.missingHint, "info", 6500);
    return;
  }
  const steps = view.steps ?? [];
  const idx = steps.findIndex((s) => (s.id ?? "") === preferred);
  if (idx < 0) {
    toast(opts.missingHint, "info", 6500);
    return;
  }
  const step = steps[idx];
  const title =
    (step.title ?? preferred).replace(/\s*—\s*.*$/, "").trim() || preferred;
  const status = (step.status ?? "").toLowerCase();
  const isCurrent =
    (view.current_index ?? -1) === idx || (view.current?.id ?? "") === preferred;

  if (status === "done" || status === "skipped") {
    toast(`«${title}» already confirmed — Continue publishing for the next gate`, "ok", 5000);
    return;
  }
  if (!isCurrent) {
    toast(`«${title}» isn’t the current Publish gate yet`, "info", 8000, [
      {
        id: "open-publish",
        label: "Open Publish",
        icon: "open",
        run: () => {
          void continuePublishingHandoff({
            preferredStepId: preferred,
            missingHint: opts.handoffMissingHint,
            genericHint: opts.handoffGenericHint,
          });
        },
      },
    ]);
    return;
  }

  setView("publish");
  focusPublishStepIndex(idx);
  await publishConfirmWithDirtySoftGate();
}

function preferredDeployPublishStep(view: PublishView | null): string | null {
  const steps = view?.steps ?? [];
  for (const id of ["live_check", "deploy.hosts", "oauth.hosts"]) {
    const hit = steps.find((s) => (s.id ?? "") === id);
    if (hit && !["done", "skipped"].includes((hit.status ?? "").toLowerCase())) {
      return id;
    }
  }
  const deploy = steps.find((s) => (s.id ?? "").startsWith("deploy."));
  return deploy?.id ?? null;
}

function preferredSignPublishStep(view: PublishView | null): string | null {
  const steps = view?.steps ?? [];
  const byView = steps.find(
    (s) =>
      (s.desktop_view ?? "").trim() === "sign" &&
      !["done", "skipped"].includes((s.status ?? "").toLowerCase()),
  );
  if (byView?.id) return byView.id;
  for (const id of ["ship.desktop_cut", "signet.release", "sign.graduate"]) {
    const hit = steps.find((s) => (s.id ?? "") === id);
    if (hit && !["done", "skipped"].includes((hit.status ?? "").toLowerCase())) {
      return id;
    }
  }
  return null;
}

let publishWatchTimer: number | null = null;
let publishWatchLastOk = false;
let publishWatchLastStep = "";

function stopPublishWatch(opts?: { uncheck?: boolean }) {
  if (publishWatchTimer !== null) {
    window.clearInterval(publishWatchTimer);
    publishWatchTimer = null;
  }
  if (opts?.uncheck) {
    const el = document.querySelector<HTMLInputElement>("#opt-publish-watch");
    if (el) el.checked = false;
  }
  document
    .querySelector("#btn-publish-confirm")
    ?.classList.remove("watch-ready");
}

async function tickPublishWatch() {
  if (running) return;
  const project = projectPath();
  if (!project) return;
  if (lastPublish?.finished) {
    stopPublishWatch({ uncheck: true });
    return;
  }
  try {
    const result = await invoke<CmdResult>("run_orbityard", {
      project,
      args: publishArgs(["watch", "--once"]),
    });
    const raw = (result?.stdout ?? "").trim();
    if (!raw) return;
    const line = raw.split(/\r?\n/).filter(Boolean).pop() ?? "";
    const parsed = JSON.parse(line) as {
      ok?: boolean;
      message?: string;
      step_id?: string;
      prompt?: string | null;
      finished?: boolean;
    };
    const ok = Boolean(parsed.ok);
    const step = parsed.step_id ?? "";
    document
      .querySelector("#btn-publish-confirm")
      ?.classList.toggle("watch-ready", ok);
    syncPublishGateButtons();
    if (ok && (!publishWatchLastOk || step !== publishWatchLastStep)) {
      toast(parsed.prompt ?? "Step ready — Confirm on the green button", "ok", 6000);
      const status = await invoke<CmdResult>("run_orbityard", {
        project,
        args: publishArgs(),
      });
      if (status?.ok && status.stdout) {
        try {
          applyPublishView(JSON.parse(status.stdout) as PublishView);
        } catch {
          /* ignore */
        }
      }
      syncPublishGateButtons();
    }
    publishWatchLastOk = ok;
    publishWatchLastStep = step;
    if (parsed.finished) stopPublishWatch({ uncheck: true });
  } catch {
    /* watch is best-effort */
  }
}

function startPublishWatch() {
  stopPublishWatch();
  publishWatchLastOk = false;
  publishWatchLastStep = "";
  void tickPublishWatch();
  publishWatchTimer = window.setInterval(() => {
    void tickPublishWatch();
  }, 15_000);
}

type ContinueResult = {
  ok?: boolean;
  message?: string;
  stopped?: string;
  advanced?: number;
  publish?: PublishView;
};

async function runPublishContinueOnce(chain = 1): Promise<ContinueResult | null> {
  const result = await run([...publishArgs(["continue"]), "--chain", String(chain)], {
    step: "paste",
    quietToast: true,
  });
  if (!result?.stdout) return null;
  try {
    return JSON.parse(result.stdout) as ContinueResult;
  } catch {
    return null;
  }
}

/** Walk Auto gates one-at-a-time with ~2s dwell so 2→7 never flashes. */
async function publishContinuePaced() {
  if (publishPacing) {
    toast("Already checking checkpoints…", "info", 2500);
    return;
  }
  if (running) {
    toast("Busy — Cancel unlocks Publish if stuck", "err", 4000);
    return;
  }
  publishPacing = true;
  setStagePacingUi(true);
  applyPublishUi("stages");
  setView("publish");
  let totalAdvanced = 0;
  const wf = savedWorkflow();
  const wfLabel = wf ? WORKFLOW_PRESETS[wf].label : null;
  try {
    while (true) {
      const beforeIdx = lastPublish?.current_index ?? 0;
      const beforeTitle =
        lastPublish?.current?.title ?? lastPublish?.steps?.[beforeIdx]?.title ?? "checkpoint";
      stageFocusIndex = beforeIdx;
      if (lastPublish) renderPublishStage(lastPublish);
      toast(`Checking «${shortStageLabel(beforeTitle)}»…`, "info", Math.min(paceDwellMs(), 1600));

      const parsed = await runPublishContinueOnce(1);
      if (!parsed) break;

      const advanced = parsed.advanced ?? 0;
      totalAdvanced += advanced;
      const gate = parsed.stopped === "human_gate" ? parsed.publish?.current : null;
      const related = (gate?.desktop_view ?? "").trim();
      const leaveForPanel = Boolean(gate && shouldLeavePublishForRelated(related));
      if (parsed.publish) {
        applyPublishView(parsed.publish, { reveal: !leaveForPanel });
        stageFocusIndex = parsed.publish.current_index ?? stageFocusIndex;
        renderPublishStage(parsed.publish);
      }

      const msg = parsed.message ?? (parsed.ok === false ? "Continue failed" : "Continued");

      if (parsed.stopped === "human_gate") {
        const title = parsed.publish?.current?.title ?? lastPublish?.current?.title ?? "this step";
        if (leaveForPanel && gate) {
          toast(
            totalAdvanced > 0
              ? `Checked ${totalAdvanced} automatic step(s), then open «${title}» — Confirm when finished.`
              : `Open «${title}» — finish there, then Confirm. Not finished shipping.`,
            "info",
            6500,
          );
          await navigatePublishStep(gate);
        } else {
          toast(
            totalAdvanced > 0
              ? `Checked ${totalAdvanced} automatic step(s). Pause at «${title}» — Confirm on Publish when done.`
              : `Paused at «${title}» — Confirm on Publish when this check is done.`,
            "info",
            6500,
          );
        }
        break;
      }

      if (parsed.stopped === "finished") {
        toast(
          wfLabel
            ? `${wfLabel} — required gates for this pass are done (${totalAdvanced} checked).`
            : totalAdvanced > 0
              ? `Required gates done — checked ${totalAdvanced} automatic step(s).`
              : "Required gates for this pass are done",
          "ok",
          5500,
        );
        break;
      }

      if (parsed.stopped === "verify_failed" || parsed.ok === false) {
        const title =
          parsed.publish?.current?.title?.replace(/\s*—\s*.*$/, "").trim() ||
          lastPublish?.current?.title ||
          "this step";
        const polished = polishShipctlUserMessage(msg) || msg;
        toast(
          `Continue stopped at «${title}»: ${polished.slice(0, 120)}. Fix that, then Continue again.`,
          "err",
          8000,
          [
            {
              id: "continue",
              label: "Continue",
              icon: "continue",
              run: () => {
                void publishContinuePaced();
              },
            },
            {
              id: "preview",
              label: "Preview log",
              icon: "open",
              run: () => openOutputPreview(),
            },
          ],
        );
        break;
      }

      if (advanced === 0) {
        toast(msg || "Nothing to advance — next checkpoint is still ahead.", "info", 4500);
        break;
      }

      // Dwell so the operator sees Done → next marker before the jump.
      await sleepMs(paceDwellMs());
    }
  } finally {
    publishPacing = false;
    setStagePacingUi(false);
    if (lastPublish) renderPublishStage(lastPublish);
    await refreshSessionNow();
  }
}

async function publishAction(sub: string[]) {
  const ordered = sub.length === 0 ? publishArgs() : publishArgs(sub);
  // Own toasts — never stack "publish failed" + raw orbityard copy.
  const result = await run(ordered, { step: "paste", quietToast: true });
  if (!result) {
    toast("Publish is busy — Cancel to unlock, then retry.", "err", 7000, [
      {
        id: "cancel",
        label: "Cancel",
        icon: "open",
        run: () => document.querySelector<HTMLButtonElement>("#btn-cancel")?.click(),
      },
      {
        id: "retry",
        label: "Retry",
        icon: "continue",
        run: () => {
          void publishAction(sub);
        },
      },
    ]);
    return;
  }
  if (result.cancelled) {
    toast("Cancelled", "err");
    return;
  }

  const combined = `${result.stderr}\n${result.stdout}`.trim();
  if (/Confirm or Verify|still pending/i.test(combined)) {
    if (result.stdout.trim()) {
      try {
        const parsed = JSON.parse(result.stdout) as PublishView & { publish?: PublishView };
        applyPublishView(parsed.publish ?? parsed);
      } catch {
        /* ignore */
      }
    }
    toastPublishGatePending();
    return;
  }

  if (!result.stdout.trim()) {
    const detail =
      polishShipctlUserMessage(result.stderr) ||
      result.stderr.trim().split(/\r?\n/).filter(Boolean).pop() ||
      "Publish action failed with no details.";
    toast(detail.slice(0, 200), "err", 8000, [
      {
        id: "preview",
        label: "Preview log",
        icon: "open",
        run: () => openOutputPreview(),
      },
      {
        id: "retry",
        label: "Retry",
        icon: "continue",
        run: () => {
          void publishAction(sub);
        },
      },
      {
        id: "continue",
        label: "Continue",
        icon: "continue",
        run: () => {
          setView("publish");
          document.querySelector<HTMLButtonElement>("#btn-publish-continue")?.click();
        },
      },
    ]);
    return;
  }

  try {
    const parsed = JSON.parse(result.stdout) as PublishView & {
      publish?: PublishView;
      ok?: boolean;
      message?: string;
    };
    applyPublishView(parsed.publish ?? parsed);
    if (parsed.message) {
      appendStream({ stream: "meta", text: parsed.message });
      const polished = polishShipctlUserMessage(parsed.message);
      if (parsed.ok === false) {
        if (/Confirm or Verify|still pending|after Open\/Run succeeds/i.test(parsed.message)) {
          toastPublishGatePending();
        } else if (polished) {
          toast(polished, "err", 6500, [
            {
              id: "confirm",
              label: "Confirm",
              icon: "confirm",
              run: () => {
                setView("publish");
                document.querySelector<HTMLButtonElement>("#btn-publish-confirm")?.click();
              },
            },
            {
              id: "continue",
              label: "Continue",
              icon: "continue",
              run: () => {
                setView("publish");
                document.querySelector<HTMLButtonElement>("#btn-publish-continue")?.click();
              },
            },
          ]);
        }
      } else if (polished) {
        toast(polished, "ok");
      }
    } else if (result.ok) {
      const subCmd = sub[0];
      if (subCmd === "confirm") {
        // After Confirm, burn Auto gates so strangers are not stuck on dual Next.
        const cur = lastPublish?.current;
        const st = (cur?.status ?? "").toLowerCase();
        const kind = (cur?.kind ?? "").toLowerCase();
        const canChain =
          !lastPublish?.finished &&
          (st === "done" ||
            st === "skipped" ||
            (st === "pending" && (kind === "auto" || gateToastKind(cur) === "continue")));
        if (canChain) {
          toast("Confirmed — checking next gates…", "ok", 2200);
          void publishContinuePaced();
        } else {
          toast("Confirmed — press the green button", "ok");
        }
      } else if (subCmd === "next") toast("Advanced to next checkpoint", "ok");
      else if (subCmd === "verify") toast("Verify ok — Confirm if this step is done", "ok");
    } else if (/verify failed/i.test(combined)) {
      toastPublishGatePending();
    } else {
      toastPublishGatePending();
    }
    await refreshSessionNow();
  } catch {
    appendStream({ stream: "stderr", text: result.stdout.slice(0, 400) });
    toast("Could not read Publish result — open Preview for the log.", "err", 7000, [
      {
        id: "preview",
        label: "Preview log",
        icon: "open",
        run: () => openOutputPreview(),
      },
    ]);
  }
}

function launchStepBand(id: string): "prep" | "lane" | "cut" {
  const key = id.trim();
  if (key === "doctor" || key === "configure" || key === "intent") return "prep";
  if (
    key === "flow_dry_run" ||
    key === "deploy" ||
    key === "selfhost.deploy" ||
    key.startsWith("deploy.") ||
    key === "listing.packages"
  ) {
    return "cut";
  }
  return "lane";
}

function launchLaneGroup(id: string): string {
  if (id === "oauth.hosts" || id === "deploy.panel") return "deployment";
  if (id === "sign.panel") return "sign";
  if (id === "integrations.panel") return "payments";
  if (id === "env.sprint") return "env";
  if (id === "scopes") return "targets";
  if (id === "legal.baseline") return "legal";
  return id;
}

function launchLaneMeta(group: string): { title: string; blurb: string; view?: string } {
  switch (group) {
    case "deployment":
      return {
        title: "Deployment",
        blurb: "Host login · Put · Open dashboard — pick one primary host.",
        view: "platforms",
      };
    case "sign":
      return { title: "Sign", blurb: "Self-sign · official certs · store portals.", view: "sign" };
    case "payments":
      return {
        title: "Payments",
        blurb: "Polar / Stripe / … — vendor bookmarks under Suite.",
        view: "integrations",
      };
    case "env":
      return { title: "Env / tokens", blurb: "Put secrets in the terminal — never store values here.", view: "env" };
    case "targets":
      return { title: "Targets", blurb: "What ships — Web / API / Desktop scopes.", view: "scopes" };
    case "legal":
      return { title: "Legal", blurb: "LICENSE · SECURITY · TRUST baseline.", view: "dashboard" };
    default:
      return { title: group, blurb: "Optional launch lane.", view: undefined };
  }
}

function applyLaunchView(view: LaunchView | null) {
  lastLaunch = view;
  void lastLaunch;
  const currentEl = document.querySelector<HTMLElement>("#launch-current");
  const prepEl = document.querySelector<HTMLElement>("#launch-prep");
  const prepSummary = document.querySelector<HTMLElement>("#launch-prep-summary");
  const prepList = document.querySelector<HTMLElement>("#launch-prep-list");
  const lanesEl = document.querySelector<HTMLElement>("#launch-lanes");
  const laneGrid = document.querySelector<HTMLElement>("#launch-lane-grid");
  const cutEl = document.querySelector<HTMLElement>("#launch-cut");
  const cutList = document.querySelector<HTMLElement>("#launch-cut-list");
  const hint = document.querySelector<HTMLElement>("#launch-hint");
  const openBtn = document.querySelector<HTMLButtonElement>("#btn-launch-open");
  if (!currentEl) return;
  if (!view?.steps?.length) {
    currentEl.innerHTML =
      '<p class="detail empty-hint">Refresh Launch to build the adaptive plan for this repo.</p>';
    if (prepEl) prepEl.hidden = true;
    if (lanesEl) lanesEl.hidden = true;
    if (cutEl) cutEl.hidden = true;
    if (openBtn) openBtn.textContent = "Open / Run";
    return;
  }
  setView("launch");
  const cur = view.current;
  const curKind = (cur?.kind ?? "").toLowerCase();
  const curRelated = (cur?.desktop_view ?? "").trim();
  const curLogin = loginProviderForStep(cur);
  const curHasUrl = Boolean(cur?.entry_url);
  const curHasRun = Boolean(cur?.run?.length);
  if (openBtn) {
    openBtn.textContent =
      curRelated === "platforms"
        ? "Open Deployment"
        : curRelated && RELATED_VIEW_LABELS[curRelated]
          ? RELATED_VIEW_LABELS[curRelated].replace(/^Open /, "Open ")
          : curKind === "oauth" || curLogin
            ? "Login CLI"
            : curHasUrl && !curHasRun
              ? "Open portal"
              : curHasRun && !curHasUrl
                ? "Run local"
                : "Open / Run";
  }
  const laneCount = new Set(
    (view.steps ?? [])
      .filter((s) => launchStepBand(s.id ?? "") === "lane")
      .map((s) => launchLaneGroup(s.id ?? "")),
  ).size;
  if (hint) {
    hint.textContent = view.finished
      ? "Launch finished — prefer Publish for the remaining checklist."
      : `Choice board · ${laneCount} lane${laneCount === 1 ? "" : "s"} · step ${(view.current_index ?? 0) + 1}/${view.total ?? 0} — Open a dashboard, then Verify → Confirm.`;
  }

  const renderCtas = (
    step: NonNullable<LaunchView["current"]>,
    isCurrent: boolean,
  ): string => {
    const kind = (step.kind ?? "").toLowerCase();
    const related = (step.desktop_view ?? "").trim();
    const url = step.entry_url ?? "";
    const hasRun = Boolean(step.run?.length);
    const loginProv = loginProviderForStep(step);
    const parts: string[] = [];

    if (related && RELATED_VIEW_LABELS[related]) {
      const label =
        related === "platforms"
          ? "Open Deployment"
          : related === "sign"
            ? "Open Sign"
            : related === "integrations"
              ? "Open Suite"
              : related === "env"
                ? "Open Env"
                : RELATED_VIEW_LABELS[related];
      parts.push(
        `<button type="button" class="primary launch-step-related" data-view="${escapeHtml(
          related,
        )}">${escapeHtml(label)}</button>`,
      );
    } else if (kind === "oauth" && loginProv) {
      parts.push(
        `<button type="button" class="primary launch-step-login" data-provider="${escapeHtml(
          loginProv,
        )}">Login CLI</button>`,
      );
    } else {
      if (loginProv) {
        parts.push(
          `<button type="button" class="primary launch-step-login" data-provider="${escapeHtml(
            loginProv,
          )}">Login CLI</button>`,
        );
      }
      if (url) {
        parts.push(
          `<button type="button" class="${
            loginProv ? "" : "primary "
          }launch-step-open" data-url="${escapeHtml(url)}">Open portal</button>`,
        );
      }
      if (hasRun) {
        parts.push(
          `<button type="button" class="${
            loginProv || url ? "" : "primary "
          }launch-step-run" data-current="1" ${
            isCurrent ? "" : "disabled"
          }>${isCurrent ? "Run local" : "Run when current"}</button>`,
        );
      }
    }
    if (isCurrent) {
      parts.push(
        `<button type="button" class="launch-step-skip" title="Confirm this optional lane and advance">Skip</button>`,
      );
    }
    if (!parts.length) {
      parts.push(
        `<button type="button" class="primary" disabled>${
          isCurrent ? "Confirm when ready" : "Advance with Next"
        }</button>`,
      );
    }
    return `<div class="btns launch-portal-ctas">${parts.join("")}</div>`;
  };

  const renderStepRow = (
    s: NonNullable<LaunchView["steps"]>[number],
    i: number,
  ): string => {
    const active = i === view.current_index ? " active-step" : "";
    const kind = (s.kind ?? "").toLowerCase();
    return `<li class="portal-step${active}" data-launch-idx="${i}">
        <div class="meta">
          <div class="title">${statusKindHtml(s.status)}${
            kind
              ? `<span class="kind kind-${escapeHtml(kind)}">${escapeHtml(kind)}</span>`
              : ""
          }${escapeHtml(s.title ?? s.id ?? "")}</div>
          <p class="detail">${escapeHtml(s.detail ?? "")}</p>
        </div>
        ${renderCtas(s, i === view.current_index)}
      </li>`;
  };

  const steps = view.steps ?? [];
  const prep = steps
    .map((s, i) => ({ s, i }))
    .filter(({ s }) => launchStepBand(s.id ?? "") === "prep");
  const cut = steps
    .map((s, i) => ({ s, i }))
    .filter(({ s }) => launchStepBand(s.id ?? "") === "cut");
  const laneSteps = steps
    .map((s, i) => ({ s, i }))
    .filter(({ s }) => launchStepBand(s.id ?? "") === "lane");

  if (prepEl && prepList && prepSummary) {
    if (!prep.length) {
      prepEl.hidden = true;
      prepList.innerHTML = "";
    } else {
      prepEl.hidden = false;
      const done = prep.filter(({ s }) => (s.status ?? "").toLowerCase() === "done").length;
      prepSummary.textContent = `Prep · ${done}/${prep.length} ok`;
      prepList.innerHTML = prep.map(({ s, i }) => renderStepRow(s, i)).join("");
    }
  }

  if (lanesEl && laneGrid) {
    if (!laneSteps.length) {
      lanesEl.hidden = true;
      laneGrid.innerHTML = "";
    } else {
      lanesEl.hidden = false;
      const groups = new Map<
        string,
        {
          indices: number[];
          statuses: string[];
          views: string[];
          details: string[];
          suggested: boolean;
          optional: boolean;
        }
      >();
      for (const { s, i } of laneSteps) {
        const g = launchLaneGroup(s.id ?? "");
        const entry = groups.get(g) ?? {
          indices: [],
          statuses: [],
          views: [],
          details: [],
          suggested: false,
          optional: true,
        };
        entry.indices.push(i);
        entry.statuses.push((s.status ?? "").toLowerCase());
        if (s.desktop_view) entry.views.push(s.desktop_view);
        if (s.detail) entry.details.push(s.detail);
        if (s.suggested) entry.suggested = true;
        if (s.optional === false) entry.optional = false;
        groups.set(g, entry);
      }
      const curId = cur?.id ?? "";
      const curGroup = curId ? launchLaneGroup(curId) : "";
      laneGrid.innerHTML = [...groups.entries()]
        .map(([group, info]) => {
          const meta = launchLaneMeta(group);
          const viewId = info.views[0] ?? meta.view ?? "";
          const allDone = info.statuses.every((st) => st === "done" || st === "skipped");
          const isCurrent = info.indices.includes(view.current_index ?? -1) || group === curGroup;
          const badge = isCurrent
            ? `<span class="launch-badge">Current</span>`
            : allDone
              ? `<span class="launch-badge">Done</span>`
              : info.suggested
                ? `<span class="launch-badge">Suggested</span>`
                : info.optional
                  ? `<span class="launch-badge">Optional</span>`
                  : "";
          const blurb = info.details[0] ?? meta.blurb;
          return `<button type="button" class="int-card launch-lane-card${
            isCurrent ? " is-active" : ""
          }${allDone ? " is-done" : ""}" data-lane="${escapeHtml(group)}" data-view="${escapeHtml(
            viewId,
          )}" data-launch-idx="${info.indices[0] ?? 0}">
            ${badge}
            <span class="int-card-title">${escapeHtml(meta.title)}</span>
            <span class="int-card-blurb">${escapeHtml(blurb)}</span>
          </button>`;
        })
        .join("");
    }
  }

  if (cutEl && cutList) {
    if (!cut.length) {
      cutEl.hidden = true;
      cutList.innerHTML = "";
    } else {
      cutEl.hidden = false;
      cutList.innerHTML = cut.map(({ s, i }) => renderStepRow(s, i)).join("");
    }
  }

  currentEl.innerHTML = cur
    ? `<div class="title">${statusKindHtml(cur.status ?? cur.kind)}${escapeHtml(cur.title ?? "")}</div>
       <p class="detail">${escapeHtml(cur.detail ?? "")}${
         cur.verify_hint ? ` · verify: ${escapeHtml(cur.verify_hint)}` : ""
       }</p>
       ${renderCtas(cur, true)}`
    : "<p class=\"detail\">No current step</p>";

  const bindLogin = (root: ParentNode) => {
    root.querySelectorAll<HTMLButtonElement>(".launch-step-login").forEach((btn) => {
      btn.addEventListener("click", () => {
        const provider = btn.getAttribute("data-provider") || "";
        void openPortalLoginTerminal(provider).then(() => {
          window.setTimeout(() => {
            void refreshLaunch();
          }, 1500);
        });
      });
    });
  };
  const bindRelated = (root: ParentNode) => {
    root.querySelectorAll<HTMLButtonElement>(".launch-step-related").forEach((btn) => {
      btn.addEventListener("click", () => {
        const viewId = (btn.getAttribute("data-view") || "").trim();
        if (viewId === "platforms") {
          openPlatformsCatalog();
        } else if (viewId === "integrations") {
          setView("integrations");
          renderIntegrations();
        } else if (viewId) {
          setView(viewId);
        }
        toast(
          `${RELATED_VIEW_LABELS[viewId] ?? viewId} — finish there, then Verify → Confirm`,
          "info",
          5000,
        );
      });
    });
  };
  const bindOpenRun = (root: ParentNode) => {
    root.querySelectorAll<HTMLButtonElement>(".launch-step-open").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const url = btn.getAttribute("data-url");
        if (url) await openUrl(url);
      });
    });
    root.querySelectorAll<HTMLButtonElement>(".launch-step-run").forEach((btn) => {
      btn.addEventListener("click", () => {
        if (btn.disabled) return;
        void openLaunchCurrentGate();
      });
    });
    root.querySelectorAll<HTMLButtonElement>(".launch-step-skip").forEach((btn) => {
      btn.addEventListener("click", () => {
        void launchAction(["confirm"]).then(() => launchAction(["next"]));
      });
    });
  };

  bindLogin(currentEl);
  bindRelated(currentEl);
  bindOpenRun(currentEl);
  if (prepList) {
    bindLogin(prepList);
    bindRelated(prepList);
    bindOpenRun(prepList);
  }
  if (cutList) {
    bindLogin(cutList);
    bindRelated(cutList);
    bindOpenRun(cutList);
  }
  laneGrid?.querySelectorAll<HTMLButtonElement>(".launch-lane-card").forEach((btn) => {
    btn.addEventListener("click", () => {
      const viewId = (btn.getAttribute("data-view") || "").trim();
      if (viewId === "platforms") {
        openPlatformsCatalog();
      } else if (viewId === "integrations") {
        setView("integrations");
        renderIntegrations();
      } else if (viewId) {
        setView(viewId);
      }
      toast(
        `${RELATED_VIEW_LABELS[viewId] ?? btn.querySelector(".int-card-title")?.textContent ?? "Lane"} — finish there, then Verify → Confirm`,
        "info",
        5000,
      );
    });
  });
}

async function refreshLaunch() {
  const result = await run(["launch", "--project", projectPath()], {
    step: "paste",
  });
  if (!result?.ok || !result.stdout) return;
  try {
    applyLaunchView(JSON.parse(result.stdout) as LaunchView);
  } catch {
    /* shown in output */
  }
}

async function launchAction(sub: string[]) {
  const args = ["launch", ...sub, "--project", projectPath()];
  // clap: parent flags before subcommand is awkward; use: launch --project . verify
  const ordered =
    sub.length === 0
      ? ["launch", "--project", projectPath()]
      : ["launch", "--project", projectPath(), ...sub];
  const result = await run(ordered, { step: "paste" });
  if (!result?.stdout) return;
  try {
    const parsed = JSON.parse(result.stdout) as LaunchView & {
      launch?: LaunchView;
      ok?: boolean;
      message?: string;
    };
    applyLaunchView(parsed.launch ?? parsed);
    if (parsed.message) {
      appendStream({ stream: "meta", text: parsed.message });
    }
  } catch {
    /* raw output shown */
  }
  void args;
}

async function exportVault() {
  const project = projectPath();
  if (!project) {
    show("Open a project folder first.");
    return;
  }
  if (!lastSecrets?.hints?.length) {
    await loadSecrets();
  }
  const hints = (lastSecrets?.hints ?? []).filter(
    (h): h is typeof h & { name: string } => !!h.name && !h.name.includes("<"),
  );
  if (!hints.length) {
    appendStream({
      stream: "meta",
      text: "No secret hints — add wrangler # Secrets or empty .dev.vars keys, or use CLI: orbityard vault export --out ./ship-secrets.km",
    });
    return;
  }

  const out = await invoke<string | null>("pick_vault_save", {
    defaultName: "ship-secrets.km",
  });
  if (!out) return;

  const pass = window.prompt("Vault passphrase (remember this — needed to open in Clavis)");
  if (!pass) return;
  const again = window.prompt("Confirm passphrase");
  if (pass !== again) {
    appendStream({ stream: "stderr", text: "Passphrases do not match." });
    return;
  }

  const entries: Array<{ title: string; value: string; url?: string; notes?: string }> =
    [];
  for (const h of hints) {
    const value = window.prompt(
      `Paste value for ${h.name} (Cancel skips this name)`,
      "",
    );
    if (value == null || value === "") continue;
    entries.push({
      title: h.name,
      value,
      url: h.entry_url ?? "",
      notes: "Exported from Orbit Yard desktop",
    });
  }
  if (!entries.length) {
    appendStream({ stream: "meta", text: "No values entered — vault not written." });
    return;
  }

  if (running) {
    show("Already running — wait for the current command.");
    return;
  }
  setBusy(true, "Exporting vault…");
  setProjectUi(true);
  let entriesPath = "";
  try {
    entriesPath = await invoke<string>("write_vault_entries_temp", {
      json: JSON.stringify(entries),
    });
    const result = await invoke<CmdResult>("run_orbityard_env", {
      project,
      args: [
        "vault",
        "export",
        "--out",
        out,
        "--entries-file",
        entriesPath,
        "--name",
        "Orbit Yard secrets",
      ],
      env: { SHIP_VAULT_PASSPHRASE: pass },
    });
    if (entriesPath) {
      await invoke("delete_path", { path: entriesPath }).catch(() => undefined);
    }
    const pretty = prettyMaybe(result.stdout);
    show(
      `vault export · exit ${result.code}\n\n${pretty ?? result.stdout}${
        result.stderr.trim() ? `\n\n[stderr]\n${result.stderr.trim()}` : ""
      }`,
    );
    if (result.ok) {
      appendStream({
        stream: "meta",
        text: `Encrypted vault saved · ${out} — open in Clavis / Keys Manager`,
      });
    }
  } catch (err) {
    if (entriesPath) {
      await invoke("delete_path", { path: entriesPath }).catch(() => undefined);
    }
    appendStream({ stream: "stderr", text: String(err) });
  } finally {
    setBusy(false, "Ready");
    setProjectUi(true);
  }
}

async function loadPortal(openAll = false) {
  const args = ["portal", "--project", projectPath()];
  const result = await run(args, { step: "portal", quietToast: true });
  if (!result) {
    toast("Portal busy — Cancel to unlock, then retry", "err");
    return;
  }
  if (!result.ok || !result.stdout.trim()) {
    toast(`Portal: ${cmdFailDetail(result)}`, "err", 8000, [
      { id: "preview", label: "Preview log", icon: "open", run: () => openOutputPreview() },
    ]);
    return;
  }
  try {
    const plan = JSON.parse(result.stdout) as PortalPlan;
    applyPortalPlan(plan);
    setStep("portal", "done");
    if (openAll) {
      const urls = [
        ...new Set(
          (plan.steps ?? [])
            .map((s) => s.entry_url)
            .filter((u): u is string => !!u),
        ),
      ];
      for (const url of urls) await openUrl(url);
    } else {
      toast("Portal plan loaded", "ok", 2500);
    }
  } catch {
    toast("Portal: could not parse plan — open Preview", "err", 7000, [
      { id: "preview", label: "Preview log", icon: "open", run: () => openOutputPreview() },
    ]);
  }
}

function applyDetected(detected?: Detected) {
  lastDetected = detected;
  // Structure banners must follow detect after rebind (not stay on prior monorepo fail).
  if (activeViewId === "platforms") paintPlatformWizard();
  else if (activeViewId === "sign") paintSignWizard();
  const host = document.querySelector<HTMLElement>("#detect-chips");
  if (!host) {
    syncProjectIdentity();
    return;
  }
  if (!detected) {
    host.hidden = true;
    host.innerHTML = "";
    syncProjectIdentity();
    return;
  }
  const flags: Array<[string, boolean | undefined]> = [
    ["signet.toml", detected.signet_toml],
    ["package.json", detected.package_json],
    ["tauri", detected.tauri],
    ["wrangler", detected.wrangler],
    ["vercel", detected.vercel],
    ["netlify", detected.netlify],
    ["fly", detected.fly],
    ["railway", detected.railway],
    [
      "pages",
      Boolean(
        detected.marketing_site &&
          (detected.marketing_host === "pages" || !detected.marketing_host),
      ) || detected.marketing_host === "pages",
    ],
    ["github", detected.github && !detected.marketing_site],
    ["polar", detected.polar],
    ["orbit", detected.yard_configured],
    ["umami", detected.umami],
  ];
  host.hidden = false;
  const onFlags = flags.filter(([, on]) => on);
  if (!onFlags.length) {
    host.hidden = true;
    host.innerHTML = "";
  } else {
    host.innerHTML = onFlags
      .map(
        ([label]) =>
          `<button type="button" class="chip on" data-chip="${escapeHtml(label)}" title="Open ${escapeHtml(label)}">${escapeHtml(label)}</button>`,
      )
      .join("");
    host.querySelectorAll<HTMLButtonElement>("[data-chip]").forEach((btn) => {
      btn.addEventListener("click", () => {
        const chip = btn.getAttribute("data-chip");
        if (chip) routeDetectChip(chip);
      });
    });
  }
  // Soft-select Deployment host card when already on the view (no navigation).
  // Do not stomp an explicit primary_host from studio.json.
  const preferred = preferredHostingPlatformId(detected);
  if (preferred && activeViewId === "platforms" && !savedPrimaryHost()) {
    if (selectedPlatform !== preferred) {
      selectedPlatform = preferred;
      renderPlatforms();
    }
  }
  syncProjectIdentity();
}

function applyDoctor(report: DoctorReport) {
  const signetOk = !!report.signet?.found;
  const orbitOk = !!report.orbit?.found;
  setPill("pill-signet", signetOk ? "ok" : "bad", signetOk ? "Found" : "Missing");
  setPill("pill-orbit", orbitOk ? "ok" : "bad", orbitOk ? "Found" : "Missing");
  setPill("pill-doctor", report.ok ? "ok" : "bad", report.ok ? "Healthy" : "Issues");

  const metaSignet = document.querySelector("#meta-signet");
  const metaOrbit = document.querySelector("#meta-orbit");
  if (metaSignet) {
    metaSignet.textContent = signetOk
      ? `${report.signet?.version ?? "signet"} · ${report.signet?.path ?? ""}`
      : "Not on PATH (set SIGNET_PATH)";
  }
  if (metaOrbit) {
    metaOrbit.textContent = orbitOk
      ? `${report.orbit?.version ?? "orbit"} · ${report.orbit?.path ?? ""}`
      : "Not on PATH (set ORBIT_PATH)";
  }

  const notes = document.querySelector("#notes-list");
  if (notes) {
    const items = report.notes?.length
      ? report.notes
      : report.detected?.hints ?? ["No notes."];
    notes.innerHTML = items.map((n) => `<li>${escapeHtml(n)}</li>`).join("");
  }

  applyDetected(report.detected);
  setStep("doctor", report.ok ? "done" : "fail");
  applyNow(lastPublish);
}

function applyLastRun(last: ShipState["last_run"]) {
  lastRunState = last;
  paintDeployResultsBay();
  // Deploy card is driven by pulse; keep this for refreshShipState compatibility.
  if (!document.querySelector("#pill-deploy")) return;
  if (!last) {
    setPill("pill-deploy", "muted", "None");
    const meta = document.querySelector("#meta-deploy");
    if (meta && !lastPulse) meta.textContent = "No .ship/last-run.json yet";
    return;
  }
  const ok = !!last.ok;
  if (!lastPulse) {
    setPill("pill-deploy", ok ? "ok" : "bad", ok ? "Last run ok" : "Failed");
    const meta = document.querySelector("#meta-deploy");
    const steps = (last.steps ?? [])
      .map((s) => `${s.id ?? "?"}${s.ok === false ? "✗" : "✓"}`)
      .join(" → ");
    const when = last.finished_at ? ` · ${last.finished_at}` : "";
    if (meta) {
      meta.textContent = `${last.message ?? "last run"}${steps ? ` · ${steps}` : ""}${when}`;
    }
  }
  for (const s of last.steps ?? []) {
    if (s.id === "configure" || s.id === "sign" || s.id === "deploy") {
      setStep(s.id, s.ok === false ? "fail" : "done");
    }
  }
}

function applyStudio(studio: ShipState["studio"]) {
  const sa = signArgsEl();
  const da = deployArgsEl();
  if (sa) sa.value = joinArgs(studio?.sign_args, "doctor --json");
  if (da) da.value = joinArgs(studio?.deploy_args, "status");
  if (studio?.detected) applyDetected(studio.detected);
  if (studio?.notes?.length) {
    const notes = document.querySelector("#notes-list");
    if (notes && notes.querySelectorAll("li").length <= 1) {
      notes.innerHTML = studio.notes.map((n) => `<li>${escapeHtml(n)}</li>`).join("");
    }
  }
  syncLaunchPaymentsToggle(Boolean(studio?.launch_payments));
  const host = (studio?.primary_host ?? "").trim().toLowerCase();
  // Drop parked primary_host ids (flowsint / medusa) so Deployment stays on catalog cards.
  const hostOk =
    Boolean(host) &&
    hostingCatalogEntries({ orbitConfigured: true }).some((e) => e.id === host);
  rememberedPrimaryHost = hostOk ? host : null;
  if (hostOk) {
    selectedPlatform = host;
    if (activeViewId === "platforms") renderPlatforms();
  } else if (host && (host === "flowsint" || host === "medusa")) {
    rememberedPrimaryHost = null;
    selectedPlatform = preferredHostingPlatformId(lastDetected) ?? "selfhost";
    if (activeViewId === "platforms") renderPlatforms();
  }
}

function syncLaunchPaymentsToggle(on: boolean) {
  const el = document.querySelector<HTMLInputElement>("#opt-launch-payments");
  if (el) el.checked = on;
}

async function refreshShipState() {
  const project = projectPath();
  if (!project) return;
  try {
    const state = await invoke<ShipState>("load_ship_state", { project });
    applyStudio(state.studio);
    applyLastRun(state.last_run);
    const reveal = document.querySelector<HTMLButtonElement>("#btn-reveal");
    if (reveal) reveal.disabled = !project;
  } catch (err) {
    console.warn(err);
  }
}

function parseDoctor(stdout: string): DoctorReport | null {
  const raw = stdout.trim();
  if (!raw) return null;
  try {
    return JSON.parse(raw) as DoctorReport;
  } catch {
    // Stream noise may precede the report — take the last JSON object.
    const start = raw.lastIndexOf("{");
    const end = raw.lastIndexOf("}");
    if (start >= 0 && end > start) {
      try {
        return JSON.parse(raw.slice(start, end + 1)) as DoctorReport;
      } catch {
        return null;
      }
    }
    return null;
  }
}

async function run(
  args: string[],
  opts?: {
    step?: string;
    quietHeader?: boolean;
    silent?: boolean;
    quietToast?: boolean;
    busyLabel?: string;
    /** Override bound project (e.g. Targets → Deploy on apps/website). */
    projectOverride?: string;
  },
): Promise<CmdResult | undefined> {
  const project = opts?.projectOverride ?? projectPath();
  if (!project) {
    show("Open a project folder first.");
    return;
  }
  if (running) {
    show("Already running — wait for the current command, or Cancel to unlock.");
    const now = Date.now();
    if (now - lastBusyToastAt > 8000) {
      lastBusyToastAt = now;
      toast("Busy — Cancel unlocks Publish if stuck", "info", 4000);
    }
    return;
  }
  if (opts?.step) setStep(opts.step, "active");
  setBusy(true, opts?.busyLabel ?? "Running…");
  if (!opts?.silent) {
    streamBuf = opts?.quietHeader ? "" : `orbityard ${args[0]}\n`;
    show(streamBuf);
  }

  let endLabel = "Ready";
  let endFailed = false;
  try {
    const result = await invoke<CmdResult>("run_orbityard", { project, args });
    // Final pretty pass for JSON-heavy commands
    if (
      !opts?.silent &&
      (args[0] === "doctor" ||
        args[0] === "configure" ||
        args[0] === "portal" ||
        args[0] === "secrets" ||
        args[0] === "guide" ||
        args[0] === "ship" ||
        args[0] === "human" ||
        args[0] === "launch" ||
        args[0] === "publish" ||
        args[0] === "pulse" ||
        args[0] === "scopes" ||
        args[0] === "env" ||
        args[0] === "sign-paths" ||
        args[0] === "assist" ||
        args[0] === "status" ||
        args[0] === "selfhost" ||
        args[0] === "hostdeploy" ||
        args[0] === "flow")
    ) {
      const pretty = prettyMaybe(result.stdout);
      if (pretty) {
        const stderr = result.stderr.trim()
          ? `\n\n[stderr]\n${result.stderr.trim()}`
          : "";
        show(`orbityard ${args[0]} · exit ${result.code}\n\n${pretty}${stderr}`);
      }
    }

    if (args[0] === "doctor") {
      const report = parseDoctor(result.stdout);
      if (report) applyDoctor(report);
    }
    if (opts?.step) setStep(opts.step, result.ok ? "done" : "fail");
    if (args[0] === "configure" && result.ok) setStep("configure", "done");
    if (
      args[0] === "flow" ||
      args[0] === "configure" ||
      args[0] === "status" ||
      args[0] === "sign"
    ) {
      await refreshShipState();
    }
    endLabel = result.cancelled ? "Cancelled" : result.ok ? "Ready" : "Failed";
    endFailed = !result.ok && !result.cancelled;
    // Expected gate pauses / soft portal errors are not sticky FAILED.
    {
      const err = `${result.stderr}\n${result.stdout}`.trim();
      if (/Confirm or Verify|still pending/i.test(err)) {
        endLabel = "Ready";
        endFailed = false;
      }
      if (isSoftCmdFailure(result)) {
        endLabel = "Ready";
        endFailed = false;
      }
      // Verify prints JSON then exits 1 when not ready — still a pause, not a crash.
      if (
        args.includes("verify") &&
        /"ok"\s*:\s*false/.test(result.stdout) &&
        !/spawn |not a directory|lock poisoned/i.test(err)
      ) {
        endLabel = "Ready";
        endFailed = false;
      }
    }
    if (!opts?.quietHeader && !opts?.silent && !opts?.quietToast) {
      const cmd = args[0] ?? "orbityard";
      if (result.cancelled) toast(`${cmd} cancelled`, "err");
      else if (result.ok) {
        // Never say "publish · done" — that reads as the whole ship finished.
        if (cmd === "publish" && args.includes("continue")) {
          /* publishContinue owns the toast */
        } else if (cmd === "publish") {
          const sub = args.find(
            (a) =>
              a === "confirm" ||
              a === "next" ||
              a === "verify" ||
              a === "open" ||
              a === "reset" ||
              a === "watch",
          );
          if (sub === "confirm") toast("Confirmed — press the green button", "ok");
          else if (sub === "next") toast("Advanced to next gate", "ok");
          else if (sub === "verify") toast("Verify finished — Confirm if ready", "ok");
          else toast(`Publish ${sub ?? "command"} finished`, "ok");
        } else if (cmd !== "portal") {
          // Portal success toast is owned by loadPortal / openPortalProvider.
          toast(`${cmd} finished`, "ok");
        }
      } else {
        const err = `${result.stderr}\n${result.stdout}`.trim();
        if (/Confirm or Verify|still pending/i.test(err)) {
          toastPublishGatePending();
        } else {
          toast(`${cmd} failed — ${cmdFailDetail(result)}`, "err", 8000, [
            {
              id: "preview",
              label: "Preview log",
              icon: "open",
              run: () => openOutputPreview(),
            },
          ]);
        }
      }
    }
    return result;
  } catch (err) {
    if (!opts?.silent) appendStream({ stream: "stderr", text: String(err) });
    if (opts?.step) setStep(opts.step, "fail");
    endLabel = "Failed";
    endFailed = true;
    if (!opts?.quietHeader && !opts?.silent) toast(String(err), "err");
  } finally {
    // Always unlock — prevents Refresh/Open stuck after cancel, hang, or throw.
    setBusy(false, endLabel, endFailed);
  }
}

function flowArgs(dryRun: boolean): string[] {
  const args = ["flow", "--project", projectPath()];
  if (dryRun) args.push("--dry-run");
  if (offline()) args.push("--offline");
  if (offline() || !includeDeploy()) args.push("--skip-deploy");
  return args;
}

/** Class N — Advanced online Deploy / Flow-with-deploy may prompt (Orbit auth). Prefer terminal. */
function ritualNetworkMayPrompt(): boolean {
  return studioMode() === "advanced" && !offline();
}

async function openRitualDeployTerminal(): Promise<boolean> {
  const project = projectPath();
  if (!project) {
    toast("Bind a project first", "info");
    return false;
  }
  return openShipctlTerminal(["deploy", "--project", project], {
    title: "Orbit Yard deploy",
    meta: "Launched terminal: orbityard deploy — finish auth/prompts there, then refresh pulse.",
    okToast: "Deploy opened in terminal — finish there if Orbit prompts",
  });
}

async function openRitualFlowTerminal(args: string[]): Promise<boolean> {
  return openShipctlTerminal(args, {
    title: "Orbit Yard flow",
    meta: "Launched terminal: orbityard flow — finish deploy/auth prompts there.",
    okToast: "Flow opened in terminal — finish deploy there if prompted",
  });
}

async function bindProject(path: string, autoDoctor = true) {
  const input = pathEl();
  if (input) input.value = path;
  lastPublish = null;
  lastPulse = null;
  dirtyConfirmArmed = false;
  lastDetected = undefined;
  saveRecent(path);
  await setTitle(path);
  resetSteps();
  setProjectUi(true);
  applyNow(null);
  show(`Working in ${projectName(path)}\n${path}\n`);
  toast(`Bound ${projectName(path)}`, "ok");
  await refreshShipState();
  // Serial only — parallel loadJsonCmd races the global running lock and drops pulse.
  await refreshSessionNow();
  // S1.16 S3 — quiet detect so Targets cards fill without a manual Detect click.
  await detectScopes({ quiet: true });
  if (autoDoctor) {
    await run(["doctor", "--project", path], { step: "doctor", quietHeader: true });
    await refreshSessionNow();
  }
  if (activeViewId === "platforms") {
    renderPlatforms();
    paintPlatformWizard();
  } else if (activeViewId === "sign") {
    renderSignCatalog();
    paintSignWizard();
  }
}

window.addEventListener("DOMContentLoaded", () => {
  const offlineSaved = localStorage.getItem(OFFLINE_KEY);
  const deploySaved = localStorage.getItem(DEPLOY_KEY);
  if (offlineEl() && offlineSaved !== null) offlineEl()!.checked = offlineSaved !== "0";
  if (deployEl() && deploySaved !== null) deployEl()!.checked = deploySaved === "1";

  offlineEl()?.addEventListener("change", syncDeployToggle);
  deployEl()?.addEventListener("change", syncDeployToggle);
  syncDeployToggle();
  // Output dock off by default — Preview in statusbar; Dock restores the strip.
  applyOutputDock(localStorage.getItem(OUTPUT_DOCK_KEY) === "1");
  applyStudioMode(studioMode());
  applyShipIntent(shipIntent());
  applyPublishUi(publishUiMode());
  syncWorkflowCards();
  document.querySelectorAll<HTMLButtonElement>(".mode-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const next = btn.dataset.mode === "advanced" ? "advanced" : "general";
      if (next === studioMode()) return;
      applyStudioMode(next, { rebuild: Boolean(projectPath()) });
    });
  });
  document.querySelectorAll<HTMLButtonElement>(".intent-btn").forEach((btn) => {
    btn.addEventListener("click", () => {
      const next = btn.dataset.intent === "local" ? "local" : "public";
      if (next === shipIntent()) return;
      applyShipIntent(next, { rebuild: Boolean(projectPath()) });
    });
  });
  renderRecent(loadRecent());
  void refreshShipctlPath();
  wireNavSections();
  setView("dashboard");
  setTitle(null);
  wireWindowChrome();
  document.querySelector("#btn-update-check")?.addEventListener("click", () => {
    void runUpdateCheck();
  });
  // Quiet boot notice — only toasts when a newer release exists (S0.8).
  window.setTimeout(() => {
    void runUpdateCheck({ quiet: true });
  }, 4000);

  document.querySelectorAll<HTMLButtonElement>(".nav-item[data-nav]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.dataset.nav;
      if (!id) return;
      setView(id);
      if (id === "publish" && projectPath() && !lastPublish?.steps?.length) {
        afterPaint(() => {
          void refreshPublish();
        });
      }
    });
  });

  document.querySelector("#btn-back-view")?.addEventListener("click", () => {
    if (!goBackView()) toast("No previous page", "info", 2000);
  });
  document.querySelector("#btn-back-publish")?.addEventListener("click", () => {
    setView("publish");
    if (!lastPublish?.steps?.length) {
      afterPaint(() => {
        void refreshPublish();
      });
    } else toast("Back on Publish", "info", 1800);
  });
  document.querySelector("#btn-search")?.addEventListener("click", () => cmdkOpen());
  document.querySelector("[data-cmdk-close]")?.addEventListener("click", () => cmdkClose());
  document.querySelector("#cmdk-input")?.addEventListener("input", (ev) => {
    const value = (ev.target as HTMLInputElement).value;
    cmdkIndex = 0;
    renderCmdk(value);
  });
  document.querySelector("#cmdk-input")?.addEventListener("keydown", (ev) => {
    const kev = ev as KeyboardEvent;
    if (kev.key === "ArrowDown") {
      kev.preventDefault();
      cmdkIndex = Math.min(cmdkIndex + 1, Math.max(0, cmdkFiltered.length - 1));
      renderCmdk((kev.target as HTMLInputElement).value);
    } else if (kev.key === "ArrowUp") {
      kev.preventDefault();
      cmdkIndex = Math.max(cmdkIndex - 1, 0);
      renderCmdk((kev.target as HTMLInputElement).value);
    } else if (kev.key === "Enter") {
      kev.preventDefault();
      runCmdk(cmdkIndex);
    } else if (kev.key === "Escape") {
      kev.preventDefault();
      cmdkClose();
    }
  });

  document.querySelector("#dash-open")?.addEventListener("click", () => {
    document.querySelector<HTMLButtonElement>("#btn-open")?.click();
  });
  document.querySelector("#next-ship-open")?.addEventListener("click", () => {
    const path =
      document.querySelector<HTMLButtonElement>("#next-ship-open")?.dataset.path?.trim() ?? "";
    if (!path) {
      toast("No other local recent — Open folder for the next repo", "info", 5000);
      return;
    }
    void bindProject(path, true);
  });
  document.querySelector("#now-primary")?.addEventListener("click", () => {
    if (!projectPath()) {
      document.querySelector<HTMLButtonElement>("#btn-open")?.click();
      return;
    }
    const id = document.querySelector<HTMLButtonElement>("#now-primary")?.dataset.pulseId ?? "";
    const view =
      document.querySelector<HTMLButtonElement>("#now-primary")?.dataset.pulseView || "publish";
    void runPulseAction(id || "publish_start", view);
  });
  document.querySelector("#now-switch")?.addEventListener("click", (ev) => {
    // Same-click document listener would close the switcher without this.
    ev.stopPropagation();
    toggleProjectSwitcher(true);
    document.querySelector<HTMLButtonElement>("#chrome-project")?.focus();
    toast("Choose a project", "info");
  });
  document.querySelector("#now-human")?.addEventListener("click", () => {
    void runHumanPortal({ openSources: true });
  });
  document.querySelector("#now-portal")?.addEventListener("click", () => {
    void loadPortal(false);
  });
  document.querySelector("#now-env")?.addEventListener("click", async () => {
    setView("env");
    const plan = (await loadJsonCmd(["env", "--project", projectPath()], {
      user: true,
      label: "Env",
    })) as EnvPortal | null;
    applyEnv(plan);
  });
  document.querySelector("#now-polar")?.addEventListener("click", () => {
    setupPolarPortal();
  });
  document.querySelector("#suite-catalog")?.addEventListener("click", async (ev) => {
    const t = ev.target as HTMLElement | null;
    const openBtn = t?.closest<HTMLButtonElement>("button.suite-open");
    const url = openBtn?.dataset.suiteUrl?.trim();
    if (url) {
      await openUrl(url);
      toast("Opened suite link in your browser", "info", 3500);
      return;
    }
    const whyBtn = t?.closest<HTMLButtonElement>("button.suite-why");
    const id = whyBtn?.dataset.suiteId?.trim();
    if (id === "velocity") {
      toast(
        "Velocity is a private modular scaffold — hire a wall or license it; no fake public storefront here. Get live still uses Publish / Deployment on this project.",
        "info",
        8000,
      );
    }
  });
  document.querySelector("#int-open")?.addEventListener("click", async () => {
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    if (!wiz) return;
    await openIntegrationVendor(wiz.openUrl, wiz.title);
  });
  document.querySelector("#int-open-links")?.addEventListener("click", async (ev) => {
    const btn = (ev.target as HTMLElement | null)?.closest<HTMLButtonElement>("[data-open-url]");
    const url = btn?.getAttribute("data-open-url")?.trim();
    if (!url) return;
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    await openIntegrationVendor(url, btn?.textContent?.trim() || wiz?.title || "Paddle");
  });
  document.querySelector("#int-portal-steps")?.addEventListener("click", () => {
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    if (!wiz?.provider || !isPortalProvider(wiz.provider)) {
      toast("No Portal steps for this wizard — use Open dashboard", "info");
      return;
    }
    void openPortalProvider(wiz.provider);
  });
  document.querySelector("#int-docs")?.addEventListener("click", async () => {
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    if (!wiz?.docsUrl) {
      toast("No Learn more link for this wizard", "info");
      return;
    }
    await openUrl(wiz.docsUrl);
    toast(`Opened ${wiz.title} docs — setup still happens on the vendor`, "info", 4500);
  });
  document.querySelector("#int-put")?.addEventListener("click", () => {
    const putName = INTEGRATION_HOST_PUT[selectedIntegration];
    if (!putName) {
      toast("This lane has no host Put — use page Opens, then Secrets / Env Put", "info", 5000);
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    void putNamedSecretOnHost(putName, selectedIntegration);
  });
  document.querySelector("#int-confirm-gate")?.addEventListener("click", () => {
    const stepId = INTEGRATION_PUBLISH_STEP[selectedIntegration] ?? null;
    void confirmWizardPublishGate({
      preferredStepId: stepId,
      missingHint: stepId
        ? selectedIntegration === "resend"
          ? "No env.sprint yet — switch to Public intent so env gates appear on Publish."
          : "No matching listing step yet — use Advanced + Public with that provider detected."
        : "No Publish gate mapped for this wizard — Continue publishing.",
      handoffMissingHint:
        selectedIntegration === "resend"
          ? "No env.sprint yet — Public intent required. Publish is open."
          : "No matching listing step yet — use Advanced + Public with that provider detected. Publish is open.",
      handoffGenericHint: "Back on Publish — Confirm the current gate when ready",
    });
  });
  document.querySelector("#int-gate-orient-go")?.addEventListener("click", () => {
    goToCurrentPublishGate();
  });
  document.querySelector("#int-continue-publish")?.addEventListener("click", () => {
    const stepId = INTEGRATION_PUBLISH_STEP[selectedIntegration] ?? null;
    void continuePublishingHandoff({
      preferredStepId: stepId,
      missingHint:
        selectedIntegration === "resend"
          ? "No env.sprint yet — Public intent required. Publish is open."
          : "No matching listing step yet — use Advanced + Public with that provider detected. Publish is open.",
      genericHint: "Back on Publish — Confirm the current gate when ready",
    });
  });
  document.querySelector("#plat-open")?.addEventListener("click", async () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz) return;
    if (platformLocked(wiz)) {
      toastHostingNeedsPublic(() => {
        void document.querySelector<HTMLButtonElement>("#plat-open")?.click();
      });
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    if (offline() && isHostedCliDeployCard(wiz.id)) {
      ensureHostingReady({ toastOk: true });
    }
    // Hard spawn lanes: Self-host / Umami / Flowsint / Medusa here; cloud hostdeploy asserts inside runHostedCliDeploy.
    if (
      wiz.id === "selfhost" ||
      wiz.id === "umami" ||
      wiz.id === "flowsint" ||
      wiz.id === "medusa"
    ) {
      const structureOk = await assertStructureProceed(wiz.id);
      paintPlatformWizard();
      if (!structureOk) return;
    } else if (wiz.id === "github-pages" || wiz.id === "orbit") {
      const soft = evaluateStructureGate(wiz.id, lastDetected, { projectPath: projectPath() });
      paintDeployStructureGate(wiz.id);
      if (soft && !soft.ok) {
        toast(soft.reason, "info", 9000, structureGateToastActions(soft));
      }
    }
    if (wiz.id === "selfhost") {
      selfhostResultsDismissed = false;
      applyOutputDock(true);
      paintPlatformWizard();
      const result = await run(["selfhost"], {
        quietToast: true,
        busyLabel: "Checking…",
      });
      await refreshShipState();
      await refreshSessionNow();
      paintPlatformWizard();
      if (!result) return;
      if (result.cancelled) toast("Self-host check cancelled", "ok", 4000);
      else if (result.ok) {
        toast("Self-host check ok — Open live to keep serving on loopback", "ok", 5500, [
          {
            id: "open-live",
            label: "Open live",
            icon: "continue",
            run: () => void startSelfhostServe({ openLive: true }),
          },
        ]);
      } else toast("Self-host check failed — see Output", "err", 5000);
      return;
    }
    if (wiz.id === "umami") {
      applyOutputDock(true);
      paintPlatformWizard();
      toast("Umami Deploy — compose + heartbeat in Output", "ok", 3500);
      const result = await run(["umami", "deploy"], {
        quietToast: true,
        busyLabel: "Umami…",
      });
      await refreshShipState();
      await refreshSessionNow();
      paintPlatformWizard();
      if (!result) return;
      if (result.cancelled) {
        toast("Umami deploy cancelled", "ok", 4000);
        return;
      }
      const report = parseUmamiDeployReport(result.stdout);
      const adminUrl =
        report?.admin_url ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:3000";
      if (result.ok && report?.ok !== false) {
        toast(`Umami Admin ready — ${adminUrl}`, "ok", 9000, [
          {
            id: "open-admin",
            label: "Open Admin",
            icon: "open",
            run: () => {
              void openUrl(adminUrl).then(() =>
                toast("Change default admin password, then Confirm on Publish", "info", 5000),
              );
            },
          },
        ]);
      } else {
        const detail =
          report?.message ||
          result.stderr.trim().split("\n").filter(Boolean).pop() ||
          result.stdout.trim().split("\n").filter(Boolean).pop() ||
          "Umami deploy failed — see Output";
        toast(detail, "err", 8000);
      }
      return;
    }
    if (wiz.id === "flowsint") {
      applyOutputDock(true);
      paintPlatformWizard();
      toast("Flowsint Deploy — compose.prod + App proof in Output", "ok", 3500);
      const result = await run(["flowsint", "deploy"], {
        quietToast: true,
        busyLabel: "Flowsint…",
      });
      await refreshShipState();
      await refreshSessionNow();
      paintPlatformWizard();
      if (!result) return;
      if (result.cancelled) {
        toast("Flowsint deploy cancelled", "ok", 4000);
        return;
      }
      const report = parseFlowsintDeployReport(result.stdout);
      const appUrl =
        report?.app_url ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:5173";
      if (result.ok && report?.ok !== false) {
        toast(`Flowsint App ready — ${appUrl}`, "ok", 9000, [
          {
            id: "open-app",
            label: "Open App",
            icon: "open",
            run: () => {
              void openUrl(`${appUrl.replace(/\/+$/, "")}/register`).then(() =>
                toast("Create an account at /register, then Confirm on Publish", "info", 5000),
              );
            },
          },
        ]);
      } else {
        const detail =
          report?.message ||
          result.stderr.trim().split("\n").filter(Boolean).pop() ||
          result.stdout.trim().split("\n").filter(Boolean).pop() ||
          "Flowsint deploy failed — see Output";
        toast(detail, "err", 8000);
      }
      return;
    }
    if (wiz.id === "medusa") {
      applyOutputDock(true);
      paintPlatformWizard();
      toast("Medusa Deploy — compose + Admin health in Output", "ok", 3500);
      const result = await run(["medusa", "deploy"], {
        quietToast: true,
        busyLabel: "Medusa…",
      });
      await refreshShipState();
      await refreshSessionNow();
      paintPlatformWizard();
      if (!result) return;
      if (result.cancelled) {
        toast("Medusa deploy cancelled", "ok", 4000);
        return;
      }
      const report = parseMedusaDeployReport(result.stdout);
      const adminUrl =
        report?.admin_url ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:9000/app";
      if (result.ok && report?.ok !== false) {
        toast(`Medusa Admin ready — ${adminUrl}`, "ok", 9000, [
          {
            id: "open-admin",
            label: "Open Admin",
            icon: "open",
            run: () => {
              void openUrl(adminUrl).then(() =>
                toast(
                  "Create/sign in Admin, then Confirm on Publish (payments stay vendor dashboards)",
                  "info",
                  5000,
                ),
              );
            },
          },
        ]);
      } else {
        const detail =
          report?.message ||
          result.stderr.trim().split("\n").filter(Boolean).pop() ||
          result.stdout.trim().split("\n").filter(Boolean).pop() ||
          "Medusa deploy failed — see Output";
        toast(detail, "err", 8000);
      }
      return;
    }
    if (isHostedCliDeployCard(wiz.id)) {
      await runHostedCliDeploy(wiz);
      return;
    }
    await openHostDashboard(wiz);
    if (wiz.deployArgs) {
      const dep = deployArgsEl();
      if (dep && !dep.disabled) {
        dep.value = wiz.deployArgs;
        toast(`Ritual deploy_args set to «${wiz.deployArgs}»`, "ok", 4500);
        return;
      }
    }
  });
  document.querySelector("#plat-dashboard")?.addEventListener("click", async () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.openUrl) return;
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    await openHostDashboard(wiz);
  });
  document.querySelector("#plat-open-live")?.addEventListener("click", async () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (wiz?.id === "selfhost") {
      await startSelfhostServe({ openLive: true });
      return;
    }
    if (wiz?.id === "umami") {
      const btn = document.querySelector<HTMLButtonElement>("#plat-open-live");
      const url =
        btn?.dataset.url?.trim() ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:3000";
      await openUrl(url);
      toast("Opened Umami Admin — change default password, then Confirm on Publish", "ok", 5000);
      return;
    }
    if (wiz?.id === "flowsint") {
      const btn = document.querySelector<HTMLButtonElement>("#plat-open-live");
      const url =
        btn?.dataset.url?.trim() ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:5173";
      await openUrl(`${url.replace(/\/+$/, "")}/register`);
      toast("Opened Flowsint — register at /register, then Confirm on Publish", "ok", 5000);
      return;
    }
    if (wiz?.id === "medusa") {
      const btn = document.querySelector<HTMLButtonElement>("#plat-open-live");
      const url =
        btn?.dataset.url?.trim() ||
        (lastRunState?.urls ?? []).find((u) => isLoopbackDeployUrl(String(u))) ||
        "http://127.0.0.1:9000/app";
      await openUrl(url);
      toast(
        "Opened Medusa Admin — create/sign in, then Confirm on Publish (payments stay vendor dashboards)",
        "ok",
        5000,
      );
      return;
    }
    const btn = document.querySelector<HTMLButtonElement>("#plat-open-live");
    const url = btn?.dataset.url?.trim();
    if (!url) {
      toast("No live URL in last deploy evidence yet", "info");
      return;
    }
    await openUrl(url);
    toast("Opened live URL", "ok", 3500);
  });
  document.querySelector("#plat-results-dashboard")?.addEventListener("click", async () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (!wiz) {
      toast("No host selected", "info");
      return;
    }
    await openHostDashboard(wiz);
  });
  document.querySelector("#plat-troubleshoot")?.addEventListener("click", () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (!wiz) return;
    const recovery = classifyHostDeployFailure(
      `${lastPulse?.deploy?.detail ?? ""}\n${lastRunState?.message ?? ""}`,
      wiz.provider ?? wiz.id,
    );
    runHostRecoveryAction(recovery, wiz);
  });
  document.querySelector("#plat-cancel-deploy")?.addEventListener("click", () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (!wiz) return;
    runPlatCancelDeploy(wiz, {
      mode: wiz.id === "selfhost" ? undefined : "clear",
    });
  });
  document.querySelector("#plat-cancel-deploy-bay")?.addEventListener("click", () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (!wiz) return;
    runPlatCancelDeploy(wiz, {
      mode: wiz.id === "selfhost" ? undefined : "dashboard",
    });
  });
  document.querySelector("#plat-retry-deploy")?.addEventListener("click", () => {
    const wiz = deployCatalogEntries().find((w) => w.id === selectedPlatform);
    if (!wiz || !isHostedCliDeployCard(wiz.id)) {
      toast("Retry Deploy is for Cloudflare / Vercel / Netlify", "info");
      return;
    }
    void runHostedCliDeploy(wiz, {
      name: lastHostDeployName || defaultHostProjectName(),
      skipConfirm: true,
    });
  });
  document.querySelectorAll("[data-host-deploy-close]").forEach((el) => {
    el.addEventListener("click", () => closeHostDeployDialog());
  });
  document.querySelector("#btn-recent-history")?.addEventListener("click", () => {
    openRecentHistoryDialog();
  });
  document.querySelector("#btn-switcher-history")?.addEventListener("click", () => {
    openRecentHistoryDialog();
  });
  document.querySelectorAll("[data-recent-history-close]").forEach((el) => {
    el.addEventListener("click", () => closeRecentHistoryDialog());
  });
  document.querySelector("#recent-history-search")?.addEventListener("input", () => {
    renderRecentHistoryList();
  });
  document.querySelector("#recent-history-search")?.addEventListener("keydown", (ev) => {
    if ((ev as KeyboardEvent).key === "Escape") closeRecentHistoryDialog();
  });
  document.querySelector("#recent-history-open-folder")?.addEventListener("click", async () => {
    closeRecentHistoryDialog();
    try {
      const picked = await invoke<string | null>("pick_project");
      if (picked) await bindProject(picked, true);
    } catch (err) {
      show(String(err));
    }
  });
  document.querySelector("#host-deploy-confirm")?.addEventListener("click", () => {
    void confirmHostDeployAllInOne();
  });
  document.querySelector("#host-deploy-back-setup")?.addEventListener("click", () => {
    showHostDeployStep("setup");
  });
  document.querySelector("#host-deploy-name")?.addEventListener("keydown", (ev) => {
    if ((ev as KeyboardEvent).key === "Enter") {
      document.querySelector<HTMLButtonElement>("#host-deploy-confirm")?.click();
    }
    if ((ev as KeyboardEvent).key === "Escape") {
      closeHostDeployDialog();
    }
  });
  document.querySelector("#sign-open")?.addEventListener("click", async () => {
    const wiz = signCatalogEntries().find((w) => w.id === selectedSignLane);
    if (!wiz?.openUrl) {
      toast("No dashboard URL for this signing lane", "info");
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    await openUrl(wiz.openUrl);
    toast(`Opened ${wiz.title} — finish on the vendor site`, "ok", 4500);
  });
  document.querySelector("#sign-confirm-gate")?.addEventListener("click", () => {
    void confirmWizardPublishGate({
      preferFromView: preferredSignPublishStep,
      missingHint: "No signing Publish gate yet — Open Publish to see the current checkpoint.",
      handoffMissingHint: "Back on Publish — Confirm the signing gate when it is current",
      handoffGenericHint: "Back on Publish — Confirm the signing gate when ready",
    });
  });
  document.querySelector("#sign-continue-publish")?.addEventListener("click", () => {
    void continuePublishingHandoff({
      preferFromView: preferredSignPublishStep,
      missingHint: "Back on Publish — Confirm the signing gate when it is current",
      genericHint: "Back on Publish — Confirm the signing gate when ready",
    });
  });
  document.querySelector("#plat-put")?.addEventListener("click", () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.provider || !providerHasEnvPut(wiz.provider)) {
      toast("Put is for Cloudflare / Vercel / Netlify — use Portal steps otherwise", "info");
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    void openPortalEnvPut(wiz.provider);
  });
  document.querySelector("#plat-import-env")?.addEventListener("click", () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.provider || !providerHasEnvPut(wiz.provider)) {
      toast("Import env is for Cloudflare / Vercel / Netlify / Fly / Railway", "info");
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    void openEnvImportDialog(wiz);
  });
  document.querySelectorAll("[data-env-import-close]").forEach((el) => {
    el.addEventListener("click", () => closeEnvImportDialog());
  });
  document.querySelector("#env-pack-continue")?.addEventListener("click", () => {
    void continueEnvImportFromPack();
  });
  document.querySelector("#env-import-back-packs")?.addEventListener("click", () => {
    showEnvImportStep("packs");
  });
  document.querySelector("#env-import-confirm")?.addEventListener("click", () => {
    void confirmEnvImport();
  });
  document.querySelector("#plat-docs")?.addEventListener("click", async () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.docsUrl) {
      toast("No Learn more link for this platform", "info");
      return;
    }
    await openUrl(wiz.docsUrl);
    toast(`Opened ${wiz.title} tutorial — Put stays in Studio when the host supports it`, "info", 4500);
  });
  document.querySelector("#plat-portal-steps")?.addEventListener("click", () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.provider || !isPortalProvider(wiz.provider)) {
      toast("No Portal steps for this platform — use Open dashboard", "info");
      return;
    }
    void openPortalProvider(wiz.provider);
  });
  document.querySelector("#plat-confirm-gate")?.addEventListener("click", () => {
    void confirmWizardPublishGate({
      preferFromView: preferredDeployPublishStep,
      missingHint: "No Live check / host gate yet — Open Publish to see the current checkpoint.",
      handoffMissingHint: "Back on Publish — Confirm Live check or the host gate when it is current",
      handoffGenericHint: "Back on Publish — Confirm Live check or the host gate when ready",
    });
  });
  document.querySelector("#plat-continue-publish")?.addEventListener("click", () => {
    void continuePublishingHandoff({
      preferFromView: preferredDeployPublishStep,
      missingHint: "Back on Publish — Confirm Live check or the host gate when it is current",
      genericHint: "Back on Publish — Confirm Live check or the host gate when ready",
    });
  });
  document.querySelector("#plat-use-local")?.addEventListener("click", () => {
    applyShipIntent("local", { rebuild: true });
    toast("Local intent — hosted deploy / live check omitted", "ok", 4500);
    paintPlatformWizard();
  });

  void listen<StreamLine>("orbityard-line", (event) => {
    appendStream(event.payload);
  });

  window.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape") {
      if (cmdkVisible()) {
        ev.preventDefault();
        cmdkClose();
        return;
      }
      if (outputPreviewVisible()) {
        ev.preventDefault();
        closeOutputPreview();
        return;
      }
      const historyDlg = document.querySelector<HTMLElement>("#recent-history-dialog");
      if (historyDlg && !historyDlg.hidden) {
        ev.preventDefault();
        closeRecentHistoryDialog();
        return;
      }
      if (running || selfhostServing) {
        ev.preventDefault();
        cancelSelfhostServe();
      }
      return;
    }
    const ctrl = ev.ctrlKey || ev.metaKey;
    if (ctrl && (ev.key === "f" || ev.key === "F") && outputPreviewVisible()) {
      ev.preventDefault();
      document.querySelector<HTMLInputElement>("#output-preview-search")?.focus();
      document.querySelector<HTMLInputElement>("#output-preview-search")?.select();
      return;
    }
    if (ctrl && (ev.key === "k" || ev.key === "K")) {
      ev.preventDefault();
      if (cmdkVisible()) cmdkClose();
      else cmdkOpen();
      return;
    }
    if (isTypingTarget(ev.target)) return;
    if (ev.altKey && (ev.key === "ArrowLeft" || ev.key === "Left")) {
      ev.preventDefault();
      if (!goBackView()) toast("No previous page", "info", 2000);
      return;
    }
    if (!ctrl || !projectPath()) return;
    if (ev.key === "d" || ev.key === "D") {
      ev.preventDefault();
      void run(["doctor", "--project", projectPath()], { step: "doctor" });
    } else if (ev.key === "s" || ev.key === "S") {
      ev.preventDefault();
      const args = ["sign", "--project", projectPath()];
      if (offline()) args.push("--offline");
      void run(args, { step: "sign" });
    } else if (ev.key === "Enter" && ev.shiftKey) {
      ev.preventDefault();
      if (includeDeploy() && !offline()) {
        if (!confirm("Run full flow including Orbit deploy (network)?")) return;
      }
      void run(flowArgs(false));
    } else if (ev.key === "Enter") {
      ev.preventDefault();
      void run(flowArgs(true));
    }
  });

  document.querySelector("#chrome-project")?.addEventListener("click", (ev) => {
    ev.stopPropagation();
    toggleProjectSwitcher();
  });
  document.querySelector("#btn-open-switch")?.addEventListener("click", () => {
    toggleProjectSwitcher(false);
    document.querySelector<HTMLButtonElement>("#btn-open")?.click();
  });
  document.addEventListener("click", (ev) => {
    const sw = document.querySelector<HTMLElement>("#project-switcher");
    const trig = document.querySelector<HTMLElement>("#chrome-project");
    if (!sw || sw.hidden) return;
    const t = ev.target as Node;
    if (sw.contains(t) || trig?.contains(t)) return;
    toggleProjectSwitcher(false);
  });

  document.querySelector("#btn-assist")?.addEventListener("click", async () => {
    setView("assist");
    const plan = (await loadJsonCmd(["assist", "--project", projectPath()], {
      user: true,
      label: "Assist",
    })) as AssistPlan | null;
    applyAssist(plan);
  });
  document.querySelector("#btn-assist-start")?.addEventListener("click", async () => {
    setView("publish");
    const raw = await loadJsonCmd(["assist", "--project", projectPath(), "--start"], {
      user: true,
      label: "Assist",
    });
    if (raw && typeof raw === "object" && "assist" in raw) {
      applyAssist((raw as { assist: AssistPlan }).assist);
    }
    if (raw && typeof raw === "object" && "publish" in raw) {
      applyPublishView((raw as { publish: PublishView }).publish);
    } else if (raw) {
      document.querySelector<HTMLButtonElement>("#btn-publish")?.click();
    }
    // Fail: loadJsonCmd toasted — do not chain into Publish refresh.
  });
  document.querySelector("#btn-scopes")?.addEventListener("click", () => {
    void detectScopes();
  });
  document.querySelector("#btn-scopes-save")?.addEventListener("click", () => {
    void saveScopes();
  });
  document.querySelector("#btn-stage-scopes-detect")?.addEventListener("click", () => {
    void detectScopes();
  });
  document.querySelector("#btn-stage-scopes-save")?.addEventListener("click", () => {
    void saveScopes();
  });
  const onScopeCheckboxChange = (ev: Event) => {
    if (!(ev.target as HTMLElement).matches("[data-scope-id]")) return;
    syncScopesDirtyCue();
    // Live enable Confirm when at least one box checked (before Save).
    const primaryBtn = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
    if (!primaryBtn || primaryBtn.dataset.stageAction !== "confirm") return;
    const n = document.querySelectorAll("#stage-scope-grid [data-scope-id]:checked").length;
    primaryBtn.disabled = running || n === 0;
    primaryBtn.title = n > 0 ? "Save selection is applied on Confirm" : "Select at least one scope";
  };
  document.querySelector("#scope-grid")?.addEventListener("change", onScopeCheckboxChange);
  document.querySelector("#stage-scope-grid")?.addEventListener("change", onScopeCheckboxChange);
  document.querySelector("#btn-env")?.addEventListener("click", async () => {
    const plan = (await loadJsonCmd(["env", "--project", projectPath()], {
      user: true,
      label: "Env",
    })) as EnvPortal | null;
    applyEnv(plan);
  });
  document.querySelector("#btn-sign-paths")?.addEventListener("click", () => {
    void refreshStatusProbes({ views: ["sign"], animate: true });
  });

  void (async () => {
    const kept = await pruneMissingRecents();
    const saved = localStorage.getItem(LAST_PROJECT_KEY);
    if (saved && kept.some((p) => recentPathKey(p) === recentPathKey(saved))) {
      await bindProject(saved, true);
    } else if (saved) {
      localStorage.removeItem(LAST_PROJECT_KEY);
      renderRecent(kept);
    } else {
      renderRecent(kept);
    }
  })();

  document.querySelector("#btn-open")?.addEventListener("click", async () => {
    try {
      const picked = await invoke<string | null>("pick_project");
      if (picked) await bindProject(picked, true);
    } catch (err) {
      show(String(err));
    }
  });

  document.querySelector("#btn-reveal")?.addEventListener("click", async () => {
    const project = projectPath();
    if (!project) return;
    try {
      await openPath(`${project}\\.ship`);
      toast("Opened .ship folder", "ok");
    } catch {
      try {
        await openPath(project);
        toast("Opened project folder", "ok");
      } catch (err) {
        show(String(err));
        toast(String(err), "err");
      }
    }
  });

  document.querySelector("#btn-save-args")?.addEventListener("click", async () => {
    const project = projectPath();
    if (!project) return;
    const sign_args = splitArgs(signArgsEl()?.value ?? "");
    const deploy_args = splitArgs(deployArgsEl()?.value ?? "");
    if (sign_args.length === 0) {
      show("sign_args cannot be empty.");
      return;
    }
    if (deploy_args.length === 0) {
      show("deploy_args cannot be empty.");
      return;
    }
    setBusy(true, "Saving…");
    try {
      const studio = await invoke<Record<string, unknown>>("save_ritual_args", {
        project,
        signArgs: sign_args,
        deployArgs: deploy_args,
      });
      show(`Saved .ship/studio.json\n\n${JSON.stringify(studio, null, 2)}`);
      setStep("configure", "done");
      setBusy(false, "Ready");
      toast("Ritual args saved", "ok");
    } catch (err) {
      show(String(err));
      setBusy(false, "Failed", true);
      toast(String(err), "err");
    }
  });

  document.querySelector("#btn-copy")?.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(streamBuf || outputEl()?.textContent || "");
      setBusy(false, "Copied");
      toast("Copied output", "ok", 1800);
      setTimeout(() => setBusy(false, "Ready"), 800);
    } catch (err) {
      show(String(err));
      toast(String(err), "err");
    }
  });

  document.querySelector("#btn-output-preview")?.addEventListener("click", () => {
    openOutputPreview();
  });
  document.querySelector("#btn-statusbar-preview")?.addEventListener("click", () => {
    openOutputPreview();
  });
  document.querySelector("#btn-statusbar-dock")?.addEventListener("click", () => {
    applyOutputDock(!outputDockVisible());
  });
  document.querySelector("#btn-output-dock-hide")?.addEventListener("click", () => {
    applyOutputDock(false);
  });
  document.querySelector("#btn-output-cancel")?.addEventListener("click", () => {
    cancelSelfhostServe();
  });
  document.querySelector("#btn-statusbar-cancel")?.addEventListener("click", () => {
    cancelSelfhostServe();
  });
  document
    .querySelector("[data-output-preview-close]")
    ?.addEventListener("click", () => closeOutputPreview());
  document
    .querySelector("#btn-output-preview-close")
    ?.addEventListener("click", () => closeOutputPreview());
  document
    .querySelector("#btn-output-preview-copy")
    ?.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(
          streamBuf || outputEl()?.textContent || "",
        );
        toast("Copied output", "ok", 1800);
      } catch (err) {
        toast(String(err), "err");
      }
    });
  document
    .querySelector("#output-preview-search")
    ?.addEventListener("input", (ev) => {
      previewSearchQuery = (ev.target as HTMLInputElement).value;
      previewMatchIndex = 0;
      renderOutputPreviewBody({ stickBottom: false });
    });
  document
    .querySelector("#output-preview-search")
    ?.addEventListener("keydown", (ev) => {
      const kev = ev as KeyboardEvent;
      if (kev.key === "Enter") {
        kev.preventDefault();
        stepPreviewMatch(kev.shiftKey ? -1 : 1);
      } else if (kev.key === "Escape") {
        kev.preventDefault();
        closeOutputPreview();
      }
    });
  document
    .querySelector("#btn-output-preview-prev")
    ?.addEventListener("click", () => stepPreviewMatch(-1));
  document
    .querySelector("#btn-output-preview-next")
    ?.addEventListener("click", () => stepPreviewMatch(1));

  document.querySelector("#btn-clear")?.addEventListener("click", () => {
    show("");
    forceUnlockUi("Ready");
  });

  document.querySelector("#btn-cancel")?.addEventListener("click", async () => {
    try {
      const killed = await invoke<boolean>("cancel_orbityard");
      if (!killed) {
        appendStream({ stream: "meta", text: "nothing to cancel — unlocking UI" });
      } else {
        appendStream({ stream: "meta", text: "cancel signal sent" });
      }
    } catch (err) {
      appendStream({ stream: "stderr", text: String(err) });
    } finally {
      // Band #25: always unlock Refresh even if Rust pid was already cleared.
      selfhostServing = false;
      paintPlatformWizard();
      forceUnlockUi("Cancelled");
      toast("Unlocked — you can Refresh Publish again", "ok");
    }
  });

  document.querySelectorAll<HTMLButtonElement>(".preset").forEach((btn) => {
    btn.addEventListener("click", () => {
      const sign = btn.getAttribute("data-sign");
      const deploy = btn.getAttribute("data-deploy");
      if (sign && signArgsEl()) signArgsEl()!.value = sign;
      if (deploy && deployArgsEl()) deployArgsEl()!.value = deploy;
    });
  });

async function runWizard() {
  appendStream({ stream: "meta", text: "wizard: guide → doctor → portal → secrets → configure → dry-run → open entries" });
  const guideResult = await run(["guide", "--project", projectPath()]);
  await run(["doctor", "--project", projectPath()], { step: "doctor" });
  await loadPortal(false);
  await loadSecrets();
  await run(["configure", "--project", projectPath()], { step: "configure" });
  await run(flowArgs(true));
  let opened = 0;
  try {
    const plan = guideResult?.stdout ? (JSON.parse(guideResult.stdout) as { entry_urls?: string[] }) : null;
    const urls = [...new Set(plan?.entry_urls ?? [])];
    if (urls.length && confirm(`Open ${urls.length} provider/marketplace entry page(s) in the browser?`)) {
      for (const url of urls) {
        await openUrl(url);
        opened += 1;
      }
    }
  } catch {
    /* ignore */
  }
  appendStream({
    stream: "meta",
    text: `wizard offline done · opened ${opened} url(s) — paste secrets via TUI/CLI, then Flow when ready`,
  });
}

  document.querySelector("#btn-doctor")?.addEventListener("click", () =>
    run(["doctor", "--project", projectPath()], { step: "doctor" }),
  );
  document.querySelector("#btn-wizard")?.addEventListener("click", () => {
    void runWizard();
  });
  document.querySelector("#btn-ship")?.addEventListener("click", () => {
    const open = confirm("Also open provider/marketplace entry pages in the browser?");
    const args = ["ship", "--project", projectPath()];
    if (open) args.push("--open");
    void run(args);
  });
  document.querySelector("#btn-human")?.addEventListener("click", () => {
    void runHumanPortal({ openSources: true });
  });
  document.querySelector("#btn-launch")?.addEventListener("click", () => {
    void refreshLaunch();
  });
  document.querySelector("#opt-launch-payments")?.addEventListener("change", () => {
    const el = document.querySelector<HTMLInputElement>("#opt-launch-payments");
    const project = projectPath();
    if (!el || !project) return;
    void (async () => {
      try {
        await invoke("set_launch_payments", { project, enabled: el.checked });
        // Rebuild Launch so Payments lane appears/disappears.
        await run(["launch", "--project", project, "reset"], { quietHeader: true, quietToast: true });
        await refreshLaunch();
        toast(
          el.checked ? "Payments lane on — Refresh shows Suite vendor bookmarks" : "Payments lane off",
          "ok",
          3500,
        );
      } catch (err) {
        el.checked = !el.checked;
        toast(String(err), "err");
      }
    })();
  });
  document.querySelector("#btn-publish")?.addEventListener("click", () => {
    void refreshPublish();
  });
  document.querySelector("#btn-publish-related")?.addEventListener("click", () => {
    void (async () => {
      const view =
        document.querySelector<HTMLButtonElement>("#btn-publish-related")?.dataset.relatedView ||
        lastPublish?.current?.desktop_view ||
        "";
      if (!view) return;
      const ok = await openRelatedStudioView(view);
      if (ok) toast(`${RELATED_VIEW_LABELS[view] ?? view} — then Back to Publish`, "info");
    })();
  });
  const goPublishStepNav = (el: HTMLElement | null) => {
    if (!el) return;
    const view = (el.dataset.desktopView ?? "").trim();
    if (!view) return;
    void navigatePublishStep({ desktop_view: view });
  };
  document.querySelector("#publish-steps")?.addEventListener("click", (ev) => {
    const t = (ev.target as HTMLElement).closest<HTMLElement>(".portal-step-nav");
    if (!t) return;
    ev.preventDefault();
    goPublishStepNav(t);
  });
  document.querySelector("#publish-steps")?.addEventListener("keydown", (ev) => {
    const kev = ev as KeyboardEvent;
    if (kev.key !== "Enter" && kev.key !== " ") return;
    const t = (ev.target as HTMLElement).closest<HTMLElement>(".portal-step-nav");
    if (!t) return;
    kev.preventDefault();
    goPublishStepNav(t);
  });
  document.querySelector("#publish-current")?.addEventListener("click", (ev) => {
    const cur = document.querySelector<HTMLElement>("#publish-current");
    if (!cur?.classList.contains("launch-current-nav")) return;
    if ((ev.target as HTMLElement).closest("button,a")) return;
    goPublishStepNav(cur);
  });
  document.querySelector("#publish-current")?.addEventListener("keydown", (ev) => {
    const kev = ev as KeyboardEvent;
    if (kev.key !== "Enter" && kev.key !== " ") return;
    const cur = document.querySelector<HTMLElement>("#publish-current");
    if (!cur?.classList.contains("launch-current-nav")) return;
    kev.preventDefault();
    goPublishStepNav(cur);
  });
  document.querySelector("#btn-publish-open")?.addEventListener("click", () => {
    void (async () => {
      const project = projectPath();
      if (!project) return;
      const cur = lastPublish?.current;
      const related = (cur?.desktop_view ?? "").trim();
      // Band #16: Open matches Related for every RELATED_VIEW_LABELS target (incl. dashboard).
      const studioDetail = Boolean(related && RELATED_VIEW_LABELS[related]);
      if (studioDetail) {
        // Live check / legal → dashboard was a bounce (flash Publish ↔ Dashboard).
        if (!shouldLeavePublishForRelated(related)) {
          toast(
            related === "dashboard"
              ? "Stay on Publish — Confirm when this check is done (Local may have no live URL)."
              : "Stay on Publish — use Confirm when ready",
            "info",
            5000,
          );
          return;
        }
        await openRelatedStudioView(related);
        toast(`${RELATED_VIEW_LABELS[related] ?? related} — Login CLI / Open if needed, then Confirm`, "info");
        // Panel-first steps (Scopes / Env / …): stay there — do not force Publish or a terminal.
        if (PANEL_FIRST_VIEWS.has(related)) return;
      }
      const kind = (cur?.kind ?? "").toLowerCase();
      const loginProvider = loginProviderForStep(cur);
      if (kind === "oauth" && loginProvider) {
        await openPortalLoginTerminal(loginProvider);
        window.setTimeout(() => {
          void (async () => {
            const status = await invoke<CmdResult>("run_orbityard", {
              project,
              args: publishArgs(),
            });
            if (status?.ok && status.stdout) {
              try {
                applyPublishView(JSON.parse(status.stdout) as PublishView);
              } catch {
                /* ignore */
              }
            }
          })();
        }, 1500);
        return;
      }
      const needsTerminal =
        Boolean(cur?.run?.length) ||
        kind === "oauth" ||
        kind === "sign" ||
        kind === "deploy";
      if (needsTerminal) {
        const opened = await openShipctlTerminal(
          ["publish", "--project", project, "open"],
          {
            title: "Orbit Yard publish",
            meta: "Launched terminal: orbityard publish open — complete auth / run there, then Verify/Confirm here.",
            okToast: "Terminal opened for this step",
          },
        );
        if (opened) {
          window.setTimeout(() => {
            void (async () => {
              const status = await invoke<CmdResult>("run_orbityard", {
                project,
                args: publishArgs(),
              });
              if (status?.ok && status.stdout) {
                try {
                  applyPublishView(JSON.parse(status.stdout) as PublishView);
                } catch {
                  /* ignore */
                }
              }
            })();
          }, 1500);
        } else {
          void publishAction(["open"]);
        }
      } else {
        void publishAction(["open"]);
      }
    })();
  });
  document.querySelector("#btn-publish-verify")?.addEventListener("click", () => {
    void publishAction(["verify"]);
  });
  document.querySelector("#opt-publish-watch")?.addEventListener("change", (ev) => {
    const on = (ev.target as HTMLInputElement).checked;
    if (on) {
      if (!projectPath()) {
        (ev.target as HTMLInputElement).checked = false;
        toast("Open a project first", "err");
        return;
      }
      startPublishWatch();
      toast(
        "Watching — Verify every 15s (disk · local CLI · official CLI probe). No Studio-held secrets.",
        "info",
        4500,
      );
    } else {
      stopPublishWatch();
    }
  });
  document.querySelector("#btn-publish-continue")?.addEventListener("click", () => {
    const cur = lastPublish?.current;
    const kind = (cur?.kind ?? "").toLowerCase();
    const pending = (cur?.status ?? "").toLowerCase() === "pending";
    const humanGate =
      pending &&
      (kind === "human" ||
        kind === "oauth" ||
        kind === "deploy" ||
        kind === "list" ||
        kind === "check");
    if (humanGate) {
      document.querySelector<HTMLButtonElement>("#btn-publish-open")?.click();
      return;
    }
    void publishContinuePaced();
  });
  document.querySelector("#btn-publish-confirm")?.addEventListener("click", () => {
    void publishConfirmWithDirtySoftGate();
  });
  document.querySelector("#btn-publish-next")?.addEventListener("click", () => {
    void publishAction(["next"]);
  });
  document.querySelectorAll<HTMLButtonElement>(".workflow-card").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.dataset.workflow;
      if (!isWorkflowId(id)) return;
      void startWorkflow(id);
    });
  });
  document.querySelector("#btn-publish-stages")?.addEventListener("click", () => {
    applyPublishUi("stages");
    if (lastPublish) renderPublishStage(lastPublish);
  });
  document.querySelector("#btn-publish-list")?.addEventListener("click", () => {
    applyPublishUi("list");
  });
  document.querySelector("#btn-stage-prev")?.addEventListener("click", () => {
    if (stageFocusIndex <= 0) return;
    stageFocusIndex -= 1;
    if (lastPublish) renderPublishStage(lastPublish);
  });
  document.querySelector("#btn-stage-next")?.addEventListener("click", () => {
    const steps = lastPublish?.steps ?? [];
    if (!steps.length || stageFocusIndex >= steps.length - 1) return;
    const focus = steps[stageFocusIndex];
    const status = (focus?.status ?? "").toLowerCase();
    const curIdx = lastPublish?.current_index ?? 0;
    const focusDone = status === "done" || status === "skipped";
    // Honesty: never advance past a pending Confirm via stage Next.
    if (!focusDone && stageFocusIndex >= curIdx) return;
    if (stageFocusIndex === curIdx && focusDone) {
      document.querySelector<HTMLButtonElement>("#btn-publish-next")?.click();
      return;
    }
    stageFocusIndex += 1;
    if (lastPublish) renderPublishStage(lastPublish);
  });
  document.querySelector("#btn-stage-primary")?.addEventListener("click", () => {
    onStagePrimary();
  });
  document.querySelector("#stage-rail")?.addEventListener("click", (ev) => {
    const dot = (ev.target as HTMLElement).closest<HTMLButtonElement>(".stage-dot");
    if (!dot) return;
    if (dot.dataset.stageMore === "1") {
      applyPublishUi("list");
      return;
    }
    const idx = Number(dot.dataset.stageIndex);
    if (!Number.isFinite(idx) || !lastPublish?.steps?.[idx]) return;
    stageFocusIndex = idx;
    renderPublishStage(lastPublish);
  });
  document.querySelector("#stage-scrub-track")?.addEventListener("click", (ev) => {
    const seg = (ev.target as HTMLElement).closest<HTMLButtonElement>(".stage-scrub-seg");
    if (!seg) return;
    const idx = Number(seg.dataset.stageIndex);
    if (!Number.isFinite(idx) || !lastPublish?.steps?.[idx]) return;
    stageFocusIndex = idx;
    applyPublishUi("stages");
    renderPublishStage(lastPublish);
  });
  document.querySelector("#btn-launch-open")?.addEventListener("click", () => {
    void openLaunchCurrentGate();
  });
  document.querySelector("#btn-launch-verify")?.addEventListener("click", () => {
    void launchAction(["verify"]);
  });
  document.querySelector("#btn-launch-confirm")?.addEventListener("click", () => {
    void launchAction(["confirm"]);
  });
  document.querySelector("#btn-launch-next")?.addEventListener("click", () => {
    void launchAction(["next"]);
  });
  document.querySelector("#btn-human-open")?.addEventListener("click", () => {
    void (async () => {
      if (!lastHuman) await runHumanPortal({ openSources: false });
      const urls = [...new Set(lastHuman?.open_order ?? [])];
      for (const url of urls) await openUrl(url);
      appendStream({
        stream: "meta",
        text: `Opened ${urls.length} paste-source page(s).`,
      });
    })();
  });
  document.querySelector("#btn-human-put")?.addEventListener("click", async () => {
    const project = projectPath();
    if (!project) return;
    const opened = await openShipctlTerminal(
      ["human", "--project", project, "--no-open", "--put"],
      {
        title: "Orbit Yard paste",
        meta: "Launched terminal: orbityard human --no-open --put — paste each value when prompted.",
        okToast: "Paste terminal opened — finish puts there",
      },
    );
    if (opened) setStep("paste", "done");
  });
  document.querySelector("#btn-guide")?.addEventListener("click", () =>
    run(["guide", "--project", projectPath()]),
  );
  document.querySelector("#btn-configure")?.addEventListener("click", () =>
    run(["configure", "--project", projectPath()], { step: "configure" }),
  );
  document.querySelector("#btn-portal")?.addEventListener("click", () => {
    void loadPortal(false);
  });
  document.querySelector("#btn-portal-open")?.addEventListener("click", () => {
    void loadPortal(true);
  });
  document.querySelector("#btn-portal-refresh")?.addEventListener("click", () => {
    void loadPortal(false);
  });
  document.querySelector("#btn-portal-open-all")?.addEventListener("click", async () => {
    const urls = [
      ...new Set(
        (lastPortal?.steps ?? [])
          .map((s) => s.entry_url)
          .filter((u): u is string => !!u),
      ),
    ];
    if (urls.length === 0) {
      await loadPortal(true);
      return;
    }
    for (const url of urls) await openUrl(url);
  });
  document.querySelector("#btn-secrets")?.addEventListener("click", () => {
    void loadSecrets();
  });
  document.querySelector("#btn-vault")?.addEventListener("click", () => {
    void exportVault();
  });
  document.querySelector("#btn-vault-export")?.addEventListener("click", () => {
    void exportVault();
  });
  document.querySelector("#btn-sign")?.addEventListener("click", () => {
    const args = ["sign", "--project", projectPath()];
    if (offline()) args.push("--offline");
    return run(args, { step: "sign" });
  });
  document.querySelector("#btn-deploy")?.addEventListener("click", async () => {
    if (offline()) {
      show("Deploy is blocked while Offline is on.");
      toast("Deploy blocked — turn Offline off", "info");
      return;
    }
    const ritualGate = evaluateStructureGate("ritual-orbit-deploy", lastDetected, {
      projectPath: projectPath(),
    });
    if (ritualGate && !ritualGate.ok) {
      toast(ritualGate.reason, "info", 10000, structureGateToastActions(ritualGate));
    }
    if (
      !confirm(
        "Run Orbit deploy (network)? Uses deploy_args from .ship/studio.json (or the Ritual field after Save).",
      )
    ) {
      return;
    }
    // Class N: Advanced → terminal so Orbit/vendor CLIs can prompt; General stays headless J.
    if (ritualNetworkMayPrompt()) {
      await openRitualDeployTerminal();
      return;
    }
    return run(["deploy", "--project", projectPath()], { step: "deploy" });
  });
  document.querySelector("#btn-flow-dry")?.addEventListener("click", () =>
    run(flowArgs(true)),
  );
  document.querySelector("#btn-flow")?.addEventListener("click", async () => {
    const withDeploy = includeDeploy() && !offline();
    if (withDeploy) {
      if (!confirm("Run full flow including Orbit deploy (network)?")) return;
    }
    const args = flowArgs(false);
    // Class N only when deploy is in the flow; dry-run / skip-deploy stay J.
    if (withDeploy && ritualNetworkMayPrompt()) {
      await openRitualFlowTerminal(args);
      return;
    }
    return run(args);
  });
  document.querySelector("#btn-status")?.addEventListener("click", () =>
    run(["status", "--project", projectPath()]),
  );
});

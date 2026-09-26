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
  WORKFLOW_KEY,
  NAV_SECTIONS_KEY,
  RELATED_VIEW_LABELS,
  VIEW_META,
} from "./constants";
import {
  integrationIconHtml,
  providerIconHtml,
  SCOPE_KIND_ORDER,
  scopeIconFile,
  iconImg,
} from "./icons";
import { INTEGRATION_WIZARDS } from "./integrations-data";
import { PLATFORM_GROUPS, PLATFORM_WIZARDS } from "./platforms-data";
import {
  highlightProviderSidebar,
  paintProviderWizard,
  renderProviderCatalogGrid,
  renderProviderSidebarTree,
} from "./provider-catalog";
import type {
  AssistPlan,
  CmdItem,
  CmdResult,
  Detected,
  DoctorReport,
  EnvPortal,
  HumanSprint,
  IntegrationWizard,
  LaunchView,
  PortalPlan,
  ProjectPulse,
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
          ? "Local intent — hosted env/deploy omitted"
          : "Public intent — hosted final-mile when detected",
        "ok",
      );
      await refreshSessionNow();
    })();
  }
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
      toast(mode === "general" ? "General mode — minimal publish" : "Advanced mode — full path", "ok");
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
let streamBuf = "";

function projectPath(): string {
  return pathEl()?.value.trim() ?? "";
}

function offline(): boolean {
  return offlineEl()?.checked ?? true;
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

function setBusy(busy: boolean, label = "Ready", failed = false) {
  running = busy;
  const className = busy ? "busy" : failed ? "failed" : "ready";
  for (const el of [
    stateEl(),
    document.querySelector<HTMLElement>("#run-state-bar"),
  ]) {
    if (!el) continue;
    el.textContent = label;
    el.className =
      el.id === "run-state-bar" ? `${className} statusbar-run` : className;
    if (el.id === "run-state-bar") {
      el.hidden = !busy && !failed && label === "Ready";
    }
  }
  // Always re-sync disabled state — Cancel / Clear / errors must unlock Refresh.
  setProjectUi(Boolean(projectPath()));
  syncNowQuick();
  clearBusyWatchdog();
  if (busy) {
    busyWatchdog = window.setTimeout(() => {
      if (!running) return;
      toast("Still running — use Cancel to unlock Publish if stuck", "info", 6000);
      const cancel = document.querySelector<HTMLButtonElement>("#btn-cancel");
      if (cancel) cancel.disabled = false;
    }, 90_000);
  }
}

function forceUnlockUi(reason = "Unlocked") {
  clearBusyWatchdog();
  running = false;
  for (const el of [
    stateEl(),
    document.querySelector<HTMLElement>("#run-state-bar"),
  ]) {
    if (!el) continue;
    el.textContent = reason;
    el.className = el.id === "run-state-bar" ? "ready statusbar-run" : "ready";
    if (el.id === "run-state-bar") el.hidden = true;
  }
  setProjectUi(Boolean(projectPath()));
  syncNowQuick();
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
  syncBackToPublish();
}

/** Continue is the fast path; Confirm/Next stay for explicit control. */
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
  confirmBtn.disabled = !on || running || !pending;
  nextBtn.disabled = !on || running || !done;
  if (continueBtn) {
    continueBtn.disabled = !on || running || finished || (!pending && !done);
    continueBtn.classList.add("primary");
    continueBtn.textContent = humanGate ? "Needs Open" : "Continue";
    continueBtn.title = humanGate
      ? "Human gate — use Open / Run, then Confirm"
      : "Advance Auto gates (stops at Human/Open)";
  }
  confirmBtn.classList.toggle("primary", false);
  nextBtn.classList.toggle("primary", false);
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
  { mode: StudioMode; intent: ShipIntent; label: string; toast: string }
> = {
  sign_only: {
    mode: "general",
    intent: "local",
    label: "Sign only",
    toast: "Sign only — Local · General. One checkpoint at a time on Publish.",
  },
  sign_deploy: {
    mode: "general",
    intent: "public",
    label: "Sign and deploy",
    toast: "Sign and deploy — Public · General. Hosted final-mile when detected.",
  },
  publish_platform: {
    mode: "advanced",
    intent: "public",
    label: "Publish to platforms",
    toast: "Publish to platforms — Advanced · Public. Listings and store gates included.",
  },
  deploy_only: {
    mode: "general",
    intent: "public",
    label: "Deploy focus",
    toast: "Deploy focus — Public · General. Confirm still required at deploy gates.",
  },
};

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
      return "Disk";
    case "local_cli":
      return "Local CLI";
    case "operator_cli":
      return "Official CLI probe";
    case "human_attest":
      return "Human attest";
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
  const layerPrefix = layer ? `Status · ${layer}. ` : "";
  if (status === "done" || status === "skipped") {
    return `${layerPrefix}This checkpoint is done. Press Continue to advance.`;
  }
  const kind = (step.kind ?? "").toLowerCase();
  const id = step.id ?? "";
  if ((step.verify_status ?? "").toLowerCase() === "human_attest") {
    return `${layerPrefix}Open the official UI, finish there, return → Confirm. Studio does not hold tokens.`;
  }
  if (isScopesStep(step)) {
    return `${layerPrefix}Choose what you’re shipping below, then Confirm & continue.`;
  }
  if (kind === "human" || id.includes("scope")) {
    return `${layerPrefix}Save the selection below, then Confirm & continue.`;
  }
  if (kind === "oauth" || kind === "list" || kind === "deploy" || kind === "check") {
    if ((step.id ?? "") === "live_check") {
      return `${layerPrefix}Needs a live URL or deploy evidence. If this is a desktop-only cut, switch intent to Local (Live check is omitted).`;
    }
    return `${layerPrefix}Open the official UI if needed. Confirm when you finished there.`;
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

function scopesGridHtml(plan: ScopePlan | null): string {
  const scopes = plan?.scopes ?? [];
  const active = new Set(plan?.active ?? []);
  if (!scopes.length) {
    return `<p class="detail empty-hint">No scopes detected — press Detect.</p>`;
  }
  return scopes
    .map((s) => {
      const id = s.id ?? "";
      const on = active.has(id) ? "checked" : "";
      return `<label class="scope-card">
            <input type="checkbox" data-scope-id="${escapeHtml(id)}" ${on} />
            <div>
              <strong>${escapeHtml(s.label ?? id)}</strong>
              <span>${escapeHtml(s.kind ?? "")} · ${escapeHtml(s.relative ?? ".")}${
                s.provider ? ` · ${escapeHtml(s.provider)}` : ""
              }</span>
            </div>
          </label>`;
    })
    .join("");
}

function fillScopeGrids(plan: ScopePlan | null) {
  const html = scopesGridHtml(plan);
  const main = document.querySelector("#scope-grid");
  const stage = document.querySelector("#stage-scope-grid");
  if (main) main.innerHTML = html;
  if (stage) stage.innerHTML = html;
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

function deployEvidenceFromPulse(pulse: ProjectPulse | null | undefined): boolean {
  const dep = pulse?.deploy;
  return (
    dep?.last_run_ok === true ||
    dep?.signal === "last_run_ok" ||
    dep?.signal === "orbit_deployed" ||
    (dep?.urls?.length ?? 0) > 0
  );
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
      const live =
        lastPulse?.deploy?.last_run_ok === true ||
        lastPulse?.deploy?.signal === "last_run_ok" ||
        lastPulse?.deploy?.signal === "orbit_deployed" ||
        (lastPulse?.deploy?.urls?.length ?? 0) > 0;
      guide.textContent = live
        ? "This pass’s required gates are done. Open the live URL when you want to smoke it again."
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
        "Public Live check needs a hosted URL. Local omits deploy/live check for desktop-only cuts.";
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
  toast(preset.toast, "ok", 4500);
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
  const el = document.querySelector("#shipctl-path");
  if (!el) return;
  try {
    const path = await invoke<string>("resolve_shipctl_path");
    el.textContent = `shipctl: ${path}`;
    el.setAttribute("title", path);
  } catch (err) {
    el.textContent = `shipctl: ${String(err)}`;
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
    return "That provider has no Portal steps — use Open dashboard on Platforms instead.";
  }
  if (/has no secret put CLI/i.test(t)) {
    return "This provider has no put CLI — open the vendor dashboard (or Platforms Docs).";
  }
  if (
    /program not found|cannot find|No such file|is not recognized as an internal or external command|The system cannot find the file/i.test(
      t,
    )
  ) {
    return "CLI missing on PATH — install wrangler/vercel/netlify/orbit/signet or use Platforms Docs.";
  }
  // Drop raw CLI invocations from operator-facing toasts.
  if (/^shipctl\b/i.test(t) || /\bshipctl publish\b/i.test(t)) {
    return "Finish the current checkpoint on Publish, then Confirm.";
  }
  return t;
}

/** Providers accepted by `shipctl portal --provider`. */
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

/** Class T — interactive shipctl in a real terminal (never headless stdin). */
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
    toast("No shipctl args for terminal", "err");
    return false;
  }
  try {
    await invoke("open_shipctl_terminal", {
      project,
      args,
      title: opts?.title ?? "Ship Studio",
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
      title: "Ship Studio portal login",
      meta: `Launched terminal: shipctl portal --provider ${provider} --login — complete OAuth there.`,
      okToast: `Login CLI opened for ${provider} — finish in the terminal`,
    },
  );
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
      ? `Finish «${title}» on the official site, then come back and Confirm.`
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
        openPlatformsCatalog({ preferGroup: "Official signing", selectId: "apple-sign" });
        return;
      }
      if (action === "choose-host") {
        openPlatformsCatalog({
          preferGroup: "Hosting",
          selectId: preferredHostingPlatformId(lastDetected) ?? "orbit",
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
      ? `<span class="status-probe-script" aria-hidden="true">shipctl pulse · sign-paths</span>`
      : `<button type="button" class="status-probe-recheck" data-probe-action="recheck">Check again</button>`;
  host.innerHTML = `<div class="status-probe-head" data-phase="${opts.phase}">
      <div class="status-probe-head-main">
        <span class="status-probe-spinner" aria-hidden="true"></span>
        <div>
          <span class="status-probe-kicker">Inspection</span>
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
  const deployOk =
    dep?.last_run_ok === true ||
    dep?.signal === "last_run_ok" ||
    dep?.signal === "orbit_deployed" ||
    (dep?.urls?.length ?? 0) > 0;
  const linked =
    dep?.signal === "vercel_linked" || dep?.signal === "orbit_configured";
  const deployActions: Array<{ id: string; label: string }> = [
    { id: "choose-host", label: "Choose host" },
  ];
  if (!deployOk && !linked) {
    deployActions.push({ id: "intent-local", label: "Use Local" });
  }
  rows.push({
    id: "deploy",
    label: "Deploy",
    state: deployOk ? "ok" : linked ? "guide" : "missing",
    badge: deployOk ? "Ready" : linked ? "Linked" : "No signal",
    detail: deployOk
      ? dep?.urls?.[0] || dep?.detail || "Prior deploy evidence found"
      : linked
        ? dep?.detail || "Host linked — deploy when releasing"
        : dep?.detail || "No Orbit / last-run / host link yet",
    suggestion: deployOk
      ? "Prior evidence found — redeploy when you cut again"
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

function afterPaint(fn: () => void) {
  requestAnimationFrame(() => {
    requestAnimationFrame(fn);
  });
}

const NAV_SECTION_DEFAULTS: Record<string, boolean> = {
  ship: true,
  targets: true,
  platforms: true,
  integrations: true,
  more: true,
  run: true,
};

const VIEW_TO_NAV_SECTION: Record<string, string> = {
  dashboard: "ship",
  publish: "ship",
  sign: "ship",
  env: "ship",
  scopes: "targets",
  platforms: "platforms",
  integrations: "integrations",
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
  if (id === "output") syncOutputMirror();
  if (id === "integrations") renderIntegrations();
  if (id === "platforms") renderPlatforms();
  if (id === "integrations" || prev === "integrations") highlightSidebarIntegration();
  if (id === "platforms" || prev === "platforms") highlightSidebarPlatforms();
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

function highlightSidebarIntegration() {
  highlightProviderSidebar(
    "#sidebar-integrations",
    "data-side-int",
    selectedIntegration,
    activeViewId === "integrations",
  );
}


function commandItems(): CmdItem[] {
  return [
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
      title: "Go to Scopes",
      keywords: "web api desktop directory",
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
      title: "Go to Integrations",
      keywords: "payment email polar stripe resend wizard",
      group: "Navigate",
      run: () => setView("integrations"),
    },
    {
      id: "nav-platforms",
      title: "Go to Platforms",
      keywords: "deploy host vercel cloudflare netlify orbit apple microsoft signing",
      group: "Navigate",
      run: () => openPlatformsCatalog(),
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
  if (quick) quick.hidden = !bound;
  if (polarBtn) polarBtn.hidden = !canShowPolarSetup();
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
      `No Portal plan for «${provider}» — use Open dashboard on Platforms`,
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
let selectedPlatform = "orbit";
let platformsPreferGroup: string | null = null;

function integrationAllowed(wiz: IntegrationWizard): boolean {
  if (!projectPath()) return false;
  if (wiz.needsPublic && (studioMode() !== "advanced" || shipIntent() !== "public")) return false;
  return true;
}

function platformLocked(wiz: (typeof PLATFORM_WIZARDS)[number]): boolean {
  return Boolean(wiz.needsPublic && shipIntent() !== "public");
}

function renderIntegrations() {
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
  else {
    renderIntegrations();
    highlightSidebarIntegration();
  }
}

function paintIntegrationWizard() {
  const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration) ?? null;
  paintProviderWizard({
    entry: wiz,
    panel: document.querySelector<HTMLElement>("#integrations-wizard"),
    titleEl: document.querySelector("#int-wizard-title"),
    blurbEl: document.querySelector("#int-wizard-blurb"),
    stepsEl: document.querySelector("#int-wizard-steps"),
    secondaryBtn: document.querySelector<HTMLButtonElement>("#int-portal-steps"),
    secondaryVisible: Boolean(wiz?.provider && isPortalProvider(wiz.provider)),
  });
}

function preferredHostingPlatformId(detected?: Detected | null): string | null {
  if (!detected) return null;
  // Orbit-deploy hosts first (Tier A), then CLI hosts, Pages, Orbit.
  if (detected.wrangler) return "cloudflare";
  if (detected.vercel) return "vercel";
  if (detected.netlify) return "netlify";
  if (detected.fly) return "fly";
  if (detected.railway) return "railway";
  if (detected.marketing_site && (detected.marketing_host === "pages" || !detected.marketing_host)) {
    return "github-pages";
  }
  if (detected.marketing_host === "pages") return "github-pages";
  if (detected.orbit_configured) return "orbit";
  return null;
}

function openPlatformsCatalog(opts?: { preferGroup?: string; selectId?: string }) {
  if (opts?.preferGroup) platformsPreferGroup = opts.preferGroup;
  const preferredHost = preferredHostingPlatformId(lastDetected);
  if (opts?.selectId) selectedPlatform = opts.selectId;
  else if (opts?.preferGroup === "Official signing") selectedPlatform = "apple-sign";
  else if (opts?.preferGroup === "Hosting") selectedPlatform = preferredHost ?? "orbit";
  else if (preferredHost) {
    selectedPlatform = preferredHost;
    platformsPreferGroup = "Hosting";
  }
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  setView("platforms");
  selectPlatform(selectedPlatform);
}

function renderPlatforms() {
  const host = document.querySelector<HTMLElement>("#platforms-catalog");
  if (!host) return;
  renderProviderCatalogGrid({
    host,
    entries: PLATFORM_WIZARDS,
    groups: PLATFORM_GROUPS,
    selectedId: selectedPlatform,
    preferGroup: platformsPreferGroup,
    iconHtml: providerIconHtml,
    onSelect: (id) => {
      selectPlatform(id);
    },
  });
  paintPlatformWizard();
}

function selectPlatform(id: string) {
  const wiz = PLATFORM_WIZARDS.find((w) => w.id === id);
  if (!wiz) return;
  if (!projectPath()) {
    toast("Bind a project first", "info");
    return;
  }
  selectedPlatform = id;
  if (wiz.group === "Hosting") platformsPreferGroup = "Hosting";
  if (wiz.group === "Official signing") platformsPreferGroup = "Official signing";
  if (activeViewId !== "platforms") setView("platforms");
  else {
    renderPlatforms();
    highlightSidebarPlatforms();
  }
}

function paintPlatformWizard() {
  const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform) ?? null;
  const showLocal = wiz?.group === "Hosting" && shipIntent() === "public";
  paintProviderWizard({
    entry: wiz,
    panel: document.querySelector<HTMLElement>("#platforms-wizard"),
    titleEl: document.querySelector("#plat-wizard-title"),
    blurbEl: document.querySelector("#plat-wizard-blurb"),
    stepsEl: document.querySelector("#plat-wizard-steps"),
    openBtn: document.querySelector<HTMLButtonElement>("#plat-open"),
    secondaryBtn: document.querySelector<HTMLButtonElement>("#plat-portal-steps"),
    secondaryVisible: Boolean(wiz?.provider && isPortalProvider(wiz.provider)),
    docsBtn: document.querySelector<HTMLButtonElement>("#plat-docs"),
  });
  const localBtn = document.querySelector<HTMLButtonElement>("#plat-use-local");
  if (localBtn) localBtn.hidden = !showLocal;
}

function renderSidebarPlatforms() {
  const host = document.querySelector<HTMLElement>("#sidebar-platforms");
  if (!host) return;
  renderProviderSidebarTree({
    host,
    entries: PLATFORM_WIZARDS,
    groups: PLATFORM_GROUPS,
    selectedId: selectedPlatform,
    activeView: activeViewId === "platforms",
    iconHtml: providerIconHtml,
    dataAttr: "data-side-plat",
    onSelect: (id) => {
      if (!projectPath()) {
        toast("Bind a project first", "info");
        return;
      }
      setView("platforms");
      selectPlatform(id);
    },
  });
}

function highlightSidebarPlatforms() {
  highlightProviderSidebar(
    "#sidebar-platforms",
    "data-side-plat",
    selectedPlatform,
    activeViewId === "platforms",
  );
}

function routeDetectChip(label: string) {
  const key = label.trim().toLowerCase();
  switch (key) {
    case "polar":
      setupPolarPortal();
      return;
    case "wrangler":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "cloudflare" });
      return;
    case "vercel":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "vercel" });
      return;
    case "netlify":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "netlify" });
      return;
    case "fly":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "fly" });
      return;
    case "railway":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "railway" });
      return;
    case "pages":
    case "github pages":
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "github-pages" });
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
      openPlatformsCatalog({ preferGroup: "Hosting", selectId: "orbit" });
      return;
    default:
      setView("portal");
      void loadPortal(false);
  }
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
          ? `${summary}. Continue Auto gates; Open/Confirm for required human work.`
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
    "Start the publish portal — doctor, env, sign, listing, deploy. You finish the vendor UIs; Ship Studio keeps the sequence.";
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
  const deployOk =
    pulse.deploy?.last_run_ok === true ||
    pulse.deploy?.signal === "last_run_ok" ||
    pulse.deploy?.signal === "orbit_deployed" ||
    (pulse.deploy?.urls?.length ?? 0) > 0;
  const linked =
    pulse.deploy?.signal === "vercel_linked" ||
    pulse.deploy?.signal === "orbit_configured";
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
        ? `Publish ${idx} — Continue advances Auto gates`
        : `Launch ${idx}`,
    };
  }

  if (deployOk) {
    return {
      state: "deployed",
      badge: "Deployed",
      title:
        pulse.deploy?.signal === "orbit_deployed"
          ? "Already live (Orbit)"
          : "Already deployed",
      detail:
        pulse.deploy?.urls?.[0] ||
        pulse.deploy?.detail ||
        "Prior successful deploy — redeploy only if you intend to.",
    };
  }
  if (linked) {
    return {
      state: "ready",
      badge: "Linked",
      title: "Provider linked",
      detail: pulse.deploy?.detail || "Configured locally — deploy when you need a new release.",
    };
  }
  if (localOnly) {
    return {
      state: "ready",
      badge: "Local only",
      title: "Wrangler local state",
      detail: "Dev/miniflare cache — not proof of a remote Workers deploy.",
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
    title: "Ready to ship",
    detail: "Start the publish portal when you are.",
  };
}

function buildStatusChecklist(pulse: ProjectPulse): StatusItem[] {
  const items: StatusItem[] = [];
  const signet = !!pulse.tools?.signet_found;
  const orbit = !!pulse.tools?.orbit_found;
  const linked =
    pulse.deploy?.signal === "orbit_deployed" ||
    pulse.deploy?.signal === "last_run_ok" ||
    pulse.deploy?.signal === "vercel_linked" ||
    pulse.deploy?.signal === "orbit_configured" ||
    (pulse.deploy?.urls?.length ?? 0) > 0;
  const wantsSignet = (pulse.kind ?? "").toLowerCase().includes("desktop")
    || (pulse.kind ?? "").toLowerCase().includes("tauri");
  const localIntent = shipIntent() === "local";
  const pub = pulse.publish;
  const launch = pulse.launch;
  const midWizard =
    (pub?.present && !pub.finished) || (launch?.present && !launch.finished);

  if (signet && orbit) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Tools ready",
      detail: "Signet + Orbit on PATH",
    });
  } else if (wantsSignet && signet) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Signet ready",
      detail: localIntent
        ? "Orbit optional for Local cuts"
        : "Orbit optional until you host a Public deploy",
    });
  } else if (!wantsSignet && linked) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Worker tooling OK",
      detail: "Prior live deploy evidence — Orbit optional for this stack",
    });
  } else if (wantsSignet && !signet) {
    items.push({
      id: "tools",
      state: "blocked",
      icon: "!",
      title: "Signet missing",
      detail: "Install Signet for local desktop cuts",
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
      detail: "Live check confirmed for this pass",
    });
  } else if (!midWizard) {
    const dep = pulse.deploy;
    const live =
      dep?.last_run_ok === true ||
      dep?.signal === "last_run_ok" ||
      dep?.signal === "orbit_deployed" ||
      (dep?.urls?.length ?? 0) > 0;
    if (live) {
      items.push({
        id: "deploy",
        state: "done",
        icon: "✓",
        title: "Prior deploy evidence",
        detail: dep?.urls?.[0] || dep?.detail || "Last shipctl run succeeded",
      });
    }
  }

  return items.slice(0, 3);
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
  setPill("pill-signet", signetOk ? "ok" : "bad", signetOk ? "Found" : "Missing");
  setPill("pill-orbit", orbitOk ? "ok" : "bad", orbitOk ? "Found" : "Missing");
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
  if (
    dep?.last_run_ok === true ||
    signal === "last_run_ok" ||
    signal === "orbit_deployed" ||
    (dep?.urls?.length ?? 0) > 0
  ) {
    setPill("pill-deploy", "ok", "Live");
  } else if (signal === "vercel_linked" || signal === "orbit_configured") {
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
  // Paint the page first; then load shipctl JSON so chrome does not hitch with the console work.
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
    await win.setTitle(path ? `Ship Studio — ${name}` : "Ship Studio");
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

function loadRecent(): string[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    const parsed = raw ? (JSON.parse(raw) as string[]) : [];
    return Array.isArray(parsed) ? parsed.filter((p) => typeof p === "string") : [];
  } catch {
    return [];
  }
}

function saveRecent(path: string) {
  const next = [path, ...loadRecent().filter((p) => p !== path)].slice(0, MAX_RECENT);
  localStorage.setItem(RECENT_KEY, JSON.stringify(next));
  localStorage.setItem(LAST_PROJECT_KEY, path);
  renderRecent(next);
}

function renderRecent(list: string[]) {
  const host = document.querySelector<HTMLElement>("#recent-list");
  const dash = document.querySelector<HTMLElement>("#dash-recents");
  const current = projectPath();
  const others = list.filter((p) => p !== current);
  const html = others
    .map(
      (p) =>
        `<button type="button" data-recent="${escapeHtml(p)}" title="${escapeHtml(p)}">${escapeHtml(projectName(p))}</button>`,
    )
    .join("");
  if (host) {
    if (others.length === 0) {
      host.hidden = true;
      host.innerHTML = "";
    } else {
      host.hidden = false;
      host.innerHTML = html;
      host.querySelectorAll<HTMLButtonElement>("button[data-recent]").forEach((btn) => {
        btn.addEventListener("click", () => {
          const p = btn.getAttribute("data-recent");
          if (p) void bindProject(p, true);
        });
      });
    }
  }
  if (dash) {
    if (others.length === 0 && current) {
      dash.hidden = true;
      dash.innerHTML = "";
    } else {
      const show = list.filter((p) => p !== current);
      dash.hidden = show.length === 0;
      dash.innerHTML = show
        .map(
          (p) =>
            `<button type="button" data-recent="${escapeHtml(p)}" title="${escapeHtml(p)}">${escapeHtml(projectName(p))}</button>`,
        )
        .join("");
      dash.querySelectorAll<HTMLButtonElement>("button[data-recent]").forEach((btn) => {
        btn.addEventListener("click", () => {
          const p = btn.getAttribute("data-recent");
          if (p) void bindProject(p, true);
        });
      });
    }
  }
  renderSwitcher(list);
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
  const items = list.length ? list : [];
  host.innerHTML = items.length
    ? items
        .map((p) => {
          const on = p === current ? " active" : "";
          return `<button type="button" class="switcher-item${on}" data-switch="${escapeHtml(p)}">${escapeHtml(projectName(p))}<span>${escapeHtml(p)}</span></button>`;
        })
        .join("")
    : `<p class="detail empty-hint">No recents yet — Open folder.</p>`;
  host.querySelectorAll<HTMLButtonElement>("[data-switch]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const p = btn.getAttribute("data-switch");
      toggleProjectSwitcher(false);
      if (p) void bindProject(p, true);
    });
  });
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
  const label = opts?.label ?? args[0] ?? "shipctl";
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

function applyScopes(plan: ScopePlan | null) {
  lastScopes = plan;
  fillScopeGrids(plan);
  renderSidebarTargets();
  syncStageScopesPrimary();
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
    label: "Scopes",
  })) as ScopePlan | null;
  applyScopes(plan);
  // Fail path: loadJsonCmd already toasted when user-initiated.
  if (!opts?.quiet && plan) {
    toast(plan.scopes?.length ? "Scopes detected" : "No scopes found", "ok");
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
    show("Select at least one scope.");
    toast("Select at least one scope", "err");
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
      toast("Scopes saved — Confirm on Publish", "ok");
    } else {
      toast("Scopes saved", "ok");
    }
  }
  return true;
}

function renderSidebarTargets() {
  const host = document.querySelector<HTMLElement>("#sidebar-targets");
  if (!host) return;
  const scopes = lastScopes?.scopes ?? [];
  if (!projectPath() || !scopes.length) {
    host.innerHTML = `<p class="nav-tree-empty">${projectPath() ? "No app or API targets detected." : "Bind a repo to list apps and APIs."}</p>`;
    return;
  }
  const active = new Set(lastScopes?.active ?? []);
  const kinds = [...new Set(scopes.map((s) => (s.kind ?? "root").toLowerCase()))].sort(
    (a, b) => SCOPE_KIND_ORDER.indexOf(a) - SCOPE_KIND_ORDER.indexOf(b),
  );
  host.innerHTML = kinds
    .map((kind) => {
      const rows = scopes
        .filter((s) => (s.kind ?? "root").toLowerCase() === kind)
        .map((s) => {
          const id = s.id ?? "";
          const on = active.has(id);
          const path = s.relative && s.relative !== "." ? s.relative : s.label ?? id;
          return `<button type="button" class="nav-target${on ? " is-on" : ""}" data-target="${escapeHtml(id)}" aria-pressed="${on ? "true" : "false"}">
            ${iconImg(scopeIconFile(s))}
            <span class="nav-target-mark" aria-hidden="true">${on ? "●" : "○"}</span>
            <span class="nav-target-text">
              <span class="nav-target-name">${escapeHtml(s.label ?? id)}</span>
              <span class="nav-target-path">${escapeHtml(path)}</span>
            </span>
          </button>`;
        })
        .join("");
      return `<p class="nav-kind">${escapeHtml(kind)}</p>${rows}`;
    })
    .join("");
  host.querySelectorAll<HTMLButtonElement>("[data-target]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.getAttribute("data-target");
      if (id) void toggleSidebarTarget(id);
    });
  });
}

async function toggleSidebarTarget(id: string) {
  const scopes = lastScopes?.scopes ?? [];
  const active = new Set(lastScopes?.active ?? []);
  if (active.has(id)) {
    if (active.size <= 1) {
      toast("Keep at least one deploy target", "info");
      return;
    }
    active.delete(id);
  } else {
    active.add(id);
  }
  const ids = scopes.map((s) => s.id ?? "").filter((sid) => active.has(sid));
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
  if (result?.ok) toast("Deploy targets updated", "ok");
}

function renderSidebarIntegrations() {
  const host = document.querySelector<HTMLElement>("#sidebar-integrations");
  if (!host) return;
  renderProviderSidebarTree({
    host,
    entries: INTEGRATION_WIZARDS,
    groups: ["Payments", "Email"],
    selectedId: selectedIntegration,
    activeView: activeViewId === "integrations",
    iconHtml: integrationIconHtml,
    dataAttr: "data-side-int",
    onSelect: (id) => {
      if (!projectPath()) {
        toast("Bind a project first", "info");
        return;
      }
      setView("integrations");
      selectIntegration(id);
    },
  });
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
      title: "Ship Studio env put",
      meta: `Launched terminal: shipctl env --provider ${provider} --put ${name} — paste when the CLI prompts.`,
      okToast: `Env Put opened for ${name} — finish in the terminal`,
    },
  );
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
    : `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Load env portal.</p></div></li>`;
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
      const canLogin = s.kind === "oauth" || (s.cli && s.cli.length > 0);
      return `<li class="portal-step" data-idx="${idx}">
        <div class="meta">
          <div class="title"><span class="kind">${escapeHtml(
            s.kind ?? "",
          )}</span>${escapeHtml(s.title ?? s.id ?? "step")}</div>
          <p class="detail">${escapeHtml(s.detail ?? "")}${
            cli && !url ? ` · ${escapeHtml(cli)}` : ""
          }</p>
        </div>
        <div class="btns">
          <button type="button" class="portal-open" data-url="${escapeHtml(
            url,
          )}" ${openDisabled} title="${escapeHtml(url || "No settings URL for this step")}">Open</button>
          ${
            docs
              ? `<button type="button" class="portal-docs" data-url="${escapeHtml(
                  docs,
                )}" title="${escapeHtml(docs)}">Docs</button>`
              : ""
          }
          ${
            canLogin
              ? `<button type="button" class="portal-login" data-provider="${escapeHtml(
                  s.provider ?? "",
                )}">Login CLI</button>`
              : ""
          }
        </div>
      </li>`;
    })
    .join("");

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
        return `<li class="portal-step">
          <div class="meta">
            <div class="title"><span class="kind">${escapeHtml(
              h.provider ?? "",
            )}</span>${escapeHtml(h.name ?? "")}</div>
            <p class="detail">${escapeHtml(h.detail ?? "")}${
              cmd ? ` · ${escapeHtml(cmd)}` : ""
            }</p>
          </div>
          <div class="btns">
            <button type="button" class="secret-open" data-url="${escapeHtml(
              url,
            )}" ${url ? "" : "disabled"}>Open</button>
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
      return `<li class="portal-step">
        <div class="meta">
          <div class="title"><span class="kind">${i + 1}</span>${escapeHtml(
            h.provider ?? "",
          )} · ${escapeHtml(h.name ?? "")}</div>
          <p class="detail">${escapeHtml(url || "no source url")}${
            cmd ? ` · ${escapeHtml(cmd)}` : ""
          }</p>
        </div>
        <div class="btns">
          <button type="button" class="human-open" data-url="${escapeHtml(
            url,
          )}" ${url ? "" : "disabled"}>Open source</button>
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
    syncBackToPublish();
    syncPublishGateButtons();
    applyNow(view);
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
      : `${modeLabel} · ${intentLabel} — Continue Auto gates; Open/Confirm for human work. Verify = ${
          verifyStatusLabel(cur?.verify_status) || "status probe"
        } (no secrets).`;
  }
  if (mins) {
    mins.hidden = false;
    mins.textContent = `~${view.minutes_remaining ?? 0} min remaining · ${view.minutes_total ?? 0} min total`;
  }
  if (progress) {
    const summary = publishProgressSummary(view);
    progress.hidden = !summary;
    progress.textContent = summary;
    progress.dataset.finished = view.finished ? "1" : "0";
  }
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
  syncBackToPublish();
  syncPublishGateButtons();
  applyNow(view);
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
    const result = await invoke<CmdResult>("run_shipctl", {
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
    if (ok && (!publishWatchLastOk || step !== publishWatchLastStep)) {
      toast(parsed.prompt ?? "Step ready — Confirm on the green button", "ok", 6000);
      const status = await invoke<CmdResult>("run_shipctl", {
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
  // Own toasts — never stack "publish failed" + raw shipctl copy.
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

function applyLaunchView(view: LaunchView | null) {
  lastLaunch = view;
  void lastLaunch;
  const currentEl = document.querySelector<HTMLElement>("#launch-current");
  const list = document.querySelector<HTMLElement>("#launch-steps");
  const hint = document.querySelector<HTMLElement>("#launch-hint");
  if (!currentEl || !list) return;
  if (!view?.steps?.length) {
    currentEl.innerHTML =
      '<p class="detail empty-hint">Refresh Launch to build the adaptive plan for this repo.</p>';
    list.innerHTML = "";
    return;
  }
  setView("launch");
  const cur = view.current;
  if (hint) {
    hint.textContent = view.finished
      ? "Launch workflow finished."
      : `Step ${(view.current_index ?? 0) + 1}/${view.total ?? 0} · ${view.done_count ?? 0} done — Open/Run on official platforms or local Signet/Orbit, then Verify/Confirm.`;
  }
  currentEl.innerHTML = cur
    ? `<div class="title">${statusKindHtml(cur.status ?? cur.kind)}${escapeHtml(cur.title ?? "")}</div>
       <p class="detail">${escapeHtml(cur.detail ?? "")}${
         cur.run?.length ? ` · run: ${escapeHtml(cur.run.join(" "))}` : ""
       }${
         cur.verify_hint ? ` · verify: ${escapeHtml(cur.verify_hint)}` : ""
       }</p>`
    : "<p class=\"detail\">No current step</p>";
  list.innerHTML = (view.steps ?? [])
    .map((s, i) => {
      const active = i === view.current_index ? " active-step" : "";
      return `<li class="portal-step${active}">
        <div class="meta">
          <div class="title">${statusKindHtml(s.status)}${escapeHtml(s.title ?? s.id ?? "")}</div>
        </div>
      </li>`;
    })
    .join("");
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
      text: "No secret hints — add wrangler # Secrets or empty .dev.vars keys, or use CLI: shipctl vault export --out ./ship-secrets.km",
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
      notes: "Exported from Ship Studio desktop",
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
    const result = await invoke<CmdResult>("run_shipctl_env", {
      project,
      args: [
        "vault",
        "export",
        "--out",
        out,
        "--entries-file",
        entriesPath,
        "--name",
        "Ship Studio secrets",
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
  const host = document.querySelector<HTMLElement>("#detect-chips");
  if (!host) return;
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
    ["orbit", detected.orbit_configured],
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
  // Soft-select Platforms card when already on the view (no navigation).
  const preferred = preferredHostingPlatformId(detected);
  if (preferred && activeViewId === "platforms" && platformsPreferGroup !== "Official signing") {
    if (selectedPlatform !== preferred) {
      selectedPlatform = preferred;
      platformsPreferGroup = "Hosting";
      renderPlatforms();
      highlightSidebarPlatforms();
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
    setPill("pill-deploy", ok ? "ok" : "bad", ok ? "Shipped" : "Failed");
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
  try {
    return JSON.parse(stdout.trim()) as DoctorReport;
  } catch {
    return null;
  }
}

async function run(
  args: string[],
  opts?: { step?: string; quietHeader?: boolean; silent?: boolean; quietToast?: boolean },
): Promise<CmdResult | undefined> {
  const project = projectPath();
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
  setBusy(true, "Running…");
  if (!opts?.silent) {
    streamBuf = opts?.quietHeader ? "" : `shipctl ${args[0]}\n`;
    show(streamBuf);
  }

  let endLabel = "Ready";
  let endFailed = false;
  try {
    const result = await invoke<CmdResult>("run_shipctl", { project, args });
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
        args[0] === "flow")
    ) {
      const pretty = prettyMaybe(result.stdout);
      if (pretty) {
        const stderr = result.stderr.trim()
          ? `\n\n[stderr]\n${result.stderr.trim()}`
          : "";
        show(`shipctl ${args[0]} · exit ${result.code}\n\n${pretty}${stderr}`);
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
      const cmd = args[0] ?? "shipctl";
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
    title: "Ship Studio deploy",
    meta: "Launched terminal: shipctl deploy — finish auth/prompts there, then refresh pulse.",
    okToast: "Deploy opened in terminal — finish there if Orbit prompts",
  });
}

async function openRitualFlowTerminal(args: string[]): Promise<boolean> {
  return openShipctlTerminal(args, {
    title: "Ship Studio flow",
    meta: "Launched terminal: shipctl flow — finish deploy/auth prompts there.",
    okToast: "Flow opened in terminal — finish deploy there if prompted",
  });
}

async function bindProject(path: string, autoDoctor = true) {
  const input = pathEl();
  if (input) input.value = path;
  lastPublish = null;
  lastPulse = null;
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
  const scopes = (await loadJsonCmd(["scopes", "--project", path])) as ScopePlan | null;
  applyScopes(scopes);
  if (autoDoctor) {
    await run(["doctor", "--project", path], { step: "doctor", quietHeader: true });
    await refreshSessionNow();
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
  renderSidebarTargets();
  renderSidebarIntegrations();
  renderSidebarPlatforms();
  setTitle(null);
  wireWindowChrome();

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
  document.querySelector("#int-open")?.addEventListener("click", async () => {
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    if (!wiz) return;
    if (!integrationAllowed(wiz) && wiz.needsPublic) {
      toast(
        studioMode() !== "advanced"
          ? "Payment wizards need Advanced mode"
          : "Payment wizards need Public intent",
        "info",
      );
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    await openUrl(wiz.openUrl);
    toast(`Opened ${wiz.title}`, "ok");
  });
  document.querySelector("#int-portal-steps")?.addEventListener("click", () => {
    const wiz = INTEGRATION_WIZARDS.find((w) => w.id === selectedIntegration);
    if (!wiz?.provider || !isPortalProvider(wiz.provider)) {
      toast("No Portal steps for this wizard — use Open dashboard", "info");
      return;
    }
    void openPortalProvider(wiz.provider);
  });
  document.querySelector("#int-continue-publish")?.addEventListener("click", () => {
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    setView("publish");
    toast("Back on Publish — Confirm the listing / gate when ready", "ok", 4500);
    void refreshPublish();
  });
  document.querySelector("#plat-open")?.addEventListener("click", async () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz) return;
    if (platformLocked(wiz)) {
      toast("Hosting providers need Public intent — or Use Local for desktop-only", "info");
      return;
    }
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    await openUrl(wiz.openUrl);
    if (wiz.deployArgs) {
      const dep = deployArgsEl();
      if (dep && !dep.disabled) {
        dep.value = wiz.deployArgs;
        toast(`Opened ${wiz.title} — Ritual deploy_args set to «${wiz.deployArgs}»`, "ok", 5000);
        return;
      }
    }
    toast(`Opened ${wiz.title}`, "ok");
  });
  document.querySelector("#plat-docs")?.addEventListener("click", async () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.docsUrl) {
      toast("No Docs link for this platform", "info");
      return;
    }
    await openUrl(wiz.docsUrl);
    toast(`Opened ${wiz.title} docs`, "ok");
  });
  document.querySelector("#plat-portal-steps")?.addEventListener("click", () => {
    const wiz = PLATFORM_WIZARDS.find((w) => w.id === selectedPlatform);
    if (!wiz?.provider || !isPortalProvider(wiz.provider)) {
      toast("No Portal steps for this platform — use Open / Docs", "info");
      return;
    }
    void openPortalProvider(wiz.provider);
  });
  document.querySelector("#plat-continue-publish")?.addEventListener("click", () => {
    if (!projectPath()) {
      toast("Bind a project first", "info");
      return;
    }
    setView("publish");
    toast("Back on Publish — Confirm Live check or the host gate when ready", "ok", 4500);
    void refreshPublish();
  });
  document.querySelector("#plat-use-local")?.addEventListener("click", () => {
    applyShipIntent("local", { rebuild: true });
    toast("Local intent — hosted deploy / live check omitted", "ok", 4500);
    paintPlatformWizard();
  });

  void listen<StreamLine>("shipctl-line", (event) => {
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
      if (running) {
        ev.preventDefault();
        void invoke<boolean>("cancel_shipctl");
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
  document.querySelector("#stage-scope-grid")?.addEventListener("change", (ev) => {
    if (!(ev.target as HTMLElement).matches("[data-scope-id]")) return;
    // Live enable Confirm when at least one box checked (before Save).
    const primaryBtn = document.querySelector<HTMLButtonElement>("#btn-stage-primary");
    if (!primaryBtn || primaryBtn.dataset.stageAction !== "confirm") return;
    const n = document.querySelectorAll("#stage-scope-grid [data-scope-id]:checked").length;
    primaryBtn.disabled = running || n === 0;
    primaryBtn.title = n > 0 ? "Save selection is applied on Confirm" : "Select at least one scope";
  });
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

  const saved = localStorage.getItem(LAST_PROJECT_KEY);
  if (saved) {
    void bindProject(saved, true);
  }

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
      const killed = await invoke<boolean>("cancel_shipctl");
      if (!killed) {
        appendStream({ stream: "meta", text: "nothing to cancel — unlocking UI" });
      } else {
        appendStream({ stream: "meta", text: "cancel signal sent" });
      }
    } catch (err) {
      appendStream({ stream: "stderr", text: String(err) });
    } finally {
      // Band #25: always unlock Refresh even if Rust pid was already cleared.
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
        toast(`${RELATED_VIEW_LABELS[related] ?? related} — finish, then Confirm`, "info");
        // Panel-first steps (Scopes / Env / …): stay there — do not force Publish or a terminal.
        if (PANEL_FIRST_VIEWS.has(related)) return;
      }
      const needsTerminal =
        Boolean(cur?.run?.length) ||
        cur?.kind === "oauth" ||
        cur?.kind === "sign" ||
        cur?.kind === "deploy";
      if (needsTerminal) {
        const opened = await openShipctlTerminal(
          ["publish", "--project", project, "open"],
          {
            title: "Ship Studio publish",
            meta: "Launched terminal: shipctl publish open — complete the step, then Verify/Confirm here.",
            okToast: "Terminal opened for this step",
          },
        );
        if (opened) {
          window.setTimeout(() => {
            void (async () => {
              const status = await invoke<CmdResult>("run_shipctl", {
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
    void publishAction(["confirm"]);
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
    void (async () => {
      const project = projectPath();
      if (!project) return;
      const cur = lastLaunch?.current;
      const needsTerminal =
        Boolean(cur?.run?.length) ||
        cur?.kind === "oauth" ||
        cur?.kind === "paste" ||
        cur?.kind === "sign" ||
        cur?.kind === "deploy";
      if (needsTerminal) {
        const opened = await openShipctlTerminal(
          ["launch", "--project", project, "open"],
          {
            title: "Ship Studio launch",
            meta: "Launched terminal: shipctl launch open — complete the step, then Verify/Confirm here.",
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
      } else {
        void launchAction(["open"]);
      }
    })();
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
        title: "Ship Studio paste",
        meta: "Launched terminal: shipctl human --no-open --put — paste each value when prompted.",
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

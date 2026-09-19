import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openPath, openUrl } from "@tauri-apps/plugin-opener";

type CmdResult = {
  ok: boolean;
  code: number;
  stdout: string;
  stderr: string;
  shipctl: string;
  cancelled?: boolean;
};

type StreamLine = {
  stream: "stdout" | "stderr" | "meta" | string;
  text: string;
};

type ToolStatus = {
  found?: boolean;
  path?: string | null;
  version?: string | null;
};

type Detected = {
  signet_toml?: boolean;
  package_json?: boolean;
  tauri?: boolean;
  wrangler?: boolean;
  vercel?: boolean;
  netlify?: boolean;
  github?: boolean;
  polar?: boolean;
  orbit_configured?: boolean;
  hints?: string[];
};

type DoctorReport = {
  ok?: boolean;
  signet?: ToolStatus;
  orbit?: ToolStatus;
  notes?: string[];
  detected?: Detected;
};

type PortalStep = {
  id?: string;
  provider?: string;
  kind?: string;
  title?: string;
  detail?: string;
  entry_url?: string | null;
  cli?: string[] | null;
  human?: boolean;
};

type PortalPlan = {
  providers?: string[];
  steps?: PortalStep[];
  notes?: string[];
};

type SecretsPlan = {
  hints?: Array<{
    provider?: string;
    name?: string;
    put_cli?: string[];
    entry_url?: string | null;
    detail?: string;
    source?: string;
  }>;
};

type HumanSprint = {
  minutes_hint?: string;
  open_order?: string[];
  put_queue?: Array<{
    provider?: string;
    name?: string;
    put_cli?: string[];
    entry_url?: string | null;
    detail?: string;
  }>;
  checklist?: string[];
};

type LaunchView = {
  current_index?: number;
  total?: number;
  done_count?: number;
  finished?: boolean;
  current?: {
    id?: string;
    title?: string;
    kind?: string;
    detail?: string;
    entry_url?: string | null;
    status?: string;
    verify_hint?: string | null;
    run?: string[] | null;
  } | null;
  steps?: Array<{
    id?: string;
    title?: string;
    status?: string;
    kind?: string;
  }>;
  notes?: string[];
};

type PublishView = {
  mode?: string;
  intent?: string;
  current_index?: number;
  total?: number;
  done_count?: number;
  minutes_remaining?: number;
  minutes_total?: number;
  finished?: boolean;
  current?: {
    id?: string;
    title?: string;
    kind?: string;
    detail?: string;
    entry_url?: string | null;
    status?: string;
    minutes?: number;
    run?: string[] | null;
    desktop_view?: string | null;
  } | null;
  steps?: Array<{
    id?: string;
    title?: string;
    status?: string;
    kind?: string;
    minutes?: number;
  }>;
  notes?: string[];
};

type PulseAction = {
  id?: string;
  label?: string;
  kind?: string;
  view?: string | null;
  cmd?: string[] | null;
};

type ProjectPulse = {
  name?: string;
  kind?: string;
  git?: {
    is_repo?: boolean;
    branch?: string | null;
    dirty?: boolean;
    dirty_count?: number;
    committed?: boolean;
    ahead?: number | null;
    behind?: number | null;
    last_commit?: { hash?: string; subject?: string; when?: string | null } | null;
    notes?: string[];
  };
  publish?: {
    present?: boolean;
    finished?: boolean;
    current_index?: number;
    total?: number;
    done_count?: number;
    current_id?: string | null;
    current_title?: string | null;
    minutes_remaining?: number | null;
  };
  launch?: {
    present?: boolean;
    finished?: boolean;
    current_index?: number;
    total?: number;
    current_title?: string | null;
  };
  deploy?: {
    signal?: string;
    detail?: string;
    urls?: string[];
    last_run_ok?: boolean | null;
  };
  tools?: {
    signet_found?: boolean;
    orbit_found?: boolean;
    signet_version?: string | null;
    orbit_version?: string | null;
  };
  scopes_active?: string[];
  now?: {
    title?: string;
    detail?: string;
    primary?: PulseAction;
    actions?: PulseAction[];
  };
  notes?: string[];
};

let lastLaunch: LaunchView | null = null;
let lastPublish: PublishView | null = null;
let lastPulse: ProjectPulse | null = null;
let lastDetected: Detected | undefined;
let lastHuman: HumanSprint | null = null;
let lastPortal: PortalPlan | null = null;
let lastSecrets: SecretsPlan | null = null;
let portalFilter: string | null = null;

type ShipState = {
  project: string;
  has_ship_dir: boolean;
  studio: {
    sign_args?: string[];
    deploy_args?: string[];
    notes?: string[];
    detected?: Detected;
  } | null;
  last_run: {
    ok?: boolean;
    finished?: boolean;
    message?: string;
    steps?: Array<{ id?: string; ok?: boolean; exit_code?: number; detail?: string }>;
    finished_at?: string;
  } | null;
};

const LAST_PROJECT_KEY = "ship-studio.last-project";
const RECENT_KEY = "ship-studio.recent-projects";
const OFFLINE_KEY = "ship-studio.offline";
const DEPLOY_KEY = "ship-studio.include-deploy";
const MODE_KEY = "ship-studio.mode";
const INTENT_KEY = "ship-studio.intent";
const MAX_RECENT = 6;

type StudioMode = "general" | "advanced";
type ShipIntent = "local" | "public";

function studioMode(): StudioMode {
  return localStorage.getItem(MODE_KEY) === "advanced" ? "advanced" : "general";
}

function shipIntent(): ShipIntent {
  return localStorage.getItem(INTENT_KEY) === "local" ? "local" : "public";
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

const ACTION_IDS = [
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

let running = false;
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
  const el = stateEl();
  if (el) {
    el.textContent = label;
    el.className = busy ? "busy" : failed ? "failed" : "ready";
  }
  // Always re-sync disabled state — Cancel / Clear / errors must unlock Refresh.
  setProjectUi(Boolean(projectPath()));
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
  const el = stateEl();
  if (el) {
    el.textContent = reason;
    el.className = "ready";
  }
  setProjectUi(Boolean(projectPath()));
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
  syncPublishRelated();
  syncBackToPublish();
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

function syncOutputMirror() {
  const mirror = document.querySelector<HTMLPreElement>("#output-focus");
  const out = outputEl();
  if (mirror && out) {
    mirror.textContent = out.textContent ?? "";
    mirror.scrollTop = mirror.scrollHeight;
  }
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

type ToastKind = "ok" | "err" | "info";

function toast(message: string, kind: ToastKind = "info", ms = 3200) {
  const host = document.querySelector<HTMLElement>("#toast-host");
  if (!host || !message.trim()) return;
  const el = document.createElement("button");
  el.type = "button";
  el.className = `toast ${kind}`;
  el.setAttribute("role", "status");
  el.innerHTML = `<span class="toast-mark" aria-hidden="true"></span><p class="toast-msg">${escapeHtml(message)}</p>`;
  const dismiss = () => {
    if (el.dataset.leaving === "1") return;
    el.dataset.leaving = "1";
    window.setTimeout(() => el.remove(), 170);
  };
  el.addEventListener("click", dismiss);
  host.appendChild(el);
  window.setTimeout(dismiss, ms);
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

const VIEW_META: Record<string, { title: string; desc: string }> = {
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

const RELATED_VIEW_LABELS: Record<string, string> = {
  scopes: "Open Scopes",
  env: "Open Env",
  sign: "Open Sign",
  portal: "Open Portal",
  ritual: "Open Ritual",
  tools: "Open Tools",
  launch: "Open Launch",
  dashboard: "Open Dashboard",
};

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

function setView(id: string) {
  if (!VIEW_META[id]) return;
  activeViewId = id;
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
  syncProjectIdentity();
  syncBackToPublish();
  if (id === "output") syncOutputMirror();
}

type CmdItem = {
  id: string;
  title: string;
  keywords: string;
  group: string;
  run: () => void;
};

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

function prettyMaybe(raw: string): string {
  const t = raw.trim();
  if (!t) return "";
  try {
    return JSON.stringify(JSON.parse(t), null, 2);
  } catch {
    return t;
  }
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

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function splitArgs(raw: string): string[] {
  return raw.trim().split(/\s+/).filter(Boolean);
}

function joinArgs(args: string[] | undefined, fallback: string): string {
  if (!args || args.length === 0) return fallback;
  return args.join(" ");
}

function projectName(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

function parentPath(path: string): string {
  const clean = path.replace(/[\\/]+$/, "");
  const parts = clean.split(/[\\/]/);
  if (parts.length < 2) return "";
  return parts.slice(0, -1).join("/") || clean;
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
}

function setCtaLabel(btn: HTMLButtonElement, label: string) {
  const span = btn.querySelector<HTMLElement>(".cta-label");
  if (span) span.textContent = label;
  else btn.textContent = label;
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
      hint.textContent = `Pick up the current step${minsNote}. Confirm when the vendor UI is done.`;
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
    title.textContent = "Live check is done for this pass";
    detail.textContent = `${projectName(path)} finished the publish portal. Switch project or start another pass from Publish.`;
    setNowCtaState("review");
    return;
  }
  const cur = view?.current;
  if (cur?.title) {
    const n = (view?.current_index ?? 0) + 1;
    const total = view?.total ?? 0;
    const mins = view?.minutes_remaining != null ? ` · ~${view.minutes_remaining} min left` : "";
    title.textContent = cur.title;
    detail.textContent = `${cur.detail ?? "Open/Run on the official platform, then Confirm."} (${n}/${total}${mins})`;
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
  if (detail) detail.textContent = pulse.now?.detail ?? "";
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
  const orbit = !!pulse.tools?.orbit_found;
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

  // Mid-wizard / prior Cloudflare·Vercel deploy never hard-block on Orbit.
  const toolsBlocked =
    wantsSignet && (!signet || (!orbit && !linked && !deployOk && !midWizard));

  if (toolsBlocked) {
    return {
      state: "blocked",
      badge: "Blocked",
      title: "Tools missing",
      detail: "Signet and/or Orbit not on PATH — run Doctor before shipping desktop.",
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
  if (linked && !midWizard) {
    return {
      state: "ready",
      badge: "Linked",
      title: "Provider linked",
      detail: pulse.deploy?.detail || "Configured locally — deploy when you need a new release.",
    };
  }
  if (localOnly && !midWizard) {
    return {
      state: "ready",
      badge: "Local only",
      title: "Wrangler local state",
      detail: "Dev/miniflare cache — not proof of a remote Workers deploy.",
    };
  }
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
    return {
      state: "progress",
      badge: "In progress",
      title: pub?.current_title || launch?.current_title || "Wizard in flight",
      detail: pub?.present
        ? `Publish ${(pub.current_index ?? 0) + 1}/${pub.total ?? 0}`
        : `Launch ${(launch?.current_index ?? 0) + 1}/${launch?.total ?? 0}`,
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
  if (signet && orbit) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Tools ready",
      detail: "Signet + Orbit on PATH",
    });
  } else if (!wantsSignet && linked) {
    items.push({
      id: "tools",
      state: "done",
      icon: "✓",
      title: "Worker tooling OK",
      detail: "Prior live deploy evidence — Orbit optional for this stack",
    });
  } else if (!wantsSignet) {
    items.push({
      id: "tools",
      state: "idle",
      icon: "○",
      title: "Orbit optional",
      detail: `${signet ? "Signet ok" : "Signet n/a"} · ${orbit ? "Orbit ok" : "Orbit not required for Workers"}`,
    });
  } else {
    items.push({
      id: "tools",
      state: "blocked",
      icon: "!",
      title: "Tools incomplete",
      detail: `${signet ? "Signet ok" : "Signet missing"} · ${orbit ? "Orbit ok" : "Orbit missing"}`,
    });
  }

  const git = pulse.git;
  if (!git?.is_repo) {
    items.push({
      id: "git",
      state: "idle",
      icon: "○",
      title: "No git repo",
      detail: "Optional — status is local-folder only",
    });
  } else if (git.dirty) {
    items.push({
      id: "git",
      state: "warn",
      icon: "!",
      title: "Uncommitted changes",
      detail: `${git.dirty_count ?? "?"} dirty on ${git.branch ?? "branch"}`,
    });
  } else {
    items.push({
      id: "git",
      state: "done",
      icon: "✓",
      title: "Git clean",
      detail: git.last_commit
        ? `${git.last_commit.hash} — ${git.last_commit.subject}`
        : (git.branch ?? "clean tree"),
    });
  }

  const pub = pulse.publish;
  const launch = pulse.launch;
  if (pub?.present && !pub.finished) {
    items.push({
      id: "ship",
      state: "active",
      icon: "→",
      title: "Publish in progress",
      detail: `${pub.current_title ?? "Step"} · ${(pub.current_index ?? 0) + 1}/${pub.total ?? 0}`,
    });
  } else if (pub?.finished) {
    items.push({
      id: "ship",
      state: "done",
      icon: "✓",
      title: "Publish pass finished",
      detail: "Live check confirmed for this pass",
    });
  } else if (launch?.present && !launch.finished) {
    items.push({
      id: "ship",
      state: "active",
      icon: "→",
      title: "Launch in progress",
      detail: `${launch.current_title ?? "Step"} · ${(launch.current_index ?? 0) + 1}/${launch.total ?? 0}`,
    });
  } else {
    items.push({
      id: "ship",
      state: "idle",
      icon: "○",
      title: "Ship not started",
      detail: "Open Publish when you are ready",
    });
  }

  const stepHint = (
    pub?.current_id ||
    pub?.current_title ||
    launch?.current_title ||
    ""
  ).toLowerCase();
  const pendingSubmission =
    ((pub?.present && !pub.finished) || (launch?.present && !launch.finished)) &&
    (stepHint.includes("list") ||
      stepHint.includes("polar") ||
      stepHint.includes("paste") ||
      stepHint.includes("env") ||
      stepHint.includes("secret") ||
      stepHint.includes("oauth"));

  if (pendingSubmission) {
    items.push({
      id: "submit",
      state: "warn",
      icon: "…",
      title: "Pending submission",
      detail: "Human gate — finish on the official platform, then Confirm",
    });
  } else if (pub?.finished || pulse.deploy?.last_run_ok) {
    items.push({
      id: "submit",
      state: "done",
      icon: "✓",
      title: "Submission clear",
      detail: "No open paste/listing gate in the current plan",
    });
  } else {
    items.push({
      id: "submit",
      state: "idle",
      icon: "○",
      title: "No submission gate yet",
      detail: "Appears when env, listing, or OAuth is the current step",
    });
  }

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
      title: "Already live",
      detail: dep?.urls?.[0] || dep?.detail || "Prior successful deploy — skip redundant ship",
    });
  } else if (dep?.signal === "vercel_linked" || dep?.signal === "orbit_configured") {
    items.push({
      id: "deploy",
      state: "active",
      icon: "◇",
      title: "Provider linked",
      detail: dep.detail || "Configured — deploy when you need a new release",
    });
  } else if (dep?.signal === "wrangler_local") {
    items.push({
      id: "deploy",
      state: "idle",
      icon: "○",
      title: "Local Wrangler only",
      detail: "Dev/miniflare state — not a remote Workers deploy",
    });
  } else {
    items.push({
      id: "deploy",
      state: "idle",
      icon: "○",
      title: "Not deployed yet",
      detail: "No Orbit summary or last-run deploy signal",
    });
  }

  return items;
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
    // Stay on current surface — Tools is a detail panel, not the start.
    void run(["doctor", "--project", projectPath()], { step: "doctor" });
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

type ScopePlan = {
  scopes?: Array<{
    id?: string;
    kind?: string;
    label?: string;
    relative?: string;
    provider?: string | null;
    signals?: string[];
  }>;
  active?: string[];
};

type AssistPlan = {
  sign_path?: string;
  steps?: Array<{
    id?: string;
    title?: string;
    detail?: string;
    view?: string;
    ready?: boolean;
  }>;
};

type EnvPortal = {
  actions?: Array<{
    id?: string;
    kind?: string;
    title?: string;
    detail?: string;
    entry_url?: string | null;
    put_cli?: string[] | null;
    provider?: string | null;
    name?: string | null;
  }>;
};

type SignPortal = {
  recommended?: string;
  paths?: Array<{
    id?: string;
    kind?: string;
    title?: string;
    detail?: string;
    entry_url?: string | null;
    run?: string[] | null;
  }>;
};

async function loadJsonCmd(args: string[], opts?: { silent?: boolean }): Promise<unknown | null> {
  const result = await run(args, { quietHeader: true, silent: opts?.silent !== false });
  if (!result?.ok || !result.stdout) return null;
  try {
    return JSON.parse(result.stdout);
  } catch {
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

function applyScopes(plan: ScopePlan | null) {
  const grid = document.querySelector("#scope-grid");
  if (!grid) return;
  const scopes = plan?.scopes ?? [];
  const active = new Set(plan?.active ?? []);
  grid.innerHTML = scopes.length
    ? scopes
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
        .join("")
    : `<p class="detail empty-hint">No scopes detected.</p>`;
}

async function saveScopes() {
  const ids = Array.from(
    document.querySelectorAll<HTMLInputElement>("[data-scope-id]:checked"),
  ).map((el) => el.dataset.scopeId ?? "");
  if (!ids.length) {
    show("Select at least one scope.");
    toast("Select at least one scope", "err");
    return;
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
  if (result?.ok) {
    if (publishMidFlight()) {
      setView("publish");
      toast("Scopes saved — Confirm on Publish", "ok");
    } else {
      toast("Scopes saved", "ok");
    }
  }
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
      void run(["env", "--project", projectPath(), "--provider", provider, "--put", name]);
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
          return `<li class="portal-step">
            <div class="meta">
              <div class="title"><span class="kind kind-${escapeHtml(kind)}">${escapeHtml(kind)}</span>${escapeHtml(p.title ?? "")}</div>
              <p class="detail">${escapeHtml(p.detail ?? "")}${runCmd ? ` · ${escapeHtml(runCmd)}` : ""}</p>
            </div>
            <div class="btns">
              <button type="button" class="sign-open" data-url="${escapeHtml(url)}" ${url ? "" : "disabled"}>Open vendor</button>
            </div>
          </li>`;
        })
        .join("")
    : `<li class="portal-step"><div class="meta"><p class="detail empty-hint">Load signing paths.</p></div></li>`;
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
      const cli = (s.cli ?? []).join(" ");
      const openDisabled = url ? "" : "disabled";
      const canLogin = s.kind === "oauth" || (s.cli && s.cli.length > 0);
      const loginDisabled = canLogin ? "" : "disabled";
      return `<li class="portal-step" data-idx="${idx}">
        <div class="meta">
          <div class="title"><span class="kind">${escapeHtml(
            s.kind ?? "",
          )}</span>${escapeHtml(s.title ?? s.id ?? "step")}</div>
          <p class="detail">${escapeHtml(s.detail ?? "")}${
            url ? ` · ${escapeHtml(url)}` : cli ? ` · ${escapeHtml(cli)}` : ""
          }</p>
        </div>
        <div class="btns">
          <button type="button" class="portal-open" data-url="${escapeHtml(
            url,
          )}" ${openDisabled}>Open</button>
          <button type="button" class="portal-login" data-provider="${escapeHtml(
            s.provider ?? "",
          )}" ${loginDisabled}>Login CLI</button>
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
  list.querySelectorAll<HTMLButtonElement>(".portal-login").forEach((btn) => {
    btn.addEventListener("click", () => {
      const provider = btn.getAttribute("data-provider");
      if (!provider) return;
      void run([
        "portal",
        "--project",
        projectPath(),
        "--provider",
        provider,
        "--login",
      ]);
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

function applyPublishView(view: PublishView | null) {
  lastPublish = view;
  const currentEl = document.querySelector<HTMLElement>("#publish-current");
  const list = document.querySelector<HTMLElement>("#publish-steps");
  const hint = document.querySelector<HTMLElement>("#publish-hint");
  const mins = document.querySelector<HTMLElement>("#publish-minutes");
  if (!currentEl || !list) return;
  if (!view?.steps?.length) {
    currentEl.innerHTML =
      '<p class="detail empty-hint">Refresh Publish to build the adaptive plan for this repo.</p>';
    list.innerHTML = "";
    if (mins) mins.hidden = true;
    syncPublishRelated();
    syncBackToPublish();
    applyNow(view);
    return;
  }
  setView("publish");
  const cur = view.current;
  if (hint) {
    const modeLabel = studioMode() === "general" ? "General" : "Advanced";
    const intentLabel = shipIntent() === "local" ? "Local" : "Public";
    hint.textContent = view.finished
      ? "Publish workflow finished — live check confirmed."
      : `${modeLabel} · ${intentLabel} · Step ${(view.current_index ?? 0) + 1}/${view.total ?? 0} · ~${view.minutes_remaining ?? 0} min left · ${view.done_count ?? 0} done — Related opens detail panels without leaving the spine.`;
  }
  if (mins) {
    mins.hidden = false;
    mins.textContent = `~${view.minutes_remaining ?? 0} min remaining · ${view.minutes_total ?? 0} min total`;
  }
  currentEl.innerHTML = cur
    ? `<div class="title"><span class="kind">${escapeHtml(cur.kind ?? "")}</span>${escapeHtml(cur.title ?? "")}${
        cur.minutes ? ` · ~${cur.minutes}m` : ""
      }</div>
       <p class="detail">${escapeHtml(cur.detail ?? "")}${
         cur.run?.length ? ` · run: ${escapeHtml(cur.run.join(" "))}` : ""
       }${
         cur.desktop_view && RELATED_VIEW_LABELS[cur.desktop_view]
           ? ` · studio: ${escapeHtml(RELATED_VIEW_LABELS[cur.desktop_view])}`
           : ""
       }</p>`
    : "<p class=\"detail\">No current step</p>";
  list.innerHTML = (view.steps ?? [])
    .map((s, i) => {
      const active = i === view.current_index ? " active-step" : "";
      return `<li class="portal-step${active}">
        <div class="meta">
          <div class="title"><span class="kind">${escapeHtml(s.status ?? "")}</span>${escapeHtml(s.title ?? s.id ?? "")}</div>
        </div>
      </li>`;
    })
    .join("");
  syncPublishRelated();
  syncBackToPublish();
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
    applyPublishView(JSON.parse(result.stdout) as PublishView);
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
      toast(parsed.prompt ?? "Step ready — Confirm then Next", "ok", 6000);
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

async function publishAction(sub: string[]) {
  const ordered =
    sub.length === 0 ? publishArgs() : publishArgs(sub);
  const result = await run(ordered, { step: "paste" });
  if (!result?.stdout) return;
  try {
    const parsed = JSON.parse(result.stdout) as PublishView & {
      publish?: PublishView;
      ok?: boolean;
      message?: string;
    };
    applyPublishView(parsed.publish ?? parsed);
    if (parsed.message) {
      appendStream({ stream: "meta", text: parsed.message });
      toast(parsed.message, parsed.ok === false ? "err" : "ok");
    }
    // Keep Dashboard Now honest after Confirm / Next / Open / Verify.
    await refreshSessionNow();
  } catch {
    /* raw output shown */
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
    ? `<div class="title"><span class="kind">${escapeHtml(cur.kind ?? "")}</span>${escapeHtml(cur.title ?? "")}</div>
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
          <div class="title"><span class="kind">${escapeHtml(s.status ?? "")}</span>${escapeHtml(s.title ?? s.id ?? "")}</div>
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
  const result = await run(args, { step: "portal" });
  if (!result?.ok || !result.stdout) return;
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
    }
  } catch {
    /* plan already in output */
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
    ["github", detected.github],
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
      .map(([label]) => `<span class="chip on">${escapeHtml(label)}</span>`)
      .join("");
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

async function run(args: string[], opts?: { step?: string; quietHeader?: boolean; silent?: boolean }): Promise<CmdResult | undefined> {
  const project = projectPath();
  if (!project) {
    show("Open a project folder first.");
    return;
  }
  if (running) {
    show("Already running — wait for the current command, or Cancel to unlock.");
    toast("Busy — Cancel unlocks Publish if stuck", "info", 4000);
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
    if (!opts?.quietHeader && !opts?.silent) {
      const cmd = args[0] ?? "shipctl";
      if (result.cancelled) toast(`${cmd} cancelled`, "err");
      else if (result.ok) toast(`${cmd} · done`, "ok");
      else toast(`${cmd} failed`, "err");
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
  applyStudioMode(studioMode());
  applyShipIntent(shipIntent());
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
  setView("dashboard");
  setTitle(null);
  wireWindowChrome();

  document.querySelectorAll<HTMLButtonElement>(".nav-item[data-nav]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.dataset.nav;
      if (!id) return;
      setView(id);
      if (id === "publish" && projectPath() && !lastPublish?.steps?.length) {
        void refreshPublish();
      }
    });
  });

  document.querySelector("#btn-back-publish")?.addEventListener("click", () => {
    setView("publish");
    if (!lastPublish?.steps?.length) void refreshPublish();
    else toast("Back on Publish", "info", 1800);
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
      if (running) {
        ev.preventDefault();
        void invoke<boolean>("cancel_shipctl");
      }
      return;
    }
    const ctrl = ev.ctrlKey || ev.metaKey;
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
    const plan = (await loadJsonCmd(["assist", "--project", projectPath()])) as AssistPlan | null;
    applyAssist(plan);
  });
  document.querySelector("#btn-assist-start")?.addEventListener("click", async () => {
    setView("publish");
    const raw = await loadJsonCmd(["assist", "--project", projectPath(), "--start"]);
    if (raw && typeof raw === "object" && "assist" in raw) {
      applyAssist((raw as { assist: AssistPlan }).assist);
    }
    if (raw && typeof raw === "object" && "publish" in raw) {
      applyPublishView((raw as { publish: PublishView }).publish);
    } else {
      document.querySelector<HTMLButtonElement>("#btn-publish")?.click();
    }
  });
  document.querySelector("#btn-scopes")?.addEventListener("click", async () => {
    const plan = (await loadJsonCmd(["scopes", "--project", projectPath()])) as ScopePlan | null;
    applyScopes(plan);
  });
  document.querySelector("#btn-scopes-save")?.addEventListener("click", () => {
    void saveScopes();
  });
  document.querySelector("#btn-env")?.addEventListener("click", async () => {
    const plan = (await loadJsonCmd(["env", "--project", projectPath()])) as EnvPortal | null;
    applyEnv(plan);
  });
  document.querySelector("#btn-sign-paths")?.addEventListener("click", async () => {
    const plan = (await loadJsonCmd(["sign-paths", "--project", projectPath()])) as SignPortal | null;
    applySignPaths(plan);
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
  document.querySelector("#btn-publish-open")?.addEventListener("click", () => {
    void (async () => {
      const project = projectPath();
      if (!project) return;
      const cur = lastPublish?.current;
      const related = (cur?.desktop_view ?? "").trim();
      // Band #16: Open matches Related for every RELATED_VIEW_LABELS target (incl. dashboard).
      const studioDetail = Boolean(related && RELATED_VIEW_LABELS[related]);
      if (studioDetail) {
        await openRelatedStudioView(related);
        toast(`${RELATED_VIEW_LABELS[related] ?? related} — finish, then Confirm`, "info");
      }
      const needsTerminal =
        Boolean(cur?.run?.length) ||
        cur?.kind === "oauth" ||
        cur?.kind === "sign" ||
        cur?.kind === "deploy";
      if (needsTerminal) {
        try {
          await invoke("open_publish_open_terminal", { project });
          appendStream({
            stream: "meta",
            text: "Launched terminal: shipctl publish open — complete the step, then Verify/Confirm here.",
          });
          toast("Terminal opened for this step", "ok");
          window.setTimeout(() => {
            void refreshPublish();
          }, 1500);
        } catch (e) {
          appendStream({
            stream: "stderr",
            text: `open terminal failed: ${String(e)} — falling back to in-app open`,
          });
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
      toast("Watching — local Verify every 15s", "info", 4000);
    } else {
      stopPublishWatch();
    }
  });
  document.querySelector("#btn-publish-confirm")?.addEventListener("click", () => {
    void publishAction(["confirm"]);
  });
  document.querySelector("#btn-publish-next")?.addEventListener("click", () => {
    void publishAction(["next"]);
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
        try {
          await invoke("open_launch_open_terminal", { project });
          appendStream({
            stream: "meta",
            text: "Launched terminal: shipctl launch open — complete the step, then Verify/Confirm here.",
          });
          window.setTimeout(() => {
            void refreshLaunch();
          }, 1500);
        } catch (e) {
          appendStream({
            stream: "stderr",
            text: `open terminal failed: ${String(e)} — falling back to in-app open`,
          });
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
    try {
      await invoke("open_human_put_terminal", { project });
      appendStream({
        stream: "meta",
        text: "Launched terminal: shipctl human --no-open --put — paste each value when prompted.",
      });
      setStep("paste", "done");
    } catch (err) {
      appendStream({ stream: "stderr", text: String(err) });
    }
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
  document.querySelector("#btn-deploy")?.addEventListener("click", () => {
    if (offline()) {
      show("Deploy is blocked while Offline is on.");
      return;
    }
    if (
      !confirm(
        "Run Orbit deploy (network)? Uses deploy_args from .ship/studio.json (or the field above after Save).",
      )
    ) {
      return;
    }
    return run(["deploy", "--project", projectPath()], { step: "deploy" });
  });
  document.querySelector("#btn-flow-dry")?.addEventListener("click", () =>
    run(flowArgs(true)),
  );
  document.querySelector("#btn-flow")?.addEventListener("click", () => {
    if (includeDeploy() && !offline()) {
      if (!confirm("Run full flow including Orbit deploy (network)?")) return;
    }
    return run(flowArgs(false));
  });
  document.querySelector("#btn-status")?.addEventListener("click", () =>
    run(["status", "--project", projectPath()]),
  );
});

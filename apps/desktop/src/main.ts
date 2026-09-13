import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openPath } from "@tauri-apps/plugin-opener";

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
const MAX_RECENT = 6;

const pathEl = () => document.querySelector<HTMLInputElement>("#project-path");
const outputEl = () => document.querySelector<HTMLPreElement>("#output");
const stateEl = () => document.querySelector<HTMLElement>("#run-state");
const offlineEl = () => document.querySelector<HTMLInputElement>("#opt-offline");
const deployEl = () => document.querySelector<HTMLInputElement>("#opt-deploy");
const signArgsEl = () => document.querySelector<HTMLInputElement>("#sign-args");
const deployArgsEl = () => document.querySelector<HTMLInputElement>("#deploy-args");

const ACTION_IDS = [
  "btn-doctor",
  "btn-configure",
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
] as const;

let running = false;
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

function setBusy(busy: boolean, label = "Ready", failed = false) {
  running = busy;
  const el = stateEl();
  if (!el) return;
  el.textContent = label;
  el.className = busy ? "busy" : failed ? "failed" : "ready";
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
  syncDeployToggle();
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

function show(text: string) {
  streamBuf = text;
  const out = outputEl();
  if (out) {
    out.textContent = text;
    out.scrollTop = out.scrollHeight;
  }
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
  for (const id of ["doctor", "configure", "sign", "deploy"]) {
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

async function setTitle(path: string | null) {
  try {
    const win = getCurrentWindow();
    await win.setTitle(path ? `Ship Studio — ${projectName(path)}` : "Ship Studio");
  } catch {
    /* ignore in browser preview */
  }
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
  if (!host) return;
  const current = projectPath();
  const others = list.filter((p) => p !== current);
  if (others.length === 0) {
    host.hidden = true;
    host.innerHTML = "";
    return;
  }
  host.hidden = false;
  host.innerHTML = others
    .map(
      (p) =>
        `<button type="button" data-recent="${escapeHtml(p)}" title="${escapeHtml(p)}">${escapeHtml(projectName(p))}</button>`,
    )
    .join("");
  host.querySelectorAll<HTMLButtonElement>("button[data-recent]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const p = btn.getAttribute("data-recent");
      if (p) void bindProject(p, true);
    });
  });
}

function applyDetected(detected?: Detected) {
  const host = document.querySelector<HTMLElement>("#detect-chips");
  if (!host) return;
  if (!detected) {
    host.hidden = true;
    host.innerHTML = "";
    return;
  }
  const flags: Array<[string, boolean | undefined]> = [
    ["signet.toml", detected.signet_toml],
    ["package.json", detected.package_json],
    ["tauri", detected.tauri],
    ["wrangler", detected.wrangler],
    ["vercel", detected.vercel],
    ["orbit", detected.orbit_configured],
  ];
  host.hidden = false;
  host.innerHTML = flags
    .map(
      ([label, on]) =>
        `<span class="chip ${on ? "on" : "off"}">${escapeHtml(label)}</span>`,
    )
    .join("");
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
}

function applyLastRun(last: ShipState["last_run"]) {
  const meta = document.querySelector("#meta-lastrun");
  if (!last) {
    setPill("pill-lastrun", "muted", "None");
    if (meta) meta.textContent = "No .ship/last-run.json yet";
    return;
  }
  const ok = !!last.ok;
  setPill("pill-lastrun", ok ? "ok" : "bad", ok ? "OK" : "Failed");
  const steps = (last.steps ?? [])
    .map((s) => `${s.id ?? "?"}${s.ok === false ? "✗" : "✓"}`)
    .join(" → ");
  const when = last.finished_at ? ` · ${last.finished_at}` : "";
  if (meta) {
    meta.textContent = `${last.message ?? "last run"}${steps ? ` · ${steps}` : ""}${when}`;
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

async function run(args: string[], opts?: { step?: string; quietHeader?: boolean }) {
  const project = projectPath();
  if (!project) {
    show("Open a project folder first.");
    return;
  }
  if (running) {
    show("Already running — wait for the current command.");
    return;
  }
  if (opts?.step) setStep(opts.step, "active");
  setBusy(true, "Running…");
  setProjectUi(true);
  streamBuf = opts?.quietHeader ? "" : `shipctl ${args[0]}\n`;
  show(streamBuf);

  try {
    const result = await invoke<CmdResult>("run_shipctl", { project, args });
    // Final pretty pass for JSON-heavy commands
    if (args[0] === "doctor" || args[0] === "configure" || args[0] === "status" || args[0] === "flow") {
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
    setBusy(false, result.cancelled ? "Cancelled" : result.ok ? "Ready" : "Failed", !result.ok && !result.cancelled);
    setProjectUi(true);
  } catch (err) {
    appendStream({ stream: "stderr", text: String(err) });
    if (opts?.step) setStep(opts.step, "fail");
    setBusy(false, "Failed", true);
    setProjectUi(true);
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
  saveRecent(path);
  await setTitle(path);
  resetSteps();
  setProjectUi(true);
  show(`Project: ${path}\n`);
  await refreshShipState();
  if (autoDoctor) {
    await run(["doctor", "--project", path], { step: "doctor", quietHeader: true });
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
  renderRecent(loadRecent());
  void refreshShipctlPath();

  void listen<StreamLine>("shipctl-line", (event) => {
    appendStream(event.payload);
  });

  window.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape") {
      if (running) {
        ev.preventDefault();
        void invoke<boolean>("cancel_shipctl");
      }
      return;
    }
    if (isTypingTarget(ev.target)) return;
    const ctrl = ev.ctrlKey || ev.metaKey;
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
    } catch {
      try {
        await openPath(project);
      } catch (err) {
        show(String(err));
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
    } catch (err) {
      show(String(err));
      setBusy(false, "Failed", true);
    }
  });

  document.querySelector("#btn-copy")?.addEventListener("click", async () => {
    try {
      await navigator.clipboard.writeText(streamBuf || outputEl()?.textContent || "");
      setBusy(false, "Copied");
      setTimeout(() => setBusy(false, "Ready"), 800);
    } catch (err) {
      show(String(err));
    }
  });

  document.querySelector("#btn-clear")?.addEventListener("click", () => {
    show("");
    setBusy(false, "Ready");
  });

  document.querySelector("#btn-cancel")?.addEventListener("click", async () => {
    try {
      const killed = await invoke<boolean>("cancel_shipctl");
      if (!killed) {
        appendStream({ stream: "meta", text: "nothing to cancel" });
      }
    } catch (err) {
      appendStream({ stream: "stderr", text: String(err) });
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

  document.querySelector("#btn-doctor")?.addEventListener("click", () =>
    run(["doctor", "--project", projectPath()], { step: "doctor" }),
  );
  document.querySelector("#btn-configure")?.addEventListener("click", () =>
    run(["configure", "--project", projectPath()], { step: "configure" }),
  );
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

/** S0.8 — notice newer GitHub Release; human opens download (no silent install). */

import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";

export const UPDATE_SNOOZE_KEY = "orbit-yard.update-snooze-tag";
export const RELEASES_LATEST_URL = "https://github.com/Dendro-X0/ship-studio/releases/latest";
const GITHUB_LATEST_API =
  "https://api.github.com/repos/Dendro-X0/ship-studio/releases/latest";

export type UpdateCheckResult =
  | { kind: "newer"; local: string; remote: string; htmlUrl: string }
  | { kind: "current"; local: string; remote: string }
  | { kind: "offline" }
  | { kind: "error"; detail: string };

function parseSemver(raw: string): [number, number, number] | null {
  const m = raw.trim().replace(/^v/i, "").match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!m) return null;
  return [Number(m[1]), Number(m[2]), Number(m[3])];
}

/** True when remote tag is strictly newer than local app version. */
export function isRemoteNewer(remoteTag: string, localVersion: string): boolean {
  const remote = parseSemver(remoteTag);
  const local = parseSemver(localVersion);
  if (!remote || !local) return false;
  for (let i = 0; i < 3; i++) {
    if (remote[i] > local[i]) return true;
    if (remote[i] < local[i]) return false;
  }
  return false;
}

export function loadSnoozedTag(): string | null {
  try {
    return localStorage.getItem(UPDATE_SNOOZE_KEY);
  } catch {
    return null;
  }
}

export function snoozeUpdateTag(tag: string): void {
  try {
    localStorage.setItem(UPDATE_SNOOZE_KEY, tag);
  } catch {
    /* ignore */
  }
}

export async function openLatestRelease(): Promise<void> {
  await openUrl(RELEASES_LATEST_URL);
}

export async function fetchUpdateCheck(opts?: {
  offline?: boolean;
}): Promise<UpdateCheckResult> {
  if (opts?.offline) return { kind: "offline" };
  let local: string;
  try {
    local = await getVersion();
  } catch (err) {
    return { kind: "error", detail: String(err).slice(0, 120) };
  }
  try {
    const res = await fetch(GITHUB_LATEST_API, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) {
      return { kind: "error", detail: `GitHub ${res.status}` };
    }
    const body = (await res.json()) as { tag_name?: string; html_url?: string };
    const remote = (body.tag_name ?? "").trim();
    if (!remote) return { kind: "error", detail: "No tag_name on latest release" };
    const htmlUrl = (body.html_url ?? RELEASES_LATEST_URL).trim() || RELEASES_LATEST_URL;
    if (isRemoteNewer(remote, local)) {
      return { kind: "newer", local, remote, htmlUrl };
    }
    return { kind: "current", local, remote };
  } catch (err) {
    return { kind: "error", detail: String(err).slice(0, 120) };
  }
}

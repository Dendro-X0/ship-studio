/** Bundled SVGs in src/public — never a remote icon API. */

import { escapeHtml } from "./util";

export function localIconSrc(file: string): string {
  if (file.startsWith("/")) return file;
  return `/icons/${encodeURIComponent(file)}`;
}

export type LocalIcon = { file: string; tone?: "color" | "ink" };

export function iconImg(file: string, opts?: { tone?: "color" | "ink" }): string {
  const tone = opts?.tone ?? "color";
  return `<img class="nav-ico-img" data-tone="${tone}" src="${escapeHtml(localIconSrc(file))}" alt="" width="16" height="16" />`;
}

const PROVIDER_ICONS: Record<string, LocalIcon> = {
  polar: { file: "Polar_dark.svg" },
  stripe: { file: "stripe.svg" },
  gumroad: { file: "gumroad.svg", tone: "ink" },
  lemon: { file: "lemonsqueezy.svg" },
  paddle: { file: "paddle.svg", tone: "ink" },
  resend: { file: "Resend_dark.svg" },
  orbit: { file: "lighthouse.svg" },
  cloudflare: { file: "cloudflare.svg" },
  vercel: { file: "Vercel_dark.svg" },
  netlify: { file: "netlify.svg" },
  "github-pages": { file: "GitHub_dark.svg" },
  fly: { file: "fly.svg" },
  railway: { file: "Railway_dark.svg" },
  "apple-sign": { file: "Apple_dark.svg" },
  "microsoft-sign": { file: "microsoft.svg" },
  "google-play": { file: "googleplay.svg" },
};

export function providerIcon(id: string): LocalIcon {
  return PROVIDER_ICONS[id] ?? { file: "/file.svg" };
}

export function integrationIcon(id: string): LocalIcon {
  return providerIcon(id);
}

export function integrationIconHtml(id: string): string {
  const icon = providerIcon(id);
  return iconImg(icon.file, { tone: icon.tone });
}

export function providerIconHtml(id: string): string {
  return integrationIconHtml(id);
}

export function scopeIconFile(scope: {
  kind?: string;
  provider?: string | null;
  signals?: string[];
}): string {
  const signals = (scope.signals ?? []).map((s) => s.toLowerCase());
  const kind = (scope.kind ?? "").toLowerCase();
  const provider = (scope.provider ?? "").toLowerCase();
  if (signals.includes("tauri") || kind === "desktop") return "tauri.svg";
  if (signals.includes("docker") || signals.includes("compose") || kind === "container") {
    return "docker.svg";
  }
  if (provider === "vercel" || signals.includes("vercel")) return "Vercel_dark.svg";
  if (provider === "netlify" || signals.includes("netlify")) return "netlify.svg";
  if (provider === "supabase" || signals.includes("supabase")) return "supabase.svg";
  if (signals.includes("appwrite")) return "appwrite.svg";
  if (kind === "api" || signals.includes("wrangler")) return "hono.svg";
  if (signals.includes("rust")) return "Rust_dark.svg";
  if (kind === "docs") return "/file.svg";
  if (signals.includes("node") || kind === "web") return "nodejs.svg";
  return "/file.svg";
}

export const SCOPE_KIND_ORDER = ["api", "web", "desktop", "mobile", "container", "docs", "root"];

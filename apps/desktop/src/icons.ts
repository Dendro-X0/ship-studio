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
  selfhost: { file: "lighthouse.svg" },
  cloudflare: { file: "cloudflare.svg" },
  vercel: { file: "Vercel_dark.svg" },
  netlify: { file: "netlify.svg" },
  "github-pages": { file: "GitHub_dark.svg" },
  fly: { file: "fly.svg" },
  railway: { file: "Railway_dark.svg" },
  "apple-sign": { file: "Apple_dark.svg" },
  "microsoft-sign": { file: "microsoft.svg" },
  "google-play": { file: "googleplay.svg" },
  "github-sign": { file: "GitHub_dark.svg" },
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

export function scopeIcon(scope: {
  kind?: string;
  provider?: string | null;
  signals?: string[];
}): LocalIcon {
  const kind = (scope.kind ?? "").toLowerCase();
  // Surface kinds only — never vendor/framework logos in Targets.
  switch (kind) {
    case "desktop":
      return { file: "scope-desktop.svg", tone: "ink" };
    case "mobile":
      return { file: "scope-mobile.svg", tone: "ink" };
    case "api":
      return { file: "scope-api.svg", tone: "ink" };
    case "container":
      return { file: "scope-container.svg", tone: "ink" };
    case "web":
    case "docs":
      return { file: "scope-web.svg", tone: "ink" };
    default:
      return { file: "scope-web.svg", tone: "ink" };
  }
}

/** @deprecated Prefer scopeIcon(); kept for call sites that only need the file name. */
export function scopeIconFile(scope: {
  kind?: string;
  provider?: string | null;
  signals?: string[];
}): string {
  return scopeIcon(scope).file;
}

export function scopeIconHtml(scope: {
  kind?: string;
  provider?: string | null;
  signals?: string[];
}): string {
  const icon = scopeIcon(scope);
  return iconImg(icon.file, { tone: icon.tone });
}

export const SCOPE_KIND_ORDER = ["api", "web", "desktop", "mobile", "container", "docs", "root"];

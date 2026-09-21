/** Pure helpers for Desktop shell (no DOM side effects except escape). */

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

export function splitArgs(raw: string): string[] {
  return raw.trim().split(/\s+/).filter(Boolean);
}

export function joinArgs(args: string[] | undefined, fallback: string): string {
  if (!args || args.length === 0) return fallback;
  return args.join(" ");
}

export function projectName(path: string): string {
  const parts = path.replace(/[\\/]+$/, "").split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function parentPath(path: string): string {
  const clean = path.replace(/[\\/]+$/, "");
  const parts = clean.split(/[\\/]/);
  if (parts.length < 2) return "";
  return parts.slice(0, -1).join("/") || clean;
}

export function prettyMaybe(raw: string): string {
  const t = raw.trim();
  if (!t) return raw;
  try {
    return JSON.stringify(JSON.parse(t), null, 2);
  } catch {
    return raw;
  }
}

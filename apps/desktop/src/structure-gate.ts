/** Structure proceed gate — layout match before spawn-class CTAs.
 * Spec: specs/backend/structure-proceed-gate-design.md
 */

import type { Detected } from "./types";

export type StructureGateAction = {
  id: string;
  label: string;
  icon?: "confirm" | "verify" | "open" | "continue";
};

export type StructureGate =
  | { ok: true; mode: "hard" | "soft"; summary: string }
  | {
      ok: false;
      mode: "hard" | "soft";
      reason: string;
      actions: StructureGateAction[];
    };

function openProjectAction(): StructureGateAction {
  return { id: "structure-open-project", label: "Open project…", icon: "open" };
}

/** True when the bound folder is the Umami dogfood fixture (or named like it). */
export function pathLooksLikeUmamiLayout(projectPath: string | null | undefined): boolean {
  if (!projectPath) return false;
  const n = projectPath.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  return n.endsWith("/fixtures/umami-analytics") || /(^|\/)umami-analytics$/.test(n);
}

export function umamiFixtureGuessPath(projectRoot: string | null | undefined): string | null {
  if (!projectRoot) return null;
  const sep = projectRoot.includes("\\") ? "\\" : "/";
  const trimmed = projectRoot.replace(/[/\\]+$/, "");
  const norm = trimmed.replace(/\\/g, "/");
  if (pathLooksLikeUmamiLayout(trimmed)) return trimmed;
  if (/\/fixtures$/i.test(norm)) return `${trimmed}${sep}umami-analytics`;
  return `${trimmed}${sep}fixtures${sep}umami-analytics`;
}

/** True when the bound folder is the Flowsint detect fixture (or a Flowsint clone). */
export function pathLooksLikeFlowsintLayout(projectPath: string | null | undefined): boolean {
  if (!projectPath) return false;
  const n = projectPath.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  return (
    n.endsWith("/fixtures/flowsint-osint") ||
    /(^|\/)flowsint-osint$/.test(n) ||
    (n.includes("/flowsint") &&
      !n.includes("ship-studio") &&
      (n.endsWith("/flowsint") || n.includes("reconurge")))
  );
}

export function flowsintFixtureGuessPath(projectRoot: string | null | undefined): string | null {
  if (!projectRoot) return null;
  const sep = projectRoot.includes("\\") ? "\\" : "/";
  const trimmed = projectRoot.replace(/[/\\]+$/, "");
  const norm = trimmed.replace(/\\/g, "/");
  if (pathLooksLikeFlowsintLayout(trimmed)) return trimmed;
  if (/\/fixtures$/i.test(norm)) return `${trimmed}${sep}flowsint-osint`;
  return `${trimmed}${sep}fixtures${sep}flowsint-osint`;
}

/** True when the bound folder is the Medusa detect fixture (or a Medusa app). */
export function pathLooksLikeMedusaLayout(projectPath: string | null | undefined): boolean {
  if (!projectPath) return false;
  const n = projectPath.replace(/\\/g, "/").replace(/\/+$/, "").toLowerCase();
  return (
    n.endsWith("/fixtures/medusa-commerce") ||
    /(^|\/)medusa-commerce$/.test(n) ||
    (n.includes("/medusa") &&
      !n.includes("ship-studio") &&
      (n.endsWith("/medusa") || n.includes("medusajs")))
  );
}

export function medusaFixtureGuessPath(projectRoot: string | null | undefined): string | null {
  if (!projectRoot) return null;
  const sep = projectRoot.includes("\\") ? "\\" : "/";
  const trimmed = projectRoot.replace(/[/\\]+$/, "");
  const norm = trimmed.replace(/\\/g, "/");
  if (pathLooksLikeMedusaLayout(trimmed)) return trimmed;
  if (/\/fixtures$/i.test(norm)) return `${trimmed}${sep}medusa-commerce`;
  return `${trimmed}${sep}fixtures${sep}medusa-commerce`;
}

/** Evaluate bound-folder layout vs Deployment / Ritual / Sign lane. */
export function evaluateStructureGate(
  laneId: string,
  detected: Detected | null | undefined,
  opts?: { projectPath?: string | null },
): StructureGate | null {
  const d = detected ?? null;
  const open = openProjectAction();

  if (laneId === "umami") {
    if (d?.umami || pathLooksLikeUmamiLayout(opts?.projectPath)) {
      return {
        ok: true,
        mode: "hard",
        summary: d?.umami
          ? "Umami layout detected — Deploy may run compose."
          : "Umami fixture path bound — Deploy may run compose.",
      };
    }
    const actions: StructureGateAction[] = [open];
    const fixture = umamiFixtureGuessPath(opts?.projectPath ?? null);
    if (fixture && !pathLooksLikeUmamiLayout(opts?.projectPath)) {
      actions.push({
        id: "structure-bind-fixture",
        label: "Bind fixture",
        icon: "continue",
      });
    }
    const monorepo =
      d?.studio_monorepo && !pathLooksLikeUmamiLayout(opts?.projectPath)
        ? " Bound folder looks like the Orbit Yard monorepo root."
        : "";
    return {
      ok: false,
      mode: "hard",
      reason: `Structure check failed — not an Umami layout.${monorepo} Bind the folder with the Umami compose file (or .ship/layout = analytics.umami).`,
      actions,
    };
  }

  if (laneId === "flowsint") {
    if (d?.flowsint || pathLooksLikeFlowsintLayout(opts?.projectPath)) {
      return {
        ok: true,
        mode: "hard",
        summary: d?.flowsint
          ? "Flowsint layout detected — Deploy may run compose.prod."
          : "Flowsint path bound — Deploy may run compose.prod.",
      };
    }
    const actions: StructureGateAction[] = [open];
    const fixture = flowsintFixtureGuessPath(opts?.projectPath ?? null);
    if (fixture && !pathLooksLikeFlowsintLayout(opts?.projectPath)) {
      actions.push({
        id: "structure-bind-fixture",
        label: "Bind fixture",
        icon: "continue",
      });
    }
    const monorepo =
      d?.studio_monorepo && !pathLooksLikeFlowsintLayout(opts?.projectPath)
        ? " Bound folder looks like the Orbit Yard monorepo root."
        : "";
    return {
      ok: false,
      mode: "hard",
      reason: `Structure check failed — not a Flowsint layout.${monorepo} Bind a Flowsint clone (or .ship/layout = osint.flowsint / fixtures/flowsint-osint).`,
      actions,
    };
  }

  if (laneId === "medusa") {
    if (d?.medusa || pathLooksLikeMedusaLayout(opts?.projectPath)) {
      return {
        ok: true,
        mode: "hard",
        summary: d?.medusa
          ? "Medusa layout detected — Deploy may run compose + Admin health."
          : "Medusa path bound — Deploy may run compose + Admin health.",
      };
    }
    const actions: StructureGateAction[] = [open];
    const fixture = medusaFixtureGuessPath(opts?.projectPath ?? null);
    if (fixture && !pathLooksLikeMedusaLayout(opts?.projectPath)) {
      actions.push({
        id: "structure-bind-fixture",
        label: "Bind fixture",
        icon: "continue",
      });
    }
    const monorepo =
      d?.studio_monorepo && !pathLooksLikeMedusaLayout(opts?.projectPath)
        ? " Bound folder looks like the Orbit Yard monorepo root."
        : "";
    return {
      ok: false,
      mode: "hard",
      reason: `Structure check failed — not a Medusa layout.${monorepo} Bind a Medusa app (medusa-config / @medusajs/*) or .ship/layout = commerce.medusa / fixtures/medusa-commerce.`,
      actions,
    };
  }

  if (laneId === "selfhost") {
    if (d?.selfhost) {
      return { ok: true, mode: "hard", summary: "Self-host static surface detected." };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — no Self-host static surface (index.html under public/dist/apps/website, or .ship/selfhost.json).",
      actions: [open],
    };
  }

  if (laneId === "orbit") {
    if (d?.yard_configured) {
      return { ok: true, mode: "soft", summary: "Orbit-configured — Ritual/Tools deploy path available." };
    }
    return {
      ok: false,
      mode: "soft",
      reason:
        "Structure check — this repo is not Orbit-configured (no orbit.toml / .orbit). Use Self-host or a cloud host card for Deploy.",
      actions: [open],
    };
  }

  if (laneId === "cloudflare") {
    if (d?.wrangler || d?.selfhost) {
      return {
        ok: true,
        mode: "hard",
        summary: d.wrangler
          ? "Cloudflare / wrangler surface detected."
          : "Static surface detected — Cloudflare Pages lane may apply.",
      };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — no wrangler.toml and no static index (apps/website, public, dist). Bind the app folder or add Workers/Pages config.",
      actions: [open],
    };
  }

  if (laneId === "vercel") {
    if (d?.vercel || d?.selfhost) {
      return {
        ok: true,
        mode: "hard",
        summary: d.vercel
          ? "Vercel project markers detected."
          : "Static surface detected — Vercel Deploy may apply.",
      };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — no vercel.json / .vercel and no static index. Bind the app folder or add Vercel config.",
      actions: [open],
    };
  }

  if (laneId === "netlify") {
    if (d?.netlify || d?.selfhost) {
      return {
        ok: true,
        mode: "hard",
        summary: d.netlify
          ? "Netlify config detected."
          : "Static surface detected — Netlify Deploy may apply.",
      };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — no netlify.toml and no static publish directory. Bind the site folder or add Netlify config.",
      actions: [open],
    };
  }

  if (laneId === "fly") {
    if (d?.fly) {
      return { ok: true, mode: "hard", summary: "fly.toml detected — Deploy may run after fly launch/link." };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — no fly.toml. Run `fly launch` in the app directory (human), then Retry Deploy.",
      actions: [open],
    };
  }

  if (laneId === "railway") {
    if (d?.railway) {
      return { ok: true, mode: "hard", summary: "Railway link detected — Deploy may run after railway link." };
    }
    return {
      ok: false,
      mode: "hard",
      reason:
        "Structure check failed — not linked to Railway (no .railway). Run `railway link` or `railway init` (human), then Retry Deploy.",
      actions: [open],
    };
  }

  if (laneId === "github-pages") {
    const pages =
      Boolean(d?.marketing_site && (d.marketing_host === "pages" || !d.marketing_host)) ||
      d?.marketing_host === "pages";
    if (pages || d?.github) {
      return { ok: true, mode: "soft", summary: "GitHub / Pages surface noted — Open settings when ready." };
    }
    return {
      ok: false,
      mode: "soft",
      reason:
        "Structure check — no Pages marketing surface detected. Open GitHub settings is still allowed if you know the repo.",
      actions: [open],
    };
  }

  if (laneId === "apple-sign" || laneId === "microsoft-sign") {
    if (d?.tauri) {
      return { ok: true, mode: "soft", summary: "Desktop (Tauri) cut detected — official signing may apply." };
    }
    return {
      ok: false,
      mode: "soft",
      reason:
        "Structure check — no Tauri desktop cut detected. Opening the vendor dashboard is still allowed.",
      actions: [],
    };
  }

  if (laneId === "google-play") {
    return {
      ok: true,
      mode: "soft",
      summary: "Play Console — open when mobile is in scope (Studio does not upload).",
    };
  }

  if (laneId === "github-sign") {
    if (d?.github) {
      return { ok: true, mode: "soft", summary: "GitHub remote noted — Releases stay on GitHub." };
    }
    return {
      ok: false,
      mode: "soft",
      reason: "Structure check — no GitHub remote detected. Open GitHub is still allowed to create a repo.",
      actions: [],
    };
  }

  if (laneId === "ritual-orbit-deploy") {
    if (d?.yard_configured) {
      return { ok: true, mode: "soft", summary: "Orbit-configured — Ritual deploy may run." };
    }
    return {
      ok: false,
      mode: "soft",
      reason:
        "Structure check — not Orbit-configured. Prefer Deployment → cloud host or Self-host instead of Ritual Orbit deploy.",
      actions: [open],
    };
  }

  return null;
}

/** Lanes that hard-block Deploy / spawn CTAs. */
export function isHardStructureLane(laneId: string): boolean {
  return (
    laneId === "umami" ||
    laneId === "flowsint" ||
    laneId === "medusa" ||
    laneId === "selfhost" ||
    laneId === "cloudflare" ||
    laneId === "vercel" ||
    laneId === "netlify" ||
    laneId === "fly" ||
    laneId === "railway"
  );
}

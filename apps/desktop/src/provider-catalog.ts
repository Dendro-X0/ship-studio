/** Shared catalog grid · sidebar · wizard for Integrations / Platforms. */

import type { ProviderWizard } from "./types";
import { escapeHtml } from "./util";

export function renderProviderCatalogGrid(opts: {
  host: HTMLElement;
  entries: ProviderWizard[];
  groups: readonly string[];
  selectedId: string | null;
  iconHtml: (id: string) => string;
  isLocked?: (entry: ProviderWizard) => boolean;
  onSelect: (id: string, locked: boolean) => void;
  preferGroup?: string | null;
}): void {
  const { host, entries, groups, selectedId, iconHtml, isLocked, onSelect, preferGroup } =
    opts;
  const ordered = preferGroup
    ? [...groups].sort((a, b) => (a === preferGroup ? -1 : b === preferGroup ? 1 : 0))
    : [...groups];
  host.innerHTML = ordered
    .map((group) => {
      const cards = entries
        .filter((w) => w.group === group)
        .map((w) => {
          const locked = Boolean(isLocked?.(w));
          return `<button type="button" class="int-card${selectedId === w.id ? " is-active" : ""}" data-prov="${escapeHtml(w.id)}" ${locked ? 'data-locked="true"' : ""}>
            ${iconHtml(w.id)}
            <span class="int-card-title">${escapeHtml(w.title)}</span>
            <span class="int-card-blurb">${escapeHtml(w.blurb)}</span>
          </button>`;
        })
        .join("");
      return `<section class="int-group"><h2>${escapeHtml(group)}</h2><div class="int-grid">${cards}</div></section>`;
    })
    .join("");
  host.querySelectorAll<HTMLButtonElement>("[data-prov]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.getAttribute("data-prov");
      if (!id) return;
      onSelect(id, btn.dataset.locked === "true");
    });
  });
}

export function renderProviderSidebarTree(opts: {
  host: HTMLElement;
  entries: ProviderWizard[];
  groups: readonly string[];
  selectedId: string | null;
  activeView: boolean;
  iconHtml: (id: string) => string;
  dataAttr: string;
  onSelect: (id: string) => void;
}): void {
  const { host, entries, groups, selectedId, activeView, iconHtml, dataAttr, onSelect } = opts;
  host.innerHTML = groups
    .map((group) => {
      const rows = entries
        .filter((w) => w.group === group)
        .map(
          (w) =>
            `<button type="button" class="nav-target${selectedId === w.id && activeView ? " is-on" : ""}" ${dataAttr}="${escapeHtml(w.id)}">
              ${iconHtml(w.id)}
              <span class="nav-target-name">${escapeHtml(w.title)}</span>
            </button>`,
        )
        .join("");
      return `<p class="nav-kind">${escapeHtml(group)}</p>${rows}`;
    })
    .join("");
  host.querySelectorAll<HTMLButtonElement>(`[${dataAttr}]`).forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.getAttribute(dataAttr);
      if (id) onSelect(id);
    });
  });
}

export function paintProviderWizard(opts: {
  entry: ProviderWizard | null;
  panel: HTMLElement | null;
  titleEl: HTMLElement | null;
  blurbEl: HTMLElement | null;
  stepsEl: HTMLElement | null;
  openBtn?: HTMLButtonElement | null;
  secondaryBtn?: HTMLButtonElement | null;
  secondaryVisible?: boolean;
  docsBtn?: HTMLButtonElement | null;
  /** When set, shown for Tier A put hosts (Cloudflare / Vercel / Netlify). */
  putBtn?: HTMLButtonElement | null;
  putVisible?: boolean;
}): void {
  const {
    entry,
    panel,
    titleEl,
    blurbEl,
    stepsEl,
    openBtn,
    secondaryBtn,
    secondaryVisible,
    docsBtn,
    putBtn,
    putVisible,
  } = opts;
  if (!panel) return;
  if (!entry) {
    panel.hidden = true;
    return;
  }
  panel.hidden = false;
  if (titleEl) titleEl.textContent = entry.title;
  if (blurbEl) blurbEl.textContent = entry.blurb;
  if (stepsEl) {
    stepsEl.innerHTML = entry.steps.map((s) => `<li>${escapeHtml(s)}</li>`).join("");
  }
  if (openBtn) openBtn.textContent = entry.openLabel ?? "Open dashboard";
  if (secondaryBtn) secondaryBtn.hidden = !secondaryVisible;
  if (docsBtn) {
    docsBtn.hidden = !entry.docsUrl;
    docsBtn.textContent = "Learn more";
  }
  const showPut = Boolean(putVisible);
  if (putBtn) {
    putBtn.hidden = !showPut;
    putBtn.classList.toggle("primary", showPut);
  }
  if (openBtn) {
    openBtn.classList.toggle("primary", !showPut);
  }
}

export function highlightProviderSidebar(
  root: string,
  dataAttr: string,
  selectedId: string | null,
  active: boolean,
): void {
  document.querySelectorAll<HTMLButtonElement>(`${root} [${dataAttr}]`).forEach((btn) => {
    const on = active && btn.getAttribute(dataAttr) === selectedId;
    btn.classList.toggle("is-on", on);
  });
}

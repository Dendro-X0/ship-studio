/** Shared catalog grid · wizard for Integrations / Deployment / Sign. */

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
  openLinksEl?: HTMLElement | null;
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
    openLinksEl,
  } = opts;
  if (!panel) return;
  if (!entry) {
    panel.hidden = true;
    if (openLinksEl) {
      openLinksEl.hidden = true;
      openLinksEl.innerHTML = "";
    }
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
  if (openLinksEl) {
    const links = entry.openLinks ?? [];
    openLinksEl.hidden = links.length === 0;
    openLinksEl.innerHTML = links
      .map(
        (l) =>
          `<button type="button" data-open-url="${escapeHtml(l.url)}">${escapeHtml(l.label)}</button>`,
      )
      .join("");
  }
}

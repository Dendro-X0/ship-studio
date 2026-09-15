# Marketing deploy lane — gap #9

**Status:** Done (2026-09-15) — first slice  
**Parent:** `release-surface-map.md` gap #9  
**Owner:** `config::probe`, `publish`, `pulse`  
**Updated:** 2026-09-15  

## Problem

Professional launches stall on the public surface (Obscur domain cutover, Signet HOOK Pages, Velocity saas-web, Codactrl landing). Orbit already deploys app/API targets; operators get no Publish cue for the **marketing / download / docs landing** host.

## Scope (first slice)

### Detection

Set `detected.marketing_site` when any of:

| Signal | Notes |
|--------|-------|
| Dir | `apps/website`, `website`, `apps/marketing`, `marketing` |
| GitHub Pages | Root `CNAME`, `.nojekyll`, or workflow filename contains `pages` / `gh-pages` |
| Signet-style preview | `docs/launch/preview/index.html` |
| Opt-in | `.ship/markets` / `markets.json` contains `marketing` / `site` / `pages` / `hook` |

### Advanced publish

`marketing.deploy` (Human → Confirm):

- **Detail:** Deploy or cut over the public landing (download / limitations / HOOK). Confirm when the canonical URL serves the current build. Does not claim DNS ownership.
- **entry_url** (best effort, offline):
  1. If Pages signals → `{github_repo}/settings/pages` or `https://docs.github.com/pages`
  2. Else if project has Vercel markers → `https://vercel.com/dashboard`
  3. Else if Netlify → `https://app.netlify.com/`
  4. Else → `https://vercel.com/dashboard`
- **desktop_view:** `portal`

General omits. Pulse note when detected.

## Non-goals

- DNS / registrar automation  
- Binding `obscur.app`-class domains from the bridge  
- Replacing Orbit `deploy.*` for the primary app  
- Building the marketing site in-repo  

## Proof

| Layer | Check |
|-------|-------|
| L1 | Fixture `apps/website/index.html` or markets `marketing` → `marketing_site` |
| L2 | Advanced has `marketing.deploy`; General omits |
| L3 | Related → Portal; Open host dashboard |

## Follow-ups

- Gap #10 graduate signing + commerce SKU  
- Optional: inject canonical URL into sibling suite env (Velocity ↔ Strata)  

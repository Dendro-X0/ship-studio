# Structure proceed gate — UX guarantee

**Status:** Active — product law (Desktop + CLI align)  
**Updated:** 2026-10-09  
**Owner:** Desktop proceed CTAs · `orbityard` detect / hostdeploy / umami / selfhost  
**Parents:** [PRODUCT.md](../../docs/product/PRODUCT.md) · [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md) · [umami-get-live-design](./umami-get-live-design.md) · [host-xor-value-ux-polish-design](./host-xor-value-ux-polish-design.md)  
**Supersedes:** [deploy-structure-gate-design](./deploy-structure-gate-design.md) (redirect)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Structure proceed gate — every spawn-class CTA checks layout first
PAUSED / CANCELLED:  Integrations GTM · fake one-click · auto-rebind without human
CANONICAL OWNER:     apps/desktop/src/structure-gate.ts + this spec
PROOF BEFORE DONE:   Wrong bind → banner + toast · no CLI theater · reason names fix
```

## 1. Guarantee (operator-facing)

> **Before Studio runs a project-shaped action (Deploy / compose / hostdeploy / Orbit deploy), it validates that the bound folder matches the chosen lane. If not, the action does not start — the UI names the gap and how to fix it (rebind or add the missing layout).**

This is a **product UX contract**, not a single-card feature. Open-dashboard / vendor bookmarks may soft-warn; they do not hard-block browsing.

## 2. Classes

| Class | Behavior | Surfaces |
|-------|----------|----------|
| **Hard** | Banner · disable primary · assert on click · no spawn | Umami Deploy · Self-host Deploy · Cloudflare / Vercel / Netlify / Fly / Railway Deploy |
| **Soft** | Banner or toast · action still allowed | GitHub Pages Open · Orbit docs · Sign Open · Ritual Orbit deploy (`!yard_configured`) · Import env / Put host mismatch |
| **Out of scope** | Intent / dirty / auth gates stay as today | `needsPublic` · Offline · Login CLI · Publish Confirm · Integrations soft-no |

## 3. Hard rules (detect)

| Lane | Pass when (Desktop `Detected` / probe) |
|------|----------------------------------------|
| Umami | `umami` |
| Flowsint | `flowsint` |
| Medusa | `medusa` |
| Self-host | `selfhost` |
| Cloudflare | `wrangler` **or** `selfhost` (Pages static lane) |
| Vercel | `vercel` **or** `selfhost` |
| Netlify | `netlify` **or** `selfhost` |
| Fly | `fly` (`fly.toml`) |
| Railway | `railway` (`.railway` linked) |

CLI remains fail-closed at runtime; Desktop gate must not be *looser* than CLI for Fly/Railway/Umami/Flowsint/Medusa/Self-host. Cloudflare/Vercel/Netlify accept static via `selfhost` so monorepo static cuts are not false-blocked.

## 4. UX pattern (mandatory)

1. **Paint** — structure status on the wizard when a hard/soft lane is selected.  
2. **Assert** — on hard proceed: quiet `doctor` refresh → re-evaluate → toast + return if fail.  
3. **Copy** — start with `Structure check failed —` · name the missing layout · one fix (Open project… / Bind fixture when applicable).  
4. **Never** start compose/hostdeploy/Orbit deploy after a hard fail.

## 5. Soft rules

| Lane | When | Copy intent |
|------|------|-------------|
| GitHub Pages | `!marketing_site && marketing_host ≠ pages` | No Pages surface detected — Open still allowed |
| Orbit card | `!yard_configured` | Not Orbit-configured — docs Open allowed; Ritual deploy soft-warns |
| Sign Open | lane vs `tauri` / mobile / `github` mismatch | Soft banner only |
| Import env / Put | selected host ≠ preferred put host | Soft toast; do not block if Put CLI exists |

## 6. Non-goals

- Auto-scanning sibling folders to rebind without confirm  
- Init pack wizards (Umami Later)  
- Hard-blocking vendor **Open dashboard**  
- Replacing auth / Offline / Public-intent gates  

## 7. Proof

| Layer | Check |
|-------|--------|
| L1 | Spec linked from PRODUCT + Platforms |
| L2 | Monorepo root + Umami → blocked; fixture bind → ok |
| L2 | Folder without `fly.toml` + Fly Deploy → blocked before stream |
| L3 | Soft: Pages Open still works with banner |

## 8. Next atomic

**Shipped** — `structure-gate.ts` · Deploy/Sign banners · hard assert on spawn · soft Ritual/Pages/Import · PRODUCT + Platforms links.

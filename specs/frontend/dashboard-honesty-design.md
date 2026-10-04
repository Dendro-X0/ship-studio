# Dashboard honesty polish — S1.2

**Status:** Slice 1 shipped (L1 tsc)  
**Updated:** 2026-09-28  
**Parent:** [frontend-spec](../../docs/frontend/frontend-spec.md) · pulse / status bar · [never-say](./never-say-block-design.md)  
**Owner:** `apps/desktop` Dashboard (`#status-bar` · `#health`)

## Problem

Operators on Dashboard can miss **Signet / Orbit / dirty git / step N/M**, or read **false completion**:

| Failure | Today |
|---------|--------|
| Health tiles Advanced-only | General never sees Signet/Orbit/Git/Deploy pills |
| Orbit missing soft | Often omitted when Signet is present |
| “Shipped” pill | `last-run.json` ok ≠ product all shipped |
| Publish finished row | Claims “Live check confirmed” even on Local cuts |
| Checklist cap | `slice(0, 3)` can drop dirty or tools under mid-flight |

## Goal

Dashboard answers honestly:

1. **Tools** — Signet ok / missing; Orbit ok / missing (soft when Local)  
2. **Git** — dirty count when uncommitted  
3. **Progress** — Publish/Launch **step N/M** + current title when mid-flight  
4. **Deploy** — prior hosted evidence ≠ “all shipped”

```text
Keep:     pulse-driven status bar · Now CTA · mid-flight over prior deploy
Change:   General sees health tiles · no “Shipped” · Orbit missing soft cue · honest finished copy
```

## Non-goals

- New pulse fields or orbityard JSON  
- Fake green “all shipped” aggregate  
- Vendor HTTPS status probes  
- Moving Now / workflow cards

## Slice 1

| Change | Detail |
|--------|--------|
| Health visible in General | Drop `advanced-only` from `#health` |
| Pill copy | `Shipped` → `Last run ok` |
| Checklist tools | Always surface Signet/Orbit when bound; Orbit missing = soft warn on Public |
| Checklist capacity | Allow up to **4** rows so dirty + step N/M coexist |
| Finished publish | “Publish pass finished” — no automatic live-check claim |
| Overall when deployed + dirty | Detail mentions dirty tree (checklist still warns) |

## Proof

| Layer | Check |
|-------|--------|
| L1 | `pnpm --filter orbit-yard-desktop build` (tsc) |
| L2 | Harbor bind · Local: Signet/Orbit/Git pills visible in General · status bar shows tools |
| L2 | Mid-flight: step `N/M` in checklist · no “Shipped” / “all shipped” |

## Later

- Clickable status rows → Tools / Publish / Targets  
- Persist collapsed health  

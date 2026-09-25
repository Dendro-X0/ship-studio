# GIF-ready polish — Dashboard · Publish chrome

**Status:** Implement (pre–S1.1 T1)  
**Updated:** 2026-09-24  
**Parents:** public-publish-ux · demo-subject · status-probe

## Goal

First public GIF must not look like a noisy ops dashboard. One composition: **where you are → what to do next**.

## Defects (from Dashboard screenshot)

1. **Overall vs checklist fight** — headline “Already deployed” while checklist shows “Publish in progress” (classifyOverall prefers `deployOk` over mid-wizard).  
2. **Six equal cards** — tools · git · ship · submit · deploy + headline = soup; Orbit missing screams failure on Local cuts.  
3. **Stage rail** still dumps later chips (slice B pending).  
4. **Sidebar recents** read as empty inputs, not project chips.

## Changes

| Area | Spec |
|------|------|
| `classifyOverall` | Mid-wizard / publish present → **In progress** before “Already deployed” |
| Checklist | ≤3 items: tools (only if actionable), git (if dirty), ship (if mid/finished). Drop idle submit/deploy when ship already tells the story |
| Tools (Local) | Signet ok + Local intent → “Signet ready · Orbit optional” (not blocked) |
| Rail (slice B) | Required-now only (≤6) + `+N later` chip that switches to List |
| Chrome | Recents as chips; status headline quieter; Launch stays the visual primary |

## Proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Bind Harbor mid-Publish → headline In progress, ≤3 checklist tiles, rail short |

## Non-goals

- Full nav collapse (slice F)  
- Light theme · redesign brand mark  

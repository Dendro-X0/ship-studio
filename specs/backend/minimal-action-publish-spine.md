# Minimal-action publish spine

**Status:** Implemented  
**Product principle:** Wide capability through few actions; Publish is the integrated workflow.

## Changes (desktop)

1. Honor `current.desktop_view` — **Related** opens Scopes/Env/Sign/Portal/Ritual/Tools/Launch without abandoning the pass.
2. Topbar **Back to Publish** while `.ship/publish.json` mid-flight and view ≠ publish.
3. `publishAction` / Open refresh `pulse` so Dashboard Now matches the stepper.
4. Nav → Publish auto-`refreshPublish` when no plan loaded.
5. Doctor from pulse stays on current surface (no forced Tools jump).
6. Scopes Save returns to Publish when mid-flight.
7. Assist/Launch copy demoted to overview / companion.

## Proof

- L1: `npx tsc --noEmit` in `apps/desktop`
- L3: Start publish → Related (e.g. Scopes) → Save → Back to Publish → Confirm → Next; Dashboard Now updates

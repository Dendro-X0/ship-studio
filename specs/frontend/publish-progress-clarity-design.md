# Publish progress clarity — done · required · optional

**Status:** Slice 1 implemented (Desktop) · v0.2.3+  
**Parent:** PRODUCT “minutes path” + Continue · operator reflection 2026-09-23  
**Owner:** `apps/desktop` Publish + Dashboard · existing `PublishView` / pulse JSON (no new shipctl fields for slice 1)

## Goal

Operators should **set up and ship in minutes**, always seeing:

1. **Done** — what’s already attested / auto-finished  
2. **Required next** — must Open / Confirm (or Continue when Auto) before the plan can honestly advance  
3. **Optional / later** — upcoming Auto/Sign work (and Advanced lanes) that isn’t the current honesty gate  

UI stays intuitive: one spine, Continue primary, no fake “all shipped.”

## Non-goals

- Auto-Confirm Human / OAuth / deploy / list gates  
- New publish plan semantics or secret storage  
- Replacing the step list with a kanban / Gantt  
- Forcing React/shadcn migration

## Derivation (from existing step JSON)

| Band | Rule (Desktop render) |
|------|------------------------|
| **Done** | `status` ∈ {done, skipped} |
| **Required now** | Current step if pending/done-awaiting-next · plus any pending step where `kind` ∈ {human, oauth, deploy, list, check} **or** Sign live-release (`sign` + `release` without `dry`) |
| **Optional / later** | Remaining pending steps that are **not** required-now (upcoming Auto/Sign dry work). Label **Later** in General · **Advanced lanes** in Advanced. |

Summary line (always visible above current step):

```text
{done} done · {required} required · {optional} later · ~{minutes_remaining} min
```

When `finished`: `All required gates done` (honest — never invent “live on every market”).

## Slice 1 — Desktop presentation only

1. **`#publish-progress`** strip under the minutes line — counts as above.  
2. **Grouped `#publish-steps`:** Done (collapsed by default) · Required · Later. Current step stays expanded in Required.  
3. **Dashboard Now detail** — reuse the same three counts when `pulse.publish.present` (parse from publish JSON already loaded, or add optional `progress` object in a later shipctl slice).  
4. Soften Confirm/Next visual weight (keep enabled rules) so Continue remains the obvious primary.

## Later slices

- shipctl `publish` view emits `progress: { done, required, optional, minutes_* }` for CLI/TUI/MCP parity  
- First-bind empty state: one sentence “Open a folder → Continue”  
- Watch auto-Continue for Auto-only (fast-path later)

## Proof

| Layer | Check | Result |
|-------|--------|--------|
| L1 | desktop `tsc` | Pass |
| L2 | Advanced Local fixture — scopes (+ honesty gates) in **required**; later = upcoming Auto / Advanced lanes | Pass 2026-09-23 |
| L2 | General Local — later = upcoming Auto after current (not empty when mid-flight) | Pass |
| L2 | Finished copy = “All required gates done” (no marketplace live claim) | Spec + UI |
| L2 | Continue honesty unchanged (unit + prior L2) | Unchanged |

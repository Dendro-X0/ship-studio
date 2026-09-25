# Publish journey clarity — one verb, one green button

**Status:** Slices 1–3 shipped (working tree) — outcome panel still later  
**Updated:** 2026-09-25  
**Parents:** public-publish-ux · workflow-stages · gif-ready-polish  
**Product overview:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)

## Problem (operator words)

Clicking **Next** repeatedly; unsure what happened; no sense of a portal or guided path; no intuitive “is it live?” (URL / provider dashboard). Feels like disconnected checkpoints, not a journey.

## Diagnosis (from Harbor dogfood screenshots)

| Defect | Effect |
|--------|--------|
| **Two Next buttons** | Primary green “Next” + disabled tertiary “Next” — dead clicks feel like the app is broken |
| **Confirm vs Next vs Continue** | Three advance verbs; strangers mash Next |
| **Stale guideline after Done** | Card still says “Choose scopes…” while status is DONE |
| **Workflow cards mid-flight** | Dashboard still offers “Start a workflow” while Continue publishing is live |
| **Toast pile** | Busy + Confirm + Open Dashboard while mid Continue |
| **No outcome beat** | Deploy “NO SIGNAL” never becomes “Open live URL” on Local Harbor (honest) — but we never say *what success looks like for this intent* |

Honesty stays: Studio does not fake hosted deploys. The journey must still **feel continuous**.

## North star

```text
One screen · one job · one green verb
  Pending human  → Confirm & continue
  Pending auto   → Continue
  Checkpoint done → Continue (never a second Next)
  Finished Local → “Local cut ready” + Open Sign paths
  Finished Public with URL → “Open live URL”
```

**Portal** = sequential checkpoints with memory, not a browser tab.  
**Verify ready** = status probe + (when Public) Open provider / live URL — never a fake green check.

## Slice 3 — paced Continue (this pass)

Auto gates must **not** jump 2→7 in one flash. Desktop Continue walks **one gate at a time**:

```text
Continue → verify/confirm one Auto step → show checkpoint (~2s dwell + marker)
         → next Auto … until Human gate or finished
```

| Element | Spec |
|---------|------|
| Pace | `--chain 1` loop; ~1800ms dwell (≤200ms if `prefers-reduced-motion`) |
| Marker | Stage panel `data-pacing`; toast “Checking «title»…” per step |
| Scrubber | Full-plan progress bar under Stages toggle — click any segment to review |
| Confirm | After Confirm, start paced Continue (not silent `--chain 8`) |

## Slice 2 — finish honesty (shipped)

| Defect | Fix |
|--------|-----|
| Confirm Live check with Deploy NO SIGNAL | `confirm_current` requires Verify; desktop-only Public fails until live URL or **Switch to Local** |
| “Press Continue” after finished | Finish copy explains done vs no hosted URL |
| Deploy card no CTA | **Use Local** chip when no signal |

## Slice 1 (shipped)

1. Hide tertiary **Next** while focus is on the **current** gate (primary owns advance).  
2. Primary label when status done: **Continue** (not Next).  
3. Guideline respects Done vs Pending.  
4. Hide **Start a workflow** while publish mid-flight.  
5. After Confirm toast: point at the green Continue — no “Continue or Next”.

## Later slices

| Slice | Change |
|-------|--------|
| 4 | Outcome panel: Local vs Public success copy + Open URL when `deploy.urls` |
| 5 | Quiet toasts during paced Continue (one sticky status line) |

## Proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Harbor Scopes → Confirm → paced Auto checks (~2s each) with scrubber; no 2→7 flash |

## Non-goals

- Fake live URL for Local Harbor  
- Removing Confirm honesty  
- Redesigning the whole shell in one pass  

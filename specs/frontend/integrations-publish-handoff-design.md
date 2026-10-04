# Integrations → Publish handoff — design (S1.10)

**Status:** Shipped (L1 tsc · orbityard listing desktop_view)  
**Updated:** 2026-09-27  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.10 · [provider-wizard-setup-design](./provider-wizard-setup-design.md)

```text
GOAL: Continue publishing from an Integrations wizard lands on Publish with the
      matching listing gate named (and focused) — not a vague “back on Publish”.
NOT: Auto-Confirm · Studio writing secrets · inventing listing steps when undetected.
```

## Behavior

| From | Preferred Publish step | Toast |
|------|------------------------|--------|
| Polar | `listing.polar` | Confirm when current · else “advance until …” · focus list/stages row |
| Stripe / Gumroad / Lemon / Paddle | `listing.{id}` | same |
| Resend | _(none)_ | Back on Publish — Confirm env / notify when ready |
| Step missing (General / not detected) | — | Open Publish; say listing appears in Advanced + Public when detected |

Shared helper also used by **Deployment** / **Sign** Continue publishing (prefer `live_check` / sign-ish current gate when present; else generic return toast).

## orbityard

Commerce `listing.polar|stripe|gumroad|lemon|paddle|creem|waffo` `desktop_view`: **`integrations`** (was `portal`) so Publish Related opens the Integrations catalog.

## Non-goals

- S1.11 auto-advance Confirm after Open  
- S1.12 Verify criteria depth  
- Seeking Publish current via orbityard (no seek API — focus UI + honest toast only)

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apps/desktop` `tsc` · `cargo test -p orbityard` listing desktop_view if touched |
| L2 | Harbor Advanced+Public with Polar detect → Integrations → Continue publishing names `listing.polar` |

# Publish wizard reliability — design

**Status:** Design → implement (demo-blocking)  
**Evidence:** Desktop Next on Pending doctor → `current step 'doctor' is still pending`; Dashboard “local already deployed” mid Advanced/Public  
**Owners:** `crates/shipctl/src/publish.rs` · `pulse.rs` · `apps/desktop` Publish UI

## Problem

The spine is honest (`Confirm` attests → `Next` advances), but the UI invites the wrong action:

1. Ready Auto steps say “Confirm and continue” while **Next** is the only `primary` button and stays enabled on Pending.  
2. Tools/`bindProject` doctor can look healthy without closing Publish’s `doctor` gate.  
3. Dashboard pulse appends local-deploy wording during Public mid-flight and can show a different N/M than the portal (General vs Advanced plan length).

## Invariants (keep)

- `next` without `--force` refuses Pending (attestation stays real).  
- Desktop Next never sends `--force` by default.  
- Vendors + human still finish Open/Run irreversible work.

## Changes

### A. Desktop — gate-aware CTAs (`applyPublishView`)

| Current status | Confirm | Next | Primary class |
|----------------|---------|------|---------------|
| Pending | enabled | **disabled** | Confirm |
| Done (current) | disabled / muted | enabled | Next |
| Finished plan | both muted | — | — |

- Clear Failed badge on successful Confirm/Next; map pending-next bail to a toast: “Confirm this step first (or Verify).”  
- Watch-ready still outlines Confirm (already).

### B. shipctl — doctor Auto ready at plan build

When `doctor.ok` at `build_plan_for`, create the doctor step as **Done** (with verified_at) *or* keep Pending but change detail to “Confirm this gate, then Next” — prefer **Done when ok** so step 1 does not trap demos; still require Confirm for Human/Open gates.

Decision: **Mark Auto `doctor` Done when `doctor.ok` at plan insert.** Detail: “Tools ready for this layout.”

### C. Pulse — no local-deploy aside on Public mid-publish

In `decide_now` mid-flight branch: only append “Local provider state already looks deployed” when `intent == Local` (or current step is deploy*).

### D. Copy / naming (Desktop)

- Show kind badge as `auto` → keep id; title already “Doctor”.  
- Publish hint always from `view.mode` / `view.intent` / `view.total` (not chrome-only).

## Out of scope

- Wizard rewrite / React  
- Auto `--force` Next  
- Collapsing Advanced 20 → General 7 in Public demos (operator chooses mode)

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p shipctl` publish_* ; `tsc` desktop |
| L2 | Bind ship-studio or fixture · Advanced · doctor ok → current Done or Confirm primary / Next disabled · Confirm then Next advances without FAILED |
| L2 | Public mid-flight Dashboard does not say “local already looks deployed” unless Local intent |

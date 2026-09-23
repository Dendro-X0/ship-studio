# Publish fast path — minimal interaction

**Status:** Slice 1 + L2 fixture proof · shipped in **v0.2.3**  
**Parent:** PRODUCT “few deliberate actions, one spine” · honesty gates stay  
**Owner:** `crates/shipctl` publish · `apps/desktop` Publish toolbar

## Goal

Get operators from bind → mid-flight → done with **as few clicks as possible**, without fake one-click deploy or silent OAuth.

```text
Accelerate Auto / already-proven gates.
Stop at Human / Open / irreversible vendor work.
Scripts + Watch can run the same continue loop.
```

## Non-goals (product contract)

- Auto Confirm for OAuth / listing / store submit / live deploy  
- `--force` Next as the happy path  
- Hiding Confirm forever (attestation remains for Human gates)  
- Replacing vendor UIs

## Interaction model

| Situation | One primary action | What it does |
|-----------|-------------------|--------------|
| Current **Auto** (or tools already ok) **Pending** | **Continue** | Verify (best-effort) → Confirm → Next |
| Current **Done** | **Continue** | Next |
| Current **Human / Open / OAuth / Deploy** Pending | **Open** (or Confirm after return) | Does not auto-Confirm |
| Plan finished | Done | Pulse shows live / next work |

Secondary: explicit Confirm · Next · Verify remain for power users.

## Slice 1 (shipped)

1. **`shipctl publish continue`** — one smart advance (Confirm+Next for Auto/Done path; refuse auto-Confirm on Human*).  
2. **Desktop primary `Continue`** — calls `continue --chain 12`; Confirm/Next stay visible but secondary. Human pending → Open.  
3. **`--chain N`** — keep Continue while steps are Auto and Verify ok, stop at Human.  
4. **Docs / scripts** — `scripts/publish-fast.sh|.ps1` · dogfood one-liner.  
5. **Desktop first-run** — unset intent → **Local** (General mode already default).

## Later slices

- Watch auto-Continue for Auto only  
- Persist Local as project default when no `studio.json` (CLI) without breaking Public dogfoods  

## Proof

| Layer | Check | Result |
|-------|--------|--------|
| L1 | `cargo test -p shipctl continue_` · desktop `tsc` | Pass |
| L2 | `fixtures/advanced-dogfood` General Local · Continue on scopes → `human_gate` advanced=0 | Pass 2026-09-23 |
| L2 | Confirm scopes → Continue finishes Auto spine (configure · dry_run) | Pass |
| L2 | `scripts/publish-fast.sh` same human stop | Pass |
| L2 | Unit: Human pending · Continue does not mark Done | Pass (`continue_stops_at_human_pending_without_confirm`) |

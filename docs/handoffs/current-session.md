# Current session — Ship Studio

**Updated:** 2026-09-18  
**Branch:** `main` (local may be ahead of `origin/main`)  
**Status:** Continuity restored after interrupted band #23

## Next Atomic Step

**Live Desktop L3** — Open/Run Adaptive Publish (Advanced) on a real Signet subject (e.g. aperio desktop cut), confirm cut order + no Play/Android false positives, record evidence.

Do **not** invent a new Adaptive lane unless a real ship needs it.

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| Mobile store API upload | Deferred — Confirm + vendor UI only |
| k8s controllers | Out of scope (gap backlog F.15) |

## Last closed

| Band | Commit / proof |
|------|----------------|
| #21–#22 | `b32c634` — archetypes + assess-api multi-surface |
| #23 | `d26c728` — Tauri desktop drops Play/Android lanes; L1 `cargo test -p shipctl` (signpath + publish) |

## Canonical owners

| Concern | Owner |
|---------|--------|
| Adaptive plan / Runs | `crates/shipctl/src/publish.rs` |
| Sign / submit paths | `crates/shipctl/src/signpath.rs` |
| Band queue | `specs/backend/shipping-hub-north-star.md` |
| Gap matrix | `specs/backend/release-surface-map.md` |
| Human gates | `docs/OPERATOR-NEXT.md` |

## Proof layers (when claiming done)

| Layer | Command / action |
|-------|------------------|
| L1 | `cargo test -p shipctl` |
| L2 | `bash scripts/dogfood-advanced-publish.sh` |
| L3 | Desktop Advanced Publish Open/Run on Signet subject |
| L4 | Operator Confirm on vendor UIs (`OPERATOR-NEXT.md`) — not Studio bugs |

## Boot allowlist

1. This file (Next Atomic Step)  
2. `docs/START-HERE.md` → SCOPE → PRODUCT  
3. `specs/backend/shipping-hub-north-star.md` · `release-surface-map.md`  
4. `docs/OPERATOR-NEXT.md` for human remaining work  

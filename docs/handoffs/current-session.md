# Current session — Ship Studio

**Updated:** 2026-09-23  
**Branch:** `main`  
**Status:** **v0.2.3** cut — Publish Continue (L1+L2)

## Next Atomic Step

**S1.1:** Record live silent demo GIFs per [demo SCRIPT](../assets/demo/v0.2.1/SCRIPT.md) on the **v0.2.3 installer** (Continue primary). Then deploy site → **S0.8** in-app update check.

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| Polar paid checkout E2E | Deferred — org `payment_ready`; checkout link open = Studio done |
| aperio Advanced L4 (public cut) | Parked — personal use |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |

## Last closed

| Band | Commit / link |
|------|--------|
| **v0.2.3** Publish Continue fast path | [publish-fast-path-design](../../specs/frontend/publish-fast-path-design.md) · L2 fixture proof |
| **v0.2.2** Confirm/Next + doctor Done | [v0.2.2](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.2) |
| **v0.2.1** demo-ready Release | [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) |

## Boot allowlist

1. This file  
2. [docs/CURRENT.md](../CURRENT.md) · [product/improvement-backlog.md](../product/improvement-backlog.md)  
3. [../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
4. `pnpm website:build` · `cargo test -p shipctl polar_`  

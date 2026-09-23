# Current session — Ship Studio

**Updated:** 2026-09-22  
**Branch:** `main`  
**Status:** **v0.2.1** — GitHub Release published (demo-ready installer)

## Next Atomic Step

**S1.1:** Record live silent demo GIFs on the **v0.2.1 installer** (bind → Open → Confirm → Next → Output). Then deploy site + Polar Success/Return URLs → design **S0.8 in-app update check**.

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
| Desktop nav hot-path cut (audit) | `e8cfaa9` |
| **v0.2.0** NSIS + portable | [v0.2.0](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.0) |
| Silent spawn + nav paint / mirror / sidecar prefer | (this cut) |
| **v0.2.1** demo-ready Release | [v0.2.1](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.2.1) |

## Boot allowlist

1. This file  
2. [docs/CURRENT.md](../CURRENT.md) · [product/improvement-backlog.md](../product/improvement-backlog.md)  
3. [../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
4. `pnpm website:build` · `cargo test -p shipctl polar_`  

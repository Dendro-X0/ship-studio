# Current session — Ship Studio

**Updated:** 2026-09-22  
**Branch:** `main`  
**Status:** **v0.2.0 Unreleased** — version bump for installer + in-app update check

## Next Atomic Step

**Desktop nav L2:** Dogfood aperio (or large monorepo) — spam Dashboard ↔ Publish ↔ Env; confirm chrome responds without multi-second freeze ([audit](../../specs/frontend/desktop-nav-performance-audit.md)). Then design **S0.7 Windows installer** + **S0.8 in-app update check** (specs before impl).

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
| Desktop shell boot + Stripe icon + Polar portal URLs + S0.4/S0.5 | `28f696e` |
| CI doctor gate (Signet only if signet.toml) | `503add0` |
| **S0.3** Windows portable Release | [v0.1.0](https://github.com/Dendro-X0/ship-studio/releases/tag/v0.1.0) |
| Desktop nav hot-path cut (audit) | `e8cfaa9` · [audit](../../specs/frontend/desktop-nav-performance-audit.md) |

## Boot allowlist

1. This file  
2. [docs/CURRENT.md](../CURRENT.md) · [product/improvement-backlog.md](../product/improvement-backlog.md)  
3. [../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
4. `pnpm website:build` · `cargo test -p shipctl polar_`  

# Current session — Ship Studio

**Updated:** 2026-09-22  
**Branch:** `main`  
**Status:** **v0.1.0 release week** — fix CI doctor smoke, then S0.3 Release

## Next Atomic Step

**CI:** green `shipctl test` (doctor on `fixtures/ci-smoke`) → then **S0.3** GitHub Release + SHA256.

## PAUSED / CANCELLED

| Band | Rule |
|------|------|
| Polar paid checkout E2E | Deferred — org `payment_ready`; checkout link open = Studio done |
| aperio Advanced L4 (public cut) | Parked — personal use |
| Mobile store API upload | Deferred |
| k8s controllers | Out of scope |

## Last closed

| Band | Commit |
|------|--------|
| Desktop Integrations (payment + email wizards) | `c54b298` |
| Desktop sidebar + local icons + modularization M1 | `374e04e` |
| Desktop shell boot + Stripe icon + Polar portal URLs + S0.4/S0.5 | `28f696e` |

## Boot allowlist

1. This file  
2. [docs/CURRENT.md](../CURRENT.md) · [product/improvement-backlog.md](../product/improvement-backlog.md)  
3. [../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
4. `pnpm website:build` · `cargo test -p shipctl polar_`  

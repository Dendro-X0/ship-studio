# Current session — Ship Studio

**Updated:** 2026-09-22  
**Branch:** `main`  
**Status:** **v0.1.0 release week** — Polar Pay now deferred (seller KYC)

## Objective (this week)

Stranger-facing v0.1.0: understand product → `/demo` live GIFs → Download release → Buy when Polar unlocks.  
**Not required:** Polar paid charge proof · video · PAUSED bands.

## Next Atomic Step

**S0.4 + S0.5:** `/pricing` paid delta ($29 vs OSS) + hero brand disambiguation.

Then: **S0.3** GitHub Release + SHA256 → **S1.1** live demo GIFs → deploy site + Polar Success/Return URLs.

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
| Desktop shell boot + Stripe icon slot + Polar portal URLs | uncommitted → this release-week commit |

## Boot allowlist

1. This file  
2. [docs/CURRENT.md](../CURRENT.md) · [product/improvement-backlog.md](../product/improvement-backlog.md)  
3. [../specs/backend/product-website-charter.md](../../specs/backend/product-website-charter.md)  
4. `pnpm website:build` · `cargo test -p shipctl polar_`  

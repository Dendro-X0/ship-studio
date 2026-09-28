# Never-say block — site + README (S1.3)

**Status:** Shipped (audit + caption fix)  
**Updated:** 2026-09-28  
**Parent:** [product-website-charter](../backend/product-website-charter.md) · [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · backlog S1.3  
**Surfaces:** `apps/website` · root `README.md` · `apps/website/README.md` · `/demo` captions

## Goal

Public marketing must not oversell Studio into vendor replacement or silent success theater.

## Forbidden claims (never say)

| Ban | Why |
|-----|-----|
| **“One-click deploy”** (or “one click” that means hosted cut finishes alone) | Hosted Deploy still needs auth, Confirm, and vendor ownership |
| **“We silence SmartScreen”** / skip OS trust UI | Studio does not suppress OS security |
| **“Replaces Cloudflare / Polar / Apple / …”** | Studio sequences; vendors remain authority |
| **Studio writes / holds deploy secrets** | Put stays on vendor / operator CLI config |
| **Auto-publish to npm / crates / stores** | Human Confirm on irreversible lanes |
| **Studio finishes OAuth for you** | Open / Login CLI only |

## Allowed adjacent wording

| OK | Not OK |
|----|--------|
| “Deploy streams wrangler on this machine” | “One-click Deploy to Cloudflare” |
| “Studio does not replace Cloudflare, Apple, or Polar” | “All-in-one Cloudflare alternative” |
| “Studio never writes secrets” | “We manage your API tokens” |
| “One click starts the publish workflow; you confirm each step” (Desktop spine) | “One click ships production” |

## Audit (2026-09-28)

| Surface | Result |
|---------|--------|
| `/` hero | Pass — explicit non-replace |
| `/pricing` | Pass — never writes secrets · never auto-publishes · never replaces vendor UIs |
| Root `README.md` | Pass — no banned claims |
| `apps/website/README.md` | Pass — ops notes only |
| `/demo` T7 caption | **Fixed** — dropped “One click:” lead-in |
| SCRIPT T7 on-page caption | **Synced** with `/demo` |

## Proof

| Layer | Check |
|-------|--------|
| L1 | Grep site + READMEs for ban phrases — clean after T7 fix |
| L2 | Spot-read `/` · `/pricing` · `/demo` captions |

## Maintenance

Before shipping new marketing copy: re-run the ban table against `apps/website/src` and both READMEs. Specs/design docs may name the ban as a non-goal; that is not a customer claim.

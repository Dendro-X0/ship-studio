# Integrations-as-setup is not the product

**Status:** Maintainer evidence — Polar succeeded **only** in Polar’s dashboard  
**Updated:** 2026-10-04  
**Owner:** product contract (SCOPE / PRODUCT / Integrations)  
**Parents:** [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md) · [solo-commerce-without-paddle](./solo-commerce-without-paddle-design.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Idle — no Integrations expansion
PAUSED / CANCELLED:  Portal/Integrations UX expansion; vendor coach; Paddle Solo dogfood
CANONICAL OWNER:     product thesis (not Desktop wizard copy)
PROOF:               maintainer Polar: official dashboard; Studio unused
```

## Claim that failed (cognitive flaw)

“Orbit Yard helps users **complete** third-party integrations more easily than the vendor.”

That service is **not implementable** at the layer vendors lock:

| Reality | Why a local portal cannot absorb it |
|---------|-------------------------------------|
| Walled garden dashboards | KYC, domain approval, OAuth, catalog, payouts live **only** there |
| Misleading vendor docs / MCP prompt packs | A second checklist (Studio or Paddle MCP) is another encyclopedia, not a setup |
| Operational limits | Studio must not hold secrets, finish OAuth, or auto-Confirm live |
| Provider lock-in | Convenience *is* the vendor’s product; they will not expose a one-shot “done” API for the hard gates |

Paddle made this obvious. Polar made it honest: **setup worked in Polar’s UI; Ship Studio / Orbit Yard was not in the path.** That is not a Studio bug to patch with more Opens. It is the working model.

## What that does *not* invalidate

Orbit Yard as a **release sequencer** for *this repo’s* cut:

- Detect layout → ordered Publish (sign → tag/release → deploy)
- Script what is actually scriptable (`doctor`, local Signet, `gh` list, dry-run)
- **One current gate** + pause so agents do not invent “OAuth done”
- Put **env names** on a host you already authenticated
- Honest Confirm: human attests; vendor remains authority

That job is “don’t get lost across 30 repos and eight nav items.” It is **not** “make Polar/Paddle/Netlify onboarding pleasant.” Confusing the two produced Integrations theater and the Paddle week.

## Product split (keep)

| Job | Viable? |
|-----|---------|
| Complete Polar / Paddle / Stripe **account + catalog + checkout** | **No** — vendor dashboard only. Studio must not claim this. |
| After *you* created a checkout URL / token: bind it to **this** site’s env and Confirm listing | **Thin yes** — Put NAME + Open host; you already did Polar without us |
| Sequence **your** OSS/desktop/web release | **Yes** — this is the product if anything is |

## VALUE thesis correction

Old: earn price by making **commerce gates finishable without surviving vendor encyclopedias**.

That sentence **overclaims**. Vendor encyclopedias *are* the setup. Studio cannot delete them.

New: earn price by making **this project’s cut order** finishable (what’s next, what is scriptable, what you must do in the official UI) — **without pretending** Integrations replaced Polar.

## Do not do next

- Grow Integrations wizards, vendor coach, or Paddle/Polar “setup MCP”
- Treat unused Polar success as a Studio dogfood gap
- Dual thesis (“portal” in PRODUCT.md vs “finish commerce without encyclopedias” in SCOPE)

Integrations stays a **optional deep link sheet** (PAUSED expansion). Polar/Paddle cards are bookmarks, not a product.

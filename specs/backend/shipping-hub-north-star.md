# Shipping hub — north star

**Status:** Active  
**Updated:** 2026-09-15  
**Owner:** product + `shipctl` publish / pulse / doctor  

## Mission

Ship Studio is a **local shipping hub for the final mile** of the product lifecycle:

1. **Sign** — self-sign (Signet) and graduate (OV / Azure Trusted Signing / notarization)  
2. **Release** — checksums, TRUST honesty, GitHub Release / Signet release  
3. **Deploy** — Orbit for Web/API; desktop cut is Signet release (+ optional marketing host)

Operators should rarely miss a gate. The hub sequences Open/Run → human vendor work when required → Confirm → Next.

## Explicitly not the hub’s job

- Product documentation sites, feature demos, GIF authoring, narrative marketing copy  
- Replacing Cloudflare / Vercel / Apple / Play / Gumroad UIs  
- Storing secret values in `.ship/`  
- Finishing OAuth, store review, or DNS without the human  

Developers still create docs and demos elsewhere; Ship Studio cuts the ship.

## Architecture (unchanged spine)

Detect → Adaptive Publish plan → Open or Run local CLI (Signet / Orbit / `gh`) → Confirm → Next.  
Detail panels (Scopes, Env, Sign, Portal, …) open from the current step — not a second wizard.

## Band queue

| # | Band | Status |
|---|------|--------|
| 1–10 | Release-surface Adaptive lanes | Done (first slices) |
| **11** | **Final-mile run depth** — real Signet graduate/release runs, cut order, desktop-deploy honesty, doctor readiness | Done (first slice) |
| **12** | **Suite URL sync** after marketing.deploy | Done (first slice) |
| 13 | Container build/push · mobile store API upload | Deferred / mostly non-goal |

## Band #11 acceptance

- `sign.graduate` has a Signet `run` (not only docs URL) when graduate opted in  
- Advanced Signet cut order: build → trust.pack → graduate? → release_dry → release  
- Desktop-only projects get an explicit “Signet release is the deploy” cue (no fake Orbit desktop deploy)  
- Doctor notes answer “can I cut?” (Signet / Orbit / graduate opt-in)  
- Pulse still prefers mid-publish Continue over unrelated tool noise  

## Proof

`cargo test -p shipctl` · `bash scripts/dogfood-advanced-publish.sh` · Desktop Advanced Open/Run on a Signet subject (L3 optional)

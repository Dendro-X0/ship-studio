# Personal / local ship intent — band #24

**Status:** Done (first slice, 2026-09-19)  
**Updated:** 2026-09-19  
**Parent:** `shipping-hub-north-star.md` · `studio-modes-design.md` · aperio personal-use dogfood  
**Owner:** `publish`, `config` / `.ship/studio.json`, Desktop mode chrome  

## Problem

General vs Advanced only controls **density**. It does not capture **intent**:

| Reality | What Studio did |
|---------|-----------------|
| Operator uses the product privately (e.g. aperio) | Advanced (and even General) still sequences hosted env puts, live deploy, store-shaped honesty gates |
| Marketing / public demo is a different job | Studio has no way to say “local cut only” without Confirm-skipping noise |

Ship Studio’s scope already excludes marketing/demos. The plan should **subtract** public gates when intent is local — not teach operators to ignore them.

## Goal

Add an explicit **ship intent** that filters the Adaptive Publish plan:

| Intent | Meaning |
|--------|---------|
| **`local`** | Sign → release locally (Signet build / desktop cut). No hosted env sprint, no live `deploy.*` / `live_check`, no Advanced store/listing/submit/OAuth unless already opted via markets. |
| **`public`** (default for Advanced; optional for General) | Current behavior — full hosted final-mile when signals match. |

Intent is **orthogonal** to General/Advanced:

- General + local → shortest personal spine  
- Advanced + local → richer local honesty (trust.pack, graduate, ci list) without Play/MAS noise unless mobile/desktop store signals apply  
- General + public → short hosted path (today’s General)  
- Advanced + public → today’s Advanced  

## Detection / persistence

1. **Explicit:** `.ship/studio.json` → `"ship_intent": "local" | "public"`  
2. **CLI:** `shipctl publish --intent local|public` (and `configure` / Desktop toggle)  
3. **Desktop:** topbar or Publish hint control next to General/Advanced — label **Local** / **Public**  
4. **No silent auto-local** from “empty secrets” alone (too magical). Optional *hint* in Pulse/Doctor: “No Vercel link + empty env keys — consider Local intent.”

## Plan filter (`local`)

**Omit** (even if Advanced):

- `oauth.*`
- `env.sprint` (unless `.ship/markets` or studio flag `env_required: true`)
- `deploy.*` / `live_check`
- `listing.*` / `submit.*` / `marketing.deploy` / `suite.url_sync`
- `db.provision`

**Keep** when detected:

- `doctor`, `scopes` (if multi-surface)
- `configure`
- `legal.baseline` / `trust.pack` when files missing (honesty for any cut that might leave the machine)
- Signet path: `sign.self.build` (+ Advanced local: scan / release_dry / release / graduate / `ship.desktop_cut` / `ci.release` read-only)
- `dry_run` when it only plans local CLIs

**Copy:** step details and Pulse should say “Local intent — hosted deploy skipped” so Confirm is not the skip mechanism.

## Non-goals

- Marketing / demo GIF authoring  
- Auto-creating Vercel projects  
- Storing secret values  
- Replacing General/Advanced  
- Mobile store API upload  

## Related honesty fix (same band or #24b)

Env/secrets `entry_url` for Vercel-bound names must not open Cloudflare token pages (aperio dogfood). Provider-matched URLs only.

## Acceptance

- Aperio (or Tauri+Vercel fixture) with `--intent local` has **no** `env.sprint` / `deploy.*` / `submit.*`  
- Same project `--intent public` Advanced still gets hosted steps when signals match  
- `.ship/studio.json` round-trips intent; Desktop can set it  
- Unit: local filter matrix · L2: `shipctl publish --intent local --project fixtures/…`  
- Docs: PRODUCT + OPERATOR-NEXT note Local vs Public  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl` (publish intent filter) |
| L2 | publish JSON on aperio `--intent local` vs `public` |
| L3 | Desktop Local toggle rebuilds Publish list |

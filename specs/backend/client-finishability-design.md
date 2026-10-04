# Client finishability — feasible improvement plan

**Status:** Active · S1+S2 shipped  
**Updated:** 2026-10-04  
**Owner:** `shipctl` detect · secrets · portal · publish  
**Parents:** [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · [release capability audit](../../docs/handoffs/current-session.md) · [close-cut](./close-cut-design.md)  
**Trigger:** Maintainer: implement a feasible Client-first plan for website deploy + payments + launch without pretending vendors disappear.

```text
HANDOFF ATOMIC STEP: Idle after finishability S1+S2 — S3 host bind is maintainer choice
ACTIVE BAND / SCOPE:   Finishable Client sequencing (subtraction + detect + unbound gate)
PAUSED / CANCELLED:    Vendor coach CANCELLED · Portal UX *expansion* PAUSED · Paddle Solo dogfood PAUSED
FORBIDDEN THIS TASK:   Coach theater · CDP · Desktop Integrations catalog growth · auto-finish OAuth
CANONICAL OWNER:       crates/shipctl (config · secrets · portal · publish)
PROOF BEFORE DONE:     L1 cargo test · L2 shipctl secrets/portal/publish on apps/website
SURFACE:               Kernel first (Client inherits); no Desktop wizard growth
```

## Goal

Make the **Client pathway finishable** for founder-owned accounts:

1. Correct detection (so Publish/Env show the real gates).  
2. Stop flooding every vendor when nothing is detected (obstacle-course).  
3. Later: honest “choose host” when unbound — not a fake Orbit deploy.

**Not the goal:** eliminate the CLI kernel; replace Paddle/Cloudflare UIs; zero learning of vendor dashboards.

## Feasibility vs freeze

| Band | Action |
|------|--------|
| Portal / Integrations **UX expansion** | Stays **PAUSED** — this plan subtracts noise; does not add wizards |
| Vendor handoff **coach** | Stays **CANCELLED** |
| Paddle Solo **dogfood** | Stays **PAUSED** — detect `PUBLIC_PADDLE_*` only; no checkout E2E |
| Detect / secret diet / publish honesty | **Allowed** — kernel correctness |

## Evidence (why)

L2 on `apps/website` (2026-10-04):

| Symptom | Cause |
|---------|--------|
| `paddle: false` with `.env` `PUBLIC_PADDLE_*` | Prefix matcher only looks for `PADDLE_` |
| `marketing_site: false` when bound to `apps/website` | Detector looks for *child* `apps/website/`, not folder basename |
| 14 secret hints / full portal catalog | Empty `detected_providers` → fallback `ProviderId::all()` |
| Publish still plans Orbit `deploy` | No primary host / wrangler / vercel — unbound |

## Slices

### S1 — Detect honesty + catalog diet (this band)

| Change | Detail |
|--------|--------|
| Paddle detect | Treat `PUBLIC_PADDLE_` (and `PADDLE_`) env keys as paddle |
| Marketing detect | If project dir name is `website` / `marketing` / `landing`, or `astro.config.*` present → `marketing_site` |
| Secrets diet | If `detected_providers` empty and no filter → **empty hints** + note to Platforms / markets opt-in — never expand to all |
| Portal diet | If auto-detect providers empty and no filter → **empty steps** + same note (wizard `plan_for_providers` with explicit list still may expand) |

**Proof**

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl --bin shipctl detect_paddle\|marketing\|secrets\|portal` |
| L2 | `shipctl secrets --project apps/website` → paddle PUBLIC_* only (or few); not Neon/Supabase flood |
| L2 | `shipctl pulse --project apps/website` → kind mentions marketing / markets |

### S2 — Host unbound gate (next)

When Public intent and no Tier A host config and no `primary_host`: insert human step `platforms.host` (desktop_view `platforms`) **instead of** blind `deploy` Orbit run. Detail: Choose Cloudflare / Vercel / Netlify → Login CLI or dashboard → Continue. No new catalog cards.

### S3 — Website host bind (maintainer)

Pick host for official site; write marker (`wrangler.toml` Pages / `vercel.json`) or `primary_host` via Platforms. Then H1–H6 hostdeploy path applies. Out of S1 code.

### S4 — Optional Desktop copy only

If S1–S2 leave Desktop labels stale, one-line Publish/Pulse copy — still no Integrations expansion.

## Invariants

1. CLI remains kernel under Client/MCP.  
2. Bridge never holds secrets or finishes OAuth.  
3. Subtraction preferred over new UI.  
4. Coach stays cancelled.

## Acceptance (S1)

- [x] `PUBLIC_PADDLE_*` → `detected.paddle`
- [x] Bind `apps/website` → `marketing_site`
- [x] Undetected project secrets/portal do not list all providers
- [x] Existing opt-in / wrangler / markets tests still pass

## Acceptance (S2)

- [x] Unbound Public/General plan uses `platforms.host` (not blank Orbit `deploy`)
- [x] `primary_host` without Tier A config → `deploy.host` / hostdeploy
- [x] Wrangler/Vercel/Netlify still get Orbit `deploy`

## Next

S3 — maintainer picks website host marker · or idle
# Human-gate catalog — design

**Status:** Active — O2 plain hints shipped; O3 MCP contract next  
**Updated:** 2026-09-25  
**Parent:** [ship-studio-overhaul-design.md](./ship-studio-overhaul-design.md)  
**Surfaces:** [surfaces-cli-tui-desktop.md](./surfaces-cli-tui-desktop.md) · Desktop Portal / Env / Publish Confirm  
**Reliability:** [desktop-reliability-design.md](./desktop-reliability-design.md) (TTY Put / Login / N-class)

```text
GOAL:     Map every official-channel moment to one primary Client CTA.
NOT:      Coach theater · Docs as default setup · Studio secret custody
```

## Why

Ship Studio’s paid value is **choreography of human gates** (same as a freelance DevOps handoff), not vendor documentation. Portal `kind`, Env actions, and Publish Confirm must map to this catalog.

## Gate kinds

| Kind | When | Client primary CTA | Secondary | Never primary |
|------|------|-------------------|-----------|---------------|
| **secret_put** | Env / secrets on deploy host | **Put** in terminal (`env --put` / `human --put`) | Open exact settings deep link | Docs encyclopedia |
| **oauth_login** | Provider CLI login required | **Login CLI** terminal | Open account/dashboard | Long tutorial |
| **attest_confirm** | Human finished vendor UI; Studio must advance | **Confirm** in Publish | Open step URL (re-open if needed) | Auto-Confirm |
| **sign_approve** | Signet / notarization / store approve | **Open/Run** Signet or vendor approve | — | Fake “signed” without evidence |
| **deploy_evidence** | Hosted deploy or live URL | Deploy/Flow terminal (Advanced N-class) or Confirm live URL | Platforms / Ritual | Sticky FAILED on soft auth |

## Portal step mapping

| Portal `kind` (today) | Catalog kind | O1 Client button law |
|-----------------------|--------------|----------------------|
| `env` | secret_put | Primary **Put** (when put path exists); Open = dashboard only; Docs label **Learn more** |
| `oauth` | oauth_login | Primary **Login CLI** when CLI exists; Open = dashboard |
| `token` / create URL | secret_put or attest | Put if put CLI; else Open deep link + Confirm later |
| `list` / `check` / deploy-ish | attest_confirm / deploy_evidence | Open step URL → Confirm; Deploy uses N-class terminal when Advanced+Online |

Env list (`applyEnv`): retrieve + named secret → **Put** primary (already); Open secondary.

## Copy law

1. **One sentence “Do X”** in plain language (founders).  
2. Then the primary CTA.  
3. CLI jargon (`wrangler secret put <NAME>`) may appear as **detail**, never as the only guidance.  
4. Docs / “Learn more” is optional help after the primary path is clear.

### Example (Cloudflare env) — target after O2

| Bad (today-ish) | Good |
|-----------------|------|
| Detail: `Worker secrets: wrangler secret put <NAME>…` + Open → docs/dashboard confusion | Detail: `Create or copy each secret on Cloudflare, then Put here in the terminal — never paste into Studio.` Primary: **Put** · Secondary: **Open dashboard** · Tertiary: **Learn more** |

## Invariants

1. No coach / floating overlay theater ([CANCELLED](./vendor-handoff-coach-design.md)).  
2. Put/Login always use interactive terminal helpers (reliability slices 1–2).  
3. Open ≠ Docs URLs (hosting parity).  
4. MCP may suggest or invoke the same CTAs; humans still paste keys.  
5. Confirm remains honest — no auto-Done on OAuth/store/DNS.

## Implementation owners

| Phase | Owner |
|-------|--------|
| O1 | `apps/desktop/src/main.ts` `applyPortalPlan` (+ optional Env label polish) |
| O2 | `crates/shipctl/src/portal.rs` `env_hint` / step detail / URL audit |
| O4 | Platforms/Integrations wizard copy in Desktop |

## Proof

- L1: desktop `tsc` · `cargo test -p shipctl` portal URL tests  
- L2: Harbor Public → Portal → Cloudflare env → **Put** opens terminal; **Learn more** is not the first click story  

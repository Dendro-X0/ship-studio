# Vendor handoff coach — investigation

**Status:** Active  
**Updated:** 2026-09-25  
**Surface:** Desktop Portal · Env  
**Parents:** [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md) · [provider-wizard-setup-design](../frontend/provider-wizard-setup-design.md) · reliability slices 0–4 closed

## Symptom

Portal step **Cloudflare environment / secrets** (ENV badge) exposes **Open** and **Docs**. Operators (and non-technical founders) click through and land on Workers Secrets documentation or a bare dashboard with no Studio-owned sequence. That feels like “the app abandoned me to another platform.”

## Evidence

| Fact | Where |
|------|-------|
| Card title `{label} environment / secrets`, kind `env` | `crates/shipctl/src/portal.rs` portal_step |
| Open URL (Cloudflare) | `https://dash.cloudflare.com/?to=/:account/workers-and-pages` |
| Docs URL | `https://developers.cloudflare.com/workers/configuration/secrets/` |
| Desktop Open/Docs | `applyPortalPlan` — naked `openUrl` |
| Put already exists | Env `Put` → `openEnvPutTerminal`; Human `--put` terminal |
| Product gap named | PLATFORMS-AND-PORTAL — Env Put UX still CLI-oriented |

Screenshot path in session: Open/Docs → Cloudflare Workers Secrets docs page (operator-selected Docs or prior collision).

## Root cause

Leaving Studio is treated as the **product** instead of a **step inside a Studio-owned minute loop**. Docs is useful as secondary help; it must not be the default mental model for “set up secrets.”

## Desired operator story

```text
Click Open on ENV step
  → Studio shows a short coach (toast-actions or floating card)
  → Step 1: get/create value on vendor (Open dashboard)
  → Step 2: put on host in a terminal (never paste into Studio)
  → Step 3: return to Publish → Confirm
  → Docs remains available, never the only path
```

## Non-goals (this band)

- Studio-held secrets / vault UX (reliability Later)
- Auto-Confirm after put
- Replacing wrangler / vendor UIs
- Polar E2E

## Next artifact

[vendor-handoff-coach-design.md](./vendor-handoff-coach-design.md)

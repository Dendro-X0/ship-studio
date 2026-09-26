# Ship Studio overhaul — design

**Status:** **Done** — O0–O5 closed ([evidence-harbor-client-honesty](../../docs/handoffs/evidence-harbor-client-honesty.md))  
**Updated:** 2026-09-25  
**Parents:** [SCOPE-OF-SERVICE](../../docs/product/SCOPE-OF-SERVICE.md) · [surfaces-cli-tui-desktop](./surfaces-cli-tui-desktop.md) · [human-gate-catalog-design](./human-gate-catalog-design.md) · [mcp-assist-contract-design](./mcp-assist-contract-design.md) · [vendor-handoff-coach-design](./vendor-handoff-coach-design.md) (CANCELLED)  
**North-star band:** #51 (complete)  

```text
GOAL:     Client honesty — every human gate has one concrete primary CTA
          (Put / Login CLI / exact deep link / Confirm), never a vendor
          encyclopedia as the path. MCP assists; humans hold keys. CLI = kernel.
NOT:      Greenfield Desktop rewrite · coach/overlay theater · vendor replacement ·
          Studio-held secrets · auto-OAuth · Adaptive lane sprawl (bands 1–50 stay)
```

## Problem

Breadth (Adaptive lanes, Platforms catalog, reliability TTY) already shipped. The **fee bar** fails when Client primary path for a gate is jargon docs (e.g. Cloudflare Workers Secrets). Founders need the same choreography a freelance DevOps handoff uses: one act, then return to Confirm.

## Pathways (unchanged law)

| Pathway | Surface | Role |
|---------|---------|------|
| **Client** | Desktop · TUI | UX spine; must pass honesty criterion 6 |
| **MCP** | `shipctl mcp` | Agent assist; no key custody |
| **CLI** | `shipctl` | Shared kernel |

Detail: [surfaces-cli-tui-desktop.md](./surfaces-cli-tui-desktop.md).

## Overhaul type

**Product + Client honesty** on the existing Detect → Publish → Open/Run → Confirm spine. Not a UI rewrite.

## Roadmap

| Phase | Outcome | Proof |
|-------|---------|-------|
| **O0 — Spec freeze** | This design + human-gate catalog; product docs aligned; handoff → O1 | **Done** — docs |
| **O1 — Portal CTA law** | `kind: env` (Tier A first): primary **Put** when put path exists; **Open** = dashboard deep link only; **Docs** → “Learn more”; no coach | **Done** — `applyPortalPlan` Put · L1 tsc |
| **O2 — Copy + deep-link pass** | Plain-language `env_hint` / step detail in `portal.rs` (CF/Vercel/Netlify); Open ≠ Docs | **Done** — plain hints · `tier_a_env_open_ne_docs_and_plain_hints` |
| **O3 — MCP assist contract** | Agent path documented; `ship_*` gap list for S2.4; keys human-held | **Done** — [mcp-assist-contract-design](./mcp-assist-contract-design.md) |
| **O4 — Catalog diet** | Platforms/Integrations: Put/Login primary where applicable; Docs secondary | **Done** — Platforms Put secrets · Learn more · L1 tsc |
| **O5 — Honesty proof** | Harbor Client-honesty checklist; demo/SCRIPT note; S1.2m closed | **Done** — [evidence-harbor-client-honesty](../../docs/handoffs/evidence-harbor-client-honesty.md) · L1 portal/tsc · L2 Harbor CF env |

Implementation complete — activate next work from [improvement-backlog](../../docs/product/improvement-backlog.md) (suggested: S1.1 demo GIFs).

## Non-goals

- In-app floating coach / OS-external overlay theater ([CANCELLED](./vendor-handoff-coach-design.md))  
- Studio-held secrets or auto-OAuth  
- Rewriting Adaptive lanes 1–50  
- Making CLI the buyer-facing pitch  
- Polar E2E · mobile store API · k8s (remain PAUSED)

## Fee bar

Charge for Client only if a non-technical account-holder can finish secret/OAuth gates via Put/Login without surviving vendor encyclopedias. Linking Docs is not a product.

## Activation

**Complete.** Band #51 / S1.2m closed. MCP G1–G6 stays S2.4 shelf.

## Related

| Doc | Role |
|-----|------|
| [human-gate-catalog-design.md](./human-gate-catalog-design.md) | Gate kinds + CTA law |
| [mcp-assist-contract-design.md](./mcp-assist-contract-design.md) | Agent `ship_*` inventory + gaps |
| [desktop-reliability-design.md](./desktop-reliability-design.md) | TTY Put / Login / N-class (done 0–4) |
| [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md) | Guide surfaces; honesty gaps |
| [improvement-backlog.md](../../docs/product/improvement-backlog.md) | S1.2m · S2.4 |

# Launch panel redirects — design

**Status:** Shipped (spine collapse) · **Next:** [launch-choice-board-design](./launch-choice-board-design.md)  
**Updated:** 2026-09-26  
**Parent:** [deployment-nav-design](./deployment-nav-design.md)

```text
GOAL: Launch is a short spine of Auto gates + a few Open-* panel redirects.
NOT: One row per vendor (stores · hosts · Polar · Stripe · Signet substeps).
```

## Target spine

| Keep in Launch | Opens |
|----------------|--------|
| doctor · configure · intent · scopes · legal | (auto / confirm) |
| `oauth.hosts` / `deploy.panel` | **Deployment** |
| `sign.panel` | **Sign** (identity · build · official · store submit) |
| `env.sprint` | **Env** (when secrets to put) |
| `integrations.panel` | **Integrations** — **only if** payment/commerce detected |
| flow_dry_run · deploy | local run |

Omit from Launch (live on panels): per-host `host.*`, per-store `submit.*` / `listing.*`, per-Signet `signet.*` / `sign.official.*`, marketing as its own row when Deployment covers it.

## Proof

- L1: `cargo test -p shipctl` · desktop `tsc`
- L2: Harbor Launch step count ≪ 22; Open Sign / Open Deployment / no Polar row unless Polar detected

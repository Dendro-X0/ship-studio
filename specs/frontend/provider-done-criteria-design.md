# Provider done criteria — design (S1.12)

**Status:** Slice 1 shipped (wizard Done when copy)  
**Updated:** 2026-09-28  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.12 · [verify-status-layers-design](../backend/verify-status-layers-design.md)

```text
GOAL: Each Integrations wizard states what “done” means and which Verify layer
      applies — so Confirm is never confused with Studio probing Polar/Stripe.
NOT: Webhook live probes · Studio HTTPS with secrets · auto-green Verify.
```

## Done criteria (operator-facing)

| Lane | Done when | Verify layer |
|------|-----------|--------------|
| Polar | Checkout / product live on Polar; `PUBLIC_POLAR_*` (and optional webhook) set **outside** Studio | `human_attest` on `listing.polar` |
| Stripe / Gumroad / Lemon / Paddle | SKU / checkout live on vendor; host secrets outside Studio | `human_attest` on `listing.*` |
| Resend | `RESEND_API_KEY` on deploy host; test send on Resend UI | `human_attest` on `env.sprint` (when Public) |

Desktop: one-line **Done when…** under the wizard blurb (Integrations).

## Non-goals (this slice)

- Changing `infer_verify_status`  
- Operator_cli probes for commerce dashboards  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `tsc` |
| L2 | Polar / Resend wizard shows Done when line mentioning human attest / Confirm |

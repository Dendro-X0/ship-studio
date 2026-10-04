# Resend wizard parity — design (S1.13)

**Status:** Shipped (L1 tsc)  
**Updated:** 2026-09-28  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.13 · [provider-wizard-setup-design](./provider-wizard-setup-design.md) · [wizard-completion-confirm-design](./wizard-completion-confirm-design.md)

```text
GOAL: Resend matches Polar’s minute loop — Open → Put on host → test on vendor →
      Confirm gate / Continue publishing — without Studio holding the API key.
NOT: Studio sending mail · inventing a listing.resend Publish step · Put without a host.
```

## Behavior

| Control | Resend |
|---------|--------|
| Open dashboard | Resend API keys (unchanged) |
| **Put key** | `orbityard env --provider <detected host> --put RESEND_API_KEY` — needs Cloudflare / Vercel / Netlify detect |
| Confirm gate | Prefer Publish `env.sprint` when present (Public intent) |
| Continue publishing | Same preferred step / handoff as Confirm |

Steps copy ≤5, same honesty as Polar (never paste into Studio).

## Non-goals

- Detecting Resend via package.json as a new Publish listing  
- Portal provider id for Resend  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apps/desktop` `tsc` |
| L2 | Harbor Public + Cloudflare detect → Integrations Resend shows Put · Confirm gate names env.sprint when current |

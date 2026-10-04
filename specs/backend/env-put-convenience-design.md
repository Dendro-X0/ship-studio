# Env / Secrets Put convenience

**Status:** Design + implement this slice  
**Updated:** 2026-10-03  
**Owner:** Desktop Secrets · Integrations Put · Env empty state  
**Parent:** [human-gate-catalog-design](./human-gate-catalog-design.md) · [integrations-deploy-parity-design](./integrations-deploy-parity-design.md)

## Plan alignment

- **Handoff:** Maintainer wants easier env/token configuration (same feel as Deployment Put).
- **PAUSED:** Paddle Solo API keys — do not chase website Buy tokens.
- **CANCELLED:** Vendor coach / Studio secret custody.
- **In scope:** Put button wherever a secret **name** is known; always paste in host CLI terminal (CF/Vercel/Netlify).
- **Out of scope:** Creating vendor API keys inside Studio; storing values in `.ship/`; writing `apps/website/.env` from Desktop.

## Contracts

1. **Secrets** list: each hint gets **Put** → `openEnvPutTerminal(host, name)` where `host` is hint provider if Tier A, else `preferredEnvPutHost()`.
2. **Human paste queue:** same Put button (not only Copy CLI).
3. **Integrations:** named Put for Resend + commerce host secrets (STRIPE_SECRET_KEY, CREEM_API_KEY, …) when a Tier A host is detected — same terminal path as Resend.
4. **Env** empty retrieve: show “Add empty `NAME=` lines to `.env` / `.dev.vars` or wrangler `# Secrets:` — then Put appears.”
5. Open vendor page remains secondary for copy; Put never captures the value in Studio.

## Acceptance

- [x] Secrets rows show Put when a host Put path exists
- [x] Human queue shows Put
- [x] Integrations Put covers Resend + common commerce host secret names
- [x] Env empty state names how to get Put rows

## Proof

| Layer | Command |
|-------|---------|
| L1 | `pnpm exec tsc --noEmit` in `apps/desktop` |
| L3 | Desktop Secrets → Put opens terminal (maintainer) |

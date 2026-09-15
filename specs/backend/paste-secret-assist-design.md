# Paste-secret assist — design

**Status:** Active  
**Updated:** 2026-09-15  
**Owner:** `crates/shipctl` (`secrets` module)

## Goal

After portal opens the credential page, help the operator **paste** values into the correct provider CLI — without shipctl storing secrets on disk.

## Commands

```text
shipctl secrets --project . [--provider ID] [--open]
shipctl secrets put --project . --provider cloudflare --name GITHUB_TOKEN
```

- `secrets`: JSON plan of hinted names + put CLI + entry URL (offline-safe).
- `secrets put`: interactive provider CLI (value typed/pasted in terminal; not logged by shipctl).
- `--open`: open provider token/env entry URL once.

## Hint sources (priority)

1. `wrangler.toml` / nested: `# Secrets …` comment block (Orbit-compatible)
2. Empty keys in `.dev.vars` / `.env` / `.env.local` (name only, never values)
3. Catalog fallbacks when provider detected but no names found (optional generic note)
4. **Graduate / commerce catalog** (when `detected.graduate_sign` / `gumroad` / `lemon`): name-only rows — open dashboard URL; no real put CLI (CI / vendor consoles only; never `.ship/`)

### Graduate catalog names

`SIGNET_OV_CERT`, `SIGNET_AZURE_CLIENT_ID`, `SIGNET_AZURE_CLIENT_SECRET`, `SIGNET_AZURE_TENANT_ID`, `SIGNET_NOTARY_PROFILE`, `WIN_CERT_PFX_PASS`, `APPLE_API_KEY_ID`, `APPLE_API_ISSUER_ID`

### Commerce catalog names

- Gumroad: `GUMROAD_ACCESS_TOKEN`, `GUMROAD_PRODUCT_ID`, `GUMROAD_CHECKOUT_URL`
- Lemon: `LEMON_API_KEY`, `LEMONSQUEEZY_WEBHOOK_SECRET`, `LEMON_CHECKOUT_URL`

## Put mapping

| Provider | Put command |
|----------|-------------|
| Cloudflare | `wrangler secret put <NAME>` |
| Vercel | `vercel env add <NAME>` |
| Netlify | `netlify env:set <NAME>` |
| GitHub | no put — open tokens page |
| graduate / gumroad / lemon | no put — open docs/dashboard |

## Invariants

1. Never write secret values to `.ship/` or logs.
2. Never print pasted values.
3. Desktop lists hints + copy command / open URL; interactive put prefers CLI/TUI TTY.

## Proof

- L1: parse secrets comment + empty `.dev.vars` keys unit tests
- L1b: markets `graduate`+`gumroad` → secrets plan includes catalog names
- L2: `shipctl secrets --project <fixture>` JSON
- TUI/Desktop: Secrets action surfaces same plan

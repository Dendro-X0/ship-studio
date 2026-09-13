# Paste-secret assist — design

**Status:** Active  
**Updated:** 2026-09-13  
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

## Put mapping

| Provider   | Put command                          |
|------------|--------------------------------------|
| Cloudflare | `wrangler secret put <NAME>` (cwd = wrangler dir) |
| Vercel     | `vercel env add <NAME>`              |
| Netlify    | `netlify env:set <NAME>` (prompts)   |
| GitHub     | no put — open tokens page + remind to set on deploy target |

## Invariants

1. Never write secret values to `.ship/` or logs.
2. Never print pasted values.
3. Desktop lists hints + copy command / open URL; interactive put prefers CLI/TUI TTY.

## Proof

- L1: parse secrets comment + empty `.dev.vars` keys unit tests
- L2: `shipctl secrets --project <fixture>` JSON
- TUI/Desktop: Secrets action surfaces same plan

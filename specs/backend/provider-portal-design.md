# Provider portal — design

**Status:** Active — extend via [hosting-portal-parity-design](./hosting-portal-parity-design.md)  
**Updated:** 2026-09-25  
**Owner:** `crates/shipctl` (`portal` module)  
**Desktop overview:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)

## Goal

Semi-automatic **shipping portal**: detect which deploy/auth providers apply, emit ordered human steps with **canonical entry URLs + docs URLs + CLI login commands**, optionally open the browser / run login CLIs. Does **not** replace Cloudflare / Vercel / Netlify / GitHub / Fly / Railway; does **not** invent secrets.

Desktop: **Open** = settings UI (`entry_url`); **Docs** = official tutorial (`docs_url`); **Login CLI** = interactive terminal for OAuth rows only.

## In scope (v1 — current code)

| Provider   | Detect                         | Entry kinds                                      |
|------------|--------------------------------|--------------------------------------------------|
| Cloudflare | wrangler.toml / json / jsonc   | OAuth (`wrangler`/`orbit login`), API token URL, Workers secrets hint |
| Vercel     | vercel.json / `.vercel`        | OAuth (`vercel`/`orbit login`), token URL, env hint |
| Netlify    | netlify.toml / `.netlify`      | OAuth (`netlify`/`orbit login`), PAT URL, env hint |
| GitHub     | `.git` or `GITHUB_*` hints     | `gh auth login`, tokens settings URL             |

## Planned hosting extension

See [hosting-portal-parity-design](./hosting-portal-parity-design.md): Fly/Railway Login CLI + distinct env Docs; GitHub Pages ≠ PAT; URL hygiene; Tier D hosts stay Advanced until catalog expand.

## Commands

```text
shipctl portal --project . [--provider ID] [--open] [--login]
```

- Default: JSON plan of steps (offline-safe; URLs are data only).
- `--open`: open each step’s primary `entry_url` in the system browser (operator-initiated network/UI).
- `--login`: for selected providers, run the preferred local login CLI (interactive); still human completes OAuth in browser.

## Invariants

1. Bridge does not call vendor HTTPS APIs itself.
2. Catalog URLs are versioned constants in `portal.rs` (aligned with Orbit AuthGuide where possible).
3. Secrets stay in provider CLIs / operator paste — shipctl never stores tokens.
4. Multi-provider: when several match, plan lists all; `--provider` filters.

## Proof

- L1: unit tests for detect + catalog IDs + suggested deploy_args including netlify
- L2: `shipctl portal --project <tmp>` prints JSON with expected providers
- Desktop: Portal button runs `portal` and lists steps; Open uses opener on `entry_url`

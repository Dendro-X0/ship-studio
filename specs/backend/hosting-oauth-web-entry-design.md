# Hosting OAuth web entry — design

**Status:** Slice 1 shipped  
**Updated:** 2026-09-27  
**Parent:** [hosting-portal-parity-design](./hosting-portal-parity-design.md)

```text
GOAL:   Portal OAuth steps expose a browser dashboard/sign-in Open
        (users often already logged into Google/GitHub in the browser).
NOT:    Studio-held OAuth · detecting login success · replacing Login CLI
```

## Problem

CLI-capable hosts (Cloudflare · Vercel · Netlify · …) emitted `*.oauth` with `entry_url: None`, so Desktop **Open** was disabled and only **Login CLI** / **Docs** showed. Dashboard deploy/sign-in is often faster than starting a CLI.

## Slice 1 (shipped)

| Change | Detail |
|--------|--------|
| `oauth_web_url(provider)` | Stable vendor login/dashboard URL (≠ token_page, ≠ env Open) |
| Portal `*.oauth` | Set `entry_url`; title “sign-in”; hint prefers web when already signed in |
| Desktop Portal | OAuth **Sign in (web)** primary when URL present; **Login CLI** secondary |

## Honesty

- Web Open = vendor session in the browser (dashboard deploy / create project).  
- Login CLI = local CLI credentials when you deploy from this machine.  
- Studio still does not call vendor HTTPS or mark OAuth Done automatically.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p orbityard -- cloudflare_steps_have_distinct_open_targets vercel_oauth_has_web_sign_in_entry` — green |
| L2 | Portal filter Cloudflare → OAuth **Sign in (web)** enabled |

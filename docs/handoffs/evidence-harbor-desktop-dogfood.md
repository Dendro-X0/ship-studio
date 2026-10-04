# Evidence — Harbor Desktop UI dogfood (Publish · Payments · primary host)

**Date:** 2026-09-28  
**Subject:** `fixtures/harbor`  
**Binary:** `target/release/orbit-yard-desktop.exe` (rebuilt this session)  
**Method:** WebView2 CDP `:9223` + Playwright `connectOverCDP`  
**Related CLI:** [evidence-harbor-launch-l3-l4](./evidence-harbor-launch-l3-l4.md)

## Setup

1. `pnpm desktop:release`
2. `powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1`
3. Launch with `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9223`

## L2 results (Desktop CDP)

| Check | Result |
|-------|--------|
| Bind Harbor | Pass |
| Workflow previews (`About N steps · ~min`) | Pass — all four Dashboard cards |
| Publish copy (`We'll check…` on stage) | Pass — Sign only → Scopes stage |
| Payments off → no Payments lane | Pass |
| Payments on → `launch_payments: true` + lane | Pass |
| Nav open Deployment does **not** invent `primary_host` | Pass (fix this session) |
| Cloudflare card → `primary_host=cloudflare` | Pass |
| Reload → Cloudflare still `is-active` | Pass |

## Fix landed with dogfood

`openPlatformsCatalog` no longer calls `selectPlatform` on soft open (nav / Launch related). Soft open **highlights** preferred/saved host; **persist** only on explicit card click or `selectId` (chip / Targets Deploy / command). Matches Launch L4 “Detect suggests; human chooses.”

## Cleanup

`harbor-reset` after run; Desktop process stopped.

## Verdict

**Desktop UI L2 pass** for Publish copy · Payments · primary host on Harbor.

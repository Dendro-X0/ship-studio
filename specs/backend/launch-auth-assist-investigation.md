# Launch auth assist — investigation

**Status:** Fixed — implement in Desktop Launch + orbityard open  
**Updated:** 2026-09-26  
**Symptom:** Launch / Confirm gates feel pointless — “authenticate on the vendor site, then return” with no auth link or official CLI launch.

## Evidence

Harbor Launch plan includes `oauth.cloudflare|vercel|netlify` with **no** `entry_url` and **no** `run`. After portal-card UI:

| Step | Card CTA before | Actual effect |
|------|-----------------|---------------|
| `oauth.*` | Disabled “Confirm when ready” | **No Login CLI** — stranded |
| `signet.identity` | Run local → `launchAction(["open"])` | Headless `signet identity list` (no create, no TTY) |
| `signet.release` / GitHub | Open portal → URL only | `github.com/new` — **no** `gh auth login` |

Header `#btn-launch-open` still opened an interactive terminal for oauth/sign — card CTAs bypassed it.

Catalog law ([human-gate-catalog-design](./human-gate-catalog-design.md)): oauth_login primary = **Login CLI**; Open = secondary dashboard.

## Root cause

Portal-card Launch UI subtracted the auth assist path. Copy (`return → Confirm`, honesty toast) reinforced “go elsewhere first.”

## Fix (shipped)

1. Launch cards: oauth → primary **Login CLI**; GitHub release gates → Login CLI (`gh`) + Open portal; Run local → `openShipctlTerminal(launch open)`.
2. `launch open` / `signet.identity`: if no identity, run interactive `signet identity create`.
3. Copy: Login CLI / Open auth first; Confirm after Verify.

## Proof

- L1: `cargo test -p orbityard` (118 ok) · desktop `tsc --noEmit`
- L2: Harbor Launch → oauth current → **Login CLI**; identity Run → create-when-missing in TTY

# Ship Studio — product contract

**Status:** Active — desktop v0 shell usable  
**Updated:** 2026-09-12  

```text
GOAL:     One local Ship workflow: configure → sign (Signet) → deploy (Orbit)
NOT:      Merge Signet/Orbit source · cloud control plane · launch/checkout bands
RUNTIME:  Local + offline-first (bridge never requires network; adapters may call local CLIs only)
SHELL:    Desktop client akin to CodaCtrl (one window = one repo) — `apps/desktop`
PROOF:    Signet build+sign dogfood · assess-api Orbit Cloudflare deploy via shipctl
DONE:     Bridge shell dogfooded end-to-end (Signet sign + Orbit deploy) — secrets/gaps are app-side
```

## Architecture

```text
Desktop shell (apps/desktop)
        │
        ▼
   shipctl  (CLI / MCP)  ← beside exe, SHIPCTL_PATH, workspace target, or PATH
        │
   ┌────┴────┐
   ▼         ▼
 signet     orbit
 (PATH / SIGNET_PATH / ORBIT_PATH / sibling ../ship)
```

## Commands (v0)

| Command | Meaning |
|---------|---------|
| `doctor` | Check Signet/Orbit, probe project, optional `.ship/studio.json` |
| `configure` | Probe repo + merge `.ship/studio.json` (`sign_args` / `deploy_args`) |
| `sign` | Invoke `signet` (defaults from studio.json; `--offline` → `doctor --json`) |
| `deploy` | Invoke `orbit` (defaults from studio.json; refuses `--offline`) |
| `flow` | configure → sign → deploy (sign uses studio `sign_args`, not `--help`) |
| `status` | Read `.ship/last-run.json` (per-step exit codes + RFC3339 timestamps) |
| `mcp` | Stdio MCP: doctor, configure, sign, deploy, flow, flow_dry_run, status |

## Desktop

```bash
# Preferred release stage (copies shipctl next to the exe)
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

Shortcuts: `Esc` cancel · `Ctrl+D` doctor · `Ctrl+S` sign · `Ctrl+Enter` flow dry-run · `Ctrl+Shift+Enter` flow

## `.ship/studio.json`

- `sign_args` — default `["doctor","--json"]`, or `["scan","--json"]` when no `signet.toml`
- `deploy_args` — default `["status"]` (non-interactive). With wrangler → `deploy --provider cloudflare`; with only vercel → `deploy --provider vercel`. Legacy `["ship"]` auto-migrates when CF/Vercel detected.
- Edit in UI (presets + Save) or by hand

## Offline / security

- Bridge process does **not** call vendor HTTPS itself.
- Network only happens if the operator runs Signet/Orbit steps that need it (explicit).
- `--offline` refuses deploy; sign falls back to local `signet doctor`.

## Non-goals (v0)

- Rewriting wrangler/vercel inside this repo
- Multi-root portfolio hub
- Paid unlock / Gumroad / traffic
- Full CodaCtrl Design/Perf/Verify lanes

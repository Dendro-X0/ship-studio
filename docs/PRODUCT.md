# Ship Studio — product contract

**Status:** Active — CLI + TUI + desktop  
**Updated:** 2026-09-13  

```text
GOAL:     Shipping portal on three surfaces: CLI (JSON) · TUI · Desktop — same shipctl engine
NOT:      Replace Cloudflare/Vercel/Netlify/GitHub · merge Signet/Orbit · invent cloud secrets
RUNTIME:  Local + offline-first (bridge never requires network; open/login are operator-initiated)
SHELLS:   shipctl CLI/MCP · shipctl tui · apps/desktop (Tauri)
PROOF:    cargo test -p shipctl · guide · portal · secrets · vault · tui --help
DONE:     Portal + secrets + vault.km export + guide --open + ship + human + TUI/desktop
```

## Architecture

```text
┌─ Desktop (Tauri) ─┐     ┌─ TUI (ratatui) ─┐     ┌─ CLI / MCP ─┐
│  apps/desktop     │     │  shipctl tui    │     │  shipctl *  │
└────────┬──────────┘     └────────┬────────┘     └──────┬──────┘
         └────────────┬────────────┴─────────────────────┘
                      ▼
                   shipctl
         guide · portal · secrets · configure · signet · orbit
```

Design: `specs/backend/surfaces-cli-tui-desktop.md` · `specs/backend/provider-portal-design.md` · `specs/backend/paste-secret-assist-design.md` · `specs/backend/vault-export-design.md`

## Commands

| Command | Meaning |
|---------|---------|
| `doctor` | Check Signet/Orbit; includes portal providers + secret hint count |
| `guide` | Unified offline checklist; `--open` entry URLs |
| `ship` | One-shot offline prep → `.ship/last-guide.json` |
| `human` | **Portal sprint:** open Polar→GitHub→dashboards, then `--put` paste queue |
| `configure` | Write `.ship/studio.json` |
| `portal` | Provider entry plan; `--open` / `--login` |
| `secrets` | Paste-secret assist; `--put NAME --provider …` |
| `vault` | **Encrypted `.km` export** (Clavis-compatible); `export` / `add` / `list` / `show` |
| `tui` | Interactive terminal wizard |
| `sign` / `deploy` / `flow` / `status` | Signet / Orbit / pipeline / last-run |
| `mcp` | Stdio MCP (`ship_guide`, `ship_portal`, `ship_secrets`, `ship_vault`, …) |

## Providers (portal)

| Provider | Detect | OAuth CLI | Token page |
|----------|--------|-----------|------------|
| Cloudflare | wrangler.* | wrangler / orbit login | dash API tokens |
| Vercel | vercel.json / `.vercel` | vercel / orbit login | account tokens |
| Netlify | netlify.toml / `.netlify` | netlify / orbit login | PATs |
| GitHub | `.git` | `gh auth login` | settings/tokens |
| Polar | `POLAR_*` / polar.sh markers | dashboard (no CLI OAuth) | polar.sh/dashboard |

## Desktop

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

Buttons: **Human portal** · Wizard · Ship · Guide · Portal · Secrets · **Export vault** · …

## TUI

```bash
cargo build -p shipctl --release
./target/release/shipctl.exe tui --project .
# Home: Ship wizard · Guide · Portal · Secrets · …
```

## Offline / security

- Bridge does **not** call vendor HTTPS itself.
- `guide` / `portal` / `secrets` JSON offline-safe; open/login/put are explicit.
- Secret values never stored in `.ship/` plaintext.
- Optional **vault export**: Argon2id + AES-256-GCM `kmvault` file (open in Clavis / Keys Manager). Passphrase via TTY or `SHIP_VAULT_PASSPHRASE`.

```bash
shipctl vault export --out ./ship-secrets.km --from-hints --project .
shipctl vault list --file ./ship-secrets.km
```

## Non-goals (v0)

- Rewriting provider CLIs inside this repo
- Completing OAuth without the human
- Multi-root portfolio hub / paid unlock bands

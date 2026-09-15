# Ship Studio — product contract

**Status:** Active — CLI + TUI + desktop  
**Updated:** 2026-09-15  

```text
GOAL:     Local shipping hub — final-mile sign → release → deploy (Adaptive Publish spine)
NOT:      Product docs/demo authoring · replace vendor UIs · store secrets · finish OAuth/store review without the human
RUNTIME:  Local + offline-first (bridge never requires network; open/login/put/verify are operator-initiated)
SHELLS:   shipctl CLI/MCP · shipctl tui · apps/desktop (Tauri sidebar dashboard + Ctrl+K search)
PROOF:    cargo test -p shipctl · scripts/dogfood-advanced-*.sh · Desktop Publish Advanced
DONE:     Publish portal + final-mile cut order + graduate Run + desktop_cut honesty + container.build Run + scopes/env/listings/commerce + assist + desktop shell
NOT YET:  Operator completes paste / graduate certs / live release / marketplace / registry push / deploy on vendor platforms (see OPERATOR-NEXT)
```

## UX principle — minimal actions, one spine

Access a wide range of shipping functions through **few deliberate actions**. Publish is the integrated workflow: Open/Run → (vendor UI) → Confirm → Next. Scopes, Env, Sign, Portal, Ritual, and Tools are **detail panels** opened from the current publish step (`desktop_view`), not competing start points. Assist is a checklist overview; Launch is a companion stepper — prefer Publish for the full minute path.

**Modes:** **General** (default) — shortest publish plan + focused nav. **Advanced** — full OAuth/official-sign/listing plan + Assist/Launch/Portal/Ritual/Tools. See `specs/backend/studio-modes-design.md`.

DO NOT: make operators reassemble the release from eight peer nav destinations · force Assist → Scopes → Env → Sign → Portal → Publish as the happy path.
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

Design: `specs/backend/shipping-hub-north-star.md` · `specs/backend/surfaces-cli-tui-desktop.md` · `specs/backend/provider-portal-design.md` · `specs/backend/paste-secret-assist-design.md` · `specs/backend/vault-export-design.md` · `specs/backend/guided-launch-design.md` · `specs/backend/studio-scopes-design.md` · `specs/backend/publish-portal-design.md` · `specs/backend/release-surface-map.md`

## Commands

| Command | Meaning |
|---------|---------|
| `doctor` | Check Signet/Orbit; includes portal providers + secret hint count |
| `guide` | Unified offline checklist; `--open` entry URLs |
| `ship` | One-shot offline prep → `.ship/last-guide.json` |
| `publish` | **Publish portal:** minute wizard open/run → verify/confirm → next |
| `pulse` | **Project pulse:** git · wizards · deploy signals · next action (local) |
| `launch` | Guided launch: open/run → verify/confirm → next (sign · release · list · deploy) |
| `scopes` | Detect / select Web · API · Desktop directories |
| `env` | ENV & token portal (configure / retrieve / create URLs) |
| `sign-paths` | Self-sign (Signet) vs official Apple/Windows/Play/GitHub |
| `assist` | Full-stack checklist; `--start` loads publish portal |
| `human` | Portal sprint: open Polar→GitHub, then `--put` paste queue |
| `configure` | Write `.ship/studio.json` |
| `portal` | Provider entry plan; `--open` / `--login` |
| `secrets` | Paste-secret assist; `--put NAME --provider …` |
| `vault` | **Encrypted `.km` export** (Clavis-compatible); `export` / `add` / `list` / `show` |
| `tui` | Interactive terminal wizard |
| `sign` / `deploy` / `flow` / `status` | Signet / Orbit / pipeline / last-run |
| `mcp` | Stdio MCP (`ship_publish`, `ship_guide`, `ship_portal`, …) |

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

Buttons: **Publish** (minute wizard) · **Assist** checklist · **Launch** · **Human portal** · Wizard · Ship · Guide · Portal · Secrets · Export vault · …

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

- Authoring product docs, feature demos, or marketing GIFs inside Ship Studio  
- Rewriting provider CLIs inside this repo  
- Completing OAuth / store review / DNS without the human  
- Multi-root portfolio hub / paid unlock bands  

Charter: `specs/backend/shipping-hub-north-star.md` · surface map: `specs/backend/release-surface-map.md`

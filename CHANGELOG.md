# Changelog

## 0.1.0 — 2026-09-13

### Added

- **Guided launch** (`shipctl launch`): open official entry → verify/confirm → next until deploy
- **Human portal sprint** opens paste-source tabs only (Polar→GitHub); Desktop **Paste in terminal**; `--open-all` for full dashboards
- **CI** (GitHub Actions): `cargo test -p shipctl` + CLI smoke on `main`
- **Desktop / TUI / MCP** vault export surfaces (`Export vault`, TUI `v`, `ship_vault`)
- **Guide** step `vault` (optional encrypted backup after secrets)
- **`shipctl vault`** encrypted `.km` export (Clavis / Keys Manager `kmvault` v1): `export` / `add` / `list` / `show`
- **Portal** for Cloudflare, Vercel, Netlify, GitHub, Polar (entry URLs + OAuth/dashboard navigation)
- **`shipctl secrets`** paste-assist (wrangler `# Secrets` + empty `.dev.vars`; never stores values)
- **`shipctl guide`** unified offline checklist; **`--open`** batch-opens entry URLs
- **`shipctl ship`** one-shot offline prep (guide → configure → flow dry-run; writes `.ship/last-guide.json`)
- **`shipctl tui`** ratatui wizard (providers, portal, secrets, ship prep)
- **Desktop** Tauri shell: Wizard / Ship / Guide / Portal / Secrets
- **MCP**: `ship_guide`, `ship_ship`, `ship_portal`, `ship_secrets`, …
- Doctor reports `portal_providers`, `secret_hint_count`, `provider_clis`

### Notes

- Bridge does not call vendor HTTPS; OAuth and secret paste stay operator-initiated.
- Optional vault export uses Argon2id + AES-256-GCM; passphrase via TTY or `SHIP_VAULT_PASSPHRASE`.
- Signet and Orbit remain separate products — Ship Studio is the bridge + shells.

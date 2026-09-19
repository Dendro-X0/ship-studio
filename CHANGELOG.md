# Changelog

## 0.1.0 — 2026-09-14

### Added

- **Desktop busy unlock** — Cancel/Clear always re-enable Publish Refresh; `run()` clears busy in `finally`; 90s stuck hint
- **Env entry URL honesty** — secret Open URLs match value source or put provider (Vercel-bound names no longer open Cloudflare tokens)
- **Personal / local ship intent** — `--intent local|public` (orthogonal to General/Advanced); Local omits hosted env/deploy/store lanes; Desktop Local/Public toggle; persists in `.ship/studio.json`
- **aperio dogfood** — desktop Tauri no longer pulls Play/Android official/submit lanes (MAS + Microsoft Store stay)
- **Archetype expansion** — PWA manifest / vite-plugin-pwa detection (pulse + assist); Advanced `listing.huggingface` for Hub docs + Confirm (no upload); extended release-surface archetypes + launch matrix doc
- **Scope of service** — canonical product boundary: detect → plan → sequence final-mile; vendors + humans finish irreversible work ([docs/SCOPE-OF-SERVICE.md](./docs/SCOPE-OF-SERVICE.md))
- **README hub pointer** — shipping-hub mission + north star / release-surface / Advanced dogfood links for contributors
- **Assist Run notes** — checklist cues for `gh run/release list`, npm/crates `--dry-run`, and container build
- **release.github Run** — Advanced `gh release list` (read-only) + Verify when a release exists; never creates releases
- **Pulse cut hints** — Dashboard Now mid-publish detail cues for CI / npm·crates dry-run / container / GitHub Release
- **Desktop Open/Related parity** — Publish Open navigates Dashboard/Tools/Launch (and every Related target) the same as Related
- **Adaptive doctor + Verify** — layout-aware `doctor.ok`; Verify re-probes legal/trust; `gh` for ci.release; sticky pending baseline steps
- **Cut order + CI/registry Runs** (Advanced) — `ci.release` / `container.*` after Signet release; `gh run list` + `npm|cargo publish --dry-run` Runs; live publish stays Confirm
- **Container final-mile** (Advanced) — `container.build` Runs local `docker build` / `compose build`; `container.deploy` stays push docs + Confirm (never auto-push); doctor docker PATH notes
- Desktop **Dashboard Launch CTA** — emerald Start/Continue publishing button with hint + breathe; Assist matches
- **Professional launch baseline** (Advanced) — `legal.baseline` · `trust.pack` · `release.github` when LICENSE/SECURITY/TRUST or GitHub origin signals match
- **Package registries** (Advanced) — `listing.npm` / `listing.crates` for publishable packages or `.ship/markets` opt-in
- **Marketing deploy** (Advanced) — `marketing.deploy` for `apps/website` / GitHub Pages / HOOK preview / markets opt-in
- **Graduate signing + commerce** (Advanced) — `sign.graduate` · `listing.gumroad` · `listing.lemon` via markets/env opt-in (honesty: no verified-publisher claims)
- **Graduate / commerce secret catalog** — `shipctl secrets` name-only hints for SIGNET_*/WIN_CERT_*/GUMROAD_*/LEMON_* (never stored in `.ship/`)
- **Shipping hub final-mile** — cut order build→trust→graduate→release; `signet graduate notes` Run; `ship.desktop_cut` for desktop-only; doctor “can I cut?” notes
- **Suite URL sync** (Advanced) — `suite.url_sync` from `.ship/suite.json` sibling env key names (never writes sibling `.env`)
- **Publish portal** (`shipctl publish` / open / verify / confirm / next) — minute wizard through the full manual ship path
- Desktop **Publish** view (primary) + TUI Publish screen (`P`) + MCP `ship_publish`
- Assist `--start` loads the publish portal; Assist remains the checklist overview
- **Deploy assist** (`shipctl assist`) + Desktop Assist view — scopes → env → sign → publish
- **Scopes** (`shipctl scopes` / `set --ids`) — Web / API / Desktop / Mobile / Container directories drive per-scope deploy
- **ENV portal** (`shipctl env`) — configure / retrieve / create (no secret values)
- **Sign paths** (`shipctl sign-paths`) — self-sign vs official certs vs store **submit** (ASC / Play / MS)
- **General / Advanced modes** — short spine vs full OAuth / listing / submit / DB / CI / container plan
- **Release surface lanes** — mobile Play/ASC listing, Steam/itch/Epic (opt-in), DB provision (Neon/Supabase/D1/Turso), CI release check, container docs
- **Project pulse** (`shipctl pulse`) — Dashboard Now from git + `.ship` + deploy signals (skip when already live)
- Desktop project switcher (titlebar recents) + custom frameless chrome + logo
- Dogfood: `fixtures/advanced-dogfood` + `scripts/dogfood-advanced-*.sh`
- **Guided launch** (`shipctl launch` / `open` / `run`): adaptive plan through Signet build→release, Polar listing, Orbit deploy
- **TUI Launch** screen (`L` / home item): o open/run · v verify · c confirm · n next
- Desktop Launch **Open / Run** opens a terminal for Sign/OAuth/Deploy steps
- Verify uses PATH-resolved CLIs (`wrangler.cmd` on Windows) with a 20s timeout
- Tauri/`signet.toml` projects get Signet steps; worker-only repos skip them
- Configure defaults `sign_args=[build]` when Tauri or signet.toml is present
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

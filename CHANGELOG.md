# Changelog

## Unreleased

### Added

- **Hosting portal parity (slices 0–4)** — URL hygiene · Fly/Railway Login CLI · Pages≠PAT · Tier A–E docs · detect→Platforms highlight ([hosting-portal-parity-design](./specs/backend/hosting-portal-parity-design.md))
- **Portal Docs button** — Open goes to vendor settings; Docs opens official tutorials (`docs_url` on portal steps); Cloudflare env Open → Workers & Pages
- **Portal Open targets** — Cloudflare no longer triples the same API-tokens URL (drop duplicate recover; env → Workers secrets docs); Login CLI only on OAuth rows
- **Desktop silent-failure reliability** — Portal Login CLI opens an interactive terminal (not headless); Orbit no longer calls unknown portal provider; fail toasts include stderr; soft errors no longer sticky-FAILED ([desktop-silent-failures-investigation](./specs/backend/desktop-silent-failures-investigation.md))
- **Provider wizard setup** — Hosting / Payments / Email checklists name Open → put env → Confirm; **Continue publishing** handoff; Cloudflare uses `cloudflare.svg` ([provider-wizard-setup-design](./specs/frontend/provider-wizard-setup-design.md))
- **Collapsible sidebar sections** — Ship · Targets · Platforms · Integrations · More · Run fold like dropdowns (persisted); platform icons use Apple / Microsoft / Play / Fly / Railway assets
- **Platforms catalog** — Hosting + Official signing list (Integrations-style); probe **Choose host** / **Choose platform**; shared `provider-catalog` util ([platforms-catalog-design](./specs/frontend/platforms-catalog-design.md))
- **Paced Publish Continue** — Auto gates advance one-at-a-time (~2s dwell + scrubber); Confirm no longer flashes 2→7 ([publish-journey-clarity-design](./specs/frontend/publish-journey-clarity-design.md) slice 3)
- **Live check honesty** — Confirm blocked without deploy evidence; Public desktop-only offers **Switch to Local**; finished copy no longer says Continue when the pass is done
- **Publish Continue bounce** — Live check no longer flashes Publish↔Dashboard; paused honesty gates stay on Publish with Confirm
- **Publish journey clarity (slice 1)** — one green verb; hide dual Next on current gate; Continue (not Next) after Done; hide workflow cards mid-flight; Confirm auto-chains Auto gates ([publish-journey-clarity-design](./specs/frontend/publish-journey-clarity-design.md))
- **Nested fixture pulse** — bind under a monorepo (e.g. `fixtures/harbor`) no longer inherits parent dirty git; Orbit missing soft-cues only (Signet absence still hard-blocks desktop)
- **GIF-ready Dashboard polish** — mid-flight headline wins over “Already deployed”; ≤3 status tiles; Local treats Signet-only as ready; stage rail diet (`+N later` → List); Launch hero surface ([gif-ready-polish-design](./specs/frontend/gif-ready-polish-design.md))
- **Harbor demo fixture** — `fixtures/harbor` (Desktop + Docs + Signet) + `scripts/harbor-reset`; public GIFs bind this, not ship-studio or advanced-dogfood ([demo-subject-design](./specs/frontend/demo-subject-design.md))
- **Status probe inspection bay** — Sign / Publish: three lane cards (icons · Ready/Guide badges · suggestions · CTAs), checking shimmer + `shipctl pulse · sign-paths` cue; stage checkpoint amber rail ([status-probe-ux-design](./specs/frontend/status-probe-ux-design.md))
- **Public Publish UX (A+C)** — Stages mode: one primary on the checkpoint card (toolbar keeps Refresh · Watch); Scopes gate embeds detect/save inline with **Confirm & continue** ([public-publish-ux-design](./specs/frontend/public-publish-ux-design.md))
- **Actionable gate toasts** — pending Next / Verify classify the gate; stderr-only Next failures emit JSON; sticky **FAILED** no longer shown for expected pauses; empty-output toasts include Preview / Retry / Cancel
- **Verify status layers** — each Publish step carries `verify_status` (`disk` · `local_cli` · `operator_cli` · `human_attest`); Desktop Offline / Watch / Publish hint explain graduated status checks without Studio-held secrets ([verify-status-layers-design](./specs/backend/verify-status-layers-design.md))
- **Workflow stage flow** — Dashboard cards (Sign only · Sign and deploy · Publish to platforms · Deploy focus) set mode/intent and open Publish as a linear stage pager (checkpoint rail · guideline · Back/Next); Stages/List toggle restores progress bands ([workflow-stages-design](./specs/frontend/workflow-stages-design.md))
- **Publish progress clarity** — Desktop summary `N done · M required · K later · ~min` + grouped step bands (Done collapsed · Required · Advanced lanes); Dashboard Now mirrors counts ([publish-progress-clarity-design](./specs/frontend/publish-progress-clarity-design.md))

### Fixed

- **Verify / Watch honesty** — successful local Verify no longer auto-marks Human/OAuth/deploy gates Done (Confirm still required). Fixes MCP `ship_publish_watch` silently advancing Scopes when active scopes exist
- **Continue toast honesty** — no more `publish · done` when Continue only pauses at a human gate; copy says paused / Confirm next

### Changed

- **Docs — Hosting portal parity specs** — static audit + vendor tutorial review; tiered improvement plan ([investigation](./specs/backend/hosting-portal-parity-investigation.md) · [design](./specs/backend/hosting-portal-parity-design.md))
- **Docs — Platforms · Portal · Integrations** — product overview of current Desktop guide surfaces, objectives, and known shortcomings ([PLATFORMS-AND-PORTAL](./docs/product/PLATFORMS-AND-PORTAL.md)); reliability deferred to a design-first slice
- **Busy toast debounce** — “Busy — Cancel unlocks…” at most once per 8s (demo/status bar no longer stacks)
- **Status chip colors** — PENDING (amber) · DONE (emerald) · SKIPPED (slate) distinct on Publish/Launch lists
- **Output dock hidden by default** — statusbar **Preview** opens the large console; **Dock** restores the bottom strip when wanted
- **Publish navigates detail panels** — Needs Open / Open / step click / Continue-at-human-gate open Scopes · Env · Sign · … and stay there (no yank back to Publish)

## 0.2.3 — 2026-09-23

### Added

- **Publish Continue** — `shipctl publish continue [--chain N]` advances Auto/ready gates (Verify→Confirm→Next); stops at Human/Open. Desktop primary **Continue** (+ Dashboard mid-flight CTA) · Confirm/Next remain for explicit control · `scripts/publish-fast.sh|.ps1` · first-run Desktop intent defaults to **Local** ([publish-fast-path-design](./specs/frontend/publish-fast-path-design.md))

## 0.2.2 — 2026-09-23

### Fixed

- **Publish Confirm/Next gating** — Pending steps make Confirm available and disable Next until Done
- **Doctor Auto when tools ok** — new plans mark doctor Done so the spine opens on the first real gate
- **Dashboard mid-publish copy** — “local already deployed” only when intent is Local
- **Human Next errors** — toast “Confirm this step first” instead of raw CLI FAILED for pending gates

## 0.2.1 — 2026-09-22

### Fixed

- **Silent shipctl / git spawn** — Windows `CREATE_NO_WINDOW` on Desktop background runs so page loads / Verify / doctor no longer flash a console (Open / Run still opens an intentional terminal)
- **Installer shipctl preference** — installed / portable sidecar beside the Desktop exe (or under `resources/`) always wins over a newer workspace build by mtime
- **Nav paint before shipctl** — Publish refresh and related-page loads wait for a paint frame before invoking shipctl
- **Output mirror weight** — Output page mirrors only the last ~120KB of the log for snappy opens (full stream remains in the live pane)

### Added

- **Release v0.2.1** — Windows x64 NSIS installer + portable zip + SHA256 (demo-ready cut)

## 0.2.0 — 2026-09-22

### Planned (follow-up on this track)

- **In-app update check** — Desktop notices newer GitHub Releases without silent auto-force install

### Fixed

- **Desktop nav freeze** — `setView` no longer rebuilds sidebar integrations or runs full identity sync on every page switch; inactive views get `content-visibility` / contain (see `specs/frontend/desktop-nav-performance-audit.md`)
- **Desktop sidebar scroll** — thin trackless themed scrollbar on `.nav` (replaces OS chrome on long Targets lists)

### Added

- **Release v0.2.0** — Windows x64 **NSIS installer** + portable zip (`ship-studio-desktop.exe` + `shipctl.exe`) + SHA256 on GitHub Releases
- **S0.7 Windows installer** — `tauri build` NSIS (`currentUser`); `shipctl` bundled under `resources/`; `pnpm desktop:installer` / `scripts/stage-desktop.sh --installer`

## 0.1.0 — 2026-09-14

### Added

- **Release v0.1.0** — Windows x64 portable zip (`ship-studio-desktop.exe` + `shipctl.exe`) + SHA256 on GitHub Releases
- **CI / Doctor** — Signet is hard-required only when `signet.toml` exists (Tauri-without-init is a note). Unblocks `shipctl doctor --project .` on ubuntu CI for this monorepo; `fixtures/ci-smoke` available for tighter smoke later
- **Website S0.4 / S0.5** — `/pricing` paid delta ($29 vs OSS) + hero disambiguation vs unrelated ship.studio
- **Polar portal entry URLs** — dashboard / credentials / env Open to distinct pages (OAT + webhook docs, or org deep links when `POLAR_ORGANIZATION_SLUG` is in `.env`)
- **Desktop Stripe icon** — symbol uses the same 16px sidebar and 22px card slot as the other vendors (the wide wordmark box was shifting the row and shrinking the mark)
- **Desktop shell boot** — debug window stays hidden until Vite answers; otherwise a branded waiting shell (titlebar + retry) replaces Edge `ERR_CONNECTION_REFUSED`, then returns to the UI when port 1420 is up
- **Desktop modularization M1** — extract `types` · `util` · `icons` · `integrations-data` · `constants` from `main.ts` (see `specs/frontend/desktop-modularization-design.md`)
- **Desktop local icons** — sidebar targets and integration wizards use bundled SVGs in `apps/desktop/src/public` (no runtime icon fetch)
- **Desktop sidebar** — Ship / Targets (monorepo scopes) / Integrations shortcuts / More · click a target to include it in deploy
- **Desktop Integrations** — Advanced nav section with payment wizards (Polar · Stripe · Gumroad · Lemon · Paddle) and Resend email checklist; Open dashboard only
- **Desktop independent options** — Dashboard `#now-quick` (Human / Portal / Env / Set up Polar) · clickable detect chips · CmdK “Set up Polar” (Advanced + Public)
- **Product website W4** — local license key file (`/license` · `scripts/issue-license.py`) + refund dogfood guide; success/account/refunds wired
- **Product website W3** — `/demo` silent GIF shelf (bind · Open · Confirm→Next · Output Preview) + `docs/assets/demo/v0.1.0` script
- **Product website W2** — Polar env-gated checkout · `/checkout/success|cancel` · `/account` portal · configurable refund window; `apps/website/docs/POLAR-SETUP.md`
- **Product website W1** — `/docs/*` renders repo Markdown (START-HERE · scope · product · human-gates · current)
- **Product website W0** — Astro site at `apps/website` (hero · pricing/Polar stubs · docs index · demo · refunds); `pnpm website:dev` / `website:build`
- **Desktop pnpm launch** — repo-root `pnpm dev` runs Tauri + Vite; VS Code “Desktop: pnpm dev” launch config
- **Docs shelf layout** — Obscur-style `docs/` TOC (`README` · `CURRENT` · shelves: product / dogfood / frontend / handoffs)
- **Launch legal / trust / graduate / release parity** — Guided Launch sequences LICENSE/SECURITY baseline, TRUST pack, graduate notes, `gh release list`, and desktop-cut honesty
- **Launch CI + container parity** — Guided Launch sequences `gh run list` (read-only) + local `docker build` / compose build; registry push stays Confirm
- **Launch DB + marketing + suite parity** — Guided Launch sequences DB provision / marketing landing / suite URL sync (Open + Confirm; no DNS or sibling `.env` writes)
- **Launch registry + HF listing parity** — Guided Launch sequences npm / crates.io dry-run Runs + Hugging Face Hub docs Open (live publish/upload stays Confirm)
- **Alt host expand (Cloud Run / Azure Static)** — Advanced + Launch `host.cloudrun` / `host.azurestatic` Open dashboard URLs
- **Launch mobile / store submit parity** — Guided Launch sequences Play / App Store / Microsoft Store Open + Confirm (no upload; Tauri skips Play)
- **Launch marketplace submit parity** — Guided Launch sequences Steam / itch / Epic listing + submit docs (Open + Confirm; no upload)
- **Launch host / BaaS parity** — Guided Launch sequences alt-host dashboards + mobile BaaS provision (Open + Confirm)
- **Alt host expand (Heroku / Amplify)** — Advanced `host.heroku` / `host.amplify` Open dashboard URLs (deploy stays on vendor CLI/UI)
- **Launch commerce parity** — Guided Launch lists Gumroad / Lemon / Stripe / Paddle (Open + Confirm) beside Polar
- **Commerce portal catalog** — Gumroad / Lemon / Stripe / Paddle join Polar as `shipctl portal` providers (dashboard Open; no SKU creation)
- **Guide notes expand** — `shipctl guide` adds Publish step + host / BaaS / commerce / Watch notes (prefer Publish over Flow)
- **Doctor notes expand** — cut-readiness notes for alt hosts / mobile BaaS / Stripe·Paddle + optional flyctl/railway/doctl; Watch cue; missing host CLIs do not fail doctor ok
- **Assist notes expand** — checklist notes for alt hosts / mobile BaaS / Stripe·Paddle / marketplace submit + Watch cue on Publish step
- **Pulse cut hints expand** — Dashboard Now mid-publish detail cues for `host.*` / `baas.provision` / `submit.*` / commerce listings
- **Commerce expand** — Advanced `listing.stripe` / `listing.paddle` Open dashboard URLs + name-only secret catalog (no Payment Link creation)
- **MCP publish watch** — `ship_publish_watch` one-shot local Verify probe for agents (optional `auto_confirm`; no vendor HTTPS / stdio pollution)
- **TUI publish watch** — Publish screen `w` toggles local Verify poll (~15s); READY status when Confirm is safe (parity with CLI/Desktop Watch)
- **Alt host expand** — Advanced `host.render` / `host.digitalocean` Open dashboard URLs (deploy stays on vendor CLI/UI)
- **Marketplace submit parity** — Advanced `submit.itch` / `submit.epic` open butler / Epic publishing docs (Open + Confirm; no upload from bridge)
- **Alt host portal** — Advanced `host.fly` / `host.railway` Open dashboard URLs (deploy stays on Fly/Railway CLI/UI)
- **Steam submit portal** — Advanced `submit.steam` opens Steamworks depot/build docs (Open + Confirm; no Steam API upload)
- **Mobile BaaS portal** — Advanced `baas.provision` Open URL + Confirm for Firebase / Appwrite / Convex / mobile+Supabase Auth (no vendor HTTPS; Local/General omit)
- **Publish progress watch** — `shipctl publish watch [--once|--interval-secs|--auto-confirm]` local Verify poller; Desktop Publish **Watch** toggle toasts when Confirm is ready (portal nudge; no vendor HTTPS)
- **PowerShell Advanced dogfood** — `scripts/dogfood-advanced-publish.ps1` for Windows hosts without WSL bash
- **Desktop busy unlock** — Cancel/Clear always re-enable Publish Refresh; `run()` clears busy in `finally`; 90s stuck hint
- **Env entry URL honesty** — secret Open URLs match value source or put provider (Vercel-bound names no longer open Cloudflare tokens)
- **Personal / local ship intent** — `--intent local|public` (orthogonal to General/Advanced); Local omits hosted env/deploy/store lanes; Desktop Local/Public toggle; persists in `.ship/studio.json`
- **aperio dogfood** — desktop Tauri no longer pulls Play/Android official/submit lanes (MAS + Microsoft Store stay)
- **Archetype expansion** — PWA manifest / vite-plugin-pwa detection (pulse + assist); Advanced `listing.huggingface` for Hub docs + Confirm (no upload); extended release-surface archetypes + launch matrix doc
- **Scope of service** — canonical product boundary: detect → plan → sequence final-mile; vendors + humans finish irreversible work ([docs/product/SCOPE-OF-SERVICE.md](./docs/product/SCOPE-OF-SERVICE.md))
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

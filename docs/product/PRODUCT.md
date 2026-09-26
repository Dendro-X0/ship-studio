# Ship Studio — product contract

**Status:** Active — Client · MCP · CLI kernel  
**Updated:** 2026-09-25  

**Scope of service (who / what / boundary):** [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md)  
**Platforms · Portal · Integrations (Desktop guide surfaces):** [PLATFORMS-AND-PORTAL.md](./PLATFORMS-AND-PORTAL.md)

```text
GOAL:     Local shipping hub — Client for humans, MCP for agents, CLI as kernel;
          final-mile sign → release → deploy without vendor-doc theater
NOT:      Product docs/demo authoring · replace vendor UIs · store secrets ·
          finish OAuth/store review without the human · in-app coach theater
RUNTIME:  Local + offline-first (bridge never requires network; open/login/put are operator-initiated; Verify/Watch = graduated status layers, not Studio-held secrets)
SHELLS:   Client = Desktop (+ TUI) · MCP = shipctl mcp · CLI = shipctl kernel
PROOF:    cargo test -p shipctl · scripts/dogfood-advanced-*.sh|.ps1 · Desktop Publish Advanced
DONE:     Publish portal + adaptive doctor + Verify honesty + Continue fast path + final-mile cut order + graduate/container/CI Runs + registry dry-run + desktop_cut + scopes/env/listings/commerce (Polar/Gumroad/Lemon/Stripe/Paddle portal + listing) + assist + desktop shell + Local/Public ship intent + publish progress watch (CLI/Desktop/TUI/MCP) + mobile BaaS portal + Steam/itch/Epic submit + Fly/Railway/Render/DO/Heroku/Amplify/Cloud Run/Azure Static host portals + Launch companion parity + Pulse/Assist/Doctor/Guide notes for those lanes + Desktop Platforms catalog + provider wizards + Portal Open/Docs/Login-CLI terminal + reliability slices 0–4 + surface law (Client·MCP·CLI)
NOT YET:  Operator completes paste / graduate certs / live release / marketplace / registry push / deploy on vendor platforms (see OPERATOR-NEXT) · Client honesty bar for founders · Reliability Later · CDP dogfood
```

## Scope of service (summary)

| We provide | We do not provide |
|------------|-------------------|
| Detect project shipping layout | Product docs / demos / marketing GIFs |
| Adaptive Publish plan (General / Advanced × Local / Public) | Replacement for Cloudflare / Apple / Play / Gumroad UIs |
| Open/Run → Confirm → Next sequencing | Finishing OAuth, store review, or DNS without you |
| Safe local CLI Runs + vendor Open URLs | `docker push`, live npm/cargo publish, store API upload |
| Doctor / Pulse / Assist orientation | Secret values stored in `.ship/` |
| Optional encrypted vault **export** | Being your team password manager |

Full definition: [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · Human gates: [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)

## UX principle — minimal actions, one spine

Access a wide range of shipping functions through **few deliberate actions**. Ship Studio is a **portal and guide**, not a replacement for official providers. Prefer **Continue** / scripts for highly automatable gates; **Open → Confirm** wizards only for official-channel work. Publish is the integrated workflow. Operators pick a **workflow card** on the Dashboard (Sign only · Sign and deploy · Publish to platforms · Deploy focus), then follow a **linear stage pager** on Publish — one checkpoint, guideline, and primary action at a time ([workflow-stages-design](../../specs/frontend/workflow-stages-design.md)). The Publish surface should always answer: **what’s done**, **what’s required next**, and **what’s optional/later** ([publish-progress-clarity-design](../../specs/frontend/publish-progress-clarity-design.md)). Scopes, Env, Sign, Portal, Ritual, and Tools are **detail panels** opened from the current publish step (`desktop_view`), not competing start points.

**Modes:** **General** (default) — shortest publish plan + focused nav. **Advanced** — full OAuth/official-sign/listing plan + Assist/Launch/Portal/Ritual/Tools. See `specs/backend/studio-modes-design.md`.

**Fast path:** [publish-fast-path-design.md](../../specs/frontend/publish-fast-path-design.md)

**Operator profile:** indie / multi-repo (many OSS + a few commercial) — same tool across cuts, marketplaces, deploys; value vs SaaS starter kits is **practical release utility**, not another template. See [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md).

DO NOT: make operators reassemble the release from eight peer nav destinations · force Assist → Scopes → Env → Sign → Portal → Publish as the happy path · auto-Confirm OAuth/deploy/store gates · flatten every Advanced lane as equally mandatory · claim to replace Cloudflare / Polar / store consoles.
## Architecture

```text
┌─ Client (Desktop) ─┐     ┌─ Client (TUI) ─┐     ┌─ MCP (agents) ─┐
│  apps/desktop      │     │  shipctl tui   │     │  shipctl mcp   │
└────────┬───────────┘     └────────┬───────┘     └────────┬───────┘
         └────────────┬─────────────┴──────────────────────┘
                      ▼
                   shipctl (CLI kernel)
         guide · portal · secrets · configure · signet · orbit
```

Surfaces law: [SCOPE-OF-SERVICE — Delivery surfaces](./SCOPE-OF-SERVICE.md#delivery-surfaces-client--mcp--cli) · [surfaces-cli-tui-desktop.md](../../specs/backend/surfaces-cli-tui-desktop.md)

Design: `specs/backend/shipping-hub-north-star.md` · `specs/backend/provider-portal-design.md` · `specs/backend/paste-secret-assist-design.md` · `specs/backend/vault-export-design.md` · `specs/backend/guided-launch-design.md` · `specs/backend/studio-scopes-design.md` · `specs/backend/publish-portal-design.md` · `specs/backend/release-surface-map.md`

## Commands

| Command | Meaning |
|---------|---------|
| `doctor` | Check tools for **this** layout (Signet if desktop; Orbit or host CLI if Web/API) |
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
| Polar | `POLAR_*` / polar.sh markers | dashboard (no CLI OAuth) | dashboard · OAT docs · webhook docs (or `{slug}/products|settings|webhooks` when `POLAR_ORGANIZATION_SLUG` is set) |
| Gumroad | markets / `GUMROAD_*` | dashboard (no CLI OAuth) | app.gumroad.com |
| Lemon | markets / `LEMON_*` | dashboard (no CLI OAuth) | app.lemonsqueezy.com |
| Stripe | markets / `STRIPE_*` | dashboard (no CLI OAuth) | dashboard.stripe.com |
| Paddle | markets / `PADDLE_*` | dashboard (no CLI OAuth) | vendors.paddle.com |

## Desktop

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

**Primary spine:** Dashboard workflow cards → **Publish** (Stages pager · paced Continue · status probe).

**Provider guide surfaces** (detail panels, not competing starts): **Platforms** (Hosting · Official signing) · **Integrations** (Payments · Email) · **Portal** (Open settings · Docs · Login CLI). Objectives, current behavior, and known gaps: [PLATFORMS-AND-PORTAL.md](./PLATFORMS-AND-PORTAL.md).

Also: **Assist** checklist · **Launch** companion · **Human portal** sprint · Wizard · Ship · Guide · Secrets · Export vault · Ritual / Tools (Advanced).

## TUI

```bash
cargo build -p shipctl --release
./target/release/shipctl.exe tui --project .
# Home: Ship wizard · Guide · Portal · Secrets · …
```

## Offline / security

- **Offline-first bridge:** Studio does **not** call vendor HTTPS with secrets. Env, tokens, and OAuth stay **Open → paste/login → Confirm** on official UIs.
- **Status detection (graduated Verify)** — detect publish/sign readiness without becoming an online control plane ([verify-status-layers-design](../../specs/backend/verify-status-layers-design.md)):
  - **Disk** — project files / `.ship` metadata (never secret values)
  - **Local CLI** — Signet, doctor, deploy pulse / last-run
  - **Official CLI probe** — read-only `gh` / `wrangler whoami` / … only when you run **Verify** or **Watch**
  - **Human attest** — no automated green light; Confirm after you finish on the provider
- `guide` / `portal` / `secrets` JSON offline-safe; open/login/put are explicit.
- Secret values never stored in `.ship/` plaintext.
- Optional **vault export**: Argon2id + AES-256-GCM `kmvault` file (open in Clavis / Keys Manager). Passphrase via TTY or `SHIP_VAULT_PASSPHRASE`.
- Desktop **Offline** toggle = prefer offline-safe sign / refuse deploy — the **bridge** still does not require network; Verify/Watch remain operator-initiated status checks.

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

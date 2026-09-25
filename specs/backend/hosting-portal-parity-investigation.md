# Hosting portal parity — investigation

**Status:** Investigation (no code this band)  
**Updated:** 2026-09-25  
**Owner:** `crates/shipctl` (`portal` · `config` detect) + Desktop Platforms catalog  
**Product:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Parents:** [provider-portal-design](./provider-portal-design.md) · [platforms-catalog-design](../frontend/platforms-catalog-design.md) · [desktop-silent-failures-investigation](./desktop-silent-failures-investigation.md)

```text
QUESTION:  What do we claim to support for hosting, what does the code actually do,
           and what do official vendor docs imply we should open / run / document?
OUTCOME:   Evidence for a design slice — no Desktop/shipctl diffs in this file.
```

## Method

1. Static inventory of Desktop Platforms Hosting · `ProviderId` · detect · portal step builder.
2. Read official CLI / secrets / Pages tutorials for the Hosting set we plan to support in-product.
3. Classify gaps (parity · stale URL · duplicate Open · wrong mental model).

## Inventory (static)

### Three catalogs, three truths

| Surface | Hosting-related IDs |
|---------|---------------------|
| **Desktop Platforms** (`platforms-data.ts`) | Orbit · Cloudflare · Vercel · Netlify · GitHub Pages · Fly · Railway |
| **`shipctl portal` ProviderId** | Cloudflare · Vercel · Netlify · Github · Fly · Railway · Render · DigitalOcean · Heroku · Amplify · CloudRun · AzureStatic (+ non-host commerce/DB/BaaS) |
| **Publish `host.*` gates** (`launch.rs`) | Fly · Railway · Render · DigitalOcean · Heroku · Amplify · CloudRun · AzureStatic |

**Mismatch A — Desktop vs portal:** Platforms lists Orbit (no `ProviderId`) and omits Render / DO / Heroku / Amplify / Cloud Run / Azure SWA even though portal + Advanced Publish already know them.

**Mismatch B — GitHub Pages vs GitHub auth:** Platforms card `github-pages` sets `provider: "github"`, so Portal steps are PAT + Actions secrets — not Settings → Pages.

**Mismatch C — Orbit:** Intentionally no portal id (avoids `unknown provider 'orbit'`). Open = GitHub Orbit repo; deploy via Ritual / Tools.

### Portal step shapes (code)

`plan_for_providers` always emits, per provider:

| When | Steps |
|------|--------|
| `oauth_cli` non-empty | oauth → token_page → (recover if create≠token) → env |
| `oauth_cli` empty | **dashboard** → token_page → (recover…) → env |

Kind-aware `entry_url` / `docs_url` today are rich mainly for **Cloudflare · Vercel · Netlify · Github · Polar · D1**. Everyone else falls back to `cat.token_url` / `cat.docs_url` for env Open — so Fly/Railway often open the **same dashboard URL three times** (dashboard · token · env).

### Detect signals (`config::probe`)

| Provider | Signal (summary) |
|----------|------------------|
| Cloudflare | `wrangler.toml` / `.json` / `.jsonc` (incl. one-level nested) |
| Vercel | `vercel.json` / `.vercel` |
| Netlify | `netlify.toml` / `.netlify` |
| GitHub | `.git` or env hints |
| Fly | `fly.toml` / `.fly` / package mention |
| Railway | `railway.toml` / `.json` / `.railway` / package mention |
| Render…Azure | analogous file / dir / package heuristics |

Desktop Platforms does **not** yet highlight the card matching Pulse/detect (known gap).

### Desktop wiring

- Cloudflare / Vercel / Netlify / GitHub Pages → `openPortalProvider(wiz.provider)`.
- Fly / Railway / Orbit → Open dashboard only (`provider` unset on Fly/Railway).
- Login CLI terminal only when portal step kind is oauth and CLI is present.

## Vendor docs review (Hosting set we plan to guide)

Sources fetched 2026-09-25. Studio remains Open / Docs / Login CLI — never vendor HTTPS with secrets.

### Cloudflare (Workers / Pages)

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Auth | Prefer `wrangler login` OAuth; API tokens secondary | Match — `oauth_cli: wrangler login` |
| Tokens UI | Profile API tokens | `dash.cloudflare.com/profile/api-tokens` |
| Token docs | Create-token fundamentals | OK |
| Secrets | `wrangler secret put`; Workers secrets docs | Env Open → Workers & Pages; Docs → workers secrets — **good after recent fix** |
| Install | Prefer local `npx wrangler` | Login CLI assumes PATH `wrangler` — honesty gap for operators without global install |

Docs: [Wrangler commands](https://developers.cloudflare.com/workers/wrangler/commands/) · [Workers secrets](https://developers.cloudflare.com/workers/configuration/secrets/) · [Create API token](https://developers.cloudflare.com/fundamentals/api/get-started/create-token/)

### Vercel

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Auth | `vercel login`; CI uses `VERCEL_TOKEN` | Match OAuth path |
| Deploy loop | `vercel link` → `env pull` → `deploy` / `--prod` | Wizard copy mentions login + env; does not name **link** as first-class |
| Env | `vercel env add` / dashboard Settings → Env | Env hint OK; env Open = dashboard (coarse vs project Settings) |
| Token docs URL | Tokens live under account settings; REST “creating token” anchors move | `docs_url` = `vercel.com/docs/rest-api#creating-an-access-token` — **likely stale / fragile** |

Docs: [CLI overview](https://vercel.com/docs/cli) · [Deploy from CLI](https://vercel.com/docs/projects/deploy-from-cli) · [Environment variables](https://vercel.com/docs/projects/environment-variables)

### Netlify

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Auth | `netlify login` or PAT (`NETLIFY_AUTH_TOKEN`) | Match |
| PAT UI | User settings → Applications → Personal access tokens | Hash URL present |
| CLI get-started | **Moved** under `/api-and-cli-guides/cli-guides/get-started-with-cli/` | Catalog `docs_url` still `docs.netlify.com/cli/get-started/#authentication` — **stale** |
| Env Docs | Env get-started / overview paths reorganized | `step_docs_url(env)` → `/environment-variables/overview/` — **verify / replace** |
| Deploy | `netlify deploy --prod`; site id via link / `NETLIFY_SITE_ID` | Wizard honest; portal does not surface site-link step |

Docs: [Get started with CLI](https://docs.netlify.com/api-and-cli-guides/cli-guides/get-started-with-cli/) · [Env get started](https://docs.netlify.com/build/environment-variables/get-started)

### GitHub Pages (≠ GitHub PAT)

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Publish | Repo **Settings → Pages**; branch or Actions workflow | Platforms Open = `https://github.com` (too shallow) |
| Auth for CLI/CI | `gh auth login` / PAT | Portal via `provider: github` — **wrong primary story for Pages lane** |
| Docs | Creating a Pages site · publishing source | Not linked from Platforms wizard |

Docs: [Creating a GitHub Pages site](https://docs.github.com/en/pages/getting-started-with-github-pages/creating-a-github-pages-site)

### Fly.io

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Auth | `fly auth login` (browser) | `oauth_cli: []` — **no Login CLI** |
| Secrets | `fly secrets set` / list (values never readable) | Env hint names `fly secrets set`; Open URL = dashboard for all steps |
| Launch | Hands-on launch-app docs | `docs_url` OK |
| Tokens | Access tokens for CI (`--access-token` / dashboard tokens) | `token_url` = `fly.io/dashboard` — **not token page**; dashboard/token/env Open collide |

Docs: [fly auth login](https://fly.io/docs/flyctl/auth-login/) · [Secrets](https://fly.io/docs/apps/secrets/) · [Launch app](https://fly.io/docs/hands-on/launch-app/)

### Railway

| Concern | Official posture | Our catalog today |
|---------|------------------|-------------------|
| Auth | `railway login` (browser / `--browserless`) | `oauth_cli: []` — **no Login CLI** |
| Tokens | `RAILWAY_TOKEN` (project) vs `RAILWAY_API_TOKEN` (account) — **mutually exclusive** | once_hint vague; no distinction |
| Token UI | Account Settings → Tokens vs project token | `token_url` = dashboard — **coarse** |
| Docs | CLI login + deploying pages | `docs_url` = `docs.railway.com/` root |

Docs: [CLI login](https://docs.railway.com/cli/login) · [CLI deploying](https://docs.railway.com/cli/deploying)

### Orbit (Studio-adjacent)

Not a public SaaS host dashboard. Detect via Orbit config; Desktop Open = Orbit GitHub. Portal exclusion is correct; wizard must keep saying Studio does not upload.

## Failure / gap classes (hosting-specific)

| ID | Class | Evidence |
|----|-------|----------|
| H1 | **Duplicate Open URLs** on dashboard-only hosts | Fly/Railway: dashboard ≡ token ≡ env Open |
| H2 | **Missing Login CLI** where vendor has interactive login | Fly `fly auth login`; Railway `railway login` |
| H3 | **Stale / fragile docs_url** | Netlify CLI path; Vercel REST hash |
| H4 | **Catalog set drift** | Platforms ≠ portal hosts ≠ Publish `host.*` |
| H5 | **Wrong mental model** | GitHub Pages → GitHub PAT portal |
| H6 | **Detect → UI** | Pulse lights do not select Platforms card |
| H7 | **PATH / install honesty** | Wrangler/Vercel often project-local (`npx`); terminal assumes global binary |

Non-goals for this investigation: Polar E2E, CDP dogfood, Desktop Tools/Env Put terminal reliability (separate reliability design).

## Recommended next artifact

Design: [hosting-portal-parity-design.md](./hosting-portal-parity-design.md) — tiered host model, URL matrix, Desktop catalog policy, ordered slices + L1/L2 proof.

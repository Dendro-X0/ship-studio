# Hosting portal parity — design

**Status:** Slices 0–4 shipped (working tree) — hosting parity band complete; park/commit or later Tier D  
**Updated:** 2026-09-25  
**Investigation:** [hosting-portal-parity-investigation.md](./hosting-portal-parity-investigation.md)  
**Product:** [PLATFORMS-AND-PORTAL.md](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Owners:** `portal.rs` · `platforms-data.ts` · (later) pulse → Platforms highlight

```text
GOAL:     Hosting lanes in Desktop + shipctl portal match vendor reality:
          distinct Open · Docs · Login CLI; honest detect; no fake deploy.
NOT:      Studio-held secrets · vendor HTTPS · one-click deploy · full cloud console clone
```

## North star — host tiers

| Tier | Hosts | Studio promise |
|------|-------|----------------|
| **A — Orbit deploy path** | Cloudflare · Vercel · Netlify | Detect → OAuth Login CLI → token fallback → env Open≠Docs → Publish Confirm |
| **B — CLI host (guide)** | Fly · Railway | Detect → Login CLI → secrets docs · dashboard Open · Publish `host.*` Confirm |
| **C — Static / Pages** | GitHub Pages | Settings→Pages Open + Pages docs; **do not** default to PAT portal |
| **D — Advanced host (portal+Publish, optional Platforms)** | Render · DigitalOcean · Heroku · Amplify · Cloud Run · Azure SWA | Same shape as B when activated; keep Advanced Publish `host.*` even if Desktop catalog stays slim |
| **E — Local Orbit** | Orbit | No portal id; Ritual/Tools deploy; Open = Orbit project docs/repo |

**Desktop Platforms Hosting (v1 product list):** keep A+B+C+E visible. Tier D stays portal/Publish until a later catalog expand (document in PLATFORMS-AND-PORTAL).

## Invariants (unchanged)

1. Bridge never calls vendor HTTPS with secrets.
2. Open = settings / console UI; Docs = official tutorial; Login CLI = interactive terminal only.
3. Prefer CLI OAuth over long-lived tokens when the vendor documents it.
4. Duplicate `entry_url` across steps for one provider is a bug (see Cloudflare env split).
5. Desktop actions map to `shipctl portal` / detect / Publish — no parallel secret state.

## Catalog matrix (target)

| Provider | Detect | Login CLI | Open (settings) | Docs (primary) | Env Open | Env Docs |
|----------|--------|-----------|-----------------|----------------|----------|----------|
| Cloudflare | wrangler.* | `wrangler login` | API tokens **or** Workers&Pages for env | create-token / workers secrets | Workers & Pages | workers secrets |
| Vercel | vercel.json / `.vercel` | `vercel login` | account tokens | [CLI](https://vercel.com/docs/cli) + env vars doc | project env (dashboard OK if no deep link) | projects/environment-variables |
| Netlify | netlify.toml / `.netlify` | `netlify login` | PAT applications | [CLI get-started](https://docs.netlify.com/api-and-cli-guides/cli-guides/get-started-with-cli/) | app.netlify.com (site) | [env get-started](https://docs.netlify.com/build/environment-variables/get-started) |
| GitHub (auth/CI) | `.git` | `gh auth login` | tokens / new | PAT docs | Actions secrets | Actions secrets guide |
| GitHub Pages | marketing/pages heuristics | none (or gh only if already github) | **repo Settings → Pages** (best-effort URL) | Creating a Pages site | n/a or Actions secrets if workflow | Pages / Actions docs |
| Fly | fly.toml / `.fly` | `fly auth login` | dashboard **or** account tokens page | launch-app | dashboard apps (or secrets UI) | [Secrets](https://fly.io/docs/apps/secrets/) |
| Railway | railway.toml / `.railway` | `railway login` | account tokens page | [CLI login](https://docs.railway.com/cli/login) | project / variables UI | CLI deploying / variables docs |
| Orbit | orbit config | — | Orbit GitHub / docs | Orbit README | — | — |

Implementers: prefer **stable** vendor URLs from the investigation; if a deep link 404s, fall back to dashboard + accurate Docs (never three identical Opens).

## Structural code changes (design intent)

### 1. Provider catalog fields

Extend `ProviderCatalog` (or parallel host profile) so dashboard-only hosts are not forced through the “token_page = same dashboard” template:

- `oauth_cli` for Fly / Railway (and later Tier D where CLI login exists).
- Distinct `token_url` vs `create_url` only when the vendor has a real tokens page; otherwise **omit token_page** (dashboard + env only).
- Kind-aware `env_entry_url` / `step_docs_url` for Fly secrets and Railway variables (mirror Cloudflare/Vercel).

### 2. GitHub Pages lane

- Platforms: stop wiring Pages solely as `provider: "github"` for Portal steps.
- Options (pick one in slice 2):
  - **A:** New portal kind / filter `github_pages` that emits Pages settings + docs steps only; or
  - **B:** Platforms Pages uses Open+Docs only (no Portal steps); GitHub PAT stays under Integrations/Tools or separate “GitHub auth” card.
- Prefer **B** for smallest honesty win; **A** if we need `shipctl portal --provider …` for Pages.

### 3. Desktop catalog policy

- Document Tier D as “Advanced Publish / `shipctl portal --provider`” until Platforms expands.
- Fly/Railway: set `provider: "fly" | "railway"` so Portal steps + Login CLI work once catalog has OAuth.
- Optional: `deployArgs` hints remain Ritual-only where Orbit does not deploy that host.

### 4. Detect → highlight (later slice)

Pulse/detect → select matching Platforms card (wrangler → Cloudflare). No auto-Done.

## Slices (ordered)

| Slice | Change | Proof |
|-------|--------|-------|
| **0 — Docs/URL hygiene** | Fix stale Netlify `docs_url` + env docs; harden Vercel docs_url to stable CLI/env pages; spot-check Cloudflare/GitHub | **Done** — L1 portal tests |
| **1 — Fly + Railway parity** | `oauth_cli`; omit Fly token_page; Railway account tokens Open ≠ dashboard env; env Docs; Desktop `provider` | **Done** — L1 portal tests |
| **2 — GitHub Pages honesty** | Pages Open/Docs without PAT portal conflation | **Done** — option B: no `provider`; Docs + Open GitHub |
| **3 — Catalog / product doc sync** | PLATFORMS-AND-PORTAL + Platforms design: Tier A–E; Tier D Advanced-only | **Done** — docs/spec sync |
| **4 — Detect → highlight** | Pulse lights Platforms card | **Done** — Choose host / detect chips → preferred Hosting card; soft-select on Platforms view |
| **Later** | Tier D Platforms cards; `npx`/local CLI honesty; project-deep env links | Separate band |

**Do not** fold Desktop Tools/Env Put/Ritual terminal reliability into these slices — that remains the reliability design band.

## Out of scope

- Polar / commerce E2E
- CDP / CodaCtrl without Tauri remote-debug recipe
- Studio auto-detecting OAuth success
- Replacing Orbit deploy for Cloudflare/Vercel/Netlify

## Proof plan (per activated slice)

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p shipctl portal::tests` · desktop `tsc` |
| L2 | Harbor Public: Cloudflare Open ≠ Docs; Login CLI terminal |
| L2 | Temp dir with `fly.toml`: `shipctl portal --provider fly` → oauth CLI present; env Docs = secrets URL; no triple identical Open |
| L2 | Platforms GitHub Pages: no `portal failed` / no PAT-only checklist as the primary story |

## Activation

Handoff **Next Atomic Step** names the slice (start with **0** or **0+1**). No code until that quote exists.

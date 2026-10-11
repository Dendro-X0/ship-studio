# Host deploy — env import (secure)

**Status:** Slice 1–4 shipped (CLI + Desktop confirm + monorepo packs)  
**Updated:** 2026-10-07  
**Owner:** `secrets` · `hostdeploy` · Desktop Deployment  
**Parents:** [paste-secret-assist](./paste-secret-assist-design.md) · [hosted-deploy-ops-value-bar](./hosted-deploy-ops-value-bar-design.md) · [host-xor-value-ux-polish](./host-xor-value-ux-polish-design.md) · [SCOPE](../../docs/product/SCOPE-OF-SERVICE.md)  
**Trigger:** Deploy already streams CLI + project name; operators want bulk env / `.env` import **before or beside** Deploy, with clear privacy confidence.

## Plan alignment

```text
HANDOFF ATOMIC STEP: Host XOR polish (env import is a depth bet under Deploy)
PAUSED / CANCELLED:  Studio cloud secret custody · CDP Create-Token fill ·
                     Integrations product · vendor coach encyclopedia
CANONICAL OWNER:     secrets::import_env_to_host (+ Desktop Deployment CTA)
PROOF THIS SLICE:    L1 no-value-in-.ship · L2 CF/Vercel/Netlify put from fixture .env
```

## Goal

Let the operator **push environment variables to the selected host** from a local file or selected NAME=value rows, using the **same vendor CLI path** Put already uses — without Orbit Yard becoming a password manager or logging values.

**Job words (General):** “Add secrets to host” · “Import from .env”  
**Done:** Host has the selected names (vendor authority); Studio shows **names + ok/fail counts only**.

## What exists today

| Surface | Behavior |
|---------|----------|
| **Put secrets** | One NAME → interactive `wrangler secret put` / `vercel env add` / `netlify env:set` (stdin inherit; human types value) |
| Hint discovery | Empty keys in `.env` / `.env.local` / `.dev.vars` + wrangler `# Secrets:` — **names only** |
| Deploy | Streams `hostdeploy`; does **not** import env |
| Invariant | Never write secret **values** to `.ship/` or Output |

## Allowed model (recommend)

### Path A — Import local file → vendor CLI (default)

```text
Operator picks file (.env / .dev.vars / .env.local / custom path on disk)
  → Studio parses locally (in-process)
  → Operator confirms NAME checklist (values masked in UI)
  → For each selected NAME: spawn vendor put; value on stdin only (never argv)
  → Output / last-run: names + exit codes only
  → Clear parse buffer when done
```

| Trust claim (UI copy) | Mechanism |
|-----------------------|-----------|
| “Values stay on this PC” | No Studio cloud; no vendor HTTPS from bridge |
| “Not saved in Orbit Yard” | Never write values to `.ship/`, vault unless operator later exports |
| “Host receives them via their CLI” | Same binaries as Login CLI / Deploy (`wrangler` / `vercel` / `netlify`) |
| “Output never shows values” | Redact; log `PUT NAME ok|fail` only |

### Path B — Louder link to project env (thin slice)

Detect `.env` / `.dev.vars` next to the host root; list **names** (and which are empty); primary CTA still Path A or single Put. No new custody.

### Path C — Ephemeral in-app form (later / higher risk)

Type NAME=value in Desktop memory → flush to CLI → wipe. Harder to prove safe (IPC, crash dumps, screenshots). **Defer** until Path A ships and dogfood asks for it.

## Explicitly reject

| Idea | Why |
|------|-----|
| Upload `.env` to Orbit / Netlify / any Studio server | Custody product; out of SCOPE |
| Store values in `.ship/publish.json` or last-run | Forbidden invariant |
| Pass values on CLI argv (`secret put NAME=value`) | Leak via process list |
| Echo values into Output / MCP JSON | Agent + log leak |
| Auto-import every key on Deploy without confirm | Surprise production secrets; trust death |
| “Studio syncs env across hosts” | Password-manager pitch |

## Provider matrix (Tier A first)

| Host | Put mechanism | Notes |
|------|---------------|-------|
| Cloudflare | `wrangler secret put <NAME>` per secret | Workers secrets; Pages may need `wrangler pages secret` / dashboard — detect layout |
| Vercel | `vercel env add <NAME> <env>` | Need target (production/preview/development); default production for Public Deploy |
| Netlify | `netlify env:set <NAME> <value>` | Prefer stdin/file form that does not print value; verify CLI flags |
| Self-host | Copy into local `.env` / compose only if operator opts | Not “host secrets”; different copy |
| Fly / Railway | Guide Open + their CLI later | No fake parity in slice 1 |

## CLI / MCP contract (proposed)

```text
orbityard secrets import-env --project . --provider cloudflare \
  --file .dev.vars [--dry-run] [--name KEY]...
```

- `--dry-run`: list NAMES that would be put (never values).  
- Default: interactive confirm of names.  
- MCP: `yard_env_import` may pass **file path + name list only**; values must not appear in tool args — Desktop/TTY preferred (same as `yard_env_put`).

JSON result (safe):

```json
{
  "schema": "orbit-yard/secrets-import/v1",
  "provider": "cloudflare",
  "file": ".dev.vars",
  "put": [{"name": "API_TOKEN", "ok": true}],
  "skipped": [{"name": "LOCAL_ONLY", "reason": "unchecked"}],
  "notes": ["Values were not logged."]
}
```

## Desktop UX (Deployment panel)

After host selected (Cloudflare in screenshot):

1. Keep **Deploy** as primary green when auth ok.  
2. Add secondary **Import env…** (peer of Put secrets — or merge: Put secrets menu → “One name” | “From .env file”).  
3. Modal: file picker (defaults: `.dev.vars`, `.env`, `.env.local` under host work dir) → checklist of names with masked values (`••••`) → **Push to host** → progress by name.  
4. Trust strip (always visible): three bullets from the table above.  
5. Optional: “Then Deploy” continues to existing confirm/name dialog.

**U2:** Do not make Import equal to Deploy. Deploy remains the outcome action; Import is prep.

## Security checklist (must pass before “implemented”)

1. Grep proof: no write of value bytes under `.ship/`.  
2. Output scrubber: if vendor CLI echoes value, treat as soft-fail + warn (do not mirror raw line).  
3. File read: only paths operator picked; refuse network URLs.  
4. Skip empty values; skip comments; support `export KEY=`.  
5. Optional deny-list for local-only names (`*_LOCAL`, `PATH`, …) with override.  
6. Never attach import payload to MCP success blobs.

## Slice plan

| Slice | Scope |
|-------|-------|
| **0** | This design + never-say / trust copy locked — **done** |
| **1** | `secrets --import-env` Tier A + L1 tests — **done** |
| **2** | Desktop **Import env…** confirm modal — **done** |
| **3** | Optional “Deploy after import” — **done** (MCP path-only tool still later) |
| **4** | **Monorepo env-pack picker** — `--list-packs` / `--pack` · Desktop grid — **done** |
| **5** | **Deploy all-in-one** — target + env pack in Deploy dialog · import-then-deploy · stale CLI toast — **done** |
| **6** | **Rigor** — import provider = Deploy card; CLI preflight `[missing_cli]`; soft-fail → Deploy without import; host mismatch warning — **done** |
| **7** | **Affinity + mismatch CTA + name confirm on Deploy** — A1–A3 — **done** |
| **8** | **Tier B Put/import** — Fly · Railway only — **done** |
| **9** | **Dogfood opacity** — `--host-name` → Pages `--project-name`; scrubbed put `error` in report; mismatch Switch on fail toast — **done** |
| **10** | **Import polish** — strip ANSI from put errors; Pages create-if-missing before put; clearer Deploy-first toast — **done** |
| **11** | **Empty-key honesty** — label `empty in .env` + ready/empty counts (not “broken” checkboxes) — **done** |

## Slice 10 — Import polish (dogfood round 2)

| Symptom | Root cause | Fix |
|---------|------------|-----|
| Toast shows `[31mX` / `[41;31mERROR` | wrangler color codes left in stderr scrub | Strip CSI/ANSI in `scrub_cli_detail` |
| `Project "ship-studio" does not exist` on every PUT | Pages secrets need an existing project; Deploy creates later | Ensure `wrangler pages project create` once before Pages puts (hostdeploy parity) |
| Classifier missed missing project | Error omits the word `pages` | Broaden `is_missing_pages_project` |
| Operator still on CF+Netlify pack | Soft warn only | Keep Switch CTA; missing-project toast prefers Deploy without import |

## Slice 9 — Dogfood (opaque CF put fail)

| Symptom | Root cause | Fix |
|---------|------------|-----|
| `PUT … fail` with no reason | stderr discarded (`Stdio::null`) | Capture + scrub into `ImportPutResult.error` + stderr line |
| CF Deploy name `ship-studio` but Pages put used folder `website` | Pages argv used workdir basename | `--host-name` / Desktop Deploy name → `--project-name` |
| Operator pushed Netlify pack to CF anyway | Warn + Import off, but re-check allowed | Names-step banner + fail toast **Switch to {hint}** |

## Slice 7 — Affinity · Switch Deploy · name confirm (A1–A3)

### A1 — Affinity defaults

When opening Deploy dialog for provider `P`:

1. Prefer target scope whose `provider` / colocated signal matches `P` (Netlify → website with `netlify.toml`; Cloudflare → wrangler/API).
2. Prefer env pack with `host_hint == P` under that target.
3. Keep `lastDeployFocusProject` as tie-break only when it already matches `P` affinity; do not stick on a Netlify website focus when the card is Cloudflare.

### A2 — Mismatch CTA

If pack `host_hint` ≠ Deploy card and hint is a Put-capable host: show warn + button **Switch to {hint} Deploy** → close dialog → open that host’s Deploy dialog (same target/pack selection). Do **not** silently change import provider (slice 6).

### A3 — Name checklist on all-in-one

When Import is checked on Deploy confirm: dry-run → name checklist (reuse Import env… list) → Push selected → then Deploy. Skipping checklist (push all nonempty) is removed for the all-in-one path.

### Proof (slice 7)

| Layer | Check |
|-------|--------|
| L1 | N/A (Desktop) |
| L2 | Open Cloudflare Deploy on ship-studio → default target/pack is API/wrangler-ish or Import off; Netlify pack shows Switch to Netlify |
| L2 | Import checked → name list before put |

## Slice 6 — Rigor (failure taxonomy)

### Lessons from dogfood

| Symptom | Root cause | Rule |
|---------|------------|------|
| `unknown --pack apps/website/.env` | Import ran under Deploy focus cwd | Always `--file` from monorepo root |
| `netlify … program not found` + Deploy skipped | Auto-switched import to pack host; hard-failed Deploy | Import uses **Deploy card** provider; missing CLI is soft |
| Cloudflare Deploy + Netlify pack toast | Pack hint ≠ card | Warn in dialog; default import off on mismatch |

### Invariants

1. **One host for the run** — Import provider = selected Deploy card (`cloudflare` / `vercel` / `netlify`). Do not silently redirect to `host_hint`.
2. **Preflight** — Before any put: vendor CLI on PATH (`wrangler` / `vercel` / `netlify`). Fail with `[missing_cli]` — same class as hostdeploy auth-check.
3. **Soft-fail Deploy** — Import failure never blocks Deploy without a recovery path. Desktop offers **Deploy without import**.
4. **Mismatch honesty** — If pack `host_hint` ≠ Deploy card, show warning and default **Import** unchecked (operator can still check it).
5. **Values** — Still never in `.ship/` / Output.

### CLI

Before puts (non-dry-run): resolve via same helper as hostdeploy (`resolve_auth_cli`) → on miss `bail!("… [missing_cli]")`. On Windows prefer `*.cmd` / `*.exe` over the extensionless npm shim (avoids CreateProcess hang). Each put child times out (~45s) so a stuck vendor CLI soft-fails instead of freezing Desktop.

### Desktop Deploy confirm

```text
mismatch? → warn + import unchecked by default
import checked?
  → preflight / import
  → missing_cli | fail → toast + [Deploy without import] [Preview]
  → ok → Deploy
else → Deploy
```

## Slice 8 — Tier B Put / import (Fly · Railway)

### Matrix

| Host | Put mechanism | Notes |
|------|---------------|-------|
| Fly | `fly secrets set NAME=-` (+ `--stage` on bulk import) | stdin until EOF; workdir near `fly.toml` |
| Railway | `railway variable set NAME --stdin` (+ `--skip-deploys` on bulk) | Prefer stdin; never log values |

Same invariants as Tier A: local parse, names-only JSON, no `.ship/` values, `resolve_auth_cli` + `[missing_cli]`, put timeout, soft-fail where Deploy exists.

### Desktop

- `providerHasEnvPut` includes `fly` / `railway`.
- **Import env…** on those cards; **Deploy after import** only for Tier A hosted CLI Deploy cards.
- Pack `host_hint`: `fly.toml` → fly; `railway.toml` / `railway.json` / `.railway` → railway.

### Never-say

- “We sync env across clouds”
- “Fly/Railway Deploy all-in-one” (unless hostdeploy stream exists)

### Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: dry-run Fly/Railway lists names; missing CLI tag; fixture `fly.toml` pack hint |
| L2 | Desktop: Import env on Fly card → name confirm → Push (throwaway app) |

### Reject

Render / DO / Heroku / Amplify / Cloud Run / Azure in this slice; whole-file `fly secrets import` without name confirm; Studio HTTPS put; CF→Fly copy.

## Slice 4 — Monorepo / multi-`.env` (optimal UX)

### Problem (today)

`discover_env_import_files` only looks at the bound root (+ wrangler dir) and `resolve_import_file` takes the **first** hit. In ship-studio that misses `apps/website/.env` when the bound project is the monorepo root — or silently picks the wrong pack when both `apps/api/.dev.vars` and `apps/website/.env` exist.

### Principle (same spine as Targets Deploy)

```text
One ship unit at a time.
Env pack → Host → Confirm names → Push → (optional) Deploy that same unit.
Never merge every .env in the repo onto one host without an explicit multi-select.
```

| Wrong | Right |
|-------|-------|
| Dump all keys from all apps into Netlify | Push **website** pack → Netlify (Deploy focus `apps/website`) |
| Auto-pick first `.env` found | Show a **pack grid**; default = Deploy focus / selected host affinity |
| Separate “env product” nav | Stay inside **Import env…** dialog (step 0 → step 1) |

This **complements** hosts: each app’s secrets land on the host that serves that app; Studio does not become a cross-host env sync platform.

### Env pack object (detect)

Walk **scopes** (same roots as `orbityard scopes`: `apps/*`, api/web/desktop heuristics) plus bound root. For each directory, collect files:

`.dev.vars` · `.env` · `.env.local` · `.env.production` (optional) — skip `.env.example` (names-only template, usually empty values).

```json
{
  "id": "website-env",
  "label": "Website",
  "kind": "web",
  "rel_dir": "apps/website",
  "file": "apps/website/.env",
  "host_hint": "netlify",
  "key_count": 8,
  "nonempty_count": 3
}
```

- **label / kind** from scopes (`Website` / `API` / `Desktop` / `Project`).  
- **host_hint** from colocated signals (`netlify.toml` → netlify, `wrangler.*` → cloudflare, `vercel.json` → vercel).  
- **Counts only** — never values in JSON/UI metadata.

CLI:

```text
orbityard secrets --import-env --provider netlify --list-packs
orbityard secrets --import-env --provider netlify --pack apps/website --dry-run
```

`--file` remains for power users; `--pack <rel_dir>` means “prefer env files under that scope dir.”

### Desktop flow (two-step modal — still one panel)

**Step 0 — Pick pack** (only when `packs.length > 1` or focus ≠ sole pack):

- Compact **grid of cards** (reuse Platforms/Targets catalog chrome — not a dashboard of widgets):  
  - Title: scope label (`Website`)  
  - Path: `apps/website/.env`  
  - Meta: host chip · `N keys`  
- Single-select by default (matches Deploy focus).  
- Preselect: `lastDeployFocusProject` pack if present; else pack whose `host_hint` matches the Deployment card; else sole pack.  
- Advanced (collapsed): “Select multiple packs” → sequential push to the **same** host (rare); warn if host_hint mismatches.

**Step 1 — Confirm names** (current checklist): masked values · deny/empty disabled · Push · optional Deploy after.

If only one pack: skip Step 0 (today’s UX).

### Coupling to Deploy

| Operator path | Env default |
|---------------|-------------|
| Targets → Deploy on Website | Pack `apps/website` + host from that scope |
| Deployment card Cloudflare with API wrangler | Pack next to wrangler (`.dev.vars`) |
| Root bound, no focus | Step 0 grid required when ≥2 packs |

**Deploy after import** must use the same `projectOverride` as the selected pack’s `rel_dir` (already partly via `lastDeployFocusProject`).

### Minimalist rules (U2)

1. One primary CTA at a time: **Continue** (pack) → **Push to host** → **Deploy**.  
2. No third sidebar for “Env packs.”  
3. Job words: “Which app’s secrets?” not “multi-root env matrix.”  
4. Trust strip unchanged.

### Reject

| Idea | Why |
|------|-----|
| Always import every pack | Surprise cross-app secret blast |
| Studio merges packs into one virtual `.env` | Custody + wrong host semantics |
| Cloud sync of packs across machines | Out of SCOPE |
| Treating `.env.example` as import source by default | Empty/changeme noise |

### Proof (slice 4)

| Layer | Check |
|-------|--------|
| L1 | Fixture monorepo: `apps/web/.env` + `apps/api/.dev.vars` → `--list-packs` returns 2; dry-run with `--pack apps/web` never reads api values into report |
| L2 | Desktop: bind ship-studio root → Import env on Netlify → grid shows `apps/website/.env` → Push uses that file |
| L3 | Dogfood: import website pack → Deploy focus website → live URL |

## Relation to polish band

- Fits **host XOR** value (finishable host cut) — not Integrations revive.  
- Supports 127 **S3 proof bay** only after secrets land on host and Deploy succeeds.  
- Does **not** replace Login CLI; auth-fail still opens Login.

## Proof

| Layer | Check |
|-------|--------|
| L1 | Unit: parse fixture `.env`; import dry-run lists names; temp `.ship/` unchanged / no value bytes |
| L2 | Dogfood Harbor or fixture: import 1–2 dummy secrets to a throwaway Workers project via streamed CLI |
| L3 | Self-beta: real project Put from `.dev.vars` then Deploy; Output shows names only |

## Never-say

- “Upload your .env to Orbit”  
- “We store your secrets securely” (vault export is optional backup — different feature)  
- “One-click sets all secrets without Login”  

## Shipped usage

```bash
orbityard secrets --import-env --provider cloudflare --dry-run
orbityard secrets --import-env --provider cloudflare --yes --name API_TOKEN
```

Desktop Deployment (CF / Vercel / Netlify): **Import env…** → confirm names → **Push to host** → optional **Deploy after import**.

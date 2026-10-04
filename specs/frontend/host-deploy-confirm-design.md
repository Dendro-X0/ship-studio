# Host deploy confirm + Results isolation (design)

**Status:** Shipped · uniform CLI auth vetting on host select + Deploy  
**Updated:** 2026-10-04  
**Parent:** [hosted-deploy-ops-value-bar-design](../backend/hosted-deploy-ops-value-bar-design.md)

```text
GOAL:  Each host card shows only that host’s last-run evidence (no shared Live URL).
       Deploy opens a confirm dialog (name / lane) before streaming CLI when useful.
       Selecting any CLI host (CF/Vercel/Netlify/Fly/Railway) runs the same auth vetting.
NOT:   Studio-held secrets · CDP · inventing vendor project UIs
```

## Contract

| Field | Writer | Reader |
|-------|--------|--------|
| `last_run.host_provider` | `selfhost` → `"selfhost"`; `hostdeploy` → `cloudflare\|vercel\|netlify` | Desktop Results bay |
| `hostdeploy --name` | Optional override for Pages/site name | Dialog → CLI |
| `hostdeploy --auth-check` | Probe CLI login only | Desktop preflight |

## UX

### Results bay

Show only when `selectedPlatform` matches `last_run.host_provider` (infer from steps/message if field missing on old files). Loopback URLs never appear on hosted cards; hosted https never on Self-host.

### Confirm dialog (Cloudflare · Vercel · Netlify)

Before `shipctl hostdeploy`:

| Control | Cloudflare | Vercel | Netlify |
|---------|------------|--------|---------|
| Project / site name | Editable (default folder name) | Shown for reference (link wins) | Editable → `--site` / `--create-site` if unlinked |
| Lane summary | pages/workers · asset path | prod · cwd | prod · publish dir |
| Primary | **Deploy** | **Deploy** | **Deploy** |
| Secondary | Cancel | Cancel | Cancel |

Self-host: no dialog (local lane already one-click).

### Auth vetting (every CLI host)

Same gate for **Cloudflare · Vercel · Netlify · Fly · Railway**:

| Moment | Behavior |
|--------|----------|
| **Select** host card | `hostdeploy --auth-check` — silent if ok; else **Login CLI** + Check again |
| **Deploy** (Tier A) | Same check before streaming; Retry Deploy after auth |
| Mid-deploy auth fail | Auto Login CLI + Retry Deploy |
| Missing CLI | Toast install hint (no fake login) |

Self-host / GitHub Pages / Orbit: no CLI OAuth on the card (Pages uses Portal → GitHub).  
Payments Integrations: Open dashboard → Put (no CLI login session).

Studio never stores vendor tokens. Login CLI is the authentication page path.

## Dashboard deep links

| Host | Open dashboard lands on |
|------|-------------------------|
| Cloudflare | `?to=/:account/workers-and-pages` · after ok deploy → `pages/view/<name>` |
| Vercel | `/dashboard` (projects) |
| Netlify | `/projects` · after ok deploy → `/projects/<name>` |
| Fly / Railway | Apps / dashboard |
| GitHub Pages | `github.com/settings/pages` |
| Official signing | Certificates / Partner products / Play / New repo |

Studio never opens the bare marketing home when a list/project deep link exists.

## Also: Self-host Deploy vs serve

| Action | Behavior |
|--------|----------|
| **Deploy** | `shipctl selfhost` (one-shot check) → Ready in seconds |
| **Open live** | Starts `selfhost --serve` if needed, opens loopback on health ok |
| **Cancel serve** | Kills serve process (only while serving) |

See [selfhost-deploy-finish-fast-design](../backend/selfhost-deploy-finish-fast-design.md).

Cloud hosts (Cloudflare / Vercel / Netlify): Studio is a **portal** only — stop/delete on the vendor dashboard. **Cancel on dashboard** (Results) opens the vendor; **Clear evidence** drops local last-run so Studio syncs after a remote delete ([host-deploy-evidence-clear-design](../backend/host-deploy-evidence-clear-design.md)). `*.pages.dev` is Cloudflare-hosted, never Self-host loopback.

## Proof

| Layer | Check |
|-------|--------|
| L1 | last-run serde includes `host_provider`; classify/name sanitize; auth_check JSON |
| L2 | After CF deploy, Self-host card hides CF URL; CF card shows it |
| L2 | Deploy → dialog → rename → create/deploy uses new name |
| L2 | Unauthenticated Deploy → Login CLI opens; Retry Deploy after login |

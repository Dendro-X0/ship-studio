# Host deploy confirm + Results isolation (design)

**Status:** Shipped  
**Updated:** 2026-09-27  
**Parent:** [hosted-deploy-ops-value-bar-design](../backend/hosted-deploy-ops-value-bar-design.md)

```text
GOAL:  Each host card shows only that host’s last-run evidence (no shared Live URL).
       Deploy opens a confirm dialog (name / lane) before streaming CLI when useful.
NOT:   Studio-held secrets · CDP · inventing vendor project UIs
```

## Bugs

1. Self-host / Vercel cards show Cloudflare `host.*` steps + `.pages.dev` URL from shared `.ship/last-run.json`.
2. Deploy fires immediately — no chance to confirm Pages project name.

## Contract

| Field | Writer | Reader |
|-------|--------|--------|
| `last_run.host_provider` | `selfhost` → `"selfhost"`; `hostdeploy` → `cloudflare\|vercel\|netlify` | Desktop Results bay |
| `hostdeploy --name` | Optional override for Pages/site name | Dialog → CLI |

## UX

### Results bay

Show only when `selectedPlatform` matches `last_run.host_provider` (infer from steps/message if field missing on old files). Loopback URLs never appear on hosted cards; hosted https never on Self-host.

### Confirm dialog (Cloudflare · Vercel · Netlify)

Before `shipctl hostdeploy`:

| Control | Cloudflare | Vercel / Netlify |
|---------|------------|------------------|
| Project / site name | Editable (default folder name) | Shown; edit when CLI supports |
| Lane summary | pages/workers · asset path | prod · cwd |
| Primary | **Deploy** | **Deploy** |
| Secondary | Cancel | Cancel |

Self-host: no dialog (local lane already one-click).

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
| L1 | last-run serde includes `host_provider`; classify/name sanitize |
| L2 | After CF deploy, Self-host card hides CF URL; CF card shows it |
| L2 | Deploy → dialog → rename → create/deploy uses new name |

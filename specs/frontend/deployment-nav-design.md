# Deployment nav — design

**Status:** Shipped  
**Updated:** 2026-09-26  
**Parent:** [platforms-catalog-design](./platforms-catalog-design.md)

```text
GOAL: One Ship nav item “Deployment” → dedicated host panel (like Sign).
NOT: Sidebar tree of every host · NOT per-host oauth rows in Launch/Publish.
```

## Problem

Platforms section expands Hosting providers as individual sidebar rows. Launch/Publish also listed Cloudflare · Vercel · Netlify as separate OAuth gates. That competes with Sign’s single-entry pattern.

## Target

| Nav (Ship) | Panel |
|------------|--------|
| Sign | Signing paths · official store lanes via Sign probe |
| **Deployment** | Hosting catalog + wizard (Put / Login CLI / Open / Continue publishing) |

| Spine (Launch / Publish) | Behavior |
|--------------------------|----------|
| Host login | **One** step `oauth.hosts` → **Open Deployment** (`desktop_view: platforms`) — not N× `oauth.cloudflare` / `oauth.vercel` / … |

- Remove **Platforms** sidebar section + `#sidebar-platforms` tree.
- Keep view id `platforms` (less churn); chrome label = **Deployment**.
- Catalog groups on this panel: **Hosting only**. Official signing stays on Sign / probe → Sign.
- CmdK / probe **Choose host** → Deployment; **Choose platform** (official) → Sign.

## Proof

- L1: `apps/desktop` `tsc --noEmit` · `cargo test -p shipctl`
- L2: Harbor Launch — one Deployment gate (no oauth.cloudflare/vercel/netlify rows)

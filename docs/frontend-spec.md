# Frontend Spec — Ship Studio Desktop shell

- **Product:** Ship Studio (Tauri desktop)
- **Audience:** Solo operators shipping via Signet + Orbit
- **Reference tier:** CodaCtrl studio shell · Linear restraint
- **Stack:** Vanilla HTML/CSS/TS (existing Tauri UI — no React/shadcn migration this pass)
- **Spec status:** approved for implement
- **API dependency:** existing Tauri commands + `shipctl` (unchanged)
- **UX principle:** Minimal actions · one publish spine — detail panels open from the current step, then return
- **Modes:** General (default, short publish + focused nav) · Advanced (full plan + Assist/Launch/Portal/Ritual/Tools)

## Visual direction

- **Theme:** dark studio charcoal
- **Accent:** forest emerald (existing brand) — not purple, not cream/terracotta
- **Type:** Sora (UI) + IBM Plex Mono (paths / output)
- **Atmosphere:** frameless custom titlebar + shared shell bg; soft radial emerald + faint amber gradient (no grain)
- **Logo:** emerald rounded seal with check (shipped) — SVG in-app, PNG/ICO for OS icon
- **Density:** studio — sidebar groups, dashboard overview, command palette
- **NOT:** flat button grid as primary nav · void centered column · card-in-card · Inter/system UI fonts · default OS chrome

## App shell

```
┌─ Custom titlebar (logo · WORKING PROJECT · window controls) ──┐
├─ Sidebar ──────┬─ Topbar (view · project crumb · search) ─────┤
│ Brand LOCAL    ├─ Main view (scroll) ─────────────────────────┤
│ Nav groups     │                                              │
│ Bound project  ├─ Output dock ────────────────────────────────┤
│                └─ Status bar (shipctl · shortcuts) ───────────┘
└────────────────┴──────────────────────────────────────────────┘
+ Command palette (Ctrl+K)
```

**Project identity (always on):** the bound folder is the session — not a footnote.

- Titlebar primary control: **folder name** (large) + truncated path. Empty: “Choose a project”.
- Window title: `Ship Studio — {name}` when bound.
- Sidebar footer: “Working in” + name + full path (mono) + Open / recents.
- Topbar: view title + crumb `{name} · {path}` so you never lose the repo after navigating.
- Dashboard first viewport: **Working in {name}** (or bind empty state). No numbered pipeline.

DO NOT: hide the project behind “No project” muted chrome · lead with a 1–5 Doctor/Portal/Paste/Sign/Deploy strip · treat the dashboard as a CLI button grid.

### Nav groups

```
SHIP
- dashboard — overview, health, workflow, quick actions
- publish — minute wizard (Open/Run · Confirm · Next) — primary
- assist — full-stack checklist overview
- launch — guided open → verify → next (companion)
- portal — human sprint + provider portal + secrets
- env — ENV/token configure · retrieve · create
- sign — self-sign vs official vendor paths

CONFIG
- scopes — Web / API / Desktop directories
- ritual — sign_args / deploy_args presets
- tools — doctor / sign / deploy / flow / vault / …

RUN
- output — full output focus (dock always visible too)
```

Titlebar project name opens a recents switcher for effortless directory binding.

## Search

Command palette filters **views + actions** by title/keywords. Enter runs action or navigates. Esc closes. Does not search secret values.

## Pages

### Dashboard
Purpose: **which repo**, **local status** (git / ship / deploy), **the next human action**.  
Layout: session → **status checklist bar** (deployed / pending submission / …) → Now CTA → health tiles.  
Empty: bind a folder + recents. Bound: `shipctl pulse` fills status bar + Now from git + `.ship` + deploy signals.  
DO NOT: numbered robotic workflow · ignore mid-launch/publish state · call vendor HTTPS for status.

### Assist / Scopes / Env / Sign / Publish
Purpose: **Publish** is the live minute wizard (spine). Assist is a read-only checklist overview with **Start publish**. Scopes / Env / Sign / Portal / Ritual are detail panels — opened via **Related** from the current publish step (`desktop_view`), with **Back to Publish** in the topbar while a pass is mid-flight.  
Preserve `#btn-*` IDs; secret values never rendered.

Auto-load: navigating to Publish with a bound project and no plan calls `refreshPublish`. After Confirm / Next / Open / Verify, refresh `pulse` so Dashboard Now stays honest.

### Modes (General / Advanced)

Topbar segmented control persists in `localStorage` (`ship-studio.mode`).  
`body[data-mode=general|advanced]` drives CSS:

| General | Advanced |
|---------|----------|
| Nav: Dashboard, Publish, Env, Scopes, Sign, Output | + Assist, Launch, Portal, Ritual, Tools |
| Publish plan via `shipctl publish --mode general` | `--mode advanced` |
| Hide Deploy toggle; keep Offline | Full toggles |

Switching mode rebuilds the publish plan (`publish reset` semantics) and toasts.

**Advanced Related dogfood** (gaps #1–#6): see `specs/frontend/advanced-publish-dogfood.md` — listing / submit / db / CI / container steps open Sign, Env, Portal, or Dashboard via Related; **Open** navigates the same Related panel (incl. Dashboard) and still fires `entry_url` / Run terminal.

### Dashboard Now CTA

Hero action for the publish spine — not a quiet peer of “Switch project”.

| State | Label | Hint |
|-------|-------|------|
| Unbound | Open folder… | Bind a repo to launch the minute publish path |
| Bound, no pass | Start publishing | One click — we open the right portals; you confirm each step |
| Mid-flight | Continue publishing | Pick up the current step (~N min left when known) |
| Finished | Review publish | Open Publish to scan the completed pass |

Visual: large emerald CTA (`.cta-launch`) with arrow, soft breathe when ready to start/continue; secondary Switch stays muted. `#now-primary` id preserved.

## Motion (≥2 intentional)

1. Active nav accent bar slide / fade
2. View content fade-in on switch
3. Command palette backdrop + panel enter
4. Toast stack — slide up from bottom-right after operator actions
5. Now CTA breathe (bound + Start/Continue) — presence, not noise

## Toasts (bottom-right)

Fixed `#toast-host` above the status/output chrome (z-index above dock, below cmdk). Stack newest on top; auto-dismiss ~3.2s; click dismisses.

| Kind | When |
|------|------|
| ok | Bind project, publish/launch plan loaded, non-quiet shipctl success, copy, save ritual, open path |
| err | shipctl fail/cancel (non-quiet), open-path denied, bind/save errors |
| info | Switcher opened (“Choose a project”), quiet ops stay silent |

DO NOT: toast every `pulse`/`scopes` quiet refresh · block the UI · replace Output dock.

## Proof

- L3: `npm run tauri dev` — Advanced mode · Start publish · walk listing/submit/db/ci/container when markers present · Related → Back to Publish
- L2: `cargo run -p shipctl -- publish --mode advanced --project <dogfood-fixture>` lists new step ids
- Existing button IDs remain clickable after redesign

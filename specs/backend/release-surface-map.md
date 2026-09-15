# Release surface map — archetypes × capabilities

**Status:** Active (living checklist)  
**Updated:** 2026-09-15  
**Owner:** product + `shipctl` publish/scopes/sign/portal  
**Spine:** Do not rewrite Publish — extend adaptive plans from this matrix.

## Purpose

Enumerate every shipping task operators care about — desktop/mobile signing, backend/API/DB hosting, self vs official signing, marketplace submission — and mark what Ship Studio already sequences vs what remains a gap or non-goal.

Ship Studio stays a **local sequencing portal**: detect → adaptive plan → Open/Run (vendor UI or local CLI) → Confirm → Next. Bridge never calls vendor HTTPS; `.ship/` never stores secret values.

## Non-goals (v0)

- Replace Cloudflare / Vercel / Apple / Play / Partner Center / Steam UIs  
- Finish OAuth, store review, or marketplace listing without the human  
- Multi-root portfolio hub or cloud control plane  
- A second wizard spine beside Publish  

## Archetypes

| Archetype | Detection today | Notes |
|-----------|-----------------|-------|
| **Api** | `wrangler.toml` / worker heuristics | Cloudflare Workers via Orbit |
| **Web** | `vercel` / `netlify` / web dir names | SSR or static |
| **Desktop** | Tauri / `signet.toml` / desktop dir | Signet self-sign + official store portals |
| **Docs** | docs dir + vercel | Often companion to Api |
| **Root** | fallback node package | Weak deploy defaults |
| **Mobile** | `android/` / `ios/` / Expo / Flutter / Capacitor | Play + App Store Connect listing (Advanced) |
| **Container** | `Dockerfile` / Compose | Portal docs + Advanced `container.deploy` (no remote build/k8s) |
| **Db** | Neon / Supabase / D1 / Turso markers | Portal + env hints + Advanced `db.provision` (no migrate automation) |
| **Ci** | `.github/workflows/*release*` | Pulse note + Advanced `ci.release` (Actions URL) |

## Capability matrix

Legend: **I** = implemented step/UI · **U** = URL / open-only · **G** = gap · **N** = non-goal for v0

| Capability | Api | Web | Desktop | Docs | Mobile | Container | Db |
|------------|----|-----|---------|------|--------|-----------|-----|
| Doctor (Signet/Orbit PATH) | I | I | I | I | I | I | G |
| Scopes pick | I | I | I | I | I | I | G |
| Provider OAuth | I Adv | I Adv | — | I Adv | — | U docs | G |
| Env / secrets put | I | I | I | I | U* | N | G |
| Configure studio.json | I | I | I | I | I | I | G |
| Self-sign (Signet scan/build/release) | — | — | I | — | — | N | N |
| Official signing portals | — | — | U | — | U | N | N |
| Marketplace listing | U Polar Adv | U Polar Adv | U Polar+Steam/itch/Epic Adv | U | U Play/ASC Adv | G | N |
| Store submission (MAS / MS / Play / Steam) | N | N | U Adv | N | U Adv | N | N |
| Dry-run flow | I | I | I | I | I | I | G |
| Deploy (Orbit providers) | I CF | I Vercel/Netlify | weak | I | G | U stub | G |
| Live check + skip if already live | I | I | I | I | I | G | G |
| DB provision / migrate hosting | U Adv | U Adv | U Adv | U Adv | U Adv | G | U Adv |
| CI release authoring | U Adv | U Adv | U Adv | U Adv | U Adv | U Adv | G |

\*Mobile env put is opportunistic (shared secret hints); no Play/ASC-specific secret catalog yet.

### Modes

| Mode | Keeps | Omits |
|------|-------|-------|
| **General** | doctor, scopes, env, configure, `sign.self.build`, dry_run, deploy*, live_check | oauth.*, official sign, Polar / store listing, `submit.*`, `db.provision`, `ci.release`, `container.deploy`, sign scan/release dry/live |
| **Advanced** | Full adaptive plan | — |

See `studio-modes-design.md`.

## Task catalog (human work Ship Studio sequences)

### A. Identity & tools
1. Bind project folder  
2. Doctor — Signet / Orbit / provider CLIs on PATH  
3. Scopes — which directories ship (Web / Api / Desktop / Docs / Mobile)  

### B. Access & secrets
4. Provider OAuth (Wrangler / Vercel / Netlify / `gh`)  
5. Create tokens on vendor dashboards  
6. Put secrets into provider config (never store values in `.ship/`)  

### C. Signing
7. **Self-sign** — Signet identity → scan → build → release dry → release  
8. **Official sign** — Apple certificates / notarization · Windows Authenticode / Partner Center · Play upload key · GitHub Release assets  

### D. Listing & marketplaces
9. Polar product / checkout / pricing (confirm)  
10. Store listing & submission (MAS, Microsoft Store, Play, Steam, …) — **mostly gap / human**  
11. Package registries (npm, crates.io, …) — **gap**  

### E. Hosting & deploy
12. Backend / API — Cloudflare Workers, Vercel, Netlify (Orbit)  
13. Frontend / docs — same providers  
14. **Database hosting** — provision, migrate, connection secrets — **gap**  
15. **Containers / k8s** — build, registry, deploy — **gap**  
16. Dry-run plan → live deploy → live smoke URL  

### F. Already-live intelligence
17. Detect prior Orbit success + URLs → skip redundant deploy / live_check  
18. Distinguish local Wrangler/miniflare from remote Workers  

## Gap backlog (leverage order)

Do **not** invent a parallel wizard. Each item = detection signals + adaptive `build_plan_for` rows + Desktop Related/view + General/Advanced membership.

| # | Gap | First slice | Proof |
|---|-----|-------------|-------|
| 1 | **Mobile archetype** | ✅ Detect `android/` / `ios/` / Expo; scope kind `Mobile`; `listing.play` + `listing.app_store` (URL + confirm); ASC path on Sign portal | Fixture asserts Mobile scope + Advanced listing; General omits |
| 2 | **Official store submission steps** | ✅ Split certs vs listing vs `submit.*` (ASC / Play / MS) on Sign + Advanced publish | Mobile + Tauri fixtures; General omits `submit.*` |
| 3 | **DB hosting lane** | ✅ Neon/Supabase/D1/Turso portal + secrets hints; Advanced `db.provision` | Env portal create URLs; General omits `db.provision` |
| 4 | **CI release checklist** | ✅ Detect `*release*` workflows; pulse note; Advanced `ci.release` + Assist step | Fixture pulse/publish/assist |
| 5 | **Container deploy** | ✅ Detect Dockerfile/Compose; `ScopeKind::Container`; portal docs; Advanced `container.deploy` | Fixture scope + publish; General omits |
| 6 | **Steam / extra marketplaces** | ✅ Opt-in markers + `.ship/markets`; Advanced `listing.steam` / `itch` / `epic` | Fixture steam_appid + markets.json; General omits |

## Change protocol

1. Update **this matrix** (cell I/U/G).  
2. Add detection in `config::probe` / `scopes` / `portal` as needed.  
3. Add adaptive steps in `publish::build_plan_for` (+ General filter if short-path).  
4. Wire Desktop Related / Sign / Env only if `desktop_view` already exists or add one.  
5. Proof: L1 fixture · L2 `shipctl publish` · L3 Desktop General/Advanced.  

## Related specs

- `extra-marketplaces-design.md` — gap #6 (done)  
- `container-deploy-design.md` — gap #5 (done)  
- `ci-release-checklist-design.md` — gap #4 (done)  
- `db-hosting-lane-design.md` — gap #3 (done)  
- `mobile-listing-design.md` — gap #1 (done)  
- `store-submission-split-design.md` — gap #2 (done)  
- `publish-portal-design.md` — live stepper  
- `studio-scopes-design.md` — Web/Api/Desktop/Docs/Mobile  
- `studio-modes-design.md` — General / Advanced  
- `deploy-status-skip-design.md` — already-live skip  
- `project-pulse-design.md` — Dashboard Now  
- `docs/OPERATOR-NEXT.md` — human gates  

## Next atomic iteration (suggested)

**Backlog #1–#6 cleared + Desktop Advanced dogfood.**  

Suggested follow-ups (not parallel wizards):

1. Live click-through in the already-running Desktop (Advanced · bind fixture / assess-api).  
2. Optional matrix rows only when a real ship needs them (npm/crates.io, Fly/Railway).  
3. Keep `OPERATOR-NEXT.md` as the human-gate checklist.

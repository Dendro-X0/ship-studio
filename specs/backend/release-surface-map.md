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
| **Container** | `Dockerfile` / Compose | Advanced `container.build` (Run local docker) + `container.deploy` (push docs + Confirm; no bridge push) |
| **Db** | Neon / Supabase / D1 / Turso markers | Portal + env hints + Advanced `db.provision` (no migrate automation) |
| **Ci** | `.github/workflows/*release*` | Pulse note + Advanced `ci.release` (Actions URL) |

## Capability matrix

Legend: **I** = implemented step/UI · **U** = URL / open-only · **G** = gap · **N** = non-goal for v0

| Capability | Api | Web | Desktop | Docs | Mobile | Container | Db |
|------------|----|-----|---------|------|--------|-----------|-----|
| Doctor (Signet/Orbit PATH) | I | I | I | I | I | I | G |
| Scopes pick | I | I | I | I | I | I | G |
| Provider OAuth | I Adv | I Adv | — | I Adv | — | U docs | G |
| Env / secrets put | I | I | I+grad Adv | I | U* | N | G |
| Configure studio.json | I | I | I | I | I | I | G |
| Self-sign (Signet scan/build/release) | — | — | I | — | — | N | N |
| Official signing portals | — | — | U + graduate Adv | — | U | N | N |
| Marketplace listing | U Polar/Gumroad/Lemon Adv | U Polar/Gumroad/Lemon Adv | U Polar+Steam/itch/Epic+npm/crates+Gumroad/Lemon Adv | U | U Play/ASC Adv | G | N |
| Store submission (MAS / MS / Play / Steam) | N | N | U Adv | N | U Adv | N | N |
| Dry-run flow | I | I | I | I | I | I | G |
| Deploy (Orbit providers) | I CF | I Vercel/Netlify | Signet cut* | I | G | U stub | G |
| Live check + skip if already live | I | I | I | I | I | G | G |
| Marketing / landing deploy | U Adv | U Adv | U Adv | U Adv | U Adv | G | N |
| DB provision / migrate hosting | U Adv | U Adv | U Adv | U Adv | U Adv | G | U Adv |
| CI release authoring | U Adv | U Adv | U Adv | U Adv | U Adv | U Adv | G |
| Legal / TRUST baseline | U Adv | U Adv | U Adv | U Adv | U Adv | U Adv | U Adv |
| GitHub Release cut | U Adv* | U Adv* | I Signet / U Adv | U Adv* | U Adv* | U Adv* | G |

\*Mobile env put is opportunistic (shared secret hints); no Play/ASC-specific secret catalog yet.  
\*GitHub Release: Signet self path uses `sign.self.release`; other projects get Advanced `release.github` when `origin` is GitHub.  
\*Desktop deploy: no Orbit host → Advanced `ship.desktop_cut` (Signet release is the cut); see `shipping-hub-north-star.md`.

### Modes

| Mode | Keeps | Omits |
|------|-------|-------|
| **General** | doctor, scopes, env, configure, `sign.self.build`, dry_run, deploy*, live_check | oauth.*, official sign, Polar / store listing, `submit.*`, `db.provision`, `ci.release`, `container.build`, `container.deploy`, sign scan/release dry/live |
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
8. **Official sign** — Apple certificates / notarization · Windows Authenticode / Partner Center · Play upload key · GitHub Release assets · **graduate** (`sign.graduate`)  

### D. Listing & marketplaces
9. Polar product / checkout / pricing (confirm)  
9b. Gumroad / Lemon SKU — **I Adv** (`listing.gumroad`, `listing.lemon`)  
10. Store listing & submission (MAS, Microsoft Store, Play, Steam, …) — **mostly human** (sequenced)  
11. Package registries (npm, crates.io) — **I Adv** (`listing.npm`, `listing.crates`)  
11b. Legal / TRUST / SECURITY baseline — **I Adv** (`legal.baseline`, `trust.pack`)  
11c. GitHub Release cut — **U Adv** (`release.github`) or Signet `sign.self.release`

### E. Hosting & deploy
12. Backend / API — Cloudflare Workers, Vercel, Netlify (Orbit)  
13. Frontend / docs — same providers  
14. **Database hosting** — provision, migrate, connection secrets — **gap**  
15. **Containers / k8s** — local build Run (**I Adv** `container.build`); registry push Confirm; k8s still gap  
16. Dry-run plan → live deploy → live smoke URL  
16b. Marketing / landing cutover — **I Adv** (`marketing.deploy`)  
16c. Suite URL sync to siblings — **I Adv** (`suite.url_sync`) 

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
| 5 | **Container deploy** | ✅ Detect + portal; band #13 adds `container.build` Run | Fixture scope + publish; General omits |
| 6 | **Steam / extra marketplaces** | ✅ Opt-in markers + `.ship/markets`; Advanced `listing.steam` / `itch` / `epic` | Fixture steam_appid + markets.json; General omits |
| 7 | **Professional launch baseline** | ✅ Detect LICENSE / SECURITY / TRUST / CHANGELOG; Advanced `legal.baseline` · `trust.pack` · `release.github` (non-Signet-self) | Fixture missing legal + signet.toml; General omits |
| 8 | **Package registries** | ✅ Detect publishable npm / crates.io (+ markets opt-in); Advanced `listing.npm` / `listing.crates` | Fixture markets npm+crates; General omits |
| 9 | **Marketing deploy lane** | ✅ Detect `apps/website` / Pages / preview / markets; Advanced `marketing.deploy` | Fixture website + markets; General omits |
| 10 | **Graduate signing + commerce** | ✅ Markets/env opt-in; Advanced `sign.graduate` · `listing.gumroad` · `listing.lemon` (+ Sign portal checklist) | Fixture markets; General omits |
| 11 | **Final-mile run depth** | ✅ Cut order build→trust→graduate→release_dry→release; `signet graduate notes` run; `ship.desktop_cut`; doctor readiness | Unit cut-order test; dogfood green |
| 12 | **Suite URL sync** | ✅ `.ship/suite.json` siblings + Advanced `suite.url_sync` (never writes sibling env) | Fixture suite.json; General omits |
| 13 | **Container final-mile run** | ✅ Advanced `container.build` Run (`docker build` / `compose build`); `container.deploy` push Confirm; doctor docker PATH | Unit + dogfood; no `docker push` |
| 14 | **Cut order + CI/registry Runs** | ✅ Place ci/container after release; `gh run list`; npm/cargo `--dry-run` | Order unit test; dogfood |
| 15 | **Adaptive doctor + Verify** | ✅ Layout-aware `doctor.ok`; legal/trust/ci/desktop_cut verify; sticky pending baseline | Unit matrix + verify fixtures |
| 16 | **Desktop Open/Related parity** | ✅ Open navigates Dashboard/Tools/Launch like Related | Code + frontend-spec |
| 17 | **Pulse cut hints** | ✅ Mid-publish Now cues for CI / registry / container / release | Unit `publish_cut_hints_for_ci_and_registry` |
| 18 | **release.github Run** | ✅ `gh release list` Run + Verify when a release exists | Unit run vector; no create |
| 19 | **Assist Run notes** | ✅ Checklist cues for gh list / registry dry-run | Unit assist notes |
| 20 | **README hub pointer** | ✅ Mission + north star / map / dogfood links | README + START-HERE |

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
- `professional-launch-baseline-design.md` — gap #7 (done)  
- `package-registries-design.md` — gap #8 (done)  
- `marketing-deploy-design.md` — gap #9 (done)  
- `graduate-commerce-design.md` — gap #10 (done)  
- `shipping-hub-north-star.md` — mission + bands #11–#13  
- `suite-url-sync-design.md` — band #12 (done)  
- `container-final-mile-design.md` — band #13 (done)  
- `cut-order-ci-registry-design.md` — band #14 (done)  
- `adaptive-doctor-verify-design.md` — band #15 (done)  
- `desktop-open-dashboard-parity-design.md` — band #16 (done)  
- `pulse-cut-hints-design.md` — band #17 (done)  
- `release-github-list-run-design.md` — band #18 (done)  
- `assist-run-notes-design.md` — band #19 (done)  
- `readme-hub-pointer-design.md` — band #20 (done)  
- `publish-portal-design.md` — live stepper  
- `studio-scopes-design.md` — Web/Api/Desktop/Docs/Mobile  
- `studio-modes-design.md` — General / Advanced  
- `deploy-status-skip-design.md` — already-live skip  
- `project-pulse-design.md` — Dashboard Now  
- `docs/OPERATOR-NEXT.md` — human gates  

## Next atomic iteration (suggested)

**Gaps #1–#20 first slices cleared** (mobile store API upload still deferred).  

Suggested follow-ups (not parallel wizards):

1. Live Desktop L3 Open/Run on a Signet subject (final-mile cut).  
2. Keep `OPERATOR-NEXT.md` as the human-gate checklist.  
3. Only add a new Adaptive lane when a real ship needs it.

# Shipping hub — north star

**Status:** Active  
**Updated:** 2026-09-18  
**Owner:** product + `shipctl` publish / pulse / doctor  

## Mission

Ship Studio is a **local shipping hub for the final mile** of the product lifecycle:

1. **Sign** — self-sign (Signet) and graduate (OV / Azure Trusted Signing / notarization)  
2. **Release** — checksums, TRUST honesty, GitHub Release / Signet release  
3. **Deploy** — Orbit for Web/API; desktop cut is Signet release (+ optional marketing host)

Operators should rarely miss a gate. The hub sequences Open/Run → human vendor work when required → Confirm → Next.

## Explicitly not the hub’s job

- Product documentation sites, feature demos, GIF authoring, narrative marketing copy  
- Replacing Cloudflare / Vercel / Apple / Play / Gumroad UIs  
- Storing secret values in `.ship/`  
- Finishing OAuth, store review, or DNS without the human  

Developers still create docs and demos elsewhere; Ship Studio cuts the ship.

**Scope of service (product boundary):** [`docs/product/SCOPE-OF-SERVICE.md`](../../docs/product/SCOPE-OF-SERVICE.md)

## Architecture (unchanged spine)

Detect → Adaptive Publish plan → Open or Run local CLI (Signet / Orbit / `gh`) → Confirm → Next.  
Detail panels (Scopes, Env, Sign, Portal, …) open from the current step — not a second wizard.

## Band queue

| # | Band | Status |
|---|------|--------|
| 1–10 | Release-surface Adaptive lanes | Done (first slices) |
| **11** | **Final-mile run depth** — real Signet graduate/release runs, cut order, desktop-deploy honesty, doctor readiness | Done (first slice) |
| **12** | **Suite URL sync** after marketing.deploy | Done (first slice) |
| **13** | **Container final-mile** — local `docker build` / `compose build` Run; push stays Confirm | Done (first slice; mobile store API still deferred) |
| **14** | **Cut order + CI/registry Runs** — place ci/container after release; `gh run list` · npm/cargo `--dry-run` | Done (first slice) |
| **15** | **Adaptive doctor + Verify honesty** — layout-aware `doctor.ok`; legal/trust/ci/desktop_cut verify | Done (first slice) |
| **16** | **Desktop Open/Related parity** — Open navigates Dashboard/Tools/Launch like Related | Done (first slice) |
| **17** | **Pulse cut hints** — mid-publish cues for CI / registry / container / release.github | Done (first slice) |
| **18** | **release.github Run** — `gh release list` (read-only); Verify when a release exists | Done (first slice) |
| **19** | **Assist Run notes** — checklist cues for `gh` list / registry dry-run / container | Done (first slice) |
| **20** | **README hub pointer** — mission + north star / release-surface / Advanced dogfood links | Done (first slice) |
| **21** | **Archetype expansion** — PWA detect + Hugging Face listing + launch matrix | Done (first slice) |
| **22** | **assess-api dogfood** — deploy.yml CI detect + per-scope live skip URLs | Done (first slice) |
| **23** | **aperio dogfood** — Tauri desktop must not pull Play/Android lanes | Done (first slice) |
| **24** | **Personal / local ship intent** — Local vs Public filter orthogonal to General/Advanced | Done (first slice) |
| **24b** | **Env entry URL honesty** — provider-matched Open URLs for secrets | Done (first slice) |
| **25** | **Desktop busy unlock** — Cancel restores Refresh; `run` finally + watchdog | Done (first slice) |
| **26** | **PowerShell Advanced dogfood** — Windows L2 without WSL bash | Done (first slice) |
| **27** | **Publish progress watch** — local Verify poller + Desktop Watch toggle (portal nudge) | Done (first slice) |
| **28** | **Mobile BaaS portal** — `baas.provision` Open URL for Firebase / Appwrite / Convex / mobile+Supabase | Done (first slice) |
| **29** | **Steam submit portal** — `submit.steam` depot/build docs (Open + Confirm; no Steam API) | Done (first slice) |
| **30** | **Alt host portal** — `host.fly` / `host.railway` dashboard Open URLs | Done (first slice) |
| **31** | **Marketplace submit parity** — `submit.itch` / `submit.epic` docs (Open + Confirm) | Done (first slice) |
| **32** | **Alt host expand** — `host.render` / `host.digitalocean` dashboard Open URLs | Done (first slice) |
| **33** | **TUI publish watch** — Publish screen `w` polls local Verify (CLI/Desktop parity) | Done (first slice) |
| **34** | **MCP publish watch** — `ship_publish_watch` one-shot Verify probe for agents | Done (first slice) |
| **35** | **Commerce expand** — `listing.stripe` / `listing.paddle` Open + Confirm | Done (first slice) |
| **36** | **Pulse cut hints expand** — Now detail cues for host / BaaS / submit / commerce | Done (first slice) |
| **37** | **Assist notes expand** — checklist cues for hosts / BaaS / commerce / submit + Watch | Done (first slice) |
| **38** | **Doctor notes expand** — cut-readiness cues for hosts / BaaS / commerce + optional host CLIs | Done (first slice) |
| **39** | **Guide notes expand** — Publish step + host / BaaS / commerce / Watch cues | Done (first slice) |
| **40** | **Commerce portal catalog** — Gumroad / Lemon / Stripe / Paddle as portal providers | Done (first slice) |
| **41** | **Launch commerce parity** — companion listing.* for Gumroad / Lemon / Stripe / Paddle | Done (first slice) |
| **42** | **Alt host expand** — `host.heroku` / `host.amplify` dashboard Open URLs | Done (first slice) |
| **43** | **Launch host / BaaS parity** — companion `host.*` + `baas.provision` Open URLs | Done (first slice) |
| **44** | **Launch marketplace submit parity** — Steam / itch / Epic listing + submit Open URLs | Done (first slice) |
| **45** | **Launch mobile / store submit parity** — Play / ASC / MS listing + submit (Tauri honesty) | Done (first slice) |
| **46** | **Alt host expand** — `host.cloudrun` / `host.azurestatic` dashboard Open URLs | Done (first slice) |
| **47** | **Launch registry + HF listing parity** — companion `listing.npm` / `listing.crates` / `listing.huggingface` | Done (first slice) |
| **48** | **Launch DB + marketing + suite parity** — companion `db.provision` / `marketing.deploy` / `suite.url_sync` | Done (first slice) |
| **49** | **Launch CI + container parity** — companion `ci.release` / `container.build` / `container.deploy` | Done (first slice) |
| **50** | **Launch legal / trust / graduate / release parity** — companion `legal.baseline` / `trust.pack` / `sign.graduate` / `release.github` / `ship.desktop_cut` | Done (first slice) |

## Band #11 acceptance

- `sign.graduate` has a Signet `run` (not only docs URL) when graduate opted in  
- Advanced Signet cut order: build → trust.pack → graduate? → release_dry → release  
- Desktop-only projects get an explicit “Signet release is the deploy” cue (no fake Orbit desktop deploy)  
- Doctor notes answer “can I cut?” (Signet / Orbit / graduate opt-in)  
- Pulse still prefers mid-publish Continue over unrelated tool noise  

## Proof

`cargo test -p shipctl` · `bash scripts/dogfood-advanced-publish.sh` · Desktop Advanced Open/Run on a Signet subject (L3 optional)

## Band #13 acceptance

- Advanced `container.build` has a local `docker build` / `compose build` `run` when Dockerfile/Compose detected  
- `container.deploy` remains push docs + Confirm (no `docker push` from the bridge)  
- Doctor notes whether `docker` is on PATH for container layouts  
- Mobile store API upload remains deferred  

## Band #14 acceptance

- Plan order: configure → sign/release → listings/submit → `ci.release` → `container.*` → marketing → suite → dry_run → deploy  
- `ci.release` Runs `gh run list` (read-only)  
- `listing.npm` / `listing.crates` Run `--dry-run` only (live publish stays Confirm)  

## Band #15 acceptance

- `doctor.ok` requires Signet only when Tauri/signet.toml; Orbit or host CLI only when wrangler/vercel/netlify  
- Verify re-probes `legal.baseline` / `trust.pack`; `ci.release` via `gh`; `ship.desktop_cut` after prior release Done  
- Pending legal/trust steps stay sticky until Confirm so Verify works after files appear  

## Band #16 acceptance

- Desktop Publish **Open** navigates any `RELATED_VIEW_LABELS` target (including `dashboard`) the same as Related  
- `entry_url` / Run terminal behavior unchanged after navigation  

## Band #17 acceptance

- Mid-publish Now detail includes step-specific cut hints for `ci.release`, `listing.npm`/`crates`, `container.*`, `release.github`, legal/marketing/suite  
- Continue publishing remains the primary action  

## Band #18 acceptance

- Advanced `release.github` Runs `gh release list --limit 5` (read-only); never `gh release create`  
- Verify succeeds when `gh release list --limit 1` is non-empty  

## Band #19 acceptance

- Assist CI step/note mentions `gh run list`  
- Assist notes mention npm/crates `--dry-run` and `gh release list` when detected  

## Band #20 acceptance

- README states shipping-hub mission and links north star + release-surface map + Advanced dogfood  
- START-HERE prefers Publish over Launch as the minute path  

## Band #21 acceptance

- PWA manifest / vite-plugin-pwa detected with pulse + assist hints  
- Advanced `listing.huggingface` when markets/model card signals present; no Hub upload from bridge  

## Band #22 acceptance

- `deploy.yml` (and similar) surfaces Advanced `ci.release` with deploy-oriented copy  
- Multi-scope publish skips match provider URL (Workers vs Vercel), not always the first URL  
- `live_check` prefers live deploy URLs over Polar dashboard when both exist  

## Band #23 acceptance

- Desktop Tauri alone does **not** get Play/Android `sign.official.android` / `submit.play`  
- Mobile android/expo fixtures still get Play listing + submit  
- Apple notarization + MAS/MS Store submit remain for Tauri  

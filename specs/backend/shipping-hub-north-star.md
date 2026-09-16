# Shipping hub — north star

**Status:** Active  
**Updated:** 2026-09-15  
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

# Frontend Spec — Advanced publish dogfood (Desktop)

- **Product:** Ship Studio Desktop  
- **Depends on:** release-surface gaps #1–#10 (`shipctl` Advanced plan)  
- **Spec status:** implement  
- **Stack:** existing vanilla Tauri UI  

## Goal

Operator in **Advanced** mode can walk every new adaptive step with: **Open** (URL / Related) → **Confirm** → **Next**, without a second wizard.

## Related matrix (`desktop_view`)

| Step id | Related label | Panel | Open primary |
|---------|---------------|-------|--------------|
| `listing.play` / `listing.app_store` | Open Sign | Sign | Vendor URL |
| `listing.steam` / `itch` / `epic` / `polar` / `npm` / `crates` | Open Portal | Portal | Vendor URL |
| `submit.*` | Open Sign | Sign (kind=`submit`) | Vendor URL |
| `db.provision` | Open Env | Env | Neon/Supabase/… URL |
| `ci.release` | Dashboard | Dashboard | GitHub Actions URL |
| `legal.baseline` | Dashboard | Dashboard | — |
| `trust.pack` | Open Sign | Sign | — |
| `release.github` | Dashboard | Dashboard | GitHub Releases/new |
| `sign.graduate` | Open Sign | Sign | Azure Trusted Signing docs · Run `signet graduate notes` |
| `ship.desktop_cut` | Open Sign | Sign | GitHub Releases (desktop-only cue) |
| `listing.gumroad` / `listing.lemon` | Open Portal | Portal | Vendor dashboard |
| `marketing.deploy` | Open Portal | Portal | Host dashboard (Pages/Vercel/Netlify) |
| `container.deploy` | Open Portal | Portal (container filter) | Registry docs URL |

Invariants: General mode still omits these steps; Advanced rebuilds via mode toggle; **Back to Publish** returns from Related.

## Proof

- L2: `bash scripts/dogfood-advanced-publish.sh` — all new step ids + Related `desktop_view`  
- L2b: `bash scripts/dogfood-advanced-walk.sh <fixture|assess-api>` — Confirm through Advanced; every `desktop_view` ∈ RELATED_VIEW_LABELS  
- L3: Desktop already running (`ship-studio-desktop` + Vite `:1420`) — Advanced · bind fixture `E:\Temp\ship-studio-dogfood-advanced-*` or `assess-api` · Related / Open / Confirm  
- Rebuild `cargo build -p shipctl --release` (or newest mtime binary) so Desktop does not use a stale shipctl  

### Dogfood evidence (2026-09-15)

| Project | Advanced highlights | Walk |
|---------|---------------------|------|
| `fixtures/advanced-dogfood` | listing.* · submit.* · db · ci · container · `deploy.api.root` | related_ok |
| `assess-api` | db.provision (D1) · listing.polar · oauth · env; deploys skip when live | related_ok |

Desktop resolves the **newest** `shipctl` among sidecar + `target/{debug,release}` (stale sidecar cannot hide a fresh build).

## Non-goals

- New React views  
- Automating Steam/Butler/docker push  

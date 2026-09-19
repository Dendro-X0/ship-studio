# Operator next — manual gates only

**Updated:** 2026-09-19  
**Repo:** https://github.com/Dendro-X0/ship-studio  
**Local path:** `E:/Web Projects/ship-studio`

## Publish portal (preferred)

Minute-oriented path: **official platform for the work**, Ship Studio for sequence + verify.

**Intent (orthogonal to General/Advanced):**

| Intent | Use when | Omits |
|--------|----------|-------|
| **Local** | Personal tools / desktop cut only | `env.sprint`, `deploy.*`, `live_check`, oauth/listing/submit/marketing |
| **Public** (default) | Hosted final-mile | (none — full adaptive plan) |

```bash
./target/release/shipctl.exe publish --mode general --intent local --project "E:/Web Projects/aperio"
./target/release/shipctl.exe publish --mode advanced --intent public --project "E:/Web Projects/assess-api"
```

Desktop: topbar **Local | Public** next to General/Advanced. Persists in `.ship/studio.json` (`ship_intent`).

```bash
cd "E:/Web Projects/ship-studio"
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api"
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" open
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" verify
# after paste / listing / live release / deploy on vendor UIs…
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" confirm
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" next
# optional: poll local Verify until ready (never vendor HTTPS)
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" watch --once
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" watch --interval-secs 15
```

Desktop: topbar **Advanced** · **Publish** (spine) · **Watch** toggle polls Verify every 15s and toasts when Confirm is ready · Related opens Env / Sign / Portal / Scopes / Dashboard · **Back to Publish**.  
TUI: Publish (`P`) → o / v / c / n · **`w` Watch** (local Verify poll, READY when Confirm is safe).  
MCP: `ship_publish_watch` — one local Verify probe per call (agents poll; optional `auto_confirm`).

Progress: project `.ship/publish.json` (no secret values).

**Adaptive plan (Advanced + Public):** doctor → scopes → legal.baseline → oauth* → env → db.provision → **baas.provision?** → **host.fly?** → **host.railway?** → **host.render?** → **host.digitalocean?** → **host.heroku?** → **host.amplify?** → **host.cloudrun?** → **host.azurestatic?** → configure → **sign.self.build → trust.pack → sign.graduate? → sign.self.release_dry → sign.self.release** (or release.github) → **ship.desktop_cut?** → listing.* → submit.* → **ci.release → container.build → container.deploy** → marketing.deploy → **suite.url_sync** → dry-run → deploy* → live check.

Shipping hub: final-mile is sign → release → deploy. Docs/demos stay outside Studio (`specs/backend/shipping-hub-north-star.md`).

**General + Public** keeps the short spine (doctor, scopes, env, configure, sign.self.build, dry-run, deploy*, live_check).  
**Local** drops hosted env/deploy/stores even in Advanced.

## Dogfood

```bash
bash scripts/dogfood-advanced-publish.sh
bash scripts/dogfood-advanced-walk.sh fixtures/advanced-dogfood
# Windows without WSL:
# powershell -File scripts/dogfood-advanced-publish.ps1
# Desktop: Advanced → bind E:/Web Projects/ship-studio/fixtures/advanced-dogfood
```

Real repo example: `assess-api` surfaces **db.provision** (D1) + **listing.polar** + oauth/env; deploy steps auto-skip when already live.

## Guided launch (legacy companion)

Same open → verify → confirm → next pattern; state in `.ship/launch.json`. Prefer **Publish** for the full minute-oriented path. Launch also sequences Polar / Gumroad / Lemon / Stripe / Paddle listings, Steam / itch / Epic listing + submit, Play / App Store / Microsoft Store listing + submit (Tauri skips Play), alt-host dashboards (`host.*`), and mobile BaaS provision when detected.

## Remaining human work

| Step | You do |
|------|--------|
| Paste | Create tokens / DB URLs on vendor sites; Open → put → Confirm → Next |
| Scopes | Pick Web / API / Desktop / Mobile / Container, then Save |
| Sign | Self-sign locally; Run `signet graduate notes` then apply/ov-sign/notarize; **Submit** store review on vendor sites |
| Desktop cut | No Orbit host → Confirm `ship.desktop_cut` — Signet release is the deploy |
| Listing | Polar · Gumroad · Lemon · Stripe · Paddle · npm · crates.io · Play · ASC · Steam · itch · Epic (URL + confirm; Advanced) |
| Steam submit | After listing, Open Steamworks uploading docs (`submit.steam`) and upload depots yourself |
| itch / Epic submit | After listing, Open butler / Epic publishing docs (`submit.itch` / `submit.epic`) and upload yourself |
| Fly / Railway / Render / DO / Heroku / Amplify / Cloud Run / Azure Static | Open dashboard (`host.*`); deploy with their CLI/UI — Studio does not Orbit-deploy them |
| DB | Provision on Neon/Supabase/D1/Turso console; put connection on deploy target |
| CI | After tag/Signet release, Run `gh run list` / confirm GitHub Actions |
| Legal / trust | Add LICENSE + SECURITY.md; TRUST.md + checksums for desktop/Signet; Run `gh release list` then cut GitHub Release when prompted |
| Marketing | Deploy landing / HOOK / download site; confirm canonical URL (DNS stays manual) |
| Suite | After landing is live, paste URL into sibling env keys from `.ship/suite.json` (`suite.url_sync`) |
| Container | Run local `docker build` / `compose build`; push stays Confirm + registry docs |
| npm / crates | Run `--dry-run` from Publish; live publish + OTP stays on your machine |
| Deploy | Allow network deploy when Publish reaches deploy (skip if already live) |

# Operator next — manual gates only

**Updated:** 2026-09-15  
**Repo:** https://github.com/Dendro-X0/ship-studio  
**Local path:** `E:/Web Projects/ship-studio`

## Publish portal (preferred)

Minute-oriented path: **official platform for the work**, Ship Studio for sequence + verify.

```bash
cd "E:/Web Projects/ship-studio"
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api"
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" open
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" verify
# after paste / listing / live release / deploy on vendor UIs…
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" confirm
./target/release/shipctl.exe publish --mode advanced --project "E:/Web Projects/assess-api" next
```

Desktop: topbar **Advanced** · **Publish** (spine) · Related opens Env / Sign / Portal / Scopes / Dashboard · **Back to Publish**.  
TUI: Publish (`P`) → o / v / c / n.

Progress: project `.ship/publish.json` (no secret values).

**Adaptive plan (Advanced):** doctor → scopes → oauth* → env sprint → **db.provision** (if D1/Neon/…) → configure → sign* → **listing.*** (Polar / Play / ASC / Steam / itch / Epic) → **submit.*** → **ci.release** → **container.deploy** → dry-run → deploy* → live check.

**General** keeps the short spine (doctor, scopes, env, configure, sign.self.build, dry-run, deploy*, live_check).

## Dogfood

```bash
bash scripts/dogfood-advanced-publish.sh
bash scripts/dogfood-advanced-walk.sh fixtures/advanced-dogfood
# Desktop: Advanced → bind E:/Web Projects/ship-studio/fixtures/advanced-dogfood
```

Real repo example: `assess-api` surfaces **db.provision** (D1) + **listing.polar** + oauth/env; deploy steps auto-skip when already live.

## Guided launch (legacy companion)

Same open → verify → confirm → next pattern; state in `.ship/launch.json`. Prefer **Publish** for the full minute-oriented path.

## Remaining human work

| Step | You do |
|------|--------|
| Paste | Create tokens / DB URLs on vendor sites; Open → put → Confirm → Next |
| Scopes | Pick Web / API / Desktop / Mobile / Container, then Save |
| Sign | Self-sign locally; official certs vs **Submit** store review on vendor sites |
| Listing | Polar · Play · ASC · Steam · itch · Epic (URL + confirm; Advanced) |
| DB | Provision on Neon/Supabase/D1/Turso console; put connection on deploy target |
| CI | After tag/Signet release, confirm GitHub Actions |
| Container | Build/push locally; registry docs are open-only |
| Deploy | Allow network deploy when Publish reaches deploy (skip if already live) |

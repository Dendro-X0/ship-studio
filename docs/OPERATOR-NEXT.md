# Operator next — manual gates only

**Updated:** 2026-09-13  
**Repo:** https://github.com/Dendro-X0/ship-studio  
**Local path:** `E:/Web Projects/ship-studio`

## Guided launch (preferred)

Semi-automated path: **official platform for the work**, Ship Studio for sequence + verify.

```bash
cd "E:/Web Projects/ship-studio"
./target/release/shipctl.exe launch --project "E:/Web Projects/assess-api"
./target/release/shipctl.exe launch --project "E:/Web Projects/assess-api" open
./target/release/shipctl.exe launch --project "E:/Web Projects/assess-api" verify
# paste steps: after wrangler secret put…
./target/release/shipctl.exe launch --project "E:/Web Projects/assess-api" confirm
./target/release/shipctl.exe launch --project "E:/Web Projects/assess-api" next
```

Desktop: open assess-api → **Launch** → Open / Verify / Confirm / Next.

Progress is stored in the project’s `.ship/launch.json` (no secret values).

## Remaining human work

| Step | You do |
|------|--------|
| OAuth | Complete Wrangler/Vercel login in browser when Open starts it |
| Paste | Create GitHub PAT / Polar values on their sites; paste into wrangler |
| Deploy | Allow network deploy when Launch reaches deploy |

## Optional vault backup

```bash
./target/release/shipctl.exe vault export --out "E:/Web Projects/assess-api/ship-secrets.km" \
  --from-hints --project "E:/Web Projects/assess-api"
```

## Already done

| Step | Evidence |
|------|----------|
| GitHub repo | https://github.com/Dendro-X0/ship-studio |
| Guided launch engine | `shipctl launch` |
| Portal / human / vault | `cargo test -p shipctl` |
| Wrangler OAuth on this machine | logged in |

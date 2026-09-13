# Dogfood — Signet repo (2026-09-12)

Target: `E:/Experimental projects/Self-signed-distribution`

## Results

| Step | Result |
|------|--------|
| `shipctl doctor` | OK — signet 0.5.17, orbit via sibling `../ship`, `signet.toml` detected |
| `shipctl configure` | Wrote `.ship/studio.json` (`sign_args`: doctor / later build) |
| `shipctl flow --offline --skip-deploy` | OK — configure → `signet doctor --json` |
| `shipctl sign -- build` | OK — release build + Authenticode sign of `target/release/signet.exe` (~1m21s compile) |

## Notes

- minisign SUMS key missing (optional warning) — `signet sums-key create` if needed
- No wrangler/vercel on this repo — Orbit deploy not exercised here
- Desktop: Open folder → that path → Doctor / Sign with preset `build`

## Reproduce

```bash
SHIPCTL="E:/Web Projects/ship-studio/target/release/shipctl.exe"
PROJ="E:/Experimental projects/Self-signed-distribution"
"$SHIPCTL" doctor --project "$PROJ"
"$SHIPCTL" flow --project "$PROJ" --offline --skip-deploy
"$SHIPCTL" sign --project "$PROJ" -- build
```

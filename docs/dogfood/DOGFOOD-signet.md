# Dogfood — Signet repo (2026-09-12)

Target: `E:/Experimental projects/Self-signed-distribution`

## Results

| Step | Result |
|------|--------|
| `orbityard doctor` | OK — signet 0.5.17, orbit via sibling `../ship`, `signet.toml` detected |
| `orbityard configure` | Wrote `.ship/studio.json` (`sign_args`: doctor / later build) |
| `orbityard flow --offline --skip-deploy` | OK — configure → `signet doctor --json` |
| `orbityard sign -- build` | OK — release build + Authenticode sign of `target/release/signet.exe` (~1m21s compile) |

## Notes

- minisign SUMS key: **created** 2026-09-13 under `.signet/sums/` — `SHA256SUMS.minisig` written via `signet build --skip-build`
- No wrangler/vercel on this repo — Orbit deploy not exercised here
- Desktop: Open folder → that path → Doctor / Sign with preset `build`

## Reproduce

```bash
ORBITYARD="E:/Web Projects/orbit-yard/target/release/orbityard.exe"
PROJ="E:/Experimental projects/Self-signed-distribution"
"$ORBITYARD" doctor --project "$PROJ"
"$ORBITYARD" flow --project "$PROJ" --offline --skip-deploy
"$ORBITYARD" sign --project "$PROJ" -- build
```

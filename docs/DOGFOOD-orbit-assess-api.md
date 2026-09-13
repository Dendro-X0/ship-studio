# Dogfood — Orbit / assess-api (2026-09-12)

Target: `E:/Web Projects/assess-api`

## Results

| Step | Result |
|------|--------|
| `shipctl doctor` | OK — nested `apps/api/wrangler.toml` + `.orbit/state.json` detected (after probe fix) |
| `shipctl deploy -- status` | OK — Orbit status (Cloudflare auth + last URL) |
| `shipctl deploy -- deploy --provider cloudflare` | First attempt failed: Worker `allowlist.ts` used `fileURLToPath(import.meta.url)` (undefined under Workers) |
| Fix `apps/api/src/allowlist.ts` | Text-module imports for `*.txt` (wrangler `[[rules]]` already present) |
| Retry deploy | **OK** — uploaded Worker, URL `https://assess-api.paf437sywst688.workers.dev` (~14.5s). Orbit warns missing worker secrets. |

## Bridge lesson

- Default `deploy_args: ["ship"]` is **interactive TUI** — bad for desktop/automation.
- Prefer `deploy --provider cloudflare` or `status` in `.ship/studio.json` / UI presets.

## Reproduce

```bash
export ORBIT_PATH="E:/Web Projects/ship/orbit.exe"
SHIPCTL="E:/Web Projects/ship-studio/target/release/shipctl.exe"
PROJ="E:/Web Projects/assess-api"
"$SHIPCTL" doctor --project "$PROJ"
"$SHIPCTL" deploy --project "$PROJ" -- status
"$SHIPCTL" deploy --project "$PROJ" -- deploy --provider cloudflare
```

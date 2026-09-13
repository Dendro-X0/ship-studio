# Dogfood — Ship Studio guide / portal / secrets

**Date:** 2026-09-13  
**Target:** `E:/Web Projects/assess-api` (Cloudflare Worker + Vercel docs + Polar secrets)

## Commands

```bash
cd "E:/Web Projects/ship-studio"
cargo build -p shipctl --release
./target/release/shipctl.exe guide --project "E:/Web Projects/assess-api"
./target/release/shipctl.exe portal --project "E:/Web Projects/assess-api"
./target/release/shipctl.exe secrets --project "E:/Web Projects/assess-api" --provider cloudflare
# Operator-initiated browser batch:
./target/release/shipctl.exe guide --project "E:/Web Projects/assess-api" --open
```

## Expected

| Check | Result |
|-------|--------|
| Providers | cloudflare, vercel, github, polar (when POLAR_* present) |
| Secret hints | GITHUB_TOKEN, POLAR_* from wrangler `# Secrets` |
| `guide --open` | Opens unique dashboards (CF tokens, Vercel tokens, GitHub tokens, Polar dashboard) |
| Values | Never stored by shipctl — paste via `secrets put` / TUI |

## Human remaining

Paste real Polar/GitHub values when you have them (see [OPERATOR-NEXT.md](./OPERATOR-NEXT.md)):

```bash
shipctl human --project "E:/Web Projects/assess-api" --put
```

Also: `gh auth login`, optional vault export, redeploy.
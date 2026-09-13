# Operator next — manual gates only

**Updated:** 2026-09-13  
**Auto status:** Ship Studio CLI/TUI/Desktop/vault are implemented and staged. Offline dogfood for assess-api is green through configure + flow dry-run.

Everything below needs **you** (browser login, tokens, paste, remotes, network deploy).

---

## 1) GitHub auth (this machine)

```bash
gh auth login
# HTTPS or SSH — your choice; complete browser/device flow
gh auth status
```

## 2) Ship Studio remote — **pushed**

Local path: `E:/Web Projects/ship-studio`  
Remote: https://github.com/Dendro-X0/ship-studio  
`origin/main` tracks local `main` (pushed 2026-09-13).

If push fails again on this machine, check:
1. Broken helper pointing at `E:\Temp\gh-cli\bin\gh.exe` — use real CLI: `C:\Program Files\GitHub CLI\gh.exe`
2. HTTPS “password” must be a **Personal Access Token**, not your GitHub account password (2FA blocks passwords)
3. Network: `curl -I https://github.com` should return `200` before retrying `git push`
## 3) assess-api Worker secrets (paste)

Wrangler is already logged in. Values must come from GitHub PAT + Polar dashboard:

```bash
cd "E:/Web Projects/ship-studio"
./target/release/shipctl.exe human --project "E:/Web Projects/assess-api" --put
# or one-by-one:
./target/release/shipctl.exe secrets put --project "E:/Web Projects/assess-api" --provider cloudflare --name GITHUB_TOKEN
./target/release/shipctl.exe secrets put --project "E:/Web Projects/assess-api" --provider cloudflare --name POLAR_WEBHOOK_SECRET
./target/release/shipctl.exe secrets put --project "E:/Web Projects/assess-api" --provider cloudflare --name POLAR_CHECKOUT_URL
# optional local pepper:
./target/release/shipctl.exe secrets put --project "E:/Web Projects/assess-api" --provider cloudflare --name API_KEY_PEPPER
```

Open sources first:

```bash
./target/release/shipctl.exe human --project "E:/Web Projects/assess-api" --no-open=false
# Polar → GitHub → dashboards
```

## 4) Encrypted backup (after you have values)

```bash
./target/release/shipctl.exe vault export --out "E:/Web Projects/assess-api/ship-secrets.km" \
  --from-hints --project "E:/Web Projects/assess-api"
# or Desktop → Export vault
```

## 5) Redeploy / health (network)

```bash
./target/release/shipctl.exe deploy --project "E:/Web Projects/assess-api"
# Health from this host may still TCP-timeout to workers.dev — check from another network or CF dashboard.
```

Known URLs (already shipped earlier):

- Worker: `https://assess-api.paf437sywst688.workers.dev`
- Docs: `https://docs-two-topaz.vercel.app`

## 6) Already done offline (no action)

| Step | Evidence |
|------|----------|
| `shipctl` release + desktop staged | `target/release/shipctl.exe`, `ship-studio-desktop.exe` |
| Portal / secrets / guide / vault / human | `cargo test -p shipctl` |
| Cloudflare OAuth (wrangler) | logged in as `paf437sywst688@gmail.com` |
| assess-api guide | providers CF/Vercel/GitHub/Polar; vault step present |

Re-run offline prep anytime:

```bash
bash scripts/dogfood-offline.sh "E:/Web Projects/assess-api"
```

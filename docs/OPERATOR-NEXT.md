# Operator next — manual gates only

**Updated:** 2026-09-13  
**Repo:** https://github.com/Dendro-X0/ship-studio  
**Local path:** `E:/Web Projects/ship-studio`

Automated surfaces (CLI / TUI / Desktop / vault / CI) are in place. Offline dogfood for assess-api is green through configure + flow dry-run.

Everything below needs **you** (tokens, paste, deploy, optional `gh` CLI login).

---

## 1) GitHub CLI on this machine (optional)

Push already works via Windows Credential Manager. Optional:

```bash
gh auth login
gh auth status
```

If Git asks for a password over HTTPS with 2FA: use a **Personal Access Token**, not your account password.  
If helper points at missing `E:\Temp\gh-cli\bin\gh.exe`, point it at `C:\Program Files\GitHub CLI\gh.exe`.

## 2) Ship Studio remote — **done**

`origin/main` is published. Clone: `git clone https://github.com/Dendro-X0/ship-studio.git`

## 3) assess-api Worker secrets — **this is the remaining portal goal**

The portal now opens **paste-source pages only** (Polar/GitHub), not five dashboards. Finish the sprint:

```bash
cd "E:/Web Projects/ship-studio"
# Desktop: Open assess-api → Human portal → Paste in terminal
# or CLI:
./target/release/shipctl.exe human --project "E:/Web Projects/assess-api" --put
```

That re-opens each source URL before each `wrangler secret put`. You still must create/copy the values yourself.
## 4) Encrypted backup (after you have values)

```bash
./target/release/shipctl.exe vault export --out "E:/Web Projects/assess-api/ship-secrets.km" \
  --from-hints --project "E:/Web Projects/assess-api"
```

## 5) Redeploy / health (network)

```bash
./target/release/shipctl.exe deploy --project "E:/Web Projects/assess-api"
```

Known URLs:

- Worker: `https://assess-api.paf437sywst688.workers.dev`
- Docs: `https://docs-two-topaz.vercel.app`

## 6) Already done (no action)

| Step | Evidence |
|------|----------|
| GitHub repo + push | https://github.com/Dendro-X0/ship-studio |
| CI workflow | `.github/workflows/ci.yml` |
| `shipctl` + desktop staged | `target/release/` |
| Portal / secrets / guide / vault / human | `cargo test -p shipctl` |
| Cloudflare OAuth (wrangler) | logged in |

```bash
bash scripts/dogfood-offline.sh "E:/Web Projects/assess-api"
```

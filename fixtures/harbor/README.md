# Harbor

Lightweight **demo subject** for Ship Studio — Desktop + Docs (+ Signet). Not a product you sell.

## What you will see on bind (no prior Harbor work)

Studio **auto-probes** the folder (pulse · scopes · Signet). That is detection, not leftover Harbor history.

| Cue | Meaning |
|-----|---------|
| Targets Desktop · Docs | Layout detected from `apps/desktop` + `apps/website` |
| Signet ready | Your machine’s Signet on PATH |
| Deploy “no signal” | Expected — Harbor has never been deployed |
| Publish steps | Only after you open Publish / Refresh (writes `.ship/`) |

Nested under this monorepo → **parent dirty git is ignored**. For demos use intent **Local** (Public ends on Live check, which needs a real hosted URL — Harbor has none).

## Bind

1. Ship Studio → **Open** → this folder (`fixtures/harbor`).  
2. Mode **Advanced** · intent **Local**.  
3. Before GIFs: wipe session state:

```bash
bash scripts/harbor-reset.sh
# Windows (ExecutionPolicy often blocks -File alone):
# powershell -ExecutionPolicy Bypass -File scripts/harbor-reset.ps1
# or: Remove-Item -Recurse -Force fixtures\harbor\.ship -ErrorAction SilentlyContinue
```

## CLI check

```bash
cargo run -p shipctl -- scopes --project fixtures/harbor
# Expect: desktop.desktop · docs.website
```

## Do not

- Record public demos on the **ship-studio** monorepo.  
- Use this for Advanced multipath dogfood — that stays `fixtures/advanced-dogfood`.

See [demo-subject-design](../../specs/frontend/demo-subject-design.md) · [v0.2.1 SCRIPT](../../docs/assets/demo/v0.2.1/SCRIPT.md).

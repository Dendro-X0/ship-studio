# Self-host Deploy finish-fast — design

**Status:** Slice shipped (L1 pending dogfood)  
**Updated:** 2026-09-27  
**Parent:** [host-deploy-confirm-design](../frontend/host-deploy-confirm-design.md) · [selfhost-serve-ui-freeze-investigation](./selfhost-serve-ui-freeze-investigation.md)

```text
GOAL:  Self-host Deploy completes in seconds (Ready), matching a tiny fixture.
       Long-lived loopback is opt-in via Open live / Cancel serve — not the Deploy path.
NOT:   Speeding up hostdeploy / wrangler · changing selfhost detect algorithm
```

## Evidence

| Path | Wall time (debug orbityard) |
|------|---------------------------|
| `orbityard selfhost` (Harbor, no `--serve`) | **~140ms** |
| `orbityard selfhost --serve` | Unbounded until Cancel |

Operator “stuck at 200s” / “finished in a minute last time” = Deploy was wired to `--serve`, so the busy timer never ends after health ok.

## Contract

| Action | CLI | Busy UI | Ends when |
|--------|-----|---------|-----------|
| **Deploy** | `selfhost` | Checking… → Ready | Health written; server stopped |
| **Open live** (Self-host) | If not serving → `selfhost --serve`, open URL on `check ok` | Checking… → Serving | **Cancel serve** (wizard · Results · Output dock · status bar · Esc) |
| **Cancel serve** | kill tree | — | Serve ends |

## Copy

Deploy steps must say: check is fast; Open live holds the process. Do not imply Deploy itself serves forever.

## Proof

| Layer | Check |
|-------|--------|
| L1 | `tsc --noEmit` (desktop) |
| L2 | Harbor Deploy → Ready in &lt;5s, no Cancel serve required |
| L2 | Open live → Serving → Cancel serve → Ready |
| L2 | CLI: `orbityard selfhost --project fixtures/harbor` still &lt;1s |

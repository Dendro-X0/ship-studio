# Guided launch workflow — design

**Status:** Active  
**Updated:** 2026-09-13  
**Owner:** `crates/shipctl` (`launch`) · Desktop / TUI Launch

## Vision

Semi-automated shipping through to **product launch**: for each task, open the official entry **or run the local Signet/Orbit CLI**, operator finishes vendor UI when needed, tool verifies, then next — until sign → release → listing → deploy are done.

## Adaptive plan

| Project signal | Extra steps after secrets/configure |
|----------------|-------------------------------------|
| Tauri / `signet.toml` | identity → `signet build` → `signet ship --plan` → `signet release --dry-run` → `signet release` (confirm tag) |
| Polar markers | listing — open Polar dashboard (marketplace paste is human) |
| Wrangler / Vercel / Netlify | `shipctl deploy` (Orbit) |
| Always | doctor → oauth → paste → configure → intent → flow dry-run → deploy |

Worker-only repos skip desktop Signet build/release unless `signet.toml` exists.

## Step kinds

| kind | Open / Run | Verify |
|------|------------|--------|
| `auto` | optional `run` argv | doctor / studio.json / flow plan / identity list |
| `oauth` | provider login CLI | whoami (20s timeout) |
| `paste` | source URL + optional put | confirm or secret list |
| `sign` | `signet …` via `run` | exit 0 or confirm |
| `list` | Polar/GitHub release URL | confirm (marketplace is human) |
| `deploy` | `shipctl deploy` / orbit | last-run ok or confirm |

## Commands

```text
shipctl launch [--project .]
shipctl launch open|run     # open URL and/or execute step.run
shipctl launch verify|confirm|next|reset
```

## Surfaces

- CLI JSON + stderr prompts  
- Desktop Launch: Open/Run · Verify · Confirm · Next  
- TUI Launch: `o` · `v` · `c` · `n`

## Invariants

1. Bridge does not call vendor HTTPS itself.  
2. Never store secret values.  
3. Official platforms remain the place of marketplace listing / OAuth.  
4. Network steps (live release, deploy) require operator initiation.

## Proof

- L1: tauri fixture includes build/release steps; wrangler fixture skips them  
- L2: `shipctl launch` / verify configure / next  
- L3: Desktop + TUI Launch controls

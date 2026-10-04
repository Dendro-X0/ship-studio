# Game cut — butler push assist (itch)

**Status:** Slice 1b shipped (`ship_butler_push` + `shipctl butler push`)  
**Updated:** 2026-10-04  
**Owner:** `shipctl` MCP/CLI · Publish `submit.itch`  
**Parent:** [marketplace-submit-parity-design](./marketplace-submit-parity-design.md) · [mcp-agent-guide-design](./mcp-agent-guide-design.md) · [close-cut-design](./close-cut-design.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Idle after butler CLI — L3 dogfood optional
ACTIVE BAND / SCOPE:   Game cut · itch butler (CLI/MCP spawn) — slice 1b done
PAUSED / CANCELLED:    Vendor coach · Portal UX expansion · Paddle Solo dogfood · CDP · store API upload
FORBIDDEN THIS TASK:   Desktop Integrations growth · Steam depot automation · Epic upload API
CANONICAL OWNER:       crates/shipctl (mcp + CLI + publish cues)
PROOF BEFORE DONE:     L1 cargo test · L2 shipctl butler push --help · tools list includes ship_butler_push
SURFACE:               CLI / MCP (not Desktop wizards)
```

- Maintainer is developing a game to release later; wants finishable publish/deploy/sign → platforms.
- Freeze stays for Desktop wizard growth; **MCP/CLI spawn of official tools** is the allowed depth (same as `ship_env_put` / `ship_hostdeploy`).
- [marketplace-submit-parity](./marketplace-submit-parity-design.md) forbade *bridge* running butler silently. This band **spawns a visible terminal** (or prints a recipe) — human sees auth/errors; Studio never holds itch credentials.

## Value

itch is the easy store for indie games. Today `submit.itch` only Opens docs. Agents and operators still type `butler push` by hand. Parallel to Deploy: detect `butler` on PATH → recipe + optional spawn.

Steam/Epic stay Open+Confirm only (harder tooling; no depot API).

## Contracts

### Detection

- Existing: `.ship/markets` includes `itch` / `itch.io`, or `itch.toml` present → Advanced `listing.itch` / `submit.itch`.
- Optional: `butler` on PATH → pulse/assist note “butler available”.

### MCP / CLI: `ship_butler_push`

| Arg | Meaning |
|-----|---------|
| `project` | Absolute project path |
| `target` | itch target `user/game:channel` (required for spawn) |
| `dir` | Build directory to push (default: project or `dist`/`build` if present — **require explicit dir when ambiguous**) |
| `spawn` | Default `true` — open external terminal; `false` = recipe only |

Returns JSON: `{ ok, recipe, argv, butler_path?, spawned, hint }`.

Never accepts passwords/API keys. If `butler` missing → `ok: false` + install URL `https://itch.io/docs/butler/`.

### Publish step (optional slice 1b)

`submit.itch` Open may still open docs; if `butler` on PATH and env `SHIP_ITCH_TARGET` set, CLI `publish open` can print the same recipe (no silent push). Prefer MCP tool for agents — avoid surprising Desktop Open.

## Slice 1 (this band)

1. `envx`-style launcher module or `game.rs` / `butler.rs` with `push_launch`.
2. MCP tool `ship_butler_push`.
3. Unit tests: missing butler → ok false; recipe contains `butler push`; rejects empty target when spawn.
4. Docs: mcp-assist inventory · agent skill · handoff · OPERATOR-NEXT itch row.

## Out of scope

- SteamCMD / Steamworks upload  
- Epic Dev Portal automation  
- Desktop Integrations card for itch  
- Storing itch API keys in `.ship/`

### Slice 1b (CLI)

- `shipctl butler push --target user/game:channel [--dir dist] [--no-spawn]`
- `submit.itch` detail names the CLI/MCP path

## Acceptance

- [x] `ship_butler_push` in MCP `tools()`
- [x] Recipe-only works without spawning
- [x] Missing butler returns install hint, no hang
- [x] Skill mentions game → butler path
- [x] No Desktop wizard changes
- [x] CLI `shipctl butler push`

## Proof

| Layer | Proof |
|-------|--------|
| L1 | `cargo test -p shipctl butler` — pass |
| L2 | tools list includes `ship_butler_push` — pass |
| L3 | Maintainer with butler installed: spawn opens terminal (optional) |

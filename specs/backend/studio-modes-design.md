# Studio modes — General & Advanced

**Status:** Implemented  
**Owner:** `crates/orbityard` (`publish`) · `apps/desktop`

## Principle

Optional modes so operators choose density without losing the publish spine.

| Mode | Intent |
|------|--------|
| **General** (default) | Minimal steps to finish a publish pass — doctor → (scopes) → (env) → configure → (self-sign build) → dry-run → deploy → live check |
| **Advanced** | Full personalized path — OAuth, official store signing, Polar listing, Ritual/Tools/Launch/Assist surfaces |

## Publish plan filter (General)

Omit from the adaptive plan:

- `oauth.*`
- `sign.self.scan`, `sign.self.release_dry`, `sign.self.release`
- `sign.official.*` / `sign.official…`
- `listing.*`

Keep: `doctor`, `scopes` (when multi-dir), `env.sprint`, `configure`, `sign.self.build` (when Signet), `dry_run`, `deploy*`, `live_check`.

## Persistence

- Desktop: `localStorage` `orbit-yard.mode` = `general` | `advanced`
- Publish state: `.ship/publish.json` field `mode` — changing mode rebuilds the plan (progress for removed steps is not merged)

## CLI

```text
orbityard publish --mode general|advanced [--project .]
orbityard publish reset --mode …
```

## Desktop shell

- Topbar segmented control: General | Advanced
- `body[data-mode=…]` hides Advanced-only nav (Assist, Launch, Portal, Ritual, Tools) and Deploy toggle in General
- All `orbityard publish …` invocations pass `--mode`

## Invariants

No vendor HTTPS from bridge; no secret values in `.ship/`.

## Proof

- L1: `cargo test -p orbityard publish::`
- L1: `npx tsc --noEmit` in `apps/desktop`
- L3: Toggle General → shorter Publish list; Advanced → Related + Assist/Tools visible

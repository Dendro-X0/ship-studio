# Desktop reliability — investigation

**Status:** Investigation (extend slice-1 silent-failures; **no code this band**)  
**Updated:** 2026-09-25  
**Parents:** [desktop-silent-failures-investigation](./desktop-silent-failures-investigation.md) · [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md)  
**Owners:** `apps/desktop/src/main.ts` · `apps/desktop/src-tauri/src/lib.rs` · shipctl interactive CLIs

```text
QUESTION:  After Portal Login CLI / soft-fail toast slice 1, which Desktop paths
           still fail silently, stick FAILED, or run interactive CLIs headless?
OUTCOME:   Failure-class matrix + evidence for desktop-reliability-design.md
```

## Method

1. Re-read slice-1 investigation (F1–F4) and what shipped in [46384c4](https://github.com/Dendro-X0/ship-studio/commit/46384c4).
2. Static audit of Desktop `run()` / `run_shipctl` / terminal openers / Env Put / Ritual / loadJsonCmd.
3. Trace shipctl `secrets::put_secret` and human put for stdin requirements.

## Slice 1 status (done)

| Class | Status |
|-------|--------|
| F1 Portal Login CLI headless | Fixed — `open_portal_login_terminal` |
| F2 Orbit → portal provider | Fixed — Platforms guard / no Orbit portal id |
| F3 Silent early return after fail | Partially fixed — portal/openPortalProvider toast stderr; **loadJsonCmd still swallows** |
| F4 Sticky FAILED for soft errors | Partially fixed — gate pending + `isSoftCmdFailure`; taxonomy still thin |

Also already terminal-backed: Publish Open · Launch Open · Human Put (`open_*_terminal`).

## Remaining failure classes

### R1 — Env Put still headless (F1 sibling) — **slice 1 fixed**

Env Put now calls `open_env_put_terminal` (`shipctl env --provider … --put …` in wt/cmd). Still a fifth terminal opener until slice 2 unifies.

### R2 — Terminal opener sprawl + Windows-only — **slice 2 fixed**

All TTY flows use `open_shipctl_terminal(project, args, title?)`. Non-Windows still returns an explicit Err (later band).

### R3 — Soft-fail taxonomy incomplete — **slice 2 expanded**

`isSoftCmdFailure` now also matches missing CLI / `has no secret put CLI` / Windows “not recognized”. Further polish is fine; sticky FAILED should stay rare.

### R4 — Silent `loadJsonCmd` (F3 remainder)

```ts
run(..., { quietHeader: true, silent: true by default })
if (!result?.ok || !result.stdout) return null;
```

Used for pulse, scopes, env load, assist, publish refresh. User-initiated Refresh that fails looks like “nothing happened.” Bind-time doctor is separate and louder.

### R5 — Ritual Deploy / Sign / Flow honesty

Ritual **Deploy** / **Sign** / **Flow** use headless `run()`. Many are fine offline/JSON. Risk:

- Orbit deploy when auth expired → interactive browser/login inside headless process.
- Flow including deploy same class.
- Signet build usually non-interactive (OK headless).

Not all Ritual buttons need a terminal — need a **command class** (json-safe · interactive · network-may-prompt).

### R6 — Global busy lock UX

Single `running` gate; Cancel unlocks. Busy toast debounced. Operators still confuse “stuck FAILED” with “busy.” Reliability design must keep Cancel as the escape hatch and never leave `running` true after throw (already in `finally`).

### R7 — CDP / CodaCtrl dogfood

Still blocked without Tauri `--remote-debugging-port`. Out of reliability slices 1–3; separate attach recipe.

## Action matrix (Desktop → execution mode)

| Surface | Action | Today | Needed |
|---------|--------|-------|--------|
| Portal | Load / Open URLs | headless JSON / browser | OK |
| Portal | Login CLI | **terminal** | OK (slice 1) |
| Publish / Launch | Open / Run | **terminal** | OK |
| Human | Put loop | **terminal** | OK |
| Env | Load portal | headless JSON | OK if fail toasted when user-clicked |
| Env | **Put** | **terminal** (`open_shipctl_terminal`) | OK (slices 1–2) |
| Ritual | Doctor / Guide / Configure / Status / dry-run | headless | OK (json-safe) |
| Ritual | Sign | headless | OK if Signet non-interactive |
| Ritual | Deploy / Flow (network) | headless | Prefer terminal when Orbit may prompt; or honest toast “run in terminal” |
| Tools | Cancel / Copy | local | OK |
| Vault export | passphrase | `run_shipctl_env` | Keep env-injected passphrase; no prompt echo |

## Non-goals (this investigation)

- Softening Confirm / Verify honesty
- Polar E2E / commerce checkout
- Expanding Tier D Platforms catalog
- Full macOS/Linux terminal polish in the first code slice (design may schedule it)

## Recommended next artifact

[desktop-reliability-design.md](./desktop-reliability-design.md) — command classes, unified terminal opener, Env Put slice, soft-fail table, ordered slices + proof.

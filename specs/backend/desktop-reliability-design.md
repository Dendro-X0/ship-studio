# Desktop reliability — design

**Status:** Slices 0–3 shipped (working tree) — slice 4 next (Ritual N-class)  
**Updated:** 2026-09-25  
**Investigation:** [desktop-reliability-investigation.md](./desktop-reliability-investigation.md)  
**Parents:** [desktop-silent-failures-investigation](./desktop-silent-failures-investigation.md) (slice 1 shipped) · [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md)

```text
GOAL:     Every Desktop action is either (a) headless JSON-safe with actionable
          fail toasts, or (b) an interactive terminal the operator can complete —
          never a silent FAIL / sticky red dock for expected human work.
NOT:      Studio-held secrets · auto-Done on Human gates · CDP without attach recipe
```

## Invariants

1. **TTY rule** — If shipctl or a child CLI needs stdin/TTY (put secret, OAuth login, `publish open` vendor CLIs), Desktop opens an interactive terminal. Never `run_shipctl` with `stdin: null` for those.
2. **Toast rule** — Non-ok headless runs always surface polished stderr (or a named soft reason). User-initiated loads never fail silently.
3. **Dock rule** — Sticky **FAILED** only for spawn/crash/cancel-ambiguous hard errors. Soft failures → Ready + err toast.
4. **One opener** — New interactive flows add args to a shared Tauri helper, not a fifth copy-pasted `open_*_terminal`.
5. **Cancel** — `running` always clears in `finally`; Cancel remains the unlock.

## Command classes

| Class | Examples | Desktop execution |
|-------|----------|-------------------|
| **J — JSON-safe** | `pulse`, `portal` (plan), `env` (plan), `doctor`, `scopes`, `publish` (view/verify JSON) | `run_shipctl` headless |
| **T — Interactive TTY** | `portal --login`, `env --put` / `secrets put`, `human --put`, `publish open`, `launch open` | `open_shipctl_terminal(args)` |
| **N — Network may prompt** | `deploy`, `flow` (with deploy), Orbit when logged-out | Prefer **T** on Windows for v1; or J + toast “auth expired — Login CLI / terminal deploy” if detectably non-interactive failure |
| **L — Local UI only** | Open URL, setView, Cancel, Copy | No shipctl |

Map every button to J / T / N / L in implementation checklist; unmapped = bug.

## Soft-fail taxonomy (expand `isSoftCmdFailure`)

Treat as Ready + err toast (not sticky FAILED):

| Pattern / code | Operator copy |
|----------------|---------------|
| Confirm / Verify / still pending | Existing gate toast |
| unknown provider | Platforms Open dashboard |
| not a directory / bind missing | Bind a project |
| program not found / cannot find / No such file.*(wrangler\|vercel\|netlify\|orbit\|signet\|fly\|railway) | Install CLI or use PATH — Docs on Platforms |
| has no secret put CLI | Open vendor dashboard (honest bail) |
| cancelled | Cancelled label |

Hard FAILED: spawn failure, lock poisoned, unexpected panic text, empty fail with no classifiable stderr.

## Structural changes

### 1. Unify terminal opener (Tauri)

Replace four commands with one:

```text
open_shipctl_terminal(project, args: Vec<String>, title?: String)
```

Windows: `wt -d project shipctl …args` → fallback `cmd /C start …`.  
Non-Windows (slice later): `x-terminal-emulator` / `osascript` best-effort, or clear Err toast “open a terminal and run: …”.

Keep thin wrappers or migrate call sites to the unified command in one slice.

### 2. Env Put → class T

`applyEnv` Put button → `open_shipctl_terminal([env, --project, …, --provider, …, --put, name])` (or `secrets put` equivalent). Mirror Human Put toast: “finish paste in the terminal.”

### 3. User-initiated JSON loads

Split `loadJsonCmd`:

- **silent** (bind/pulse background): fail quiet OK  
- **user** (button Refresh / Load env): on null/non-ok → `toast(cmdFailDetail)` + Preview action  

### 4. Ritual N-class policy (slice after Env Put)

Deploy / Flow: open terminal for `shipctl deploy` / `shipctl flow` when Advanced + not Offline; keep dry-run / doctor as J. Document in Ritual hint.

## Slices (ordered)

| Slice | Change | Proof |
|-------|--------|-------|
| **0 — Spec freeze** | This design + investigation; handoff activation; action→class checklist in design | **Done** — specs + handoff |
| **1 — Env Put terminal** | Put → interactive terminal; toast parity with Human Put | **Done** — `open_env_put_terminal` · L1 tsc + cargo check |
| **2 — Unified opener + soft taxonomy** | `open_shipctl_terminal`; expand `isSoftCmdFailure`; migrate existing openers | **Done** — L1 tsc · cargo check |
| **3 — User-initiated load honesty** | Load env / Refresh assist toast on fail | **Done** — `loadJsonCmd({ user })` · L1 tsc |
| **4 — Ritual N-class** | Deploy/Flow terminal or honest auth-fail copy | L2 Harbor Public deploy path |
| **Later** | Non-Windows terminals · CDP attach recipe · vault UX | Separate bands |

## Out of scope

- Polar paid checkout E2E  
- Softening Confirm honesty  
- Drive-by Platforms catalog changes  
- Auto-detecting OAuth success  

## Proof plan

| Layer | Check |
|-------|--------|
| L1 | `pnpm exec tsc --noEmit` (desktop) · `cargo check -p ship-studio-desktop` (or desktop package name) |
| L2 | Harbor → Env → Put → terminal opens; paste works in wt/cmd |
| L2 | Portal Login CLI / Publish Open unchanged |
| L2 | Missing wrangler → Ready + toast, not sticky FAILED |
| L2 | Load env with shipctl missing → toast (user path) |

## Activation

Handoff **Next Atomic Step** names **slice 0 complete → activate slice 1** (or park). No reliability code until slice 1 is quoted.

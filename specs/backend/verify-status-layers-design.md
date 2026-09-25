# Verify status layers — disk · local CLI · operator probe

**Status:** Design → implement (slice 1)  
**Parent:** PRODUCT offline-first · portal/guide (never replace providers) · Verify honesty  
**Owner:** `shipctl` publish `verify_current` / Watch · Desktop Offline + Publish hint  
**Updated:** 2026-09-24

## Goal

Resolve offline vs “detect publish/sign status” without Studio becoming an online control plane that holds secrets.

```text
Secrets / env / OAuth  →  offline guide (Open · paste · Confirm)
Status detection       →  graduated Verify (disk → local CLI → operator CLI)
Provider remains authority for irreversible “done”
```

**Naming:** These are **status layers** (`disk` · `local_cli` · `operator_cli` · `human_attest`).  
Do **not** confuse with CI proof layers L1–L4 in other specs.

## Layers

| Status | JSON | What Studio inspects | Network |
|--------|------|----------------------|---------|
| **Disk** | `disk` | Project files, `.ship/*` (no secret values), scopes, TRUST/LICENSE | No |
| **Local CLI** | `local_cli` | Signet / Orbit / doctor / local deploy pulse / `last-run` | Only what that local tool does |
| **Operator CLI** | `operator_cli` | Read-only probes via tools the operator already uses (`gh`, `wrangler whoami`, …) | Possible — **only when Verify/Watch runs**; never Studio-owned HTTPS with tokens |
| **Human attest** | `human_attest` | No automated green light — Open official UI, then Confirm | No Studio probe |

## Invariants

1. Bridge never calls vendor HTTPS with secrets.  
2. Env / token / OAuth stay operator-initiated (Open → paste-put → Confirm).  
3. Watch polls **Verify only** (same layers as a manual Verify click).  
4. Operator CLI probes are **read-only**; Confirm still required for honesty gates.  
5. Offline topbar toggle remains “prefer offline-safe sign / refuse deploy” — separate from status layers; copy must say the **bridge** does not require network.

## Slice 1

| Surface | Change |
|---------|--------|
| `PubStep.verify_status` | Inferred at plan build; serialized on Publish view |
| PRODUCT / SCOPE / OPERATOR-NEXT | Policy section + Offline badge meaning |
| Desktop | Offline title; Publish hint + stage guideline show status layer; Watch toast |

## Non-goals (slice 1)

- Studio OAuth / store API polling  
- Background vendor HTTPS while editing env  
- Auto-Confirm when operator_cli returns ok  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `cargo test -p shipctl verify_status_` · desktop `tsc` |
| L2 | Bind fixture → Publish hint shows Disk/Local CLI/… · Watch toast mentions local Verify |

## Later

- Per-provider “done” criteria table (backlog S1.12)  
- Optional explicit **Status check** button that only runs operator_cli steps  

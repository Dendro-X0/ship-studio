# Fixture — Flowsint OSINT Get-live

Used by [flowsint-get-live-design](../../specs/backend/flowsint-get-live-design.md).

| Slice | State |
|-------|--------|
| **F1** | Detect `osint.flowsint` / compose image markers |
| **F2** | `orbityard flowsint ensure` on this stub (root `.env` only) |
| **F3** | CLI deploy exists — use a **real** [reconurge/flowsint](https://github.com/reconurge/flowsint) clone for Soft-L3 (this stub is detect/ensure only) |
| **F4** | Desktop card shipped — Soft-L3 still needs a real Flowsint clone |

For a real Soft-L3, clone [reconurge/flowsint](https://github.com/reconurge/flowsint) and bind that folder — this stub is detect-only.

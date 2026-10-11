# Evidence — F4 MCP collaborator ritual — 2026-10-07

**Status:** Kernel + skill shipped · L1 tests  
**Band:** [founder-minimal-spine-design](../../specs/backend/founder-minimal-spine-design.md) F4  
**Not:** Integrations growth · auto-Confirm · commercial ADMIT

## Claim

MCP Get-live ritual is aligned to the founder five-step spine:

`yard_setup` → (pause human) → `yard_resume` → Verify/Confirm — with `setup.spine` naming step 1–5 and soft-no on Integrations-as-path.

## Shipped

| Surface | Change |
|---------|--------|
| `SetupBrief.spine` | `orbit-yard/spine/v1` · `step` · `label` · `path` |
| `pause.prompt` | Prefixed `Founder spine N/5 (label)` |
| `agent.never` | `use Integrations as the Get live path` · `skip yard_resume…` |
| Skill | [orbit-yard-mcp-agent](../../.cursor/skills/orbit-yard-mcp-agent/SKILL.md) Get-live ritual |
| Tests | `founder_spine_maps_get_live_gates` · MCP setup asserts `spine` |

## Ritual (agent)

```text
yard_setup → read spine.step + pause
paused? → STOP · show prompt · wait for human “resume”
yard_resume → verify · need_confirm?
human Confirm → yard_publish_confirm → yard_publish_next
loop until phase=done
```

## Proof

```text
cargo test -p orbityard founder_spine_maps_get_live_gates setup_brief_mcp_returns_v1 -- --nocapture
```

## Next

**F5** — multi-repo next ship (local) · or Desktop Run B stopwatch from F3.

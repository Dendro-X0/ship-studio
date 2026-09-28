# Dirty-tree Publish cue — design (S1.15)

**Status:** Shipped  
**Updated:** 2026-09-28  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.15 · [project-pulse-design](../backend/project-pulse-design.md)

```text
GOAL: Soft warn before Confirm on release/deploy/listing gates when the tree is dirty.
NOT: Hard block · refuse Confirm · nag on every Auto Continue · nested fixture false dirty.
```

## Behavior

| Surface | When | What |
|---------|------|------|
| Publish strip | `pulse.git.dirty` && project bound | Quiet cue: “N uncommitted — Confirm on release/deploy only if you intend this tree” |
| Confirm (toolbar / stage primary) | Dirty **and** current step is deploy / listing / live_check / release / desktop_cut | Toast once → **Confirm anyway** · **Git status** |
| After Confirm anyway | Same dirty session | Arm skip until dirty clears or project rebinds |

Pulse already surfaces dirty on Dashboard checklist — this adds Publish Confirm soft gate only.

## Non-goals

- Blocking Continue Auto gates  
- Changing nested-fixture parent-git ignore (already done)  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `tsc` |
| L2 | Dirty Harbor repo · Confirm on live_check/deploy → toast · Confirm anyway advances |

## Shipped

- `#publish-dirty-cue` on Publish when `pulse.git.dirty`
- Confirm (toolbar / stage primary / wizard gate) soft-warns on deploy · listing · live_check · release · desktop_cut
- **Confirm anyway** arms until dirty clears or project rebinds
- Continue Auto gates untouched

# Wizard completion Confirm — design (S1.11)

**Status:** Shipped (L1 tsc)  
**Updated:** 2026-09-27  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.11 · [integrations-publish-handoff-design](./integrations-publish-handoff-design.md)

```text
GOAL: After Open dashboard (+ env paste outside Studio), operator can Confirm the
      matching Publish gate from the wizard — advancing `.ship/publish.json` without
      hunting the Confirm button on Publish.
NOT: Auto-Confirm after Open · confirming a non-current gate · Studio writing secrets.
```

## Behavior

| Surface | Preferred step | Confirm button |
|---------|----------------|----------------|
| Integrations | `listing.{polar\|stripe\|…}` | **Confirm gate** next to Continue publishing |
| Deployment | `live_check` / deploy host gate | same |
| Sign | sign-related current gate | same |

On click:

1. Quiet-load Publish plan  
2. Prefer step missing → honest toast (Advanced + Public / detect) — no Confirm  
3. Prefer step done → toast already done  
4. Prefer step **not current** → toast + **Open Publish** (S1.10 handoff) — no Confirm  
5. Prefer step **current** → `setView(publish)` · `publishAction(["confirm"])`  

## Non-goals

- S1.12 Verify criteria (URL vs webhook)  
- Burning Auto Continue after Confirm from wizard (Publish Confirm path already does that)  
- Resend inventing a fake listing step  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apps/desktop` `tsc --noEmit` |
| L2 | Harbor Advanced+Public · Polar listing current · Integrations **Confirm gate** advances plan |

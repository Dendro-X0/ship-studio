# Vendor handoff coach — design

**Status:** **CANCELLED** (maintainer) — 2026-09-25  
**Investigation:** [vendor-handoff-coach-investigation.md](./vendor-handoff-coach-investigation.md)  
**Parents:** [PLATFORMS-AND-PORTAL](../../docs/product/PLATFORMS-AND-PORTAL.md)

```text
CANCELLED: In-app coach card for vendor env handoff. Failed product test —
           shell-bound UI, Open still dumped operators into incomprehensible
           vendor surfaces, not founder-accessible. Do not resume without a
           new charter. Prefer official CLI / agent / chatbot for host setup;
           Studio keeps TTY Put + honest Docs, not faux onboarding theater.
```

## Why cancelled

1. Coach lived inside the Desktop shell (corner card), not a true OS-level overlay.  
2. Primary CTA still sent people at dashboards/docs they find unusable.  
3. Half-measure between “guide the vendor” and “stay CLI-native” — neither path won.

## Subtracted

Desktop `#vendor-coach` + Portal env Open intercept removed. Portal Open/Docs restored to naked `openUrl`. Env **Put** terminal path (reliability) remains.

## Do not

- Re-add in-app vendor onboarding cards without an explicit new design + maintainer activation  
- Treat Docs as the default “setup” path for non-technical founders  

## Remaining product truth

Studio = local portal / sequence / TTY put. Host account setup = vendor CLI, agents, or chatbots — not Studio theater.

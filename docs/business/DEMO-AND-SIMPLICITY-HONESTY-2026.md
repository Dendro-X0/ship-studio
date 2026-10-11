# Demo & simplicity honesty

**Status:** Lock — applies to outcome SKUs and Orbit Yard presentation  
**Updated:** 2026-10-09  
**Parents:** [OUTCOME-SKU-PAIN-DEMO-2026](./OUTCOME-SKU-PAIN-DEMO-2026.md) · [presentation-readiness](../../specs/backend/presentation-readiness-design.md) · never-say / host XOR honesty

## 1. Two products, two demos (do not mix)

| What you’re selling | Demo must show | Steps must match |
|---------------------|----------------|------------------|
| **Orbit Yard** (sequencer) | Bind → Get live → Deploy → **their** host URL | What a user does **in Studio/CLI** — not a random Docker tutorial |
| **Outcome kit / H1** (e.g. Umami live) | Compose/TLS → dashboard → pageview | What the **kit buyer** does (or what **you** do in standup) — **not** Studio chrome unless Studio is in the path |

Recording Umami via Orbit **today** would be dishonest: Desktop does **not** yet shorten Umami vs official Docker docs. That would be “extra Studio steps.”

## 2. Simplicity bar (aspire to one-click; stay honest)

```text
Script everything that is scriptable.
One loud human gate where the vendor or SSH is authority.
Never fake one-click past Login / secrets / Confirm.
```

**Justify use only if:** fewer confused steps or fewer failure modes than the official path for the **same wall**.  
If the official path is shorter → **do not sell Studio for that wall**; sell kit/H1 or build the missing automation first.

## 3. Demo production rules (low labor)

| Rule | Why |
|------|-----|
| **Silent** ≤90s (titles/captions only) **or** human-mic 3–8 min | Operator mic OK (2026-10-10); still no AI VO |
| **One wall per clip** | Durable; cheap to replace |
| **Stable UI only** | Avoid clips of chrome that churn weekly |
| **Prefer live Soft-L3 when selling** | Call/screen-share beats stale film |
| **No 10–20‑minute edited hero videos** | Time sink; stale on every update |
| **GIF shelf optional** | Only for Orbit `/demo` after product delta is real |
| **Sequence** | Obscur ops demos first · Ship second · Velocity/Strata later ([170](../../../new-business/research/170-ops-media-ready-obscur-ship.md)) |

**Do not** casually record “because the plan said Soft-L3.” Record only when (a) the path is the real buyer path and (b) a short silent **or** honest human-narrated capture is enough.

## 4. Orbit #1 implication

Until Orbit has a real Umami/VPS Get-live delta:

1. **Do not** spend days filming Orbit-as-Umami.  
2. **Either** build the simplification (detect compose → doctor Docker → deploy recipe → open Admin) **then** demo Studio,  
3. **Or** sell/standup the kit **without** Studio in the demo,  
4. **Or** park #1 and keep Orbit demos on the **already-attested** host URL wall (website/Pages) — short silent GIF/MP4 of that spine only if needed.

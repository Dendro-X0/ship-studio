# Status probe UX — Sign · Deploy readiness strip

**Status:** Design → implement (slice 2 visual)  
**Parent:** verify-status-layers · public-publish-ux  
**Updated:** 2026-09-24

## Goal

When operators open **Sign** (and Publish), show a **status probe**: brief detection animation, then clear lanes for **self-sign** vs **official** (Apple / Windows / …) vs **deploy** — local evidence only; official paths stay “guide / not attested.”

The probe is an **inspection bay** — not a flat bullet list. Discerning operators get icons, state badges, one suggestion per lane, and a dedicated layout that reads apart from the stage checkpoint.

```text
Open view → checking… (script pulse) → three lane cards settle
Official store signing = Human attest (Open vendor) — never fake Done
```

## Slice 1 (shipped)

| Surface | Behavior |
|---------|----------|
| Sign | Auto-probe on enter: Signet · official guide · deploy from Pulse |
| Publish | Compact probe under progress when plan loaded |
| Animation | ~420ms `checking`, then settle; `prefers-reduced-motion` |

## Slice 2 (this pass) — visual elevation

| Element | Spec |
|---------|------|
| Layout | Distinct **inspection bay** panel — accent edge, soft atmosphere, own grid (not a status line) |
| Lanes | Three cards: Self-sign · Official · Deploy — icon glyph + state badge + detail + **suggestion** |
| Icons | Inline SVG (key / store / rocket) — no emoji |
| Checking | Head shows “Running local probe…” + spinner; cards shimmer / pulse marks |
| CTAs | Optional chip per lane when actionable (Open Sign · View paths · Check again) — never invent vendor OAuth |
| Compact | Publish keeps bay; cards may stack below ~720px |
| Stage panel | Checkpoint gains stronger surface (left rail, denser hierarchy) so it is not a twin of the probe |

### Suggestion copy (local honesty)

| Lane state | Suggestion |
|------------|------------|
| Self-sign ok | Ready for local desktop cuts |
| Self-sign missing | Install Signet, then Check status |
| Official (any) | Finish on Apple / Microsoft / store UIs — Studio only guides |
| Official none | Self-sign is enough for this layout |
| Deploy ok | Prior evidence found — redeploy when you cut again |
| Deploy linked | Configured — run deploy when releasing |
| Deploy missing | Link host / run Orbit when you need a live URL |

## Non-goals

- Studio OAuth to Apple/Microsoft to poll signing status  
- Replacing Signet / Orbit CLIs  
- Full rail diet (slice B) in this pass  

## Proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Open Sign / Publish → bay with icons · badges · suggestions; checking pulse then settle |

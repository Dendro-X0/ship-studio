# Workflow stage flow — dashboard cards + linear Publish pager

**Status:** Design → implement (slice 1) — **Desktop wired**; L2 dogfood pending  
**Parent:** PRODUCT portal/guide · Continue honesty · progress clarity  
**Owner:** `apps/desktop` Dashboard + Publish · presets via existing `--mode` / `--intent` (no new shipctl plan engine in slice 1)

## Goal

Operators pick a **workflow** on the Dashboard (large cards), then move through a **linear stage pager** on Publish: one checkpoint at a time, short guidelines, light motion — not a flat checklist of every Adaptive lane.

```text
Dashboard cards → apply mode/intent → Publish stage pager
Continue / Open / Confirm still own honesty.
```

## Non-goals

- Replacing Adaptive Publish / Continue semantics  
- Auto-Confirm Human / OAuth / deploy / listing  
- Fake “one-click deploy” cards  
- New competing nav destinations  
- Full custom plan graphs in shipctl (later slice)

## Workflow cards (slice 1)

| Card id | Title | Mode | Intent | When to use |
|---------|-------|------|--------|-------------|
| `sign_only` | Sign only | general | local | Desktop / OSS cut — no hosted deploy |
| `sign_deploy` | Sign and deploy | general | public | Short hosted final-mile |
| `publish_platform` | Publish to platforms | advanced | public | Listings · stores · Polar / commerce |
| `deploy_only` | Deploy focus | general | public | Same General Public spine; copy stresses deploy gates (honesty still requires prior Auto/human when present) |

Cards are **interaction containers** (allowed). Selecting a card:

1. Sets mode + intent (topbar stays in sync)  
2. Resets publish plan for the bound project  
3. Opens Publish in **stage mode**  
4. Toasts one-line guideline for that workflow  

## Stage pager (Publish)

When `body[data-publish-ui=stages]` (default on after first card pick; toggle “List” restores bands):

- **Checkpoint rail** — dots/labels for Required-now + Later condensed; Done collapsed  
- **Stage card** — current step only (title, guideline, ~min, Open / Continue / Confirm)  
- **Pagination** — Back / Next *within the plan* only advances UI focus when step is Done/Skipped; otherwise Primary stays Continue/Open/Confirm  
- **Motion** — fade/slide (~180ms) on stage change; respect `prefers-reduced-motion`  

Guidelines (examples):

| Kind | Guideline |
|------|-----------|
| human / scopes | Save selection here, then Confirm on Publish |
| oauth / list / deploy | Open the official UI, finish there, return → Confirm |
| auto | Continue will verify and advance when ready |
| sign (safe) | Run local Signet, then Confirm |

## Slice 1 proof

| Layer | Check |
|-------|--------|
| L1 | desktop `tsc` |
| L2 | Bind fixture → pick Sign only → Local+General · stage pager shows Scopes/current |
| L2 | Human gate: stage Next does not skip Confirm |

## Later

- Persist `workflow_preset` in `.ship/studio.json`  
- shipctl plan filters per preset (true Deploy-only / Sign-only step sets)  
- Platform-specific cards (Polar / Steam) from detection  
- GIF T1 updated to show card pick  

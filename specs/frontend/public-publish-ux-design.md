# Public Publish UX — reliability for a $29 stranger

**Status:** Design → implement — **slices A+C shipped** (single primary · inline Scopes); B/D/E/F pending  
**Parent:** PRODUCT portal/guide · workflow-stages · progress clarity · verify status layers  
**Evidence:** Desktop dogfood screenshots 2026-09-24 (Advanced Public stage rail · General Sign-and-deploy · Scopes detail)  
**Updated:** 2026-09-24

## Verdict

Ship Studio is **not yet worth $29 to a stranger** as a *general-public* product. The Adaptive plan and honesty model are sound; the **Publish shell fails the first-session clarity test**. Operators face three competing UIs at once (toolbar · chip rail · checkpoint card), jargon without payoff, and a detail-panel bounce that feels like busywork.

Agents do not replace this product — but **this UI currently sells “another dense checklist”**, which agents undercut for free.

```text
Keep:     Detect · plan · Continue / Open→Confirm · status layers · workflow cards
Change:  One primary action · hide Advanced density · inline human gates · plain copy
```

## Audience correction

| Claimed ICP | What strangers actually need first |
|-------------|-------------------------------------|
| Indie / multi-repo shippers | Still true — but first cut must feel **General + one job** |
| “General public” | Not consumers — **technical operators who are not Studio maintainers**. Bar = install → bind → finish a Sign-only or Sign-and-deploy pass without reading PRODUCT.md |

Do **not** dumb down honesty (auto-Confirm stores). Do **dumb down chrome**.

## Failure modes (from screenshots)

### 1. Triple action bar (same job, three places)

Publish shows **Refresh · Open Scopes · Open/Run · Verify · Watch · Needs Open · Confirm · Next** *and* checkpoint **Back · Open · Next**.

- “Needs Open” vs card **Open** vs **Open Scopes** = three labels for one intent.  
- Confirm/Next live in the toolbar while the card says the story — split attention.

**Fix:** In **Stages** mode, toolbar collapses to **Refresh · Watch** (secondary). Primary actions live **only** on the checkpoint card: one green CTA + Confirm when pending human gate is ready.

### 2. Stage rail is not a pager — it’s the whole plan

Advanced Public shows **~20 chips** (Scopes → notarization → stores → marketing). That reintroduces the flat Adaptive checklist the stage pager was meant to replace.

**Fix:** Rail shows **Required-now only** (≤5–7): current + next honesty gates + one “+N later” affordance. Full plan stays behind **List** / “Show all steps”.

### 3. Human gate bounce is the product, but UX makes it feel broken

Card: *“Save in the detail panel, then return here and Confirm.”*  
Scopes panel: save → toast → hunt **Back to Publish** → find Confirm in a crowded bar.

**Fix — inline checkpoint (slice A):**

```text
Publish stage (Scopes)
  ├─ Guideline (one sentence)
  ├─ Embedded scopes picker (or full-height sheet)
  ├─ [Save selection]
  └─ [Confirm & continue]   ← only enabled after save / verify ok
```

Detail panels remain for deep edits; **happy path does not leave Publish**.

### 4. Jargon tax

| Current | Public copy |
|---------|-------------|
| Minute spine | Your publish checklist |
| Auto gates / status probe | We’ll check local files & tools |
| Needs Open | Open this step |
| PENDING · human · Opens Open Scopes | Action needed — choose what you’re shipping |
| ship-studio.json / Signet build in chip titles | Configure · Build & sign (detail on expand) |

### 5. Mode / intent invisible consequences

Advanced + Public on a dogfood repo → 50 min / 20 steps after picking “Publish to platforms”. Stranger feels punished.

**Fix:** Workflow card must **preview** before start: “About 7 steps · ~12 min · includes stores.” Switching to Advanced mid-flight shows a one-line cost: “Full path unlocked — N more optional lanes.”

### 6. Sidebar competes with the spine

Targets + five payment logos + Assist/Launch/Portal while mid-Scopes teaches “many apps,” not “one workflow.”

**Fix (General):** Collapse Integrations to **“Integrations”** single entry; hide Assist/Launch/Portal behind Advanced or “More.” Targets stay (they are the scopes outcome).

## Target experience (first 10 minutes)

1. Bind folder.  
2. Pick **one** workflow card (job-shaped).  
3. Publish shows **one** checkpoint, **one** primary button, progress `3 done · 1 next · 3 later`.  
4. Human step: edit **on the same screen** → Confirm & continue.  
5. Auto steps: Continue burns them; Watch optional.  
6. Finish: plain “This cut’s required gates are done” — not jargon.

$29 test: stranger completes Sign-only (or Sign-and-deploy on a web fixture) **without asking what Confirm means**.

### 7. Actionable toasts lied about “official path” (2026-09-24)

Configure (Auto · local `studio.json`) was offered **Open Ritual** + “official path” because `desktop_view=ritual`. Verify failures also toasted raw `shipctl publish confirm` and a useless **publish failed**.

**Fix:** Classify gates (`continue` · `open` · `confirm`); polish/suppress CLI copy; `publishAction` owns toasts (`quietToast`).

## Slice plan (implementation order)

| Slice | Change | Proof |
|-------|--------|-------|
| **A — Single primary** | Stages mode: demote toolbar; card owns Open / Continue / Confirm | **Done** — toolbar keeps Refresh · Watch; `.stages-hide` for the rest |
| **B — Rail diet** | Required-only chips + “+N later” | **Done** — ≤6 required chips + `+N later` → List ([gif-ready-polish](./gif-ready-polish-design.md)) |
| **C — Inline Scopes** | Embed scopes detect/save on checkpoint; Confirm on card | **Done** — `#stage-inline-scopes`; Confirm & continue saves then Confirms |
| **D — Copy pass** | Replace minute-spine / Needs Open / probe copy | Partial (Publish hint) |
| **E — Workflow preview** | Card shows ~steps · ~min before reset | Pending |
| **F — General nav quiet** | Collapse Integrations / More | Pending |

Non-goals this pass: redesign sidebar brand, light theme, agent chat, replacing Adaptive engine.

## Reliability (product, not just pixels)

Public trust = **predictable outcomes**:

- Verify never auto-Confirms honesty gates (already).  
- Workflow card preview matches post-reset plan length (±2 steps).  
- Confirm always visible when status is pending human (not buried).  
- Offline / status-layer copy stays one sentence on first Publish visit.

## Relationship to AI agents

Improve **instrument clarity**, not “add an agent.” Agents amplify a clear spine via MCP; they cannot rescue a 20-chip cockpit.

## Next atomic step

Maintainer picks slice **A+B** (chrome only, no Adaptive change) or **A+C** (Scopes inline — higher value, more work). Then implement from this spec; update handoff.

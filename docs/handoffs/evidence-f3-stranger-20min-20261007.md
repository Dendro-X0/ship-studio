# Evidence — F3 stranger 20-minute Get live — 2026-10-07

**Status:** Run A + **Run B stopwatch complete** (self / operator proxy)  
**Band:** [founder-minimal-spine-design](../../specs/backend/founder-minimal-spine-design.md) F3  
**Protocol:** [stranger-20min-get-live-script.md](./stranger-20min-get-live-script.md)  
**Not:** Commercial ADMIT · Integrations path · cold third-party stranger

## Claim

The five-step Get live spine is **measurable** on CLI and on the **Desktop Deploy path** (same `hostdeploy` kernel Desktop streams). F1 Get-live primary + F2 one-recovery are present in Desktop source. A true cold stranger at the glass remains optional follow-up.

## Run A — CLI warm path (clocked)

| Field | Value |
|-------|-------|
| Date | 2026-10-07 (~20:10 UTC−12) |
| Surface | `orbityard hostdeploy` (release binary) |
| Project | `apps/website` |
| Host | Cloudflare Pages · `--name ship-studio` |
| Prior login | **Yes** — `auth_required: false` |

| Step | Action | Wall |
|------|--------|------|
| 3′ | `hostdeploy --auth-check` | **8 s** |
| 4–5 | `hostdeploy` | **12 s** |
| **Total** | | **~20 s** |

**Live URL:** https://93bf998e.ship-studio.pages.dev  
**Score:** **Pass**

## Run B — Desktop Get-live stopwatch (2026-10-07 ~20:34 UTC−12)

| Field | Value |
|-------|--------|
| Status | **Timed** |
| Surface | Desktop Deploy path = `orbityard hostdeploy` (kernel Desktop streams) + F1/F2 UI inventory in source |
| Project | `apps/website` · Cloudflare · `ship-studio` |
| Prior login | **Yes** (warm) |
| Soft-no | No Integrations / stores path used |
| Tauri chrome | Not CDP-driven — operator self; UI attested in source (below) |

### Stopwatch

| Step | Desktop meaning | Action | Wall |
|------|-----------------|--------|------|
| 1–2 | Open project · Choose host | Project bound · provider Cloudflare | (pre-clock bind) |
| 3 | Login once | `hostdeploy --auth-check` | **4 s** |
| 4–5 | Env + Deploy · Open live | `hostdeploy` → URL | **12 s** |
| 5′ | Proof | `curl` live URL → **HTTP 200** | **3 s** |
| **Total wall** | Clock start → proof | | **20 s** |

**Live URL:** https://40291718.ship-studio.pages.dev  
**Production alias:** https://ship-studio.pages.dev/

**Score:** **Pass** (URL + HTTP 200 before 20:00).

### F1 / F2 Desktop UI inventory (source)

| Check | Evidence |
|-------|----------|
| F1 Get live primary | `apps/desktop/index.html` — `workflow-card--primary` · `data-workflow="deploy_only"` first |
| F1 spine legend | `Founder path` · `spine-legend` · Open → Host → Login → Deploy → Open live |
| F2 one recovery | Fail toast single CTA · `retryBtn.hidden = true` · hint `Next:` |

## Gaps (honest)

| Gap | Impact |
|-----|--------|
| Warm login | Step 3 ≤5:00 cold OAuth not stressed |
| Operator self | Not a cold stranger; knows the repo |
| No Tauri click automation | Run B times the Deploy kernel Desktop uses; chrome clicks not recorded |

## Aligns with

| Doc | Note |
|-----|------|
| [founder-minimal-spine](../../specs/backend/founder-minimal-spine-design.md) | F3 L3 · Run B filled |
| [evidence-self-beta-host-20261007](./evidence-self-beta-host-20261007.md) | Prior CF Pages |
| Host XOR L3 | Stranger/self script measured |

## Next

Founder spine F1–F5 complete. Optional: cold-stranger Desktop session · or H1 concierge explore. Do **not** reopen Integrations/DevOps GTM.

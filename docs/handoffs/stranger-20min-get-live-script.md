# Stranger script — Get live in 20 minutes

**Status:** Active protocol (F3)  
**Band:** [founder-minimal-spine-design](../../specs/backend/founder-minimal-spine-design.md)  
**Audience:** Solo founder / indie with a host account (Cloudflare, Vercel, or Netlify) — **not** DevOps fluent  
**Pass:** Public URL in browser within **20:00** wall clock · or a **named** blocking gate with one recovery CTA  
**Fail:** Lost in Integrations / Advanced / peer Ops menus · no URL and no named gate

## Preconditions (clock not running)

| Need | Why |
|------|-----|
| Orbit Yard Desktop **or** `orbityard` on PATH | Local spine |
| A project folder with a static site or host config (`apps/website`, `wrangler.toml`, `vercel.json`, `netlify.toml`, or `dist/`) | Detectable deploy surface |
| Vendor account already exists | Studio does not create accounts |
| Host CLI installed (`wrangler` / `vercel` / `netlify`) | Login + Deploy |

Do **not** start the clock until the project folder is open and the stranger is on Dashboard (Desktop) or a shell at the project root (CLI).

## Five steps (clock running)

| # | Step | Desktop | CLI proxy | Timebox |
|---|------|---------|-----------|---------|
| 1 | Open project | Dashboard → folder bound | `cd` project | ≤ 1:00 |
| 2 | Choose host | **Get live** → Deployment → Cloudflare / Vercel / Netlify | `--provider …` | ≤ 2:00 |
| 3 | Login once | Loud recovery **Login CLI** if asked; finish browser OAuth | `wrangler login` / `vercel login` / `netlify login` | ≤ 5:00 |
| 4 | Env + Deploy | Import env optional → **Deploy** (one primary) | `orbityard hostdeploy --provider … --name …` | ≤ 8:00 |
| 5 | Proof | **Open live** · Confirm when URL works | Open printed `https://…` | ≤ 4:00 |

**Soft no during the run:** Integrations, Portal encyclopedia, Publish-to-stores, Assist checklist theater.

## Scoring

| Result | Criteria |
|--------|----------|
| **Pass** | Live URL opened before 20:00 |
| **Pass (gate)** | Clock &lt; 20:00 and UI shows **one** named recovery (Login CLI / Put / Open dashboard / Preview) — stranger stops correctly |
| **Fail** | &gt; 20:00 without URL or named gate · or stranger opens Integrations as the path · or ≥2 peer recovery CTAs on fail |

## Operator notes (for the timer)

- Announce start: “Clock starts — Get live only.”
- On fail: record which **one** recovery appeared (F2).
- Record host, Desktop vs CLI, prior login yes/no.
- Evidence file: `docs/handoffs/evidence-f3-stranger-20min-YYYYMMDD.md`

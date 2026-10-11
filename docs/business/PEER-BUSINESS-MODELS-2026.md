# Peer business models — deploy / self-host / ship tools (2026)

**Status:** Study — **reference** · falsifies “OSS Gift alone = business” for this operator  
**Updated:** 2026-10-09  
**Parents:** [STRATEGY.md](./STRATEGY.md) · [MARKET-SIGNALS-2026.md](./MARKET-SIGNALS-2026.md) · [commercial-product-failed](../../specs/backend/commercial-product-failed-design.md)

```text
Maintainer evidence: 30+ public repos → no clients, no job offers;
                     attention ≈ AI scrapers, not buyers.
Peer lesson: people who get paid sell a residual the clone cannot replace.
```

## 1. What actually gets paid (field map)

| Model | Who | What buyer pays for | OSS role |
|-------|-----|---------------------|---------|
| **A. Managed cloud of same product** | Plausible Cloud; Medusa Cloud ($29–$299+) | They run it · uptime · upgrades · support | CE/self-host is optional; **ARR is hosted** |
| **B. Control-plane SaaS, your VPS** | Coolify Cloud (~$5+ / server); Outlap; KiCI tiers | Dashboard / orchestration hosted; apps on *your* metal | Often OSS core; money = plane |
| **C. Fully managed PaaS** | Railway · Vercel · Render | Compute + DX; metered | Usually **closed**; no Gift hook needed |
| **D. Paid desktop / license** | e.g. Server Compass (~$19–$29 one-time) | App that talks SSH to *your* VPS; keys stay local | Often **not** full OSS; trial/free tier limited |
| **D2. Outcome kit (template-class)** | ShipFast-era SaaS starters (~$149–$299) | Local codebase rights + “launch X fast” wall; demo before pay | Usually **paid**, not Gift; see [OUTCOME-SKU-PAIN-DEMO](./OUTCOME-SKU-PAIN-DEMO-2026.md) |
| **E. Agency / project + RUN** | Medusa shops · freelancers | Custom + keep-alive | OSS is raw material |
| **F. Pure OSS, no residual** | CapRover-class community tools | VPS only; authors unpaid | Visibility lottery — **matches your 37-repo outcome** |

Plausible’s own writeup: donations ≈ noise; **cloud subscriptions** paid the rent. That is not “selfless OSS” — it is **hosted product** with a public edition.

## 2. Orbit Yard today (honest placement)

| Dimension | Orbit Yard | Closest peer |
|-----------|------------|--------------|
| Runs buyer’s app? | No — sequences deploy to **their** host CLI | Not Coolify/Railway |
| Hosted residual you operate? | **No** | CapRover / unpaid OSS lane |
| Closed paid desktop? | Solo leftovers; cold mid SKU **failed** | Not Server Compass (yet) |
| Agency H1? | Paper only; no hitch clients | Model E without pipeline |

So “Gift + optional consultant” is **not** what Plausible/Medusa/Coolify do when they eat. They sell **A or B**. Consultant-only without distribution is **E** with no inbound — which you already falsified via OSS volume.

## 3. Models worth studying (energy ranking for *you*)

| Rank | Model | Why it fits / hurts |
|------|-------|---------------------|
| **1** | **D / D2 — Paid local product or outcome kit** (desktop license **or** template-class pack at ~$149–$299) with demo’d wall | Money = artifact + job; template grammar beats $29 suite. Generic Next kits crowded in 2026 — wall must be narrower. Distribution still required. |
| **2** | **B — Thin control plane you host** (multi-repo status, deploy triggers, secrets names sync) while deploys stay on user metal | Peer pattern Coolify/Outlap; **new product surface** — not “polish Gift.” High build cost; competes with hosts. |
| **3** | **E — Named labor SKU** only if a **non-GitHub** channel exists (communities, partners, outbound) | Valid product; OSS is irrelevant as hook given your data. |
| **4** | **A — You host their site/commerce** | Real money; you become a host — rejected as company GTM before; only revisit with eyes open. |
| **Kill as strategy** | **F — More public OSS for credibility** | Your 37-repo evidence: no social return; scrapers win. |

## 4. What is *not* a peer success story

- “We open-sourced and clients appeared” without A/B/D and a sales channel  
- Mid $29 “everything unlocked” with no hosted delta (your Solo failure rhyme)  
- Visibility from GitHub stars as a plan (stars ≠ invoices)

## 5. Implication for energy / trust

```text
Do not invest scarce trust in “contribute OSS → industry returns.”
Invest only if the bet is a residual buyers cannot git-clone:
  paid sealed (or feature-gated) product, hosted plane, or sold labor with a real channel.
```

Studying peers **supports leaving F**. It does **not** require becoming Plausible-scale overnight; it requires picking **one** of A–E and accepting build/distribution cost.

## 6. Next (study → decision, not marketing)

Maintainer picks one sentence:

1. **Paid product** — feature-gate or closed Desktop/CLI; Gift becomes demo/limited  
2. **Control-plane SaaS** — new scope (honest compete with Coolify-class)  
3. **Labor channel first** — stop OSS-as-funnel; sell H1 where humans already are  
4. **Park Ship commercial** — personal tool only; put EG elsewhere (portfolio)

No ACTIVATE until that sentence is written into handoff.

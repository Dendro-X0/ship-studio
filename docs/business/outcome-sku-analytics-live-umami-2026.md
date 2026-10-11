# Outcome SKU pressure-test — Analytics live (Umami-class)

**Status:** Pressure-test complete — **Go thin (labor or hardened pack)** · **not** generic compose zip  
**Updated:** 2026-10-09  
**Parent:** [OUTCOME-SKU-PAIN-DEMO-2026](./OUTCOME-SKU-PAIN-DEMO-2026.md) · [MARKET-SIGNALS-2026](./MARKET-SIGNALS-2026.md)  
**Candidate:** #1 Self-hosted analytics live — privacy pain

## 1. Offer sentence (buyer words)

> Get **privacy-friendly analytics on a URL you control** — cookieless tracker on your site, dashboard on your VPS — without sending visitors to Google.

**Done signal:** `https://analytics.{their-domain}` (or similar) opens Umami; a test pageview appears after the script is on one site.

## 2. P1–P5 score

| # | Result | Notes |
|---|--------|-------|
| **P1** Spend nearby | **Pass** | Already pay VPS / Plausible Cloud / agency; GDPR anxiety is budget-adjacent |
| **P2** Observable demo | **Pass** | Dashboard URL + live hit counter — easy Soft-L3 |
| **P3** AI/clone hard | **Weak / conditional** | Official `docker compose up` + dozens of “15‑minute” tutorials. **Zip of stock compose = dead SKU.** Wall that still sells: production path (TLS · secrets · no default admin · Caddy · backup) **shown working**, or **you stand it up** (H1) |
| **P4** Finite delivery | **Pass** | Pack or one-shot standup; upgrades/backups = optional later, not day-one retainer |
| **P5** Channel | **Fail until named** | Same as all Ship bets — GitHub alone won’t sell. Need privacy/indie/EU/hosting communities or hitch — **block on launch, not on kit design** |

**Verdict:** **Go thin** — sell **hardened Get-live pack + demo** and/or **H1 standup**; **Kill** “Umami compose from GitHub with a price tag.”

## 3. What we sell vs what we don’t

| In (SKU) | Out |
|----------|-----|
| Production compose: localhost bind · strong `APP_SECRET` · DB password · `DISABLE_TELEMETRY` | Replacing Umami Cloud / Plausible Cloud as a host |
| Caddy (or Nginx) TLS snippet + DNS checklist | Legal advice / “GDPR certified” claims |
| One-page runbook: up → change admin password → add website → paste script → verify hit | Building their marketing site |
| Your Soft-L3 video: blank VPS → HTTPS dashboard → pageview | Orbit Yard license as the product |
| Optional **H1**: same wall on *their* VPS once (~template band or half-day) | Unlimited support |

**Orbit role:** optional later — sequence env/deploy if/when Desktop has a VPS/Docker Get-live path; **not required** for v1 kit (SSH + compose is enough).

## 4. Fake landing (copy only — no build yet)

**Headline:** Your analytics. Your server. No Google.  
**Sub:** Self-hosted Umami, production-ready: HTTPS dashboard live, tracker on your site, visitors stay off adtech.  
**Proof strip:** 90s video — “from empty VPS to first pageview.”  
**Price anchors:**  
- **Kit** $149 — repo + runbook + lifetime updates to the pack (not Umami upstream)  
- **Done-for-you** $249 — kit + one standup on your VPS (you Login SSH; we leave you the URL)  
**CTA:** Buy kit · Book standup  
**Soft-no:** Not a hosted analytics SaaS. Not legal counsel. You bring a VPS (~$5–12/mo).

## 5. 90-second demo script (you record)

| t | Show |
|---|------|
| 0:00–0:15 | Pain: GA4 consent / “data leaves my site” — one sentence |
| 0:15–0:40 | Terminal: `docker compose up -d` on prepared pack (secrets already set) |
| 0:40–1:05 | Browser: HTTPS Umami login → dashboard |
| 1:05–1:25 | Site with script → refresh → **pageview appears** |
| 1:25–1:30 | Wall: “This URL is yours. Kit / standup.” |

Watermark: `local VPS · recorded`. No Orbit UI required for this SKU’s proof.

## 6. Risks

| Risk | Mitigation |
|------|------------|
| Buyer says “I can follow a free tutorial” | Demo + hardened defaults + standup tier; don’t compete on “compose exists” |
| Plausible Cloud cheaper in effort | Segment: must own data / refuse US SaaS; already on VPS |
| Scope creep (Matomo parity) | Umami-only wall |
| P5 no channel | Do not build pack until one channel experiment is named |

## 7. Next atomic

| If | Then |
|----|------|
| Maintainer accepts Go thin | **L1:** Record 90s Soft-L3 on a throwaway VPS (or local Docker + tunnel) · draft landing in a private doc/site — **no** public OSS dump of the pack |
| Maintainer wants channel first | Name one place to show the 90s clip (newsletter, Discord, hitch) before writing compose |
| Kill | Mark Status Kill — move to Medusa or niche kit |

**Recommended:** See [DEMO-AND-SIMPLICITY-HONESTY-2026](./DEMO-AND-SIMPLICITY-HONESTY-2026.md).

- Demo steps must = buyer path (compose → Admin → pageview), **not** Orbit chrome, until Studio shortens Umami.  
- Prefer **silent ≤90s** titles-only; **no** multi-day edited videos.  
- If you won’t sell the kit soon: **skip recording** — build Orbit delta first, or park #1.

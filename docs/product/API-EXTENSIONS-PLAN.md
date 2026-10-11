# API extensions plan — Orbit Yard / Ship-Studio

**Status:** Active boundary plan (mostly **non-goals**)  
**Updated:** 2026-10-10  
**Canonical with:** [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md) · integrations product **CANCELLED**  
**Inspiration:** [public-apis](https://github.com/public-apis/public-apis) CI / Cloud Storage / Tracking categories as anti-patterns  
**Portfolio card:** `new-business/research/164-public-apis-portfolio-inspiration.md`

---

## Intent

Ship-Studio sequences **local** sign → release → deploy with human gates. This plan records which API-shaped ideas stay **out**, and the few **local** surfaces that remain in scope.

---

## In scope (local · no secretful vendor HTTPS product)

| Extension | Shape | Notes |
|-----------|-------|-------|
| Local git / CI artifact reads | Watch | Disk + operator-gated CLI probes only |
| Human deep-links + host CLIs | Adapt | Open vendor UI; Studio does not finish OAuth |
| Optional local Verify/Watch poll | Watch | Still never completes store/DNS for the human |
| Existing dogfood assess APIs | — | See [DOGFOOD-orbit-assess-api](../dogfood/DOGFOOD-orbit-assess-api.md) |

---

## Kill (do not re-open as product)

| Extension | public-apis-ish category | Why |
|-----------|--------------------------|-----|
| Polar / Stripe / Paddle / email **setup** wizards | Business · Email | Integrations-as-product failed |
| Auto-upload releases to S3/GCS as a feature SKU | Cloud Storage | Offline bridge: no vendor HTTPS with secrets |
| URL shortener / campaign tracking on announce | URL Shorteners · Tracking | Growth-hack / honesty break |
| Finishing OAuth, store upload, live publish via API | Auth · CI | Human gate is the product |

---

## Park (copy / UX metaphor only)

| Idea | Notes |
|------|-------|
| “Status badge” language from CI APIs | Explain disk · local CLI · official probe · human attest layers — no hosted badge service |

---

## Rule

If an idea requires Studio to call a vendor HTTPS API **with secrets** as the path to “done,” it is **out of scope**. Prefer documenting the human gate.

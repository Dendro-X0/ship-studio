# Flowsint Get-live — Orbit delta (F0 study)

**Status:** **F0–F4 done** (Desktop card · structure gate · Publish Confirm)  
**Updated:** 2026-10-10  
**Owner:** `orbityard` detect · multi-env ensure · compose.prod · Desktop On-this-machine  
**Peers:** [flowsint](https://github.com/reconurge/flowsint) · parents [get-live-compose-packs-from-peers](./get-live-compose-packs-from-peers-design.md) · [umami-get-live](./umami-get-live-design.md) · [compose_pack.rs](../../crates/orbityard/src/compose_pack.rs) (G0)

## Plan alignment

```text
HANDOFF ATOMIC STEP: F0–F4 shipped — Soft-L3 optional on real Flowsint clone
PAUSED / CANCELLED:  Integrations · wiring Maigret/N8n inside Studio ·
                     enricher API key coach · Portal expansion
CANONICAL OWNER:     layout osint.flowsint (On this machine)
PROOF BEFORE F1:     Detect signals + ensure multi-env contract locked here
```

## 1. Verdict

| Decision | Sentence |
|----------|----------|
| **Go thin** | Orbit shortens Flowsint’s Windows path: **one ensure** (root + module `.env` copies) → **compose.prod up** → proof URL `http://127.0.0.1:5173` (+ API `/health`). |
| **Kill if** | Band becomes “finish OSINT vendor keys / enrichers inside Orbit.” |

Official Windows README still asks for **four** `copy .env.example` steps before `docker compose -f docker-compose.prod.yml up -d`. That is the Soft-L3 win — same class as Umami secrets ensure.

## 2. Detect signals (proposed)

Layout id: `osint.flowsint` (probe flag `flowsint`).

| Signal (any strong) | Notes |
|---------------------|-------|
| `docker-compose.prod.yml` mentions `ghcr.io/reconurge/flowsint-app` or `flowsint-api` | Canonical prod images |
| Compose service names `flowsint-app` / `flowsint-api` / `flowsint-postgres` | Corroborate |
| Root dirs `flowsint-app` + `flowsint-api` + `flowsint-core` | Monorepo clone |
| `.ship/layout` → `osint.flowsint` | Opt-in |

**Do not** detect bare Neo4j+Postgres stacks without Flowsint image/name markers.

## 3. Ensure contract (multi-env)

Official copies (Windows):

```text
.env.example → .env
.env.example → flowsint-api/.env
.env.example → flowsint-core/.env
.env.example → flowsint-app/.env
```

Orbit `orbityard flowsint ensure`:

| Key | Action |
|-----|--------|
| Missing target `.env` | Copy from nearest `.env.example` (root template preferred) |
| `AUTH_SECRET` weak / example | Replace with random hex ≥32 |
| `MASTER_VAULT_KEY_V1` example / weak | Replace with `base64:` + 32 random bytes |
| `NEO4J_PASSWORD` / `POSTGRES_PASSWORD` weak (`password`, `flowsint`) | Replace with strong random; keep username defaults |
| Never print secret values | Same as Umami |

Prod compose injects `${AUTH_SECRET}` / `${MASTER_VAULT_KEY_V1}` / Neo4j auth from **root** `.env` for `docker compose -f docker-compose.prod.yml`. Module copies still matter for **dev** compose; ensure both so either file works.

## 4. Deploy contract

```text
orbityard flowsint deploy --project <dir>
  → ensure
  → compose -f docker-compose.prod.yml up -d  (via compose_pack)
  → wait http://127.0.0.1:5173/ (or /register) 200
  → optional: http://127.0.0.1:5001/health 200 (host-bound API)
  → last-run host_provider=flowsint · Open App URL
```

| Human gate | Confirm |
|------------|---------|
| First register at `/register` | `flowsint.register` (create local account) |
| Enricher vendor keys | Soft-no — their vault UI only |

Recovery: reuse `compose_pack` log sniff + one `down -v` when volumes corrupt (Postgres/Neo4j migrate failures).

## 5. Slice queue

| ID | Slice | Done when |
|----|-------|-----------|
| **F0** | This study + detect/ensure/deploy contracts | **This file** |
| **F1** | Fixture stub or clone marker + `detect_flowsint` + unit test | Probe `flowsint=true` — **done** |
| **F2** | `orbityard flowsint ensure` multi-env | Four weak copies → strong secrets — **done** |
| **F3** | `orbityard flowsint deploy` via G0 helpers | App URL in last-run — **done** |
| **F4** | Desktop card · structure gate · Publish Confirm | Soft-L3 ≤ official README — **done** |
| **Later** | TLS/Caddy · LAN host allowlist notes | After F4 |

## 6. Soft-no

- Integrations catalog of enrichers  
- Auto-filling Maigret / breach API keys  
- Competing with Flowsint’s product UI  
- Shipping a fork of Flowsint inside Orbit  

## 7. Next atomic

**Optional Soft-L3** — bind a real [reconurge/flowsint](https://github.com/reconurge/flowsint) clone → Deploy → register → Confirm. Fixture stub is detect/ensure only.

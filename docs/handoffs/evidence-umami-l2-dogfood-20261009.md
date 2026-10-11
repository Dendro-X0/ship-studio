# Evidence — Umami L2 dogfood (2026-10-09)

**Spec:** [umami-get-live-design](../../specs/backend/umami-get-live-design.md)  
**Result:** **Pass** (CLI path)

## Steps

1. `cargo build -p orbityard`
2. `orbityard umami ensure --project fixtures/umami-analytics` → strong `.env` already present
3. `orbityard umami deploy --project fixtures/umami-analytics`
   - Used **`docker-compose`** fallback (`docker compose` plugin missing on this PATH)
   - Pulled images · compose up · heartbeat
4. `curl http://127.0.0.1:3000/api/heartbeat` → **200**
5. `.ship/last-run.json` → `ok: true`, `host_provider: umami`, URL `http://127.0.0.1:3000`

## Delta note

First attempt failed with `docker compose` (shim/plugin). Fix: prefer `docker.exe` + fall back to `docker-compose` in `umami.rs`. Without that, Orbit path fails on this machine while official `docker-compose up` works — fallback is required for honesty.

## Not done here

- Desktop bind → Umami card → Deploy (UI smoke; same CLI under the hood)
- U4 silent ≤90s film
- Admin password Confirm on Publish (human)

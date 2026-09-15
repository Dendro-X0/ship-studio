# Container final-mile run — band #13

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md` · gap #5 follow-on  
**Owner:** `config`, `publish`, `adapters` (doctor)  
**Updated:** 2026-09-15  

## Problem

`container.deploy` is docs URL + Confirm only. For a shipping hub, local **build** should be a Run step; **push** stays operator-gated (registry auth / tags).

## Scope (first slice)

1. **`container.build`** (Advanced, when `dockerfile` or `compose`):
   - Dockerfile → Run `docker build -t <dir-name>:local .`
   - Compose only → Run `docker compose build`
   - Related → portal; entry_url = container docs  
   - Confirm after local image builds  

2. **`container.deploy`** — keep as push/registry Confirm (docs URL). Detail reminds: build first, then `docker push` / host UI on your machine. No `docker push` from the bridge (auth/tag risk).

3. **Doctor** — if container detected, note whether `docker` is on PATH (“Cut ready: docker …” / missing).

## Non-goals

- `docker push` / registry login from shipctl  
- Kubernetes / Helm / Fly / Railway automation  
- Mobile store API upload (still deferred)  

## Proof

- L1: Dockerfile fixture Advanced has `container.build` with docker run  
- L2: `container.build` before `container.deploy`; General omits both  
- L2b: dogfood still green  

# Container deploy — gap #5

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #5  
**Owner:** `config::probe`, `scopes`, `portal`, `publish`

## Problem

Repos with a `Dockerfile` / Compose file get no Ship Studio cue for registry push or host docs. Operators leave the publish spine for ad-hoc docker CLI.

## Scope (first slice)

1. **Detect** `Dockerfile` / `Containerfile` / `docker-compose*.yml` / `compose.yaml` (root + one-level / `apps/*`).
2. **`ScopeKind::Container`** when docker signals dominate a directory.
3. **Portal** — `ProviderId::Container` dashboard-only (Docker Hub + GHCR docs; prefer GHCR URL when GitHub remote/repo present).
4. **Publish** — Advanced `container.deploy` (docs URL + Confirm → portal). General omits.
5. **Pulse** — kind bit + note when container detected.

## Non-goals

- ~~Running `docker build` from the bridge~~ → superseded by band #13 (`container-final-mile-design.md`)
- Running `docker push` / registry login from the bridge  
- Kubernetes / Helm controllers  
- Choosing a cloud host (Fly, Railway, ECS) beyond docs links  

## Proof

- L1: fixture with `Dockerfile` → `detected.container` + Container scope  
- L2: portal includes `container` create/docs URL  
- L3: Advanced has `container.deploy`; General omits  

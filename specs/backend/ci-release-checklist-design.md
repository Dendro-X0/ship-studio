# CI release checklist — gap #4

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #4  
**Owner:** `config::probe`, `pulse`, `publish`, `assist`

## Problem

Repos with GitHub Actions release workflows get no Ship Studio cue to open Actions after tagging/signing. Operators forget the CI gate.

## Scope (first slice)

1. **Detect** `.github/workflows/*` whose **filename** contains `release` (case-insensitive). Record names on `Detected`.
2. **Pulse** — note listing matched workflow files + Actions URL when `origin` is a GitHub remote.
3. **Publish** — Advanced `ci.release` (URL + Confirm → dashboard). General omits.
4. **Assist** — note + optional step when workflows detected.

## Non-goals

- Authoring or editing workflow YAML  
- Triggering `workflow_dispatch` / watching runs via API  
- Non-GitHub CI (GitLab, Circle)  

## Proof

- L1: fixture `.github/workflows/release.yml` → `detected.ci_release`  
- L2: pulse notes mention the workflow  
- L3: Advanced has `ci.release`; General omits it  

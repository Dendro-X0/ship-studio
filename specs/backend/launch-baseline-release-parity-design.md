# Launch legal / trust / graduate / release parity — band #50

**Status:** Done (first slice)  
**Updated:** 2026-09-19  
**Parent:** professional-launch-baseline · graduate-commerce · launch companion parity · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences legal baseline, TRUST pack, graduate signing notes, hand-cut GitHub Release (`gh release list` only), and desktop-cut honesty — same as Advanced Publish. Bridge never creates releases or claims verified publisher. Prefer Publish for the full Adaptive path.

## Behavior

| Id | When | Open | Run |
|----|------|------|-----|
| `legal.baseline` | missing LICENSE or SECURITY.md | none | none |
| `trust.pack` | tauri/signet && !TRUST.md | none | none |
| `sign.graduate` | `graduate_sign` | Azure Trusted Signing docs | `signet graduate notes` |
| `release.github` | GitHub origin && !Signet-self path | releases/new | `gh release list --limit 5` |
| `ship.desktop_cut` | Signet && no Orbit host | releases URL | none |

## Acceptance

- [x] missing LICENSE/SECURITY → `legal.baseline`  
- [x] signet without TRUST.md → `trust.pack`  
- [x] graduate markets → `sign.graduate` with notes Run  
- [x] package.json + GitHub origin (no signet) → `release.github`  
- [x] signet-only → `ship.desktop_cut`  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_baseline` |

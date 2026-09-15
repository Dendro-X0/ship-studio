# Final-mile cut order + CI/registry Runs — band #14

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md`  
**Owner:** `publish`, `adapters`  
**Updated:** 2026-09-15  

## Problem

1. `ci.release`, `release.github`, and `container.*` are inserted **before** `configure` / Signet release / listings — contradicting step copy (“After tag/Signet release”) and `OPERATOR-NEXT.md`.
2. `ci.release` is Open Actions URL only — `gh run list` is a safe read-only Run when `gh` exists.
3. `listing.npm` / `listing.crates` promise dry-run in copy but have no `run` (live publish stays Confirm).

## Scope (first slice)

1. **Reorder** Advanced plan to:
   - prep: doctor → scopes → legal → oauth* → env → db → configure  
   - cut: sign.self.* → trust → graduate? → release_dry → official* → release / **release.github** → desktop_cut?  
   - list/submit → **ci.release** → **container.build/deploy** → marketing → suite → dry_run → deploy* → live_check  

2. **`ci.release` Run** — `gh run list --workflow <first> --limit 5` (read-only). Confirm after green.

3. **`listing.npm` / `listing.crates` Run** — `npm publish --dry-run` / `cargo publish --dry-run`. Live publish stays human.

4. **Doctor** — Cut ready notes when `gh` / `npm` / `cargo` on PATH for matching layouts.

## Non-goals

- `gh release create` / live `npm|cargo publish` from the bridge  
- Adaptive `doctor.ok` without Orbit (band #15 candidate)  
- Mobile store API / k8s  

## Proof

- L1: Signet+CI fixture — `sign.self.release` index < `ci.release` < `container.build`  
- L1: `ci.release.run` starts with `gh`; listing dry-run vectors  
- L2: dogfood publish + walk still green; walk sees container after submit  

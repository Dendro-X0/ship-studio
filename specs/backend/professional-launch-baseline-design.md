# Professional launch baseline — gap #7

**Status:** Done (2026-09-15) — first slice  
**Parent:** `release-surface-map.md` gap #7  
**Owner:** `config::probe`, `publish`, `pulse`, Desktop Related  
**Updated:** 2026-09-15  

## Problem

Across Codactrl / Clavis / Signet / Obscur / Velocity / Strata, professional launch repeatedly stalls on the same local hygiene — missing LICENSE / SECURITY / TRUST, and no sequenced cue to cut a GitHub Release — while Ship Studio already walks OAuth, sign, listing, and deploy. Operators finish vendor work then forget the public trust pack.

## Scope (first slice)

1. **Detect** (root, case-insensitive name match):
   - `license` — `LICENSE`, `LICENSE.md`, `LICENSE.txt`, `COPYING`, `LICENSE-MIT`, `LICENSE-APACHE`
   - `trust_md` — `TRUST.md`
   - `security_md` — `SECURITY.md`
   - `changelog` — `CHANGELOG.md`, `CHANGELOG`, `CHANGES.md`
2. **Pulse** — note missing legal/trust files when any of license / security / (desktop→trust) are absent.
3. **Publish (Advanced only):**
   - `legal.baseline` — when `!license || !security_md` (URL none; Related → `dashboard`). Detail lists what’s missing; Confirm after files exist.
   - `trust.pack` — when `(tauri || signet_toml) && !trust_md` (Related → `sign`). Cue for TRUST.md + checksums honesty (Signet pattern).
   - `release.github` — when `github_repo_web_url` is Some (Related → `dashboard`). Open `{repo}/releases/new`; Confirm after tag/assets attached. Complements `ci.release` (Actions run) — does not replace it.
4. **General** omits all three.

## Non-goals

- Generating LICENSE text or legal advice  
- Calling GitHub HTTPS / `gh release create` from the bridge  
- Marketing-site deploy, Polar/Gumroad SKU, npm/crates.io (gaps #8+)  
- Store-grade graduate signing secrets (follow-up under signing vault)

## Detection invariants

- Bridge stays offline; detection is filesystem + local `git remote get-url origin`.  
- Never invent a second wizard — adaptive rows only.  
- Honesty: step copy must not claim “verified publisher” or signed artifacts.

## Proof

| Layer | Command / check |
|-------|-----------------|
| L1 | Fixture without LICENSE → `detected.license == false`; with TRUST.md → `trust_md` |
| L2 | Advanced plan includes `legal.baseline` / `trust.pack` / `release.github` when signals match; General omits |
| L3 | Desktop Advanced · Related on those steps → Dashboard or Sign · Back to Publish |

## Follow-ups (not this slice)

- Gap #8 package registries (`listing.npm` / `listing.crates`)  
- Gap #9 marketing deploy lane  
- Gap #10 graduate signing vault / commerce SKU  

# Adaptive doctor + Verify honesty — band #15

**Status:** Done (first slice, 2026-09-15)  
**Parent:** `shipping-hub-north-star.md`  
**Owner:** `adapters`, `publish`  
**Updated:** 2026-09-15  

## Problem

1. `doctor.ok` requires Signet **and** Orbit always — npm/crates-only and desktop-only Signet cuts fail Doctor even when the needed tools are present.
2. `verify_current` treats every `PubKind::Check` as Orbit-live, and `legal.baseline` / `trust.pack` always return false even after files exist.

## Scope (first slice)

1. **Adaptive `doctor.ok`** (pure helper + wire):
   - always require project dir exists  
   - require Signet iff `tauri || signet_toml`  
   - require Orbit **or** a host provider CLI (wrangler/vercel/netlify) iff `wrangler || vercel || netlify`  
   - library-only (no Signet, no host) → ok when path exists  

2. **`verify_current` per-step:**
   - `legal.baseline` → re-probe LICENSE + SECURITY.md  
   - `trust.pack` → TRUST.md present  
   - `ci.release` → `gh run list --json conclusion,status --limit 1` contains success  
   - `ship.desktop_cut` → prior `sign.self.release` or `release.github` Done  
   - `live_check` → deploy-live or honest desktop-only smoke message  

3. Sticky Pending `legal.baseline` / `trust.pack` until Confirm so Verify works after files appear.

4. Doctor step title/detail: “tools for this layout” not “Signet + Orbit”.

## Non-goals

- Auto-Confirm  
- Live `gh` create-release / store API  
- Desktop Open/Related dashboard parity (separate polish)  

## Proof

- L1: `doctor_tools_ok` unit matrix (library / desktop Signet / Workers)  
- L1: verify `legal.baseline` / `trust.pack` after files appear  
- L2: `cargo test -p shipctl` · dogfood still green  

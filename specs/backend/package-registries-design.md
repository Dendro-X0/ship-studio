# Package registries — gap #8

**Status:** Done (2026-09-15) — first slice  
**Parent:** `release-surface-map.md` gap #8  
**Owner:** `config::probe`, `publish`, `pulse`  
**Updated:** 2026-09-15  

## Problem

CLI/library ships (Signet, Velocity `create-velocity`, shipctl) need npm / crates.io listing cues. Studio sequences Polar + stores + Steam but not package registries.

## Scope (first slice)

### Detection (avoid app false positives)

| Registry | Auto | Opt-in |
|----------|------|--------|
| **npm** | Root `package.json` present, `"private"` is not `true`, and (`publishConfig` present **or** `"private": false` **or** package `name` looks scoped/publishable without `"private"`) | `.ship/markets` / `markets.json` contains `npm` |
| **crates.io** | Root `Cargo.toml` has `[package]` and `publish` is not `false` / `[]` | markets contains `crates` / `crates.io` |

Conservative npm rule (implemented):

1. Markets opt-in `npm` → always  
2. Else: root `package.json` parses as JSON object, `private != true`, and (`private == false` OR `publishConfig` is an object OR name starts with `@`)

Crates:

1. Markets opt-in `crates` / `crates.io` → always  
2. Else: root `Cargo.toml` contains a `[package]` table and no `publish = false` / `publish = []` on that package (line-oriented scan; no full TOML crate required)

### Advanced publish (URL + Confirm → portal)

- `listing.npm` → https://www.npmjs.com/login (detail: version bump → `npm publish` / OTP; Confirm after)  
- `listing.crates` → https://crates.io/me (detail: `cargo login` / `cargo publish`; Confirm after)

General omits both. Pulse notes when either detected.

## Non-goals

- Running `npm publish` / `cargo publish` from the bridge  
- PyPI / NuGet / Go modules  
- Monorepo member discovery beyond root  
- Changing package version numbers  

## Proof

| Layer | Check |
|-------|-------|
| L1 | Fixture with publishable `package.json` / `Cargo.toml` → `detected.npm_publish` / `crates_publish` |
| L2 | Advanced includes `listing.npm` / `listing.crates`; General omits |
| L3 | Desktop Related → Portal; Open entry URL |

## Follow-ups

- Gap #9 marketing deploy  
- Gap #10 graduate signing + commerce SKU  

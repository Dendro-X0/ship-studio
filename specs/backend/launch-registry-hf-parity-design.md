# Launch registry + Hugging Face listing parity — band #47

**Status:** Done (first slice)  
**Updated:** 2026-09-19  
**Parent:** launch-commerce-parity (#41) · package-registries · archetype HF · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences npm / crates.io / Hugging Face Hub listing Open + Confirm (dry-run Runs for npm/cargo) — same honesty as Advanced Publish. Live publish / Hub upload stays on the operator machine. Prefer Publish for the full Adaptive path.

## Behavior

| Id | When | Open | Run |
|----|------|------|-----|
| `listing.npm` | `npm_publish` | npmjs.com/login | `npm publish --dry-run` |
| `listing.crates` | `crates_publish` | crates.io/me | `cargo publish --dry-run` |
| `listing.huggingface` | `huggingface` | Hub getting-started docs | none |

## Acceptance

- [x] publishable package.json → listing.npm with dry-run  
- [x] Cargo.toml publishable → listing.crates  
- [x] HF markers → listing.huggingface  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_registry` |

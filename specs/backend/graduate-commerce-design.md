# Graduate signing + commerce — gap #10

**Status:** Done (2026-09-15) — first slice  
**Parent:** `release-surface-map.md` gap #10  
**Owner:** `config::probe`, `publish`, `pulse`, `signpath`  
**Updated:** 2026-09-15  

## Problem

Self-sign + store submit cover the OSS path. Professional launches still need:

1. **Graduate signing** — Authenticode / Azure Trusted Signing · Apple notarization (Signet `graduate`) without claiming verified publisher until real.  
2. **Commerce SKU** — Gumroad / Lemon (Polar already has `listing.polar`) for paid desks and template licenses.

## Scope (first slice)

### Detection

| Flag | Signals |
|------|---------|
| `graduate_sign` | Markets `graduate` / `authenticode` / `notarize`; env key prefixes `SIGNET_OV_`, `SIGNET_AZURE_`, `SIGNET_NOTARY_`, `WIN_CERT_`, `APPLE_API_KEY`, `NOTARY_`; `signet.toml` text contains `graduate` or `ship.path`; `.ship/graduate` marker file |
| `gumroad` | Markets `gumroad`; env `GUMROAD_`; path/doc hit `gumroad` in `.env*` keys |
| `lemon` | Markets `lemon` / `lemonsqueezy`; env `LEMON_` / `LEMONSQUEEZY_` |

Desktop/Signet without markers does **not** auto-add graduate (avoids noise). Opt in via markets or secrets layout.

### Advanced publish

| Step | When | Related | entry_url |
|------|------|---------|-----------|
| `sign.graduate` | `graduate_sign` | Sign | Azure Trusted Signing learn page (Windows) — detail also names Apple notarization; Confirm when OV/notary identity is provisioned. Honesty: do not claim SmartScreen silence. |
| `listing.gumroad` | `gumroad` | Portal | https://app.gumroad.com/ |
| `listing.lemon` | `lemon` | Portal | https://app.lemonsqueezy.com/ |

General omits all three. Polar stays on `listing.polar`.

### Sign portal

Add path `graduate.checklist` (kind `official`) when `graduate_sign` — same honesty copy; optional `signet graduate` run hint when Signet on PATH (command only, no network from bridge).

## Non-goals

- Storing cert private keys in `.ship/`  
- Calling Azure / Apple / Gumroad APIs  
- Auto-creating SKUs or Payment Links  
- Replacing `listing.polar`  

## Proof

| Layer | Check |
|-------|-------|
| L1 | Markets `graduate`+`gumroad` → flags true |
| L2 | Advanced has `sign.graduate` + `listing.gumroad`; General omits |
| L3 | Related → Sign / Portal |

## Follow-ups

- ~~Env secret catalog rows for graduate keys (paste-assist names only)~~ ✅ 2026-09-15  
- Suite URL sync after marketing.deploy  

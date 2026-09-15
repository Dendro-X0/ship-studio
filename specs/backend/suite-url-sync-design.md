# Suite URL sync — band #12

**Status:** Done (2026-09-15) — first slice  
**Parent:** `shipping-hub-north-star.md` · `release-surface-map.md`  
**Owner:** `config::probe`, `publish`, `pulse`  
**Updated:** 2026-09-15  

## Problem

After `marketing.deploy`, sibling products (e.g. Strata / Truss / Velocity) still need the live canonical URL in their env (`NEXT_PUBLIC_*_URL`). Operators forget the cross-suite paste; Studio never sequences it.

## Scope (first slice)

### Config — `.ship/suite.json`

```json
{
  "canonical_hint": "https://example.com",
  "siblings": [
    {
      "label": "strata",
      "path": "../strata",
      "env_keys": ["NEXT_PUBLIC_VELOCITY_URL"]
    }
  ]
}
```

- `path` is relative to the bound project (or absolute).  
- `env_keys` are **names only** — shipctl never writes values into sibling `.env`.  
- Missing/invalid JSON → no suite detection.

### Detection

`detected.suite_sync` when:

1. `.ship/suite.json` parses with at least one sibling that has a non-empty `path` and ≥1 `env_key`, **or**
2. Markets opt-in `suite` / `suite-sync` (step still needs suite.json for detail — if only markets, detail tells operator to add suite.json)

### Advanced publish

`suite.url_sync` (Human → Confirm), after `marketing.deploy` when possible:

- **Detail:** List sibling labels + env key names; remind to set the live landing URL there. Confirm when siblings match.  
- **entry_url:** `canonical_hint` if http(s), else marketing deploy URL, else GitHub Pages settings.  
- **desktop_view:** `dashboard`  
- General omits.

### Non-goals

- Writing sibling `.env` / Vercel env via API  
- Discovering suite members by scanning the filesystem beyond configured paths  
- Multi-root portfolio hub UI  

## Proof

| Layer | Check |
|-------|-------|
| L1 | Fixture `.ship/suite.json` → `suite_sync` |
| L2 | Advanced has `suite.url_sync`; General omits; Related → dashboard |
| L2b | Dogfood markets + suite.json |

## Follow-ups

- Optional verify: sibling path exists on disk (warn in detail)  
- Band #13 container/mobile depth (deferred)  

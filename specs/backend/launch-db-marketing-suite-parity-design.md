# Launch DB + marketing + suite parity — band #48

**Status:** Done (first slice)  
**Updated:** 2026-09-19  
**Parent:** launch-host-baas (#43) · db-hosting · marketing-deploy · suite-url-sync · north star  
**Owner:** `launch`

## Product framing

Guided Launch sequences database provision, marketing landing deploy, and suite URL sync Open + Confirm — same honesty as Advanced Publish. Studio never creates DBs, touches DNS, or writes sibling `.env` values. Prefer Publish for the full Adaptive path.

## Behavior

| Id | When | Open | Run |
|----|------|------|-----|
| `db.provision` | d1 \| neon \| supabase \| turso | vendor console (Neon → Supabase → Turso → D1) | none |
| `marketing.deploy` | `marketing_site` | `marketing_deploy_url` | none |
| `suite.url_sync` | `suite_sync` | `suite_sync_url` | none |

## Acceptance

- [x] D1/Neon/Supabase/Turso markers → `db.provision` with entry URL  
- [x] `apps/website` (or marketing markers) → `marketing.deploy`  
- [x] `.ship/suite.json` → `suite.url_sync` with canonical hint URL  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl launch_db_marketing` |

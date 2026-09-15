# DB hosting lane — gap #3

**Status:** Done (2026-09-14)  
**Parent:** `release-surface-map.md` gap #3  
**Owner:** `portal`, `secrets`, `config::probe`, `publish`

## Problem

Operators provision Neon / Supabase / Cloudflare D1 / Turso outside Ship Studio, then paste connection strings onto the deploy target. Studio has no create URLs or hints for those vendors.

## Scope (first slice)

1. **Detect** (local layout / env markers only — no vendor HTTPS):
   - **D1** — `[[d1_databases]]` / `d1_databases` in wrangler config  
   - **Neon** — `NEON_*`, `neon.tech` in env, `@neondatabase` in package.json  
   - **Supabase** — `supabase/config.toml`, `SUPABASE_*`  
   - **Turso** — `TURSO_*`, `LIBSQL_*`, `@libsql` / `@tursodatabase` in package.json  

2. **Portal** — `ProviderId::{Neon,Supabase,D1,Turso}` dashboard-only catalogs (Polar pattern): create/copy connection + docs URLs.

3. **Secrets** — catalog + empty-env hints (`DATABASE_URL`, vendor keys); `put_cli` opens portal; real put stays on Cloudflare/Vercel/Netlify (skip DB providers in put queue like Polar/GitHub).

4. **Publish** — Advanced-only `db.provision` (URL + Confirm → Env view) when any DB signal is present. General keeps secrets via `env.sprint` only.

## Non-goals

- Automating provision / migrate / schema push  
- Storing connection strings in `.ship/`  
- New ScopeKind::Db (matrix archetype stays gap for deploy)  
- RDS / generic SQL hosts beyond the four catalogs  

## Proof

- L1: fixture with `[[d1_databases]]` → portal includes `d1` create URL; Neon empty `DATABASE_URL` → neon hint  
- L2: `put_queue` does not treat neon/supabase/d1/turso as put destinations  
- L3: Advanced plan has `db.provision`; General omits it  

# Env/secrets provider-matched entry URLs — band #24b

**Status:** Done (first slice, 2026-09-19)  
**Parent:** `personal-local-intent-design.md` · aperio dogfood  
**Owner:** `portal`, `secrets`

## Problem

`source_url_for_secret_name` defaulted unknown names to the **Cloudflare** API-token page.  
Vercel-bound empty keys (`ANTHROPIC_API_KEY`, `CRON_SECRET`, …) opened the wrong dashboard.

## Fix

1. Map known **value sources** (Anthropic, Resend, Wellfound, GitHub, Polar, CF, Vercel-prefixed, …).  
2. Self-generated / unknown names use the **put provider** create URL (Vercel → Vercel tokens, etc.).  
3. Never fall through to Cloudflare unless the name or put provider is Cloudflare.

## Proof

`cargo test -p shipctl` — `source_url_*` / secrets vercel fixture tests  

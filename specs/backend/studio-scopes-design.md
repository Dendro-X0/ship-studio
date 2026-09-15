# Studio scopes, signing paths, env portal — design

**Status:** Active  
**Updated:** 2026-09-14  
**Owner:** `crates/shipctl` (`scopes`, `session`, `envx`) · Desktop

## Vision

Ship Studio is a **local shipping portal**: switch target directories instantly, pick **Web / API / Desktop / Mobile** scopes, sign (self or official), manage ENV/token **hints**, and run a multi-provider deploy assist. Official dashboards remain the place of token **creation**; the bridge never stores secret values or calls vendor HTTPS.

## Surfaces

| Capability | Mechanism |
|------------|-----------|
| Switch projects | Desktop recents + titlebar switcher; `shipctl session` lists known roots |
| Scopes | Probe tree (`apps/*`, wrangler, vercel, src-tauri, android/ios/Expo); persist `active_scopes` in `.ship/studio.json` |
| Deploy by scope | Launch/deploy `run` uses that scope’s `root` + provider args |
| Self-sign | `signet build` / identity (local) |
| Official sign | Guided: Apple certs · App Store Connect · Windows · Play vendor URLs + confirm; Signet still builds |
| Mobile listing | Advanced `listing.play` / `listing.app_store` (URL + Confirm) |
| ENV / tokens | Names + files + put CLI + create URL — never values |
| Assist wizard | Ordered checklist: bind → scopes → env → sign path → providers → dry-run → deploy |

## Invariants

1. No vendor HTTPS from the bridge.  
2. Secret values never in `.ship/` plaintext.  
3. Token **create** = open official page. **Retrieve** = open source + put CLI. **Configure** = local files / wrangler secret put.  
4. Scopes are directories inside the bound project (or the root itself).

## Proof

- L1: fixture with `apps/web` + `wrangler.toml` yields web + api scopes  
- L1b: fixture with `android/` + Expo `app.json` yields Mobile scope  
- L2: `shipctl scopes` / `shipctl envx` / `shipctl assist` JSON  
- L3: Desktop switcher + scope chips + Assist / Env / Sign views

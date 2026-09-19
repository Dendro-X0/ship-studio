# Mobile BaaS portal — band #28

**Status:** Design + first slice shipped  
**Updated:** 2026-09-19  
**Parent:** SCOPE-OF-SERVICE · north star · publish-progress-watch (gaps)  
**Owner:** `publish` · `config` · `portal`

## Product framing

Ship Studio remains a **portal**: guide operators to the official BaaS console for Auth / client keys / project setup. Human still creates projects and pastes keys. Bridge never calls vendor HTTPS; never store API upload.

Distinct from `db.provision` (SQL hosts) and from Play/ASC listing/submit.

## Step

| Id | When | UX |
|----|------|-----|
| `baas.provision` | Advanced + Public + `mobile` + any BaaS signal | Human · Open URL · Confirm · `desktop_view: env` |

### Detect signals (`Detected`)

| Flag | Signals |
|------|---------|
| `firebase` | `firebase.json` · `google-services.json` · `GoogleService-Info.plist` · `FIREBASE_` / `NEXT_PUBLIC_FIREBASE_` env · package/`pubspec` firebase |
| `appwrite` | `appwrite.json` · `APPWRITE_` · package `appwrite` |
| `convex` | `convex/` dir · `CONVEX_` · package `convex` |
| Supabase Auth | existing `supabase` **and** `mobile` |

Emit when: `mobile && (firebase \|\| appwrite \|\| convex \|\| supabase)`.

### Open URL priority

1. Firebase → `https://console.firebase.google.com/`  
2. Appwrite → `https://cloud.appwrite.io/`  
3. Convex → `https://dashboard.convex.dev/`  
4. Supabase → `https://supabase.com/dashboard`  

### Portal catalog (parity)

`ProviderId::{Firebase, Appwrite, Convex}` with empty `oauth_cli` (dashboard Open only). No forced secrets put queue in first slice.

### Filters

- Not in General (`is_general_step`)  
- Omitted by Local (`is_local_intent_step` with `db.provision`)  

## Acceptance

- [x] Advanced + mobile + `firebase.json` → `baas.provision` with Firebase console URL  
- [x] General / Local omit the step  
- [x] API-only Supabase (no mobile) does **not** add `baas.provision`  
- [x] Unit tests green  

## Non-goals

- Vendor HTTPS / Admin SDK from bridge  
- Storing Firebase/Appwrite/Convex secrets in `.ship/`  
- Replacing Auth flows  

## Proof

| Layer | Command |
|-------|---------|
| L1 | `cargo test -p shipctl baas_` |
| L2 | `shipctl publish --mode advanced --project <mobile+firebase fixture>` |
| L3 | Desktop Open on `baas.provision` (optional) |

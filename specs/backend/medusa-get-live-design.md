# Medusa Get-live — implement slices

**Status:** **M1–M4 done**  
**Updated:** 2026-10-10  
**Owner:** Publish General / Get-live packs  
**Parents:** [owned-commerce-medusa-study](./owned-commerce-medusa-study-design.md) · [founder-minimal-spine](./founder-minimal-spine-design.md) · [ops-deploy-pivot](./ops-deploy-pivot-design.md) · [structure-proceed-gate](./structure-proceed-gate-design.md)

## Plan alignment

```text
HANDOFF ATOMIC STEP: Medusa M1–M4 done — Soft-L3 optional on real clone
PAUSED / CANCELLED:  Integrations · payment-plugin coach · Solo zip-as-spine
CANONICAL OWNER:     commerce.medusa under Get-live (not Integrations)
PROOF THIS BAND:     L1 Desktop card + gate; Soft-L3 Admin = real Medusa clone
```

## Soft-no

| Never-say | Say |
|-----------|-----|
| Integrations Payments is how you sell | Get live on Deployment / Publish |
| Orbit finishes Stripe Connect | Payment plugins open the **vendor** dashboard |
| Buy our Medusa kit | Gift detect + ensure + deploy sequencer |
| We provisioned your Postgres | Compose wires local DB **or** Put `DATABASE_URL` |

## Slice board

| ID | Slice | State |
|----|-------|--------|
| **M1** | Fixture + `detect_medusa` | **Done** |
| **M2** | `orbityard medusa ensure` | **Done** |
| **M3** | `orbityard medusa deploy` + Publish steps | **Done** |
| **M4** | Desktop On-this-machine card · hard structure gate · Deploy/Open Admin | **Done** |

## Detect signals (M1)

| Signal | Strength |
|--------|----------|
| `medusa-config.{js,ts,mjs,cjs}` (root or one child dir) | Strong |
| `package.json` dep `@medusajs/medusa` or `@medusajs/framework` | Strong |
| `.ship/layout` = `commerce.medusa` | Strong |
| `src/api` + scripts `medusa develop` / `medusa start` | Corroborate |
| Stripe alone | **Never** |

## Ensure contract (M2)

`orbityard medusa ensure --project <dir>`

| Action | Rule |
|--------|------|
| Gate | `probe.medusa` must be true |
| Rotate | `JWT_SECRET` · `COOKIE_SECRET` when weak — **distinct**; never print |
| Compose Postgres | Rotate `POSTGRES_PASSWORD` · set `DATABASE_URL` / `REDIS_URL` for localhost |
| No compose | Empty `DATABASE_URL` → `needs_put` |
| Soft-no | Never write `STRIPE_*` |
| Output | JSON: `updated_keys` · `needs_put` · `message` (key names only) |

## Deploy contract (M3)

`orbityard medusa deploy --project <dir>`

| Step | Rule |
|------|------|
| 1 | `ensure` |
| 2 | No compose + `needs_put` → soft fail (Put DATABASE_URL) |
| 3 | Compose present → `compose up -d` |
| 4 | Wait `http://127.0.0.1:9000/health` |
| 5 | Ok → last-run `admin_url` = `http://127.0.0.1:9000/app` · host_provider `medusa` |
| Named gate | Deps-only compose (no `medusa:` service) → clear message: real clone / `medusa develop` |
| Publish | `medusa.deploy` Auto · `medusa.admin_login` Human Confirm |

## Desktop contract (M4)

| Surface | Rule |
|---------|------|
| Catalog | **On this machine** card `medusa` (with Self-host / Umami / Flowsint) |
| Structure | Hard gate — bind Medusa layout or Bind fixture |
| Deploy | Streams `orbityard medusa deploy` |
| Results | Loopback Admin URL · **Open Admin** |
| Soft-no | Copy never says Integrations / Stripe finished |

## Success criteria

### M1–M3

- [x] Detect · ensure · deploy CLI · Publish (see prior bands)

### M4

- [x] Desktop Medusa card in On this machine
- [x] Hard structure gate + Bind fixture
- [x] Deploy / Open Admin / results bay isolation
- [x] Soft-no payment-plugin coach in wizard steps

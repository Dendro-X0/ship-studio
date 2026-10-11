# Fixture — Medusa commerce Get-live

Used by [owned-commerce-medusa-study](../../specs/backend/owned-commerce-medusa-study-design.md) · [medusa-get-live-design](../../specs/backend/medusa-get-live-design.md).

| Slice | State |
|-------|--------|
| **M1** | Detect `commerce.medusa` / `medusa-config` / `@medusajs/*` |
| **M2** | `orbityard medusa ensure` — JWT/COOKIE · compose wires `DATABASE_URL` |
| **M3** | `orbityard medusa deploy` — compose deps up · Admin proof (fixture is deps-only) |
| **M4** | Desktop card shipped — Soft-L3 still needs a real Medusa clone |

```bash
orbityard medusa ensure --project fixtures/medusa-commerce
orbityard medusa deploy --project fixtures/medusa-commerce
```

Compose here is **Postgres + Redis only**. Soft-L3 Admin needs a real Medusa clone with a `medusa` service (`build: .`) or `medusa develop` against the wired `DATABASE_URL`. Soft-no: Stripe alone is not Medusa.

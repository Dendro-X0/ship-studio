# Integrations nav — design (S1.19)

**Status:** Shipped  
**Updated:** 2026-09-27  
**Parent:** [improvement-backlog](../../docs/product/improvement-backlog.md) S1.19 · [deployment-nav-design](./deployment-nav-design.md) · [targets-catalog-parity-design](./targets-catalog-parity-design.md)

```text
GOAL: One Ship nav item “Integrations” → Payments/Email catalog (like Sign / Deployment / Targets).
NOT: Sidebar tree of Polar / Stripe / … as primary navigation.
```

## Problem

Integrations expands every payment/email wizard as a sidebar row, plus **All wizards**. That fights the Ship pattern already used for Sign, Deployment, and Targets.

## Target

| Nav (Ship) | Panel |
|------------|--------|
| **Integrations** | Payments · Email catalog + wizard (Open → put env outside Studio → Continue publishing) |

- Remove **Integrations** sidebar section + `#sidebar-integrations` tree.
- Keep view id `integrations`.
- Selection stays on the in-page catalog grid (same as Deployment / Sign).
- CmdK **Go to Integrations** → `setView("integrations")`.

## Non-goals

- New providers or wizard steps  
- Changing Integrations page chrome / minute loop  

## Proof

| Layer | Check |
|-------|--------|
| L1 | `apps/desktop` `tsc --noEmit` |
| L2 | Harbor — Ship shows one Integrations item; catalog still selects Polar / Resend |

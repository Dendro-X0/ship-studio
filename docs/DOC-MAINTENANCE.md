# Documentation maintenance

_Last updated: 2026-09-19_

---

## Principles

1. **One front door:** [CURRENT.md](./CURRENT.md) — then [handoffs/current-session.md](./handoffs/current-session.md).
2. **Small active shelves:** product · dogfood · frontend · handoffs. Design contracts stay in `specs/backend/` (not duplicated under `docs/`).
3. **Handoff is live state** — not chat history.
4. **Root stays thin:** only `README.md`, `START-HERE.md`, `CURRENT.md`, `DOC-MAINTENANCE.md` at `docs/` root.

---

## Adding a document

| Step | Action |
|------|--------|
| 1 | Prefer updating [CURRENT.md](./CURRENT.md) or the handoff before adding files |
| 2 | Product boundary / operator gates → `product/` + link from [product/README.md](./product/README.md) |
| 3 | Dogfood walk → `dogfood/` + link from [dogfood/README.md](./dogfood/README.md) |
| 4 | Desktop UX contract → `frontend/` |
| 5 | Adaptive band / API design → `specs/backend/<concern>-design.md` + north star / release-surface |
| 6 | Update [README.md](./README.md) shelf table if a new shelf appears |

Do **not** add loose `.md` at `docs/` root except the four front-door files above.

---

## Session end

1. Update handoff (`Updated`, next atomic step).
2. If user-facing truth shifted → [CHANGELOG.md](../CHANGELOG.md) + [CURRENT.md](./CURRENT.md).
3. Grep for broken `docs/` paths after moves.

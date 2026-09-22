# START HERE — Ship Studio

**One front door.** Do not search all of `/docs`.

_Last updated: 2026-09-19_

---

## Read in this order

1. **[CURRENT.md](./CURRENT.md)** — version · idle/active · PAUSED bands
2. **[handoffs/current-session.md](./handoffs/current-session.md)** — Next Atomic Step only
3. **[product/SCOPE-OF-SERVICE.md](./product/SCOPE-OF-SERVICE.md)** — service boundary
4. **[product/OPERATOR-NEXT.md](./product/OPERATOR-NEXT.md)** — human gates still on you

That is enough for most sessions.

---

## Minute path

| Prefer | Companion |
|--------|-----------|
| `shipctl publish` — Open/Run → Confirm → Next | `shipctl launch` ([design](../specs/backend/guided-launch-design.md)) |

Extend lanes: [../specs/backend/release-surface-map.md](../specs/backend/release-surface-map.md) · [../specs/backend/shipping-hub-north-star.md](../specs/backend/shipping-hub-north-star.md)

---

## Shelves (on demand)

| Shelf | When to open |
|-------|----------------|
| [product/README.md](./product/README.md) | Scope · contract · operator checklist · [improvement backlog](./product/improvement-backlog.md) |
| [dogfood/README.md](./dogfood/README.md) | Walk scripts / paste paths |
| [frontend/README.md](./frontend/README.md) | Desktop shell |
| [handoffs/README.md](./handoffs/README.md) | Continuity + L3 evidence |
| [README.md](./README.md) | Full shelf index |

---

## Build

```bash
git clone https://github.com/Dendro-X0/ship-studio.git
cd ship-studio
cargo build -p shipctl --release
cargo test -p shipctl
```

## First useful commands

```bash
./target/release/shipctl doctor --project /path/to/project
./target/release/shipctl guide --project /path/to/project
./target/release/shipctl human --project /path/to/project
./target/release/shipctl publish --project /path/to/project
./target/release/shipctl tui --project /path/to/project
```

Desktop (Windows): `pnpm install && pnpm dev` · or `bash scripts/stage-desktop.sh`

## Non-goals

Ship Studio does **not** replace Cloudflare/Vercel/GitHub OAuth or store secret values in `.ship/`. Full boundary: [product/SCOPE-OF-SERVICE.md](./product/SCOPE-OF-SERVICE.md).

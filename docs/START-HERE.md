# START HERE — Ship Studio

1. **Scope of service:** [docs/SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md)
2. **Product contract:** [docs/PRODUCT.md](./PRODUCT.md)
3. **Publish portal (preferred):** `shipctl publish` — Open/Run → Confirm → Next ([north star](../specs/backend/shipping-hub-north-star.md))
4. **Guided launch (companion):** `shipctl launch` — open → verify → next ([design](../specs/backend/guided-launch-design.md))
5. **What still needs a human:** [docs/OPERATOR-NEXT.md](./OPERATOR-NEXT.md)
6. **Extend lanes / bands:** [specs/backend/release-surface-map.md](../specs/backend/release-surface-map.md) · [specs/backend/](../specs/backend/)

## Build

```bash
git clone https://github.com/Dendro-X0/ship-studio.git
cd ship-studio
cargo build -p shipctl --release
cargo test -p shipctl
```

## First useful commands

```bash
# Against any local project (e.g. a Worker + Polar repo)
./target/release/shipctl doctor --project /path/to/project
./target/release/shipctl guide --project /path/to/project
./target/release/shipctl human --project /path/to/project   # opens dashboards
./target/release/shipctl tui --project /path/to/project
```

Desktop (Windows): `bash scripts/stage-desktop.sh`

## Non-goals

Ship Studio does **not** replace Cloudflare/Vercel/GitHub OAuth or store secret values in `.ship/`. It gets you to the right paste surface fast, then optionally encrypts a Clavis-compatible `.km` backup. Full boundary: [SCOPE-OF-SERVICE.md](./SCOPE-OF-SERVICE.md).

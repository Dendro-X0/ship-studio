# Ship Studio

[![CI](https://github.com/Dendro-X0/ship-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/Dendro-X0/ship-studio/actions/workflows/ci.yml)

**Local shipping hub for the final mile** — sign → release → deploy — on three surfaces: **CLI** (JSON/MCP) · **TUI** · **Desktop**. Same `shipctl` engine (Signet + Orbit adapters). Adaptive **Publish** is the spine (Open/Run → Confirm → Next); Scopes / Env / Sign / Portal are detail panels, not a second wizard.

Boot: [docs/START-HERE.md](./docs/START-HERE.md) · **Scope:** [docs/SCOPE-OF-SERVICE.md](./docs/SCOPE-OF-SERVICE.md) · Contract: [docs/PRODUCT.md](./docs/PRODUCT.md) · Manual gates: [docs/OPERATOR-NEXT.md](./docs/OPERATOR-NEXT.md)

**Extend the hub (not docs/demos):** [specs/backend/shipping-hub-north-star.md](./specs/backend/shipping-hub-north-star.md) · [specs/backend/release-surface-map.md](./specs/backend/release-surface-map.md) · Advanced dogfood: `bash scripts/dogfood-advanced-publish.sh` · `bash scripts/dogfood-advanced-walk.sh fixtures/advanced-dogfood`

## Quick start

```bash
git clone https://github.com/Dendro-X0/ship-studio.git
cd ship-studio
cargo build -p shipctl --release

# Point at the project you are shipping
./target/release/shipctl ship --project /path/to/project
./target/release/shipctl guide --project /path/to/project --open
./target/release/shipctl tui --project /path/to/project
```

On Windows, binaries are `shipctl.exe` under `target/release/`.

## Layout

| Path | Role |
|------|------|
| `crates/shipctl` | CLI + TUI + MCP + publish / pulse / portal / secrets / vault |
| `apps/desktop` | Tauri shell (Publish spine + Related detail panels) |
| `specs/backend/` | Design contracts (north star + bands) |
| `scripts/` | `stage-desktop.sh`, `dogfood-advanced-*.sh`, `dogfood-offline.sh` |

## Desktop

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

## Vault (Clavis-compatible)

```bash
SHIP_VAULT_PASSPHRASE='…' MY_TOKEN='…' \
  ./target/release/shipctl vault export --out ./ship-secrets.km \
  --title GITHUB_TOKEN --value-env MY_TOKEN
```

## Proof

```bash
cargo test -p shipctl
./target/release/shipctl guide --project .
bash scripts/dogfood-advanced-publish.sh
# Windows (no WSL): powershell -File scripts/dogfood-advanced-publish.ps1
bash scripts/dogfood-offline.sh .
```

## License

MIT — see [LICENSE](./LICENSE).

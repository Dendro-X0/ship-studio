# Ship Studio

[![CI](https://github.com/Dendro-X0/ship-studio/actions/workflows/ci.yml/badge.svg)](https://github.com/Dendro-X0/ship-studio/actions/workflows/ci.yml)

**Local-first shipping portal** on three surfaces: **CLI** (JSON/MCP) · **TUI** · **Desktop** — same `shipctl` engine (Signet + Orbit adapters).

Boot: [docs/START-HERE.md](./docs/START-HERE.md) · Contract: [docs/PRODUCT.md](./docs/PRODUCT.md) · Manual gates: [docs/OPERATOR-NEXT.md](./docs/OPERATOR-NEXT.md)

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
| `crates/shipctl` | CLI + TUI + MCP + guide/portal/secrets/vault |
| `apps/desktop` | Tauri shell (Wizard / Portal / Secrets / Export vault) |
| `specs/backend/` | Design contracts |
| `scripts/` | `stage-desktop.sh`, `dogfood-offline.sh` |

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
bash scripts/dogfood-offline.sh .
```

## License

MIT — see [LICENSE](./LICENSE).

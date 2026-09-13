# Ship Studio

**Local-first shipping portal** on three surfaces: **CLI** (JSON/MCP) · **TUI** · **Desktop** — same `shipctl` engine (Signet + Orbit adapters).

## Quick start

```bash
cd "E:/Web Projects/ship-studio"
cargo build -p shipctl --release
./target/release/shipctl.exe ship --project "E:/path/to/repo"
./target/release/shipctl.exe guide --project "E:/path/to/repo" --open
./target/release/shipctl.exe tui --project "E:/path/to/repo"
```

Contract: [docs/PRODUCT.md](./docs/PRODUCT.md) · Manual gates: [docs/OPERATOR-NEXT.md](./docs/OPERATOR-NEXT.md)

## Layout

| Path | Role |
|------|------|
| `crates/shipctl` | CLI + TUI + MCP + guide/portal/secrets/vault |
| `apps/desktop` | Tauri shell (Wizard / Portal / Secrets) |
| `specs/backend/` | Design contracts |

## Desktop

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

## Proof

```bash
cargo test -p shipctl
./target/release/shipctl.exe guide --project .
# Encrypted key backup (Clavis-compatible):
SHIP_VAULT_PASSPHRASE='…' MY_TOKEN='…' \
  ./target/release/shipctl.exe vault export --out ./ship-secrets.km \
  --title GITHUB_TOKEN --value-env MY_TOKEN
```

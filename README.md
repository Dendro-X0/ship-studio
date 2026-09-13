# Ship Studio

**Local-first Ship workflow:** environment setup → Signet (sign/trust) → Orbit (deploy), offline-capable, MCP + desktop shell (CodaCtrl-shaped).

Signet (Rust) and Orbit (Go) stay **separate products**. This repo is the **bridge + client**, not a source merge.

## Product contract

See [docs/PRODUCT.md](./docs/PRODUCT.md).

## Quick start

```bash
cd "E:/Web Projects/ship-studio"
cargo build -p shipctl --release
./target/release/shipctl.exe doctor --project "E:/Web Projects/my-protfolio"
./target/release/shipctl.exe flow --project "E:/path/to/repo" --dry-run
```

MCP (stdio):

```bash
shipctl mcp
```

## Layout

| Path | Role |
|------|------|
| `crates/shipctl` | CLI + adapters + MCP |
| `apps/desktop` | CodaCtrl-like Tauri shell (Open folder → Doctor → Flow) |

## Desktop

## Desktop

```bash
bash scripts/stage-desktop.sh
./target/release/ship-studio-desktop.exe
```

Bare `cargo build -p ship-studio-desktop` keeps `cfg(dev)` and opens `localhost:1420` → `ERR_CONNECTION_REFUSED` without Vite. Always use `tauri build` / `tauri dev` / `stage-desktop.sh`.

Optional: `SHIPCTL_PATH` → `shipctl.exe` if not beside the desktop exe or under workspace `target/release`.

## Proof

```bash
cargo test -p shipctl
./target/release/shipctl.exe doctor --project .
# Desktop: Open folder → Doctor / Flow dry-run / Status
```

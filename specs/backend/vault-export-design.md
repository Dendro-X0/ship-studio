# Secure vault export — design

**Status:** Active  
**Updated:** 2026-09-13  
**Owner:** `crates/orbityard` (`vault_km`)

## Goal

Let operators **export** shipping secrets (API tokens, PATs, Polar values) into an **encrypted file** they own — compatible with Clavis / Keys Manager `vault.km` ([vault-format.md](file:///E:/Experimental%20projects/keys-manager/docs/vault-format.md)).

## Format

Same as Clavis v1:

`kmvault` + version + salt + Argon2id params + nonce + AES-256-GCM(JSON `VaultDocument`)

Entries use Clavis `entry_type: api` with `title`, `password` (secret value), `url`, `notes`, `tags`.

## Commands

```text
orbityard vault export --out PATH.km [--project DIR]
  # Interactive: passphrase (twice) + name/value pairs
orbityard vault export --out PATH.km --from-hints --project DIR
  # Titles from secret hints; values prompted
orbityard vault export --out PATH.km --title NAME --value-env ENV
  # Single-entry scripted export
orbityard vault export --out PATH.km --entries-file entries.json
  # Batch: [{title,value,url?,notes?}] — used by Desktop
orbityard vault add --file PATH.km --title TITLE [--value-env ENV]
orbityard vault list --file PATH.km
orbityard vault show --file PATH.km --title TITLE
```

## Surfaces

| Surface | Entry |
|---------|--------|
| CLI | `orbityard vault …` |
| TUI | Secrets → `v` → `./ship-secrets.km` |
| Desktop | **Export vault** (passphrase + paste prompts) |
| MCP | `orbit_vault` action `export` / `list` / `show` |
| Guide | step id `vault` after `secrets` |

## Invariants

1. Never leave plaintext secrets in `.ship/` (Desktop uses OS temp, then deletes).
2. Never log passphrase or secret values.
3. Prefer env `SHIP_VAULT_PASSPHRASE` for non-interactive; else TTY prompt.
4. Exported file is operator-portable → can open in Clavis.
5. `*.km` is gitignored.

## Proof

- L1: round-trip encode/decode unit test
- L2: `vault export` → `vault list` shows titles
- L3: Desktop **Export vault** / TUI `v` / MCP `orbit_vault` / guide step `vault`

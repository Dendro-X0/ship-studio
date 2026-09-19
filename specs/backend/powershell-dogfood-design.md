# Native PowerShell dogfood — band #26

**Status:** Done (first slice, 2026-09-19)  
**Parent:** release-surface value backlog  
**Owner:** `scripts/dogfood-advanced-publish.ps1`

## Problem

`scripts/dogfood-advanced-publish.sh` requires bash/WSL. On this Windows host WSL bash failed (`execvpe(/bin/bash)`), so L2 Advanced dogfood was blocked.

## Fix

Parity script: `scripts/dogfood-advanced-publish.ps1` — same fixture refresh + Advanced plan id / `desktop_view` asserts, resolves `shipctl.exe` from debug/release (or `SHIPCTL_PATH`).

## Usage

```powershell
# from repo root
.\scripts\dogfood-advanced-publish.ps1
# optional fixture path
.\scripts\dogfood-advanced-publish.ps1 -Fixture "E:\path\to\fixtures\advanced-dogfood"
```

## Proof

`powershell -File scripts/dogfood-advanced-publish.ps1` exits 0 with all `ok` lines.  

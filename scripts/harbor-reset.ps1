# Reset Harbor demo fixture to a clean mid-flight state (no .ship session).
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
if (-not $Root) { $Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path }
$Harbor = Join-Path $Root "fixtures\harbor"
if (-not (Test-Path $Harbor)) {
  Write-Error "missing $Harbor"
}
$Ship = Join-Path $Harbor ".ship"
if (Test-Path $Ship) {
  Remove-Item -Recurse -Force $Ship
  Write-Host "Harbor reset: removed $Ship"
} else {
  Write-Host "Harbor already clean (no .ship)"
}
Write-Host "Bind in Desktop: $Harbor"

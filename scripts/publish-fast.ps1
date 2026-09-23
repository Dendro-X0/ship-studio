# Fast path: burn Auto/ready publish gates (stops at Human/Open).
# Usage: powershell -File scripts/publish-fast.ps1 [-Project .] [-Mode general] [-Intent local] [-Chain 20]
param(
  [string]$Project = ".",
  [ValidateSet("general", "advanced")][string]$Mode = "general",
  [ValidateSet("local", "public")][string]$Intent = "local",
  [int]$Chain = 20
)
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Shipctl = $env:SHIPCTL_PATH
if (-not $Shipctl) {
  foreach ($c in @(
    "$Root\target\debug\shipctl.exe",
    "$Root\target\release\shipctl.exe"
  )) {
    if (Test-Path $c) { $Shipctl = $c; break }
  }
}
if (-not $Shipctl) {
  Push-Location $Root
  cargo build -p shipctl | Out-Null
  Pop-Location
  $Shipctl = "$Root\target\debug\shipctl.exe"
}
& $Shipctl publish --mode $Mode --intent $Intent --project $Project continue --chain $Chain
exit $LASTEXITCODE

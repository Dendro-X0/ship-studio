# Dogfood Advanced publish steps for Desktop Related matrix (Windows / PowerShell).
# Usage (repo root):
#   .\scripts\dogfood-advanced-publish.ps1
#   .\scripts\dogfood-advanced-publish.ps1 -Fixture "E:\path\to\fixture"
# Optional: $env:SHIPCTL_PATH = "E:\...\shipctl.exe"
param(
    [string]$Fixture = ""
)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
if (-not $Fixture) {
    $Fixture = Join-Path $Root "fixtures\advanced-dogfood"
}

function Write-Utf8NoBom([string]$Path, [string]$Content) {
    $dir = Split-Path -Parent $Path
    if ($dir -and -not (Test-Path $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    [System.IO.File]::WriteAllText($Path, $Content, [System.Text.UTF8Encoding]::new($false))
}

New-Item -ItemType Directory -Force -Path @(
    (Join-Path $Fixture ".ship"),
    (Join-Path $Fixture ".github\workflows"),
    (Join-Path $Fixture "android"),
    (Join-Path $Fixture "ios"),
    (Join-Path $Fixture "public"),
    (Join-Path $Fixture "apps\website")
) | Out-Null

Write-Utf8NoBom (Join-Path $Fixture "wrangler.toml") @"
name = "dogfood"
[[d1_databases]]
binding = "DB"
database_name = "x"
database_id = "..."
"@
Write-Utf8NoBom (Join-Path $Fixture "app.json") '{"expo":{"name":"dogfood","slug":"dogfood"}}'
Write-Utf8NoBom (Join-Path $Fixture "android\build.gradle") "// stub`n"
Write-Utf8NoBom (Join-Path $Fixture "Dockerfile") "FROM alpine`n"
Write-Utf8NoBom (Join-Path $Fixture ".github\workflows\release.yml") "name: release`non: push`n"
Write-Utf8NoBom (Join-Path $Fixture "steam_appid.txt") "480`n"
Write-Utf8NoBom (Join-Path $Fixture ".ship\markets.json") '["itch","epic","npm","crates","marketing","graduate","gumroad","lemon","suite","hf"]'
Write-Utf8NoBom (Join-Path $Fixture "public\manifest.webmanifest") '{"name":"dogfood","short_name":"dogfood","display":"standalone","start_url":"/"}'
Write-Utf8NoBom (Join-Path $Fixture "modelcard.md") "# Model card stub`n"
Write-Utf8NoBom (Join-Path $Fixture "apps\website\index.html") "<!doctype html><title>dogfood</title>`n"
Write-Utf8NoBom (Join-Path $Fixture ".ship\suite.json") '{"canonical_hint":"https://example.com/dogfood","siblings":[{"label":"sibling","path":"../sibling","env_keys":["NEXT_PUBLIC_DOGFOOD_URL"]}]}'
Write-Utf8NoBom (Join-Path $Fixture ".env.local") "NEON_DATABASE_URL=`n"
Write-Utf8NoBom (Join-Path $Fixture "signet.toml") "name = `"dogfood`"`n"

foreach ($f in @("LICENSE", "LICENSE.md", "SECURITY.md", "TRUST.md", "CHANGELOG.md")) {
    $p = Join-Path $Fixture $f
    if (Test-Path $p) { Remove-Item -Force $p }
}
foreach ($f in @("publish.json", "scopes.json", "studio.json")) {
    $p = Join-Path $Fixture ".ship\$f"
    if (Test-Path $p) { Remove-Item -Force $p }
}

$gitDir = Join-Path $Fixture ".git"
if (Get-Command git -ErrorAction SilentlyContinue) {
    if (-not (Test-Path $gitDir)) {
        git -C $Fixture init -q 2>$null | Out-Null
    }
    $origin = git -C $Fixture remote get-url origin 2>$null
    if (-not $origin) {
        git -C $Fixture remote add origin "https://github.com/example/ship-studio-dogfood.git" 2>$null | Out-Null
    }
}

function Resolve-Shipctl {
    if ($env:SHIPCTL_PATH -and (Test-Path $env:SHIPCTL_PATH)) {
        return (Resolve-Path $env:SHIPCTL_PATH).Path
    }
    $candidates = @(
        (Join-Path $Root "target\debug\shipctl.exe"),
        (Join-Path $Root "target\debug\shipctl"),
        (Join-Path $Root "target\release\shipctl.exe"),
        (Join-Path $Root "target\release\shipctl")
    )
    foreach ($c in $candidates) {
        if (Test-Path $c) { return (Resolve-Path $c).Path }
    }
    Push-Location $Root
    try {
        cargo build -p shipctl | Out-Null
    } finally {
        Pop-Location
    }
    foreach ($c in $candidates) {
        if (Test-Path $c) { return (Resolve-Path $c).Path }
    }
    throw "shipctl not found - run cargo build -p shipctl"
}

# Prefer a fresh debug build when SHIPCTL_PATH is unset (parity with bash script).
if (-not $env:SHIPCTL_PATH) {
    Push-Location $Root
    try {
        cargo build -p shipctl | Out-Null
    } finally {
        Pop-Location
    }
}

$Shipctl = Resolve-Shipctl
Write-Host "Fixture: $Fixture"
Write-Host "shipctl: $Shipctl"

$raw = & $Shipctl publish --mode advanced --project $Fixture 2>$null | Out-String
if (-not $raw.Trim()) {
    throw "shipctl publish returned empty output"
}
$plan = $raw | ConvertFrom-Json
$byId = @{}
foreach ($s in $plan.steps) {
    $byId[$s.id] = $s
}

$need = @(
    "listing.play",
    "listing.app_store",
    "listing.steam",
    "listing.itch",
    "listing.epic",
    "listing.npm",
    "listing.crates",
    "listing.huggingface",
    "submit.play",
    "submit.app_store",
    "db.provision",
    "ci.release",
    "container.build",
    "container.deploy",
    "legal.baseline",
    "trust.pack",
    "marketing.deploy",
    "sign.graduate",
    "listing.gumroad",
    "listing.lemon",
    "suite.url_sync"
)

$miss = 0
foreach ($id in $need) {
    if ($byId.ContainsKey($id)) {
        Write-Host "ok  $id"
    } else {
        Write-Host "MISS $id"
        $miss++
    }
}

$pairs = @(
    @{ Id = "listing.steam"; View = "portal" },
    @{ Id = "listing.npm"; View = "portal" },
    @{ Id = "listing.crates"; View = "portal" },
    @{ Id = "listing.huggingface"; View = "portal" },
    @{ Id = "listing.gumroad"; View = "portal" },
    @{ Id = "listing.lemon"; View = "portal" },
    @{ Id = "db.provision"; View = "env" },
    @{ Id = "ci.release"; View = "dashboard" },
    @{ Id = "container.build"; View = "portal" },
    @{ Id = "container.deploy"; View = "portal" },
    @{ Id = "submit.play"; View = "sign" },
    @{ Id = "legal.baseline"; View = "dashboard" },
    @{ Id = "trust.pack"; View = "sign" },
    @{ Id = "sign.graduate"; View = "sign" },
    @{ Id = "marketing.deploy"; View = "portal" },
    @{ Id = "suite.url_sync"; View = "dashboard" }
)

foreach ($p in $pairs) {
    $step = $byId[$p.Id]
    if ($step -and $step.desktop_view -eq $p.View) {
        Write-Host "ok  $($p.Id) -> $($p.View)"
    } else {
        Write-Host "MISS $($p.Id) -> $($p.View)"
        $miss++
    }
}

if ($miss -ne 0) {
    Write-Host "Dogfood plan incomplete."
    exit 1
}

Write-Host "Advanced dogfood plan OK - bind this folder in Desktop (Advanced mode):"
Write-Host "  $Fixture"

$debugDesktop = Join-Path $Root "target\debug\ship-studio-desktop.exe"
if ((Test-Path $debugDesktop) -and (Test-Path $Shipctl)) {
    Copy-Item -Force $Shipctl (Join-Path $Root "target\debug\$(Split-Path -Leaf $Shipctl)")
    Write-Host "Staged $(Split-Path -Leaf $Shipctl) next to target/debug desktop."
}

exit 0

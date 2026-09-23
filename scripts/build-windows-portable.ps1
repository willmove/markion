param(
    [string]$Version,
    [string]$ArtifactsRoot = "dist",
    [string]$BinaryPath = "target/release/markion.exe"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $repoRoot

if (-not $Version) {
    $packager = Get-Content -LiteralPath (Join-Path $repoRoot "packager.toml") -Raw
    if ($packager -notmatch '(?m)^version\s*=\s*"([^"]+)"') {
        throw "Unable to read version from packager.toml"
    }
    $Version = $Matches[1]
}

$binary = Join-Path $repoRoot $BinaryPath
if (-not (Test-Path -LiteralPath $binary -PathType Leaf)) {
    throw "Release binary not found: $binary"
}

$artifacts = Join-Path $repoRoot $ArtifactsRoot
New-Item -ItemType Directory -Force -Path $artifacts | Out-Null

$stageRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("markion-portable-" + [guid]::NewGuid().ToString("N"))
$stageApp = Join-Path $stageRoot "Markion"
New-Item -ItemType Directory -Force -Path $stageApp | Out-Null

try {
    Copy-Item -LiteralPath $binary -Destination (Join-Path $stageApp "markion.exe")
    Copy-Item -LiteralPath (Join-Path $repoRoot "assets") -Destination (Join-Path $stageApp "assets") -Recurse
    Copy-Item -LiteralPath (Join-Path $repoRoot "THIRD_PARTY_NOTICES.md") -Destination (Join-Path $stageApp "THIRD_PARTY_NOTICES.md")

    if (-not (Test-Path -LiteralPath (Join-Path $stageApp "assets/marknice-workspace") -PathType Container)) {
        throw "Portable payload is missing assets/marknice-workspace"
    }

    $zipPath = Join-Path $artifacts "markion_${Version}_x64-portable.zip"
    if (Test-Path -LiteralPath $zipPath) {
        Remove-Item -LiteralPath $zipPath -Force
    }
    Compress-Archive -Path (Join-Path $stageRoot "Markion") -DestinationPath $zipPath -CompressionLevel Optimal
    Write-Host "Wrote $zipPath"
}
finally {
    if (Test-Path -LiteralPath $stageRoot) {
        Remove-Item -LiteralPath $stageRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

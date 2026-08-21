param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("nsis", "app", "dmg", "deb", "appimage")]
    [string]$Format,
    [string]$ArtifactsRoot = "dist"
)

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$artifacts = [System.IO.Path]::GetFullPath((Join-Path $repoRoot $ArtifactsRoot))
if (-not (Test-Path -LiteralPath $artifacts -PathType Container)) {
    throw "Package artifact directory is unavailable: $artifacts"
}
$cleanupRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("markion-package-verify-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $cleanupRoot | Out-Null
$inspectionRoot = $cleanupRoot
$mountedDmg = $null

function Invoke-Native([string]$Label, [scriptblock]$Command) {
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE" }
}

try {
    switch ($Format) {
        "nsis" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*-setup.exe" -File | Select-Object -First 1
            if (-not $package) { throw "NSIS installer not found" }
            Invoke-Native "NSIS extraction" { 7z x -y "-o$cleanupRoot" $package.FullName }
        }
        "app" {
            $apps = @(Get-ChildItem -LiteralPath $artifacts -Filter "*.app" -Directory -Recurse)
            if (-not $apps) { throw "macOS app bundle not found" }
            $inspectionRoot = $artifacts
        }
        "dmg" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*.dmg" -File | Select-Object -First 1
            if (-not $package) { throw "DMG not found" }
            $mountedDmg = Join-Path $cleanupRoot "mounted"
            New-Item -ItemType Directory -Path $mountedDmg | Out-Null
            Invoke-Native "DMG mount" { hdiutil attach -readonly -nobrowse -mountpoint $mountedDmg $package.FullName }
            $inspectionRoot = $mountedDmg
        }
        "deb" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*.deb" -File | Select-Object -First 1
            if (-not $package) { throw "DEB package not found" }
            Invoke-Native "DEB extraction" { dpkg-deb -x $package.FullName $cleanupRoot }
        }
        "appimage" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*.AppImage" -File | Select-Object -First 1
            if (-not $package) { throw "AppImage not found" }
            Invoke-Native "AppImage executable permission" { chmod +x $package.FullName }
            Push-Location $cleanupRoot
            try { Invoke-Native "AppImage extraction" { & $package.FullName --appimage-extract } }
            finally { Pop-Location }
        }
    }

    $manifests = @(Get-ChildItem -LiteralPath $inspectionRoot -Filter "bundle-manifest.json" -File -Recurse |
        Where-Object { $_.Directory.Name -eq "marknice-workspace" })
    if (-not $manifests) { throw "Packaged MarkNice workspace manifest not found in $Format output" }
    foreach ($manifest in $manifests) {
        Invoke-Native "Packaged workspace verification" {
            cargo run --release -p wechat-workspace --bin verify-bundle -- $manifest.Directory.FullName
        }
    }
    Write-Host "Verified $($manifests.Count) packaged MarkNice workspace tree(s) in $Format output"
}
finally {
    if ($mountedDmg) { & hdiutil detach $mountedDmg -quiet }
    $resolvedCleanup = [System.IO.Path]::GetFullPath($cleanupRoot)
    $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if ($resolvedCleanup.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $resolvedCleanup).StartsWith("markion-package-verify-")) {
        Remove-Item -LiteralPath $resolvedCleanup -Recurse -Force -ErrorAction SilentlyContinue
    }
}

param(
    [Parameter(Mandatory = $true)]
    [ValidateSet("nsis", "app", "dmg", "deb", "appimage", "zip", "rpm")]
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

function Assert-ExportBundleClosure([System.IO.DirectoryInfo]$BundleRoot) {
    $required = @(
        "static/export-runtime.js",
        "static/marknice-format-runtime.js",
        "static/marknice-word-runtime.js",
        "static/vendor/html-docx.js",
        "LICENSE.html-docx-js.txt"
    )
    $manifestPath = Join-Path $BundleRoot.FullName "bundle-manifest.json"
    $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
    $paths = @($manifest.files | ForEach-Object { $_.path })
    foreach ($requiredPath in $required) {
        if ($paths -notcontains $requiredPath -or -not (Test-Path -LiteralPath (Join-Path $BundleRoot.FullName $requiredPath) -PathType Leaf)) {
            throw "Packaged export asset is missing: $requiredPath"
        }
    }
    $converter = @($manifest.third_party | Where-Object {
        $_.name -eq "html-docx-js" -and $_.version -eq "0.3.1" -and $_.license -eq "MIT" -and $_.license_file -eq "LICENSE.html-docx-js.txt"
    })
    if ($converter.Count -ne 1) { throw "Packaged html-docx-js provenance is incomplete" }

    $prohibited = @(Get-ChildItem -LiteralPath $BundleRoot.FullName -File -Recurse | Where-Object {
        $relative = $_.FullName.Substring($BundleRoot.FullName.Length + 1).Replace('\\', '/')
        $lower = $relative.ToLowerInvariant()
        $name = $_.Name.ToLowerInvariant()
        $lower -match '(^|/)node_modules(/|$)' -or
        $name -in @('package.json', 'package-lock.json', 'npm-shrinkwrap.json', '.npmrc', '.env', 'id_rsa', 'credentials') -or
        $name -match '\.(tgz|npm|docx|mht|pem|key|p12)$'
    })
    if ($prohibited) {
        throw "Packaged workspace contains prohibited export artifacts: $($prohibited.FullName -join ', ')"
    }
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
            
            # Verify AppImage permissions: AppRun and directories must be
            # world-executable (0755 or 0555), all files world-readable.
            # This prevents the firejail sandbox failure seen in the AppImage
            # catalog test (https://github.com/AppImage/appimage.github.io/pull/6581).
            $squashfsRoot = Join-Path $cleanupRoot "squashfs-root"
            Invoke-Native "AppImage permission verification" {
                bash -c @"
set -euo pipefail
root='$($squashfsRoot -replace '\\','/')'
cd `"`$root`"
# Check AppRun is executable by others (mode & 0001)
if [[ ! -x AppRun ]]; then
    echo 'ERROR: AppRun is not executable' >&2
    exit 1
fi
apprun_perms=`$(stat -c '%a' AppRun)
# stat -c '%a' returns 3-digit octal (e.g. '755'), so match XX5 pattern
if [[ ! `$apprun_perms =~ [0-9][0-9]5$ ]]; then
    echo `"ERROR: AppRun permissions are `$apprun_perms, expected world-executable (e.g. 0755)$`" >&2
    exit 1
fi
# Check all directories are readable+executable by others (mode & 0005 == 0005)
while IFS= read -r -d '' dir; do
    dir_perms=`$(stat -c '%a' `"`$dir`")
    if [[ ! `$dir_perms =~ [0-9][0-9]5$ ]]; then
        echo `"ERROR: Directory `$dir has permissions `$dir_perms, expected world-readable+executable (e.g. 0755)$`" >&2
        exit 1
    fi
done < <(find . -type d -print0)
# Check all files are readable by others (mode & 0004)
while IFS= read -r -d '' file; do
    file_perms=`$(stat -c '%a' `"`$file`")
    last_digit=`${file_perms: -1}
    if [[ ! `$last_digit =~ [4567] ]]; then
        echo `"ERROR: File `$file has permissions `$file_perms, not world-readable$`" >&2
        exit 1
    fi
done < <(find . -type f -print0)
echo 'AppImage permissions OK: AppRun and directories are world-executable, files are world-readable'
"@
            }
        }
        "zip" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*-portable.zip" -File | Select-Object -First 1
            if (-not $package) { throw "Portable zip not found" }
            Expand-Archive -LiteralPath $package.FullName -DestinationPath $cleanupRoot -Force
        }
        "rpm" {
            $package = Get-ChildItem -LiteralPath $artifacts -Filter "*.rpm" -File | Select-Object -First 1
            if (-not $package) { throw "RPM package not found" }
            Invoke-Native "RPM extraction" {
                bash -lc "cd '$($cleanupRoot -replace '\\','/')' && rpm2cpio '$($package.FullName -replace '\\','/')' | cpio -idmu --quiet"
            }
        }
    }

    $manifests = @(Get-ChildItem -LiteralPath $inspectionRoot -Filter "bundle-manifest.json" -File -Recurse |
        Where-Object { $_.Directory.Name -eq "marknice-workspace" })
    if (-not $manifests) { throw "Packaged MarkNice workspace manifest not found in $Format output" }
    foreach ($manifest in $manifests) {
        Invoke-Native "Packaged workspace verification" {
            cargo run --release -p wechat-workspace --bin verify-bundle -- $manifest.Directory.FullName
        }
        Assert-ExportBundleClosure $manifest.Directory
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

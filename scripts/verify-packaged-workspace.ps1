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

    # Optional providers must remain outside every core installer. The signed
    # bootstrap catalog is allowed, but workers, archives, installed-version
    # trees, native engines, and plugin-only notices are release blockers.
    $forbiddenOptionalPayloads = @(Get-ChildItem -LiteralPath $inspectionRoot -File -Recurse | Where-Object {
        $relative = $_.FullName.Substring($inspectionRoot.Length).TrimStart('\', '/').Replace('\', '/')
        $lowerPath = $relative.ToLowerInvariant()
        $lowerName = $_.Name.ToLowerInvariant()
        $lowerName -in @('pdfium.dll', 'libpdfium.dylib', 'libpdfium.so', 'markion-plugin-pdf', 'markion-plugin-pdf.exe') -or
        $lowerName -like '*.markion-plugin' -or
        $lowerPath -match '(^|/)plugins/(staging|versions|dev\.markion\.pdf)(/|$)' -or
        $lowerPath -match 'pdf-plugin.*third_party_notices'
    })
    if ($forbiddenOptionalPayloads) {
        throw "Core $Format output contains optional plugin payloads: $($forbiddenOptionalPayloads.FullName -join ', ')"
    }
    $bootstrapCatalogs = @(Get-ChildItem -LiteralPath $inspectionRoot -Filter 'catalog.json' -File -Recurse | Where-Object {
        $_.Directory.Name -eq 'plugins' -and
        (Test-Path -LiteralPath (Join-Path $_.Directory.FullName 'catalog.json.minisig') -PathType Leaf)
    })
    if ($bootstrapCatalogs.Count -lt 1) {
        throw "Core $Format output is missing the signed bootstrap plugin catalog"
    }
    $coreNotices = @(Get-ChildItem -LiteralPath $inspectionRoot -Filter 'THIRD_PARTY_NOTICES.md' -File -Recurse)
    foreach ($notice in $coreNotices) {
        $noticeText = Get-Content -LiteralPath $notice.FullName -Raw
        if ($noticeText -like '*PDFium build 7881*' -or $noticeText -like '*pdfium-render 0.9.3*') {
            throw "Core $Format notice unexpectedly claims optional PDF runtime content: $($notice.FullName)"
        }
    }
    Write-Host "Verified that $Format contains bootstrap metadata but no optional plugin or PDF runtime payload"
}
finally {
    if ($mountedDmg) { & hdiutil detach $mountedDmg -quiet }
    $resolvedCleanup = [System.IO.Path]::GetFullPath($cleanupRoot)
    $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if ($resolvedCleanup.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $resolvedCleanup).StartsWith("markion-package-verify-")) {
        Remove-Item -LiteralPath $resolvedCleanup -Recurse -Force -ErrorAction SilentlyContinue
    }
}

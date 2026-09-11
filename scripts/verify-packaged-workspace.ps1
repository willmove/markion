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

    $viewingManifest = Get-Content -LiteralPath (Join-Path $repoRoot 'config/pdf-viewing.json') -Raw | ConvertFrom-Json
    $target = if ($IsWindows) {
        @($viewingManifest.targets | Where-Object { $_.triple -eq 'x86_64-pc-windows-msvc' })
    }
    elseif ($IsMacOS) {
        @($viewingManifest.targets | Where-Object { $_.triple -eq 'aarch64-apple-darwin' })
    }
    else {
        @($viewingManifest.targets | Where-Object { $_.triple -eq 'x86_64-unknown-linux-gnu' })
    }
    if ($target.Count -ne 1) { throw 'Unable to select the native PDFium package manifest entry' }
    $runtimeNames = @('pdfium.dll', 'libpdfium.dylib', 'libpdfium.so')
    $runtimes = @(Get-ChildItem -LiteralPath $inspectionRoot -File -Recurse | Where-Object {
        $runtimeNames -contains $_.Name
    })
    if ($runtimes.Count -ne 1) {
        throw "Expected exactly one PDFium runtime in $Format output, found $($runtimes.Count): $($runtimes.FullName -join ', ')"
    }
    $runtime = $runtimes[0]
    if ($runtime.Name -ne $target[0].installed_library) {
        throw "Wrong-target PDFium runtime in $Format output: $($runtime.Name)"
    }
    if ($runtime.Length -ne $target[0].runtime_size_bytes) {
        throw "PDFium runtime size mismatch in $Format output: expected $($target[0].runtime_size_bytes), got $($runtime.Length)"
    }
    $relativeRuntime = $runtime.FullName.Substring($inspectionRoot.Length).TrimStart('\', '/').Replace('\', '/')
    if ($relativeRuntime -notmatch "(^|/)assets/pdfium/$([regex]::Escape($runtime.Name))$") {
        throw "PDFium runtime is outside the installed resource location: $relativeRuntime"
    }
    $pdfiumDirectory = $runtime.Directory
    if ($pdfiumDirectory.Name -ne 'pdfium' -or $pdfiumDirectory.Parent.Name -ne 'assets') {
        throw "PDFium runtime resource ancestry is invalid: $($runtime.FullName)"
    }
    $resourceRoot = $pdfiumDirectory.Parent.Parent.FullName

    $forbiddenPdfFiles = @(Get-ChildItem -LiteralPath $inspectionRoot -File -Recurse | Where-Object {
        $lower = $_.Name.ToLowerInvariant()
        ($lower -match 'pdfium' -and $_.FullName -ne $runtime.FullName) -or
        $lower -match '\.(h|hpp|lib|a|pdb|dSYM|tgz|zip)$' -or
        $lower -match 'pdfium.*(v8|xfa|javascript|sample|debug)'
    })
    if ($forbiddenPdfFiles) {
        throw "Packaged output contains forbidden PDF development/runtime assets: $($forbiddenPdfFiles.FullName -join ', ')"
    }

    $notices = @(Get-ChildItem -LiteralPath $inspectionRoot -Filter 'THIRD_PARTY_NOTICES.md' -File -Recurse)
    if ($notices.Count -lt 1) { throw "Packaged third-party notices are missing from $Format output" }
    $noticeText = Get-Content -LiteralPath $notices[0].FullName -Raw
    if ($noticeText -notlike '*pdfium-render 0.9.3*' -or $noticeText -notlike '*PDFium build 7881*') {
        throw "Packaged third-party notices omit PDFium provenance in $Format output"
    }

    $encodedFixture = Get-Content -LiteralPath (Join-Path $repoRoot 'crates/pdf-viewer/tests/fixtures/one-page-embedded.pdf.b64') -Raw
    $fixturePath = Join-Path $cleanupRoot 'one-page-embedded.pdf'
    [System.IO.File]::WriteAllBytes(
        $fixturePath,
        [Convert]::FromBase64String(($encodedFixture -replace '\s', ''))
    )
    Invoke-Native "Packaged PDF runtime render smoke" {
        cargo run --release --locked -p markion-pdf-viewer --bin pdf-packaged-smoke -- `
            --resource-root $resourceRoot --fixture $fixturePath
    }
    Write-Host "Verified one target-matching PDFium runtime and an offline page render in $Format output"
}
finally {
    if ($mountedDmg) { & hdiutil detach $mountedDmg -quiet }
    $resolvedCleanup = [System.IO.Path]::GetFullPath($cleanupRoot)
    $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
    if ($resolvedCleanup.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $resolvedCleanup).StartsWith("markion-package-verify-")) {
        Remove-Item -LiteralPath $resolvedCleanup -Recurse -Force -ErrorAction SilentlyContinue
    }
}

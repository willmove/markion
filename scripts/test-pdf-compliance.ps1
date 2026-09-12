param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$viewerManifestPath = Join-Path $repoRoot 'crates/pdf-viewer/Cargo.toml'
$noticesPath = Join-Path $repoRoot 'THIRD_PARTY_NOTICES.md'
$viewerManifest = Get-Content -LiteralPath $viewerManifestPath -Raw
$notices = Get-Content -LiteralPath $noticesPath -Raw

if ($viewerManifest -notmatch 'pdfium-render\s*=\s*\{\s*version\s*=\s*"=0\.9\.3"\s*,\s*default-features\s*=\s*false\s*,\s*features\s*=\s*\["pdfium_7881"\]\s*\}') {
    throw 'markion-pdf-viewer must pin pdfium-render 0.9.3 with defaults disabled and only pdfium_7881 enabled.'
}
if ($viewerManifest -match '(?im)^gpui\s*=') {
    throw 'markion-pdf-viewer must remain GPUI-free.'
}
foreach ($requiredNotice in @(
    'pdfium-render 0.9.3',
    'MIT License or the Apache License 2.0',
    'PDFium build 7881',
    'BSD-style license',
    'bblanchon/pdfium-binaries',
    'third-party components'
)) {
    if ($notices -notlike "*$requiredNotice*") {
        throw "THIRD_PARTY_NOTICES.md is missing required PDF notice text: $requiredNotice"
    }
}

$tree = & cargo tree --locked -p markion-pdf-viewer --edges normal,build
if ($LASTEXITCODE -ne 0) { throw 'Unable to inspect the PDF viewer dependency tree.' }
if ($tree -match '(?im)^.*\bgpui\b') {
    throw 'The PDF viewer dependency tree unexpectedly contains GPUI.'
}

Write-Host 'PDF dependency features and license notices passed.'

param(
    [string]$Source = "C:\Coding\EditorProjects\marknice",
    [switch]$RefreshThirdParty
)

$ErrorActionPreference = "Stop"
$expectedCommit = "c009c1ec7e7c92f89afa5a32edcb126b5296bda7"
$repoRoot = Split-Path -Parent $PSScriptRoot
$bundleRoot = Join-Path $repoRoot "assets\marknice-workspace"
$staticRoot = Join-Path $bundleRoot "static"
$vendorRoot = Join-Path $staticRoot "vendor"
$fontRoot = Join-Path $vendorRoot "fonts"

if ((git -C $Source rev-parse HEAD).Trim() -ne $expectedCommit) {
    throw "MarkNice source must be pinned at $expectedCommit"
}
if (-not (Test-Path -LiteralPath $bundleRoot -PathType Container)) {
    throw "Expected checked-in workspace shell at $bundleRoot"
}
New-Item -ItemType Directory -Force -Path $vendorRoot, $fontRoot | Out-Null

$sourceLines = Get-Content -LiteralPath (Join-Path $Source "src\main.js")
$runtime = @(
    "/* Generated from MarkNice $expectedCommit by scripts/sync-marknice-workspace.ps1. */"
    $sourceLines[216..646]
    "  statusEl.textContent = md.trim() ? locale.opened + localImageStatusSuffix() : '';"
    $sourceLines[650..682]
) -join "`n"
$runtime = $runtime.Replace(
    "    document.execCommand('copy');",
    "    if (!document.execCommand('copy')) throw new Error('clipboard denied');"
)
Set-Content -LiteralPath (Join-Path $staticRoot "theme-runtime.js") -Value $runtime -Encoding utf8NoBOM
Copy-Item -LiteralPath (Join-Path $Source "LICENSE") -Destination (Join-Path $bundleRoot "LICENSE.marknice.txt") -Force

if ($RefreshThirdParty) {
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        throw "Refreshing pinned third-party packages requires npm; normal builds do not."
    }
    $refreshRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("markion-marknice-refresh-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $refreshRoot | Out-Null
    try {
        $markedArchive = (& npm pack marked@15.0.12 --pack-destination $refreshRoot --silent | Select-Object -Last 1).Trim()
        $katexArchive = (& npm pack katex@0.16.11 --pack-destination $refreshRoot --silent | Select-Object -Last 1).Trim()
        if ($LASTEXITCODE -ne 0 -or -not $markedArchive -or -not $katexArchive) {
            throw "npm could not fetch the pinned renderer packages"
        }
        $markedExtract = Join-Path $refreshRoot "marked"
        $katexExtract = Join-Path $refreshRoot "katex"
        New-Item -ItemType Directory -Path $markedExtract, $katexExtract | Out-Null
        tar -xf (Join-Path $refreshRoot $markedArchive) -C $markedExtract
        tar -xf (Join-Path $refreshRoot $katexArchive) -C $katexExtract
        Copy-Item -LiteralPath (Join-Path $markedExtract "package\lib\marked.umd.js") -Destination (Join-Path $vendorRoot "marked.umd.js") -Force
        Copy-Item -LiteralPath (Join-Path $markedExtract "package\LICENSE.md") -Destination (Join-Path $bundleRoot "LICENSE.marked.txt") -Force
        Copy-Item -LiteralPath (Join-Path $katexExtract "package\dist\katex.min.js") -Destination (Join-Path $vendorRoot "katex.min.js") -Force
        Copy-Item -LiteralPath (Join-Path $katexExtract "package\dist\katex.min.css") -Destination (Join-Path $vendorRoot "katex.min.css") -Force
        Copy-Item -LiteralPath (Join-Path $katexExtract "package\LICENSE") -Destination (Join-Path $bundleRoot "LICENSE.katex.txt") -Force
        Copy-Item -Path (Join-Path $katexExtract "package\dist\fonts\*") -Destination $fontRoot -Force
    } finally {
        $resolvedRefresh = [System.IO.Path]::GetFullPath($refreshRoot)
        $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
        if ($resolvedRefresh.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $resolvedRefresh).StartsWith("markion-marknice-refresh-")) {
            Remove-Item -LiteralPath $resolvedRefresh -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

$requiredThirdParty = @("marked.umd.js", "katex.min.js", "katex.min.css")
foreach ($file in $requiredThirdParty) {
    if (-not (Test-Path -LiteralPath (Join-Path $vendorRoot $file) -PathType Leaf)) {
        throw "Missing vendored dependency $file; rerun with -RefreshThirdParty"
    }
}

$manifestPath = Join-Path $bundleRoot "bundle-manifest.json"
$files = Get-ChildItem -LiteralPath $bundleRoot -File -Recurse |
    Where-Object { $_.FullName -ne $manifestPath } |
    Sort-Object FullName |
    ForEach-Object {
        [ordered]@{
            path = $_.FullName.Substring($bundleRoot.Length + 1).Replace('\', '/')
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }
$manifest = [ordered]@{
    import_format_version = 1
    source_repository = "https://github.com/willmove/marknice"
    source_commit = $expectedCommit
    third_party = @(
        [ordered]@{ name = "MarkNice"; version = $expectedCommit; license = "MIT"; license_file = "LICENSE.marknice.txt" }
        [ordered]@{ name = "marked"; version = "15.0.12"; license = "MIT"; license_file = "LICENSE.marked.txt" }
        [ordered]@{ name = "KaTeX"; version = "0.16.11"; license = "MIT"; license_file = "LICENSE.katex.txt" }
    )
    files = @($files)
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
Write-Host "Synchronized $($files.Count) workspace files from MarkNice $expectedCommit"

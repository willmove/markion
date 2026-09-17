param(
    [Parameter(Mandatory = $true)][string]$BaseCatalogPath,
    [Parameter(Mandatory = $true)][string]$ReportsRoot,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [Parameter(Mandatory = $true)][string]$ReleaseTag,
    [Parameter(Mandatory = $true)][string]$Repository,
    [Parameter(Mandatory = $true)][long]$Sequence
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-RequiredPath {
    param([string]$Path, [string]$Label, [switch]$Directory)
    $resolved = [System.IO.Path]::GetFullPath($Path)
    $pathType = if ($Directory) { 'Container' } else { 'Leaf' }
    if (-not (Test-Path -LiteralPath $resolved -PathType $pathType)) {
        throw "$Label is unavailable: $resolved"
    }
    return $resolved
}

if ($ReleaseTag -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+$') {
    throw "Plugin catalogs must reference a stable release tag, got: $ReleaseTag"
}
if ($Repository -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
    throw "Invalid GitHub repository identity: $Repository"
}
if ($Sequence -lt 1) { throw 'Catalog sequence must be positive.' }

$baseCatalog = Resolve-RequiredPath -Path $BaseCatalogPath -Label 'Base plugin catalog'
$reportsDirectory = Resolve-RequiredPath -Path $ReportsRoot -Label 'Plugin report directory' -Directory
$catalog = Get-Content -LiteralPath $baseCatalog -Raw | ConvertFrom-Json
$catalog.sequence = $Sequence
$plugins = @($catalog.plugins | Where-Object { $_.plugin_id -eq 'dev.markion.pdf' })
if ($plugins.Count -ne 1) { throw 'Base catalog must contain exactly one dev.markion.pdf entry.' }
$plugin = $plugins[0]

$reports = @{}
foreach ($file in @(Get-ChildItem -LiteralPath $reportsDirectory -Filter '*.size-report.json' -File -Recurse)) {
    try { $report = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json }
    catch { continue }
    $requiredFields = @('target', 'archive_sha256', 'archive_bytes', 'installed_bytes', 'passed')
    if (@($requiredFields | Where-Object { $report.PSObject.Properties.Name -notcontains $_ }).Count -ne 0) {
        continue
    }
    $target = [string]$report.target
    if ($reports.ContainsKey($target)) { throw "Duplicate size report for $target." }
    if (-not $report.passed) { throw "Plugin size report failed for $target." }
    $reports[$target] = $report
}

$targetTriples = @{
    'windows/x86_64' = 'x86_64-pc-windows-msvc'
    'macos/aarch64' = 'aarch64-apple-darwin'
    'linux/x86_64' = 'x86_64-unknown-linux-gnu'
}
foreach ($artifact in @($plugin.artifacts)) {
    $key = "$($artifact.target.os)/$($artifact.target.arch)"
    if (-not $targetTriples.ContainsKey($key)) { throw "Unsupported catalog target: $key" }
    $triple = $targetTriples[$key]
    if (-not $reports.ContainsKey($triple)) { throw "Missing size report for $triple." }
    $report = $reports[$triple]
    $filename = "$($plugin.plugin_id)-$($plugin.version)-$triple.markion-plugin"
    $archives = @(Get-ChildItem -LiteralPath $reportsDirectory -Filter $filename -File -Recurse)
    if ($archives.Count -ne 1) { throw "Expected exactly one archive named $filename." }
    $archive = $archives[0]
    if ([long]$archive.Length -ne [long]$report.archive_bytes) {
        throw "Archive length differs from report for $triple."
    }
    $digest = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive.FullName).Hash.ToLowerInvariant()
    if ($digest -ne ([string]$report.archive_sha256).ToLowerInvariant()) {
        throw "Archive digest differs from report for $triple."
    }
    $artifact.url = "https://github.com/$Repository/releases/download/$ReleaseTag/$filename"
    $artifact.length = [long]$report.archive_bytes
    $artifact.sha256 = $digest
    $artifact.installed_size_bytes = [long]$report.installed_bytes
}

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$parent = Split-Path -Parent $resolvedOutput
if (-not (Test-Path -LiteralPath $parent -PathType Container)) {
    New-Item -ItemType Directory -Path $parent -Force | Out-Null
}
[System.IO.File]::WriteAllText(
    $resolvedOutput,
    ($catalog | ConvertTo-Json -Depth 20 -Compress),
    [System.Text.UTF8Encoding]::new($false)
)

[ordered]@{
    catalog_path = $resolvedOutput
    sequence = $Sequence
    plugin_id = $plugin.plugin_id
    plugin_version = $plugin.version
    artifacts = @($plugin.artifacts).Count
} | ConvertTo-Json -Compress

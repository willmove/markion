param(
    [Parameter(Mandatory = $true)][string]$WorkerPath,
    [Parameter(Mandatory = $true)][string]$RuntimePath,
    [Parameter(Mandatory = $true)][string]$NoticePath,
    [Parameter(Mandatory = $true)][string]$TargetTriple,
    [Parameter(Mandatory = $true)][string]$SecretKeyPath,
    [Parameter(Mandatory = $true)][string]$PublicKeyPath,
    [Parameter(Mandatory = $true)][string]$OutputRoot
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-RequiredFile {
    param([Parameter(Mandatory = $true)][string]$Path, [Parameter(Mandatory = $true)][string]$Label)
    $resolved = [System.IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "$Label is unavailable: $resolved"
    }
    return Get-Item -LiteralPath $resolved
}

function Invoke-Native {
    param([Parameter(Mandatory = $true)][string]$Label, [Parameter(Mandatory = $true)][scriptblock]$Command)
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE." }
}

function ConvertTo-CanonicalValue {
    param([Parameter(Mandatory = $false)]$Value)
    if ($null -eq $Value) { return $null }
    if ($Value -is [System.Collections.IDictionary]) {
        $ordered = [ordered]@{}
        foreach ($key in @($Value.Keys | ForEach-Object { [string]$_ } | Sort-Object)) {
            $ordered[$key] = ConvertTo-CanonicalValue $Value[$key]
        }
        return $ordered
    }
    if ($Value -is [System.Management.Automation.PSCustomObject]) {
        $ordered = [ordered]@{}
        foreach ($property in @($Value.PSObject.Properties.Name | Sort-Object)) {
            $ordered[$property] = ConvertTo-CanonicalValue $Value.$property
        }
        return $ordered
    }
    if ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [string]) {
        $items = @($Value | ForEach-Object { ConvertTo-CanonicalValue $_ })
        return ,$items
    }
    return $Value
}

function New-EmptyDirectory {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (Test-Path -LiteralPath $Path) {
        if (@(Get-ChildItem -LiteralPath $Path -Force).Count -ne 0) {
            throw "Output directory must be absent or empty: $Path"
        }
    }
    else { New-Item -ItemType Directory -Path $Path -Force | Out-Null }
}

$targets = @{
    'x86_64-pc-windows-msvc' = [ordered]@{ os = 'windows'; arch = 'x86_64'; executable = 'markion-plugin-pdf.exe'; runtime = 'pdfium.dll' }
    'aarch64-apple-darwin' = [ordered]@{ os = 'macos'; arch = 'aarch64'; executable = 'markion-plugin-pdf'; runtime = 'libpdfium.dylib' }
    'x86_64-unknown-linux-gnu' = [ordered]@{ os = 'linux'; arch = 'x86_64'; executable = 'markion-plugin-pdf'; runtime = 'libpdfium.so' }
}
if (-not $targets.ContainsKey($TargetTriple)) { throw "Unsupported target: $TargetTriple" }
$target = $targets[$TargetTriple]
$worker = Resolve-RequiredFile -Path $WorkerPath -Label 'PDF worker prototype'
$runtime = Resolve-RequiredFile -Path $RuntimePath -Label 'PDFium runtime'
$notice = Resolve-RequiredFile -Path $NoticePath -Label 'Third-party notices'
$secretKey = Resolve-RequiredFile -Path $SecretKeyPath -Label 'Fixture secret key'
$publicKey = Resolve-RequiredFile -Path $PublicKeyPath -Label 'Fixture public key'
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputRoot)
New-EmptyDirectory -Path $resolvedOutput
$stageRoot = Join-Path $resolvedOutput 'stage'
New-Item -ItemType Directory -Path (Join-Path $stageRoot 'bin') -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $stageRoot 'resources') -Force | Out-Null
$paths = [ordered]@{
    "bin/$($target.executable)" = $worker.FullName
    "resources/$($target.runtime)" = $runtime.FullName
    'THIRD_PARTY_NOTICES.md' = $notice.FullName
}
foreach ($relative in $paths.Keys) {
    Copy-Item -LiteralPath $paths[$relative] -Destination (Join-Path $stageRoot $relative)
}

$identities = [ordered]@{}
foreach ($locale in @('de', 'en', 'es', 'fr', 'ja', 'zh-Hans', 'zh-Hant')) {
    $identities[$locale] = [ordered]@{
        name = 'Markion PDF Viewer'
        description = 'Read local PDF documents in Markion.'
    }
}
$members = foreach ($relative in $paths.Keys) {
    $item = Get-Item -LiteralPath (Join-Path $stageRoot $relative)
    [ordered]@{
        path = $relative
        length = [long]$item.Length
        sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $item.FullName).Hash.ToLowerInvariant()
        executable = $relative -like 'bin/*'
    }
}
$manifest = [ordered]@{
    schema_version = 1
    plugin_id = 'dev.markion.pdf'
    version = '0.1.0'
    publisher = 'Markion'
    host_version = '>=0.3.9, <0.4.0'
    protocol = [ordered]@{ major = 1; min_minor = 0; max_minor = 0 }
    target = [ordered]@{ os = $target.os; arch = $target.arch }
    entry_point = "bin/$($target.executable)"
    identities = $identities
    permissions = @('read-selected-files')
    capabilities = @(
        [ordered]@{
            id = 'paged-document/v1'
            limits = [ordered]@{
                max_control_bytes = 65536
                max_body_bytes = 33554432
                max_in_flight_requests = 8
                request_timeout_ms = 30000
            }
        }
    )
    file_handlers = @(
        [ordered]@{
            extensions = @('.pdf')
            capability = 'paged-document/v1'
            priority = 100
            icon = 'document-pdf'
        }
    )
    archive_size_bytes = 6291456
    installed_size_bytes = 10485760
    members = @($members)
}
$manifestPath = Join-Path $stageRoot 'plugin.json'
$signaturePath = Join-Path $stageRoot 'plugin.json.minisig'
$manifestJson = ConvertTo-CanonicalValue $manifest | ConvertTo-Json -Depth 12 -Compress
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, [System.Text.UTF8Encoding]::new($false))
Invoke-Native -Label 'PDF prototype manifest signing' -Command {
    minisign -S -s $secretKey.FullName -m $manifestPath -x $signaturePath -t 'Markion PDF plugin prototype' -q
}
Invoke-Native -Label 'PDF prototype signature verification' -Command {
    minisign -Vm $manifestPath -p $publicKey.FullName -x $signaturePath -q
}

$archivePath = Join-Path $resolvedOutput "markion-pdf-0.1.0-$TargetTriple.markion-plugin"
Add-Type -AssemblyName System.IO.Compression
$archive = [System.IO.Compression.ZipFile]::Open($archivePath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    $orderedEntries = @('plugin.json', 'plugin.json.minisig') + @($paths.Keys)
    foreach ($relative in $orderedEntries) {
        $source = Join-Path $stageRoot $relative
        $entry = $archive.CreateEntry($relative, [System.IO.Compression.CompressionLevel]::Optimal)
        $mode = if ($relative -like 'bin/*') { 0x81ED } else { 0x81A4 }
        $entry.ExternalAttributes = [System.BitConverter]::ToInt32(
            [System.BitConverter]::GetBytes(([uint32]$mode) -shl 16), 0
        )
        $input = [System.IO.File]::OpenRead($source)
        $output = $entry.Open()
        try { $input.CopyTo($output) }
        finally { $output.Dispose(); $input.Dispose() }
    }
}
finally { $archive.Dispose() }

$archiveItem = Get-Item -LiteralPath $archivePath
$installedFiles = @(Get-ChildItem -LiteralPath $stageRoot -File -Recurse)
[long]$installedBytes = ($installedFiles | Measure-Object -Property Length -Sum).Sum
$archivePassed = $archiveItem.Length -le 6291456
$installedPassed = $installedBytes -le 10485760
$report = [ordered]@{
    schema_version = 1
    target = $TargetTriple
    archive_path = $archiveItem.FullName
    archive_bytes = [long]$archiveItem.Length
    archive_limit_bytes = 6291456
    archive_passed = $archivePassed
    installed_bytes = $installedBytes
    installed_limit_bytes = 10485760
    installed_passed = $installedPassed
    archive_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $archiveItem.FullName).Hash.ToLowerInvariant()
    members = @(
        $installedFiles | Sort-Object FullName | ForEach-Object {
            [ordered]@{
                path = [System.IO.Path]::GetRelativePath($stageRoot, $_.FullName).Replace('\', '/')
                bytes = [long]$_.Length
                sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLowerInvariant()
            }
        }
    )
    passed = $archivePassed -and $installedPassed
}
$reportPath = Join-Path $resolvedOutput 'size-report.json'
[System.IO.File]::WriteAllText(
    $reportPath,
    (($report | ConvertTo-Json -Depth 8) + [Environment]::NewLine),
    [System.Text.UTF8Encoding]::new($false)
)
$report | ConvertTo-Json -Depth 8 -Compress
if (-not $report.passed) { throw "PDF plugin prototype exceeds its size budget; report: $reportPath" }

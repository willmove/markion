param(
    [Parameter(Mandatory = $true)][string]$WorkerPath,
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

$targets = @{
    'x86_64-pc-windows-msvc' = [ordered]@{ os = 'windows'; arch = 'x86_64'; executable = 'plugin-fixture-worker.exe' }
    'aarch64-apple-darwin' = [ordered]@{ os = 'macos'; arch = 'aarch64'; executable = 'plugin-fixture-worker' }
    'x86_64-unknown-linux-gnu' = [ordered]@{ os = 'linux'; arch = 'x86_64'; executable = 'plugin-fixture-worker' }
}
if (-not $targets.ContainsKey($TargetTriple)) { throw "Unsupported target: $TargetTriple" }
$target = $targets[$TargetTriple]
$worker = Resolve-RequiredFile -Path $WorkerPath -Label 'Fixture worker'
$secretKey = Resolve-RequiredFile -Path $SecretKeyPath -Label 'Fixture secret key'
$publicKey = Resolve-RequiredFile -Path $PublicKeyPath -Label 'Fixture public key'
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputRoot)
$stageRoot = Join-Path $resolvedOutput 'stage'
$extractRoot = Join-Path $resolvedOutput 'extracted'
$archivePath = Join-Path $resolvedOutput "fixture-1.0.0-$TargetTriple.markion-plugin"
if (Test-Path -LiteralPath $resolvedOutput) {
    if (@(Get-ChildItem -LiteralPath $resolvedOutput -Force).Count -ne 0) {
        throw "Output directory must be absent or empty: $resolvedOutput"
    }
}
else { New-Item -ItemType Directory -Path $resolvedOutput -Force | Out-Null }
New-Item -ItemType Directory -Path (Join-Path $stageRoot 'bin') -Force | Out-Null
Copy-Item -LiteralPath $worker.FullName -Destination (Join-Path $stageRoot "bin/$($target.executable)")

$workerStagePath = Join-Path $stageRoot "bin/$($target.executable)"
$workerHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $workerStagePath).Hash.ToLowerInvariant()
$workerLength = (Get-Item -LiteralPath $workerStagePath).Length
$identities = [ordered]@{}
foreach ($locale in @('de', 'en', 'es', 'fr', 'ja', 'zh-Hans', 'zh-Hant')) {
    $identities[$locale] = [ordered]@{
        name = 'Markion Plugin Fixture'
        description = 'Cross-platform signed plugin launch fixture.'
    }
}
$manifest = [ordered]@{
    schema_version = 1
    plugin_id = 'dev.markion.fixture'
    version = '1.0.0'
    publisher = 'Markion'
    host_version = '>=0.3.9, <0.4.0'
    protocol = [ordered]@{ major = 1; min_minor = 0; max_minor = 0 }
    target = [ordered]@{ os = $target.os; arch = $target.arch }
    entry_point = "bin/$($target.executable)"
    identities = $identities
    permissions = @()
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
    file_handlers = @()
    archive_size_bytes = 6291456
    installed_size_bytes = 10485760
    members = @(
        [ordered]@{
            path = "bin/$($target.executable)"
            length = [long]$workerLength
            sha256 = $workerHash
            executable = $true
        }
    )
}
$manifestPath = Join-Path $stageRoot 'plugin.json'
$signaturePath = Join-Path $stageRoot 'plugin.json.minisig'
$manifestJson = ConvertTo-CanonicalValue $manifest | ConvertTo-Json -Depth 12 -Compress
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, [System.Text.UTF8Encoding]::new($false))
Invoke-Native -Label 'Fixture manifest signing' -Command {
    minisign -S -s $secretKey.FullName -m $manifestPath -x $signaturePath -t 'Markion plugin fixture' -q
}
Invoke-Native -Label 'Fixture manifest signature verification' -Command {
    minisign -Vm $manifestPath -p $publicKey.FullName -x $signaturePath -q
}

Add-Type -AssemblyName System.IO.Compression
$archive = [System.IO.Compression.ZipFile]::Open($archivePath, [System.IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($relative in @('plugin.json', 'plugin.json.minisig', "bin/$($target.executable)")) {
        $source = Join-Path $stageRoot $relative
        $entry = $archive.CreateEntry($relative, [System.IO.Compression.CompressionLevel]::Optimal)
        $entry.LastWriteTime = [DateTimeOffset]::new(1980, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
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
if ($archiveItem.Length -gt 6291456) { throw "Fixture archive exceeds 6 MiB: $($archiveItem.Length)" }
$report = [ordered]@{
    schema_version = 1
    target = $TargetTriple
    archive_path = $archiveItem.FullName
    archive_bytes = [long]$archiveItem.Length
    archive_sha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $archiveItem.FullName).Hash.ToLowerInvariant()
    worker_bytes = [long]$workerLength
    worker_sha256 = $workerHash
    public_key_path = $publicKey.FullName
    extraction_path = $extractRoot
}
$report | ConvertTo-Json -Depth 5 -Compress

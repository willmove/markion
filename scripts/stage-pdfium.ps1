param(
    [string]$TargetTriple,
    [string]$ManifestPath = (Join-Path $PSScriptRoot "..\config\pdf-viewing.json"),
    [string]$OutputRoot = (Join-Path $PSScriptRoot "..\target\pdfium-runtime"),
    [string]$ArchivePath,
    [switch]$ValidateManifestOnly
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Assert-RequiredProperty {
    param(
        [Parameter(Mandatory = $true)]$Value,
        [Parameter(Mandatory = $true)][string]$Name
    )

    if ($Value.PSObject.Properties.Name -notcontains $Name) {
        throw "PDF viewing manifest is missing '$Name'."
    }
}

function Assert-SafeRelativePath {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Label
    )

    $normalized = $Path.Replace('\', '/')
    if ([string]::IsNullOrWhiteSpace($normalized) -or
        [System.IO.Path]::IsPathRooted($Path) -or
        $normalized.StartsWith('/') -or
        $normalized.Split('/') -contains '..') {
        throw "$Label is not a safe relative path: $Path"
    }
}

function Assert-PathUnderTarget {
    param([Parameter(Mandatory = $true)][string]$Path)

    $repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
    $targetRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'target'))
    $resolved = [System.IO.Path]::GetFullPath($Path)
    $prefix = $targetRoot.TrimEnd(
        [System.IO.Path]::DirectorySeparatorChar,
        [System.IO.Path]::AltDirectorySeparatorChar
    ) + [System.IO.Path]::DirectorySeparatorChar

    if (-not $resolved.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "PDFium staging may write only beneath the repository target directory: $resolved"
    }
    return $resolved
}

function Read-PdfViewingManifest {
    param([Parameter(Mandatory = $true)][string]$Path)

    $resolved = [System.IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "PDF viewing manifest does not exist: $resolved"
    }

    $manifest = Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json
    foreach ($name in @(
        'schema_version',
        'base_revision',
        'byte_unit',
        'bytes_per_mib',
        'package_delta_limit_bytes',
        'installed_delta_limit_bytes',
        'control_build',
        'pdfium',
        'targets'
    )) {
        Assert-RequiredProperty -Value $manifest -Name $name
    }

    if ($manifest.schema_version -ne 1) { throw "Unsupported PDF viewing manifest schema." }
    if ($manifest.base_revision -cnotmatch '^[0-9a-f]{40}$') { throw "base_revision must be a full lowercase Git object ID." }
    if ($manifest.byte_unit -ne 'binary' -or $manifest.bytes_per_mib -ne 1048576) {
        throw "PDF size budgets must use binary MiB (1,048,576 bytes)."
    }
    if ($manifest.package_delta_limit_bytes -ne (6 * $manifest.bytes_per_mib)) {
        throw "The package delta limit must remain exactly 6 MiB for this change."
    }
    if ($manifest.installed_delta_limit_bytes -ne (10 * $manifest.bytes_per_mib)) {
        throw "The installed delta limit must remain exactly 10 MiB for this change."
    }
    if ($manifest.control_build.cargo_packager_version -ne '0.11.8') {
        throw "The control and candidate must use cargo-packager 0.11.8."
    }
    if ($manifest.pdfium.build -ne 7881 -or
        $manifest.pdfium.binding_version -ne '0.9.3' -or
        $manifest.pdfium.binding_feature -ne 'pdfium_7881') {
        throw "PDFium build 7881 and pdfium-render 0.9.3/pdfium_7881 must move together."
    }

    $targets = @($manifest.targets)
    $requiredTargets = @(
        'x86_64-pc-windows-msvc',
        'aarch64-apple-darwin',
        'x86_64-unknown-linux-gnu'
    )
    if ($targets.Count -ne $requiredTargets.Count) {
        throw "The PDF viewing manifest must define exactly the three supported native targets."
    }
    if (@($targets.triple | Sort-Object -Unique).Count -ne $targets.Count) {
        throw "The PDF viewing manifest contains duplicate target triples."
    }

    foreach ($target in $targets) {
        foreach ($name in @(
            'triple', 'archive', 'url', 'sha256', 'archive_size_bytes',
            'archive_member', 'installed_library', 'runtime_size_bytes', 'package_formats'
        )) {
            Assert-RequiredProperty -Value $target -Name $name
        }
        if ($requiredTargets -notcontains $target.triple) { throw "Unsupported PDFium target: $($target.triple)" }
        if ($target.archive -match '(?i)v8|xfa' -or $target.url -match '(?i)v8|xfa') {
            throw "V8/XFA PDFium archives are forbidden: $($target.archive)"
        }
        if ($target.url -notlike 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/*') {
            throw "PDFium must come from the pinned chromium/7881 release."
        }
        if ($target.sha256 -cnotmatch '^[0-9a-f]{64}$') { throw "Invalid SHA-256 for $($target.triple)." }
        if ($target.archive_size_bytes -le 0 -or $target.runtime_size_bytes -le 0) {
            throw "Pinned archive and runtime sizes must be positive."
        }
        Assert-SafeRelativePath -Path $target.archive_member -Label 'archive_member'
        Assert-SafeRelativePath -Path $target.installed_library -Label 'installed_library'
        if ([System.IO.Path]::GetFileName($target.installed_library) -ne $target.installed_library) {
            throw "installed_library must be a file name, not a path."
        }
        if (@($target.package_formats).Count -eq 0) { throw "Each target must define at least one package format." }
    }

    return $manifest
}

function Get-HostTargetTriple {
    $rustc = & rustc -vV
    if ($LASTEXITCODE -ne 0) { throw "rustc -vV failed." }
    $hostLine = @($rustc | Where-Object { $_ -like 'host:*' })
    if ($hostLine.Count -ne 1) { throw "Unable to determine the Rust host target." }
    return $hostLine[0].Substring('host:'.Length).Trim()
}

function Get-PdfiumTarget {
    param(
        [Parameter(Mandatory = $true)]$Manifest,
        [Parameter(Mandatory = $true)][string]$Triple
    )

    $matches = @($Manifest.targets | Where-Object { $_.triple -ceq $Triple })
    if ($matches.Count -ne 1) { throw "No unique pinned PDFium runtime for target '$Triple'." }
    return $matches[0]
}

function Test-ArchiveIntegrity {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$Target
    )

    $file = Get-Item -LiteralPath $Path
    if ($file.Length -ne $Target.archive_size_bytes) {
        throw "PDFium archive size mismatch for $($Target.triple): expected $($Target.archive_size_bytes), got $($file.Length)."
    }
    $actualHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLowerInvariant()
    if ($actualHash -cne $Target.sha256) {
        throw "PDFium archive SHA-256 mismatch for $($Target.triple)."
    }
}

function Read-PinnedRuntimeFromArchive {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)]$Target
    )

    $fileStream = [System.IO.File]::OpenRead($Path)
    $gzipStream = [System.IO.Compression.GZipStream]::new(
        $fileStream,
        [System.IO.Compression.CompressionMode]::Decompress,
        $false
    )
    $reader = [System.Formats.Tar.TarReader]::new($gzipStream, $false)
    $runtimeBytes = $null
    $runtimeCount = 0

    try {
        while ($null -ne ($entry = $reader.GetNextEntry())) {
            $entryName = $entry.Name.Replace('\', '/')
            Assert-SafeRelativePath -Path $entryName -Label 'archive entry'

            $looksLikeRuntime = $entryName -match '(?i)(^|/)(lib)?pdfium\.(dll|dylib|so)$'
            if ($looksLikeRuntime -and $entryName -cne $Target.archive_member) {
                throw "PDFium archive contains an unexpected runtime member: $entryName"
            }
            if ($entryName -cne $Target.archive_member) { continue }
            if ($null -eq $entry.DataStream) { throw "Pinned PDFium runtime member is not a regular file." }

            $runtimeCount += 1
            if ($runtimeCount -gt 1) { throw "Pinned PDFium runtime member appears more than once." }
            $memory = [System.IO.MemoryStream]::new()
            try {
                $entry.DataStream.CopyTo($memory)
                $runtimeBytes = $memory.ToArray()
            }
            finally {
                $memory.Dispose()
            }
        }
    }
    finally {
        $reader.Dispose()
        $gzipStream.Dispose()
        $fileStream.Dispose()
    }

    if ($runtimeCount -ne 1 -or $null -eq $runtimeBytes) {
        throw "Pinned PDFium runtime member was not found: $($Target.archive_member)"
    }
    if ($runtimeBytes.LongLength -ne $Target.runtime_size_bytes) {
        throw "PDFium runtime size mismatch for $($Target.triple): expected $($Target.runtime_size_bytes), got $($runtimeBytes.LongLength)."
    }
    return ,$runtimeBytes
}

$manifest = Read-PdfViewingManifest -Path $ManifestPath
if ($ValidateManifestOnly) {
    [ordered]@{
        schema_version = $manifest.schema_version
        base_revision = $manifest.base_revision
        target_count = @($manifest.targets).Count
        package_delta_limit_bytes = $manifest.package_delta_limit_bytes
        installed_delta_limit_bytes = $manifest.installed_delta_limit_bytes
    } | ConvertTo-Json -Compress
    exit 0
}

if ([string]::IsNullOrWhiteSpace($TargetTriple)) {
    $TargetTriple = Get-HostTargetTriple
}
$target = Get-PdfiumTarget -Manifest $manifest -Triple $TargetTriple
$resolvedOutputRoot = Assert-PathUnderTarget -Path $OutputRoot
$archiveDirectory = Join-Path $resolvedOutputRoot 'archives'
New-Item -ItemType Directory -Force -Path $archiveDirectory | Out-Null

if ([string]::IsNullOrWhiteSpace($ArchivePath)) {
    $resolvedArchive = Join-Path $archiveDirectory $target.archive
    if (-not (Test-Path -LiteralPath $resolvedArchive -PathType Leaf)) {
        $partial = "$resolvedArchive.partial"
        Invoke-WebRequest -Uri $target.url -OutFile $partial
        Move-Item -LiteralPath $partial -Destination $resolvedArchive -Force
    }
}
else {
    $resolvedArchive = [System.IO.Path]::GetFullPath($ArchivePath)
    if (-not (Test-Path -LiteralPath $resolvedArchive -PathType Leaf)) {
        throw "Provided PDFium archive does not exist: $resolvedArchive"
    }
}

Test-ArchiveIntegrity -Path $resolvedArchive -Target $target
$runtimeBytes = Read-PinnedRuntimeFromArchive -Path $resolvedArchive -Target $target
$stageDirectory = Join-Path $resolvedOutputRoot $target.triple
New-Item -ItemType Directory -Force -Path $stageDirectory | Out-Null
$destination = Join-Path $stageDirectory $target.installed_library
$temporary = Join-Path $stageDirectory ("." + $target.installed_library + "." + [guid]::NewGuid().ToString('N') + '.tmp')
[System.IO.File]::WriteAllBytes($temporary, $runtimeBytes)
Move-Item -LiteralPath $temporary -Destination $destination -Force

$stageRecord = [ordered]@{
    schema_version = 1
    target = $target.triple
    pdfium_build = $manifest.pdfium.build
    archive = $target.archive
    archive_sha256 = $target.sha256
    runtime_path = [System.IO.Path]::GetFullPath($destination)
    runtime_size_bytes = (Get-Item -LiteralPath $destination).Length
}
$recordPath = Join-Path $stageDirectory 'stage.json'
[System.IO.File]::WriteAllText(
    $recordPath,
    (($stageRecord | ConvertTo-Json -Depth 4) + [Environment]::NewLine),
    [System.Text.UTF8Encoding]::new($false)
)
$stageRecord | ConvertTo-Json -Depth 4 -Compress

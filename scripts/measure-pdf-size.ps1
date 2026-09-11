param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('nsis', 'dmg', 'deb', 'appimage')]
    [string]$Format,
    [Parameter(Mandatory = $true)][string]$TargetTriple,
    [Parameter(Mandatory = $true)][string]$ControlArtifact,
    [Parameter(Mandatory = $true)][string]$CandidateArtifact,
    [Parameter(Mandatory = $true)][string]$ControlPayloadRoot,
    [Parameter(Mandatory = $true)][string]$CandidatePayloadRoot,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [string]$ManifestPath = (Join-Path $PSScriptRoot "..\config\pdf-viewing.json")
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Resolve-RequiredFile {
    param([Parameter(Mandatory = $true)][string]$Path, [Parameter(Mandatory = $true)][string]$Label)
    $resolved = [System.IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) { throw "$Label is unavailable: $resolved" }
    return Get-Item -LiteralPath $resolved
}

function Resolve-RequiredDirectory {
    param([Parameter(Mandatory = $true)][string]$Path, [Parameter(Mandatory = $true)][string]$Label)
    $resolved = [System.IO.Path]::GetFullPath($Path)
    if (-not (Test-Path -LiteralPath $resolved -PathType Container)) { throw "$Label is unavailable: $resolved" }
    return Get-Item -LiteralPath $resolved
}

function Get-PayloadIndex {
    param([Parameter(Mandatory = $true)][System.IO.DirectoryInfo]$Root)

    $files = @(Get-ChildItem -LiteralPath $Root.FullName -File -Recurse)
    if ($files.Count -eq 0) { throw "Payload tree has no measurable files: $($Root.FullName)" }
    $index = @{}
    [long]$total = 0
    foreach ($file in $files) {
        $relative = [System.IO.Path]::GetRelativePath($Root.FullName, $file.FullName).Replace('\', '/')
        if ($index.ContainsKey($relative)) { throw "Duplicate relative payload path: $relative" }
        [long]$length = $file.Length
        $index[$relative] = $length
        $total += $length
    }
    return [pscustomobject]@{ Files = $index; Total = $total; Count = $files.Count }
}

function Get-CommandVersion {
    param([Parameter(Mandatory = $true)][scriptblock]$Command)
    try {
        $text = (& $Command 2>$null | Select-Object -First 1)
        if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($text)) { return 'unavailable' }
        return [string]$text
    }
    catch { return 'unavailable' }
}

$manifestFile = Resolve-RequiredFile -Path $ManifestPath -Label 'PDF viewing manifest'
$manifest = Get-Content -Raw -LiteralPath $manifestFile.FullName | ConvertFrom-Json
if ($manifest.schema_version -ne 1 -or
    $manifest.bytes_per_mib -ne 1048576 -or
    $manifest.package_delta_limit_bytes -ne 6291456 -or
    $manifest.installed_delta_limit_bytes -ne 10485760) {
    throw 'PDF viewing size budgets do not match the approved OpenSpec change.'
}

$targetMatches = @($manifest.targets | Where-Object { $_.triple -ceq $TargetTriple })
if ($targetMatches.Count -ne 1) { throw "Target is not present exactly once in the PDF viewing manifest: $TargetTriple" }
if (@($targetMatches[0].package_formats) -notcontains $Format) {
    throw "Package format '$Format' is not approved for target '$TargetTriple'."
}

$controlPackage = Resolve-RequiredFile -Path $ControlArtifact -Label 'Control package artifact'
$candidatePackage = Resolve-RequiredFile -Path $CandidateArtifact -Label 'Candidate package artifact'
$controlPayload = Resolve-RequiredDirectory -Path $ControlPayloadRoot -Label 'Control payload'
$candidatePayload = Resolve-RequiredDirectory -Path $CandidatePayloadRoot -Label 'Candidate payload'
$controlIndex = Get-PayloadIndex -Root $controlPayload
$candidateIndex = Get-PayloadIndex -Root $candidatePayload

$paths = @($controlIndex.Files.Keys + $candidateIndex.Files.Keys | Sort-Object -Unique)
$changes = foreach ($path in $paths) {
    [long]$controlBytes = if ($controlIndex.Files.ContainsKey($path)) { $controlIndex.Files[$path] } else { 0 }
    [long]$candidateBytes = if ($candidateIndex.Files.ContainsKey($path)) { $candidateIndex.Files[$path] } else { 0 }
    if ($controlBytes -ne $candidateBytes) {
        [ordered]@{
            path = $path
            control_bytes = $controlBytes
            candidate_bytes = $candidateBytes
            delta_bytes = $candidateBytes - $controlBytes
        }
    }
}
$largestChanges = @($changes | Sort-Object { [Math]::Abs([long]$_.delta_bytes) } -Descending | Select-Object -First 25)

[long]$packageDelta = $candidatePackage.Length - $controlPackage.Length
[long]$payloadDelta = $candidateIndex.Total - $controlIndex.Total
$packagePassed = $packageDelta -le [long]$manifest.package_delta_limit_bytes
$payloadPassed = $payloadDelta -le [long]$manifest.installed_delta_limit_bytes
$passed = $packagePassed -and $payloadPassed

$report = [ordered]@{
    schema_version = 1
    base_revision = $manifest.base_revision
    target = $TargetTriple
    format = $Format
    byte_unit = 'bytes'
    tool_versions = [ordered]@{
        rustc = Get-CommandVersion { rustc --version }
        cargo = Get-CommandVersion { cargo --version }
        cargo_packager = Get-CommandVersion { cargo packager --version }
        powershell = $PSVersionTable.PSVersion.ToString()
    }
    package = [ordered]@{
        control_path = $controlPackage.FullName
        candidate_path = $candidatePackage.FullName
        control_bytes = [long]$controlPackage.Length
        candidate_bytes = [long]$candidatePackage.Length
        delta_bytes = $packageDelta
        limit_bytes = [long]$manifest.package_delta_limit_bytes
        passed = $packagePassed
    }
    installed_payload = [ordered]@{
        control_root = $controlPayload.FullName
        candidate_root = $candidatePayload.FullName
        control_files = $controlIndex.Count
        candidate_files = $candidateIndex.Count
        control_bytes = [long]$controlIndex.Total
        candidate_bytes = [long]$candidateIndex.Total
        delta_bytes = $payloadDelta
        limit_bytes = [long]$manifest.installed_delta_limit_bytes
        passed = $payloadPassed
    }
    largest_changed_files = $largestChanges
    passed = $passed
}

$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$outputDirectory = Split-Path -Parent $resolvedOutput
if ([string]::IsNullOrWhiteSpace($outputDirectory)) { throw 'OutputPath must include a parent directory.' }
New-Item -ItemType Directory -Force -Path $outputDirectory | Out-Null
[System.IO.File]::WriteAllText(
    $resolvedOutput,
    (($report | ConvertTo-Json -Depth 8) + [Environment]::NewLine),
    [System.Text.UTF8Encoding]::new($false)
)

$report | ConvertTo-Json -Depth 8
if (-not $passed) {
    throw "PDF size budget exceeded for $TargetTriple/$Format; report retained at $resolvedOutput"
}

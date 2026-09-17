param(
    [Parameter(Mandatory = $true)][string]$WorkerPath,
    [Parameter(Mandatory = $true)][string]$HostPath,
    [Parameter(Mandatory = $true)][string]$ScratchRoot,
    [Parameter(Mandatory = $true)][string]$OutputPath
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

$worker = Resolve-RequiredFile -Path $WorkerPath -Label 'Extracted plugin worker'
$hostExecutable = Resolve-RequiredFile -Path $HostPath -Label 'Fixture lifecycle host'
$scratch = [System.IO.Path]::GetFullPath($ScratchRoot)
if (Test-Path -LiteralPath $scratch) {
    if (@(Get-ChildItem -LiteralPath $scratch -Force).Count -ne 0) {
        throw "Scratch directory must be absent or empty: $scratch"
    }
}
else { New-Item -ItemType Directory -Path $scratch -Force | Out-Null }

$probeLine = & $worker.FullName --probe
if ($LASTEXITCODE -ne 0) { throw "Direct plugin worker launch failed with exit code $LASTEXITCODE." }
$probe = $probeLine | ConvertFrom-Json
if ($probe.plugin_id -ne 'dev.markion.fixture' -or $probe.protocol -ne '1.0') {
    throw 'Direct plugin worker probe returned an unexpected identity.'
}
$lifecycleLine = & $hostExecutable.FullName $worker.FullName (Join-Path $scratch 'lifecycle')
if ($LASTEXITCODE -ne 0) { throw "Plugin lifecycle verification failed with exit code $LASTEXITCODE." }
$lifecycle = $lifecycleLine | ConvertFrom-Json
if (-not $lifecycle.handshake -or -not $lifecycle.forced_parent_cleanup) {
    throw 'Plugin lifecycle verification did not cover handshake and forced-parent cleanup.'
}

$policy = [ordered]@{ kind = 'none'; assessed = $false; allowed = $null; metadata_preserved = $null; detail = '' }
if ($IsMacOS) {
    $policy.kind = 'gatekeeper-quarantine'
    $policyCopy = Join-Path $scratch $worker.Name
    Copy-Item -LiteralPath $worker.FullName -Destination $policyCopy
    & xattr -w com.apple.quarantine '0081;MarkionPluginSpike;Codex;' $policyCopy
    if ($LASTEXITCODE -ne 0) { throw 'Unable to attach quarantine metadata for the Gatekeeper spike.' }
    & spctl --assess --type execute --verbose=4 $policyCopy 2>&1 | Out-String | ForEach-Object { $policy.detail = $_.Trim() }
    $policy.allowed = $LASTEXITCODE -eq 0
    $policy.assessed = $true
    & xattr -p com.apple.quarantine $policyCopy *> $null
    $policy.metadata_preserved = $LASTEXITCODE -eq 0
}
elseif ($IsWindows) {
    $policy.kind = 'windows-motw-direct-spawn'
    $policyCopy = Join-Path $scratch $worker.Name
    Copy-Item -LiteralPath $worker.FullName -Destination $policyCopy
    $zonePath = "$policyCopy`:Zone.Identifier"
    Set-Content -Path $zonePath -Value "[ZoneTransfer]`r`nZoneId=3`r`n" -NoNewline
    $policy.assessed = $true
    try {
        $policyProbe = & $policyCopy --probe 2>&1
        $policy.allowed = $LASTEXITCODE -eq 0
        $policy.detail = ($policyProbe | Out-String).Trim()
    }
    catch {
        $policy.allowed = $false
        $policy.detail = $_.Exception.GetType().Name
    }
    $policy.metadata_preserved = @(Get-Item -LiteralPath $policyCopy -Stream Zone.Identifier -ErrorAction SilentlyContinue).Count -eq 1
}
elseif ($IsLinux) {
    $policy.kind = 'linux-executable-mode'
    $modeText = (& stat -c '%a' $worker.FullName).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Unable to inspect the extracted Linux executable mode.' }
    $policy.assessed = $true
    $policy.allowed = $modeText -eq '755'
    $policy.metadata_preserved = $true
    $policy.detail = $modeText
    if (-not $policy.allowed) { throw "Extracted worker mode is $modeText instead of 755." }
}

$report = [ordered]@{
    schema_version = 1
    os = if ($IsWindows) { 'windows' } elseif ($IsMacOS) { 'macos' } elseif ($IsLinux) { 'linux' } else { 'unknown' }
    arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    direct_launch = $true
    handshake = [bool]$lifecycle.handshake
    forced_parent_cleanup = [bool]$lifecycle.forced_parent_cleanup
    policy = $policy
}
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputPath)
$parent = Split-Path -Parent $resolvedOutput
New-Item -ItemType Directory -Path $parent -Force | Out-Null
[System.IO.File]::WriteAllText(
    $resolvedOutput,
    (($report | ConvertTo-Json -Depth 6) + [Environment]::NewLine),
    [System.Text.UTF8Encoding]::new($false)
)
$report | ConvertTo-Json -Depth 6

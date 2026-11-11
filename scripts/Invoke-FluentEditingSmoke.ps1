param(
    [switch]$SkipBuild,
    [ValidateSet('release', 'debug')][string]$BuildProfile = 'release',
    [ValidateRange(30, 600)][int]$TimeoutSeconds = 180
)

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) { throw 'Run this script with PowerShell 7 (pwsh).' }
$repoRoot = Split-Path -Parent $PSScriptRoot
$fixtures = Join-Path $repoRoot 'target/tmp/fluent-editing-fixtures'
New-Item -ItemType Directory -Path $fixtures -Force | Out-Null
Push-Location $repoRoot
try {
    if (!$SkipBuild) {
        if ($BuildProfile -eq 'release') { cargo build --release --bin markion }
        else { cargo build --bin markion }
        if ($LASTEXITCODE -ne 0) { throw "$BuildProfile build failed." }
    }
    $binary = Join-Path $repoRoot "target/$BuildProfile/markion.exe"
    if (!(Test-Path -LiteralPath $binary)) { throw "Build the $BuildProfile binary first." }
    if (@(100, 300, 600, 1024 | Where-Object { !(Test-Path -LiteralPath (Join-Path $fixtures "mixed-$($_)kb.md")) }).Count) {
        $env:BENCH_FIXTURE_DIR = $fixtures
        $env:BENCH_SIZES_KB = '100,300,600,1024'
        $env:BENCH_STROKES = '1'
        $env:BENCH_VISUAL_ONLY = '1'
        cargo run --release --example bench_large_doc
        if ($LASTEXITCODE -ne 0) { throw 'Fixture generation failed.' }
    }
    $runDir = Join-Path $fixtures ('native-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    $roaming = Join-Path $runDir 'roaming'
    $local = Join-Path $runDir 'local'
    $configDir = Join-Path $roaming 'Markion'
    New-Item -ItemType Directory -Path $runDir, $configDir, $local -Force | Out-Null
    $testBinary = Join-Path $runDir 'markion.exe'
    Copy-Item -LiteralPath $binary -Destination $testBinary
    Set-Content -LiteralPath (Join-Path $configDir 'config.toml') -Encoding utf8 -Value @'
check_for_updates_on_startup = false
[auto_save]
enabled = false
'@
    $smoke = Get-Content -LiteralPath (Join-Path $repoRoot 'docs/fixtures/visual-p3-smoke.md') -Raw
    $image = (Join-Path $repoRoot 'assets/markion.png').Replace('\', '/')
    $smoke = $smoke.Replace('../../assets/markion.png', $image)
    Set-Content -LiteralPath (Join-Path $fixtures 'native-p3-smoke.md') -Value $smoke -NoNewline -Encoding utf8
    $started = [DateTime]::UtcNow
    $process = Start-Process -FilePath $testBinary -WorkingDirectory $repoRoot -WindowStyle Normal -PassThru -RedirectStandardOutput (Join-Path $runDir 'stdout.log') -RedirectStandardError (Join-Path $runDir 'stderr.log') -Environment @{
        APPDATA = $roaming
        LOCALAPPDATA = $local
        MARKION_NATIVE_SMOKE_DIR = $fixtures
        MARKION_PROFILE_VISUAL_INPUT = '1'
        RUST_LOG = 'info,markion::performance=info'
    }
    $replayTimer = [System.Diagnostics.Stopwatch]::StartNew()
    $peakResident = 0L
    $peakPrivate = 0L
    while ($true) {
        try {
            $process.Refresh()
            $peakResident = [Math]::Max($peakResident, $process.PeakWorkingSet64)
            $peakPrivate = [Math]::Max($peakPrivate, $process.PrivateMemorySize64)
        } catch {
            # A process can exit between refreshing and reading its counters.
        }
        if ($process.WaitForExit(100)) { break }
        if ($replayTimer.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
            Stop-Process -Id $process.Id -Force
            throw "Native replay timed out after $TimeoutSeconds seconds."
        }
    }
    $reportPath = Join-Path $fixtures 'native-smoke.json'
    if (!(Test-Path -LiteralPath $reportPath) -or (Get-Item -LiteralPath $reportPath).LastWriteTimeUtc -lt $started) {
        throw "Native application exited with code $($process.ExitCode) without a current report. Inspect $runDir/stderr.log and $local/Markion/Logs."
    }
    $report = Get-Content -LiteralPath $reportPath -Raw | ConvertFrom-Json
    $report | Add-Member -NotePropertyName build_profile -NotePropertyValue $BuildProfile
    $report | Add-Member -NotePropertyName process_peaks -NotePropertyValue @{
        resident_bytes = $peakResident
        private_sampled_bytes = $peakPrivate
        sample_interval_ms = 100
    }
    $report | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $reportPath -Encoding utf8
    Copy-Item -LiteralPath $reportPath -Destination (Join-Path $runDir 'native-smoke.json')
    $report.observations | Where-Object fixture_kb | ForEach-Object {
        $inputTimes = @($_.paint_samples | Where-Object kind -eq input | ForEach-Object elapsed_us | Sort-Object)
        $opening = @($_.paint_samples | Where-Object kind -eq open | ForEach-Object elapsed_us)
        [PSCustomObject]@{
            KB = $_.fixture_kb
            OpenMs = if ($opening.Count) { [math]::Round($opening[0] / 1000, 2) } else { $null }
            InputP50Ms = if ($inputTimes.Count) { [math]::Round($inputTimes[[int][math]::Floor($inputTimes.Count / 2)] / 1000, 2) } else { $null }
            InputP95Ms = if ($inputTimes.Count) { [math]::Round($inputTimes[[int][math]::Floor($inputTimes.Count * 0.95)] / 1000, 2) } else { $null }
            ResidentMiB = [math]::Round($_.resident_bytes / 1MB, 2)
            CommitMiB = [math]::Round($_.commit_bytes / 1MB, 2)
        }
    } | Format-Table -AutoSize | Out-String | Write-Output
    Write-Output "Native report: $reportPath"
    Write-Output ('Native peak resident: {0:N2} MiB; sampled peak private: {1:N2} MiB.' -f ($peakResident / 1MB), ($peakPrivate / 1MB))
    Write-Output "Isolated profile and logs: $runDir"
    if (!$report.passed) { throw ('Native replay failed: ' + $report.error) }
} finally {
    Pop-Location
}

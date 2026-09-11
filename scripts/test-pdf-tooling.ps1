$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$targetRoot = [System.IO.Path]::GetFullPath((Join-Path $repoRoot 'target'))
$testRoot = Join-Path $targetRoot ("pdf-tool-tests\" + [guid]::NewGuid().ToString('N'))
$targetPrefix = $targetRoot.TrimEnd(
    [System.IO.Path]::DirectorySeparatorChar,
    [System.IO.Path]::AltDirectorySeparatorChar
) + [System.IO.Path]::DirectorySeparatorChar
if (-not ([System.IO.Path]::GetFullPath($testRoot)).StartsWith($targetPrefix, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to create PDF tooling tests outside target: $testRoot"
}
New-Item -ItemType Directory -Path $testRoot | Out-Null

$stageScript = Join-Path $PSScriptRoot 'stage-pdfium.ps1'
$measureScript = Join-Path $PSScriptRoot 'measure-pdf-size.ps1'
$manifestPath = Join-Path $repoRoot 'config\pdf-viewing.json'
$pwsh = (Get-Command pwsh -ErrorAction Stop).Source

function Invoke-SeparatePowerShell {
    param([Parameter(Mandatory = $true)][string[]]$Arguments)
    $output = & $pwsh -NoLogo -NoProfile @Arguments 2>&1
    return [pscustomobject]@{ ExitCode = $LASTEXITCODE; Output = @($output) -join [Environment]::NewLine }
}

try {
    $valid = Invoke-SeparatePowerShell -Arguments @('-File', $stageScript, '-ManifestPath', $manifestPath, '-ValidateManifestOnly')
    if ($valid.ExitCode -ne 0) { throw "Valid PDF manifest was rejected: $($valid.Output)" }
    $summary = $valid.Output | ConvertFrom-Json
    if ($summary.target_count -ne 3 -or $summary.package_delta_limit_bytes -ne 6291456) {
        throw 'Validated PDF manifest summary is incomplete.'
    }

    $invalidPath = Join-Path $testRoot 'invalid-manifest.json'
    $invalid = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    $invalid.targets[0].archive = 'pdfium-v8-win-x64.tgz'
    [System.IO.File]::WriteAllText(
        $invalidPath,
        (($invalid | ConvertTo-Json -Depth 10) + [Environment]::NewLine),
        [System.Text.UTF8Encoding]::new($false)
    )
    $rejected = Invoke-SeparatePowerShell -Arguments @('-File', $stageScript, '-ManifestPath', $invalidPath, '-ValidateManifestOnly')
    if ($rejected.ExitCode -eq 0) { throw 'Manifest validation accepted a forbidden V8 archive.' }

    $badAppImageSizePath = Join-Path $testRoot 'bad-appimage-size-manifest.json'
    $badAppImageSize = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    $badAppImageSize.targets[2].appimage_runtime_size_bytes = $badAppImageSize.targets[2].runtime_size_bytes
    [System.IO.File]::WriteAllText(
        $badAppImageSizePath,
        (($badAppImageSize | ConvertTo-Json -Depth 10) + [Environment]::NewLine),
        [System.Text.UTF8Encoding]::new($false)
    )
    $appImageSizeRejected = Invoke-SeparatePowerShell -Arguments @('-File', $stageScript, '-ManifestPath', $badAppImageSizePath, '-ValidateManifestOnly')
    if ($appImageSizeRejected.ExitCode -eq 0) { throw 'Manifest validation accepted an invalid AppImage runtime size.' }

    $badArchivePath = Join-Path $testRoot 'bad-pdfium.tgz'
    [System.IO.File]::WriteAllBytes($badArchivePath, [byte[]](1..20))
    $badDigestManifestPath = Join-Path $testRoot 'bad-digest-manifest.json'
    $badDigestManifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json
    $badDigestManifest.targets[0].archive_size_bytes = 20
    [System.IO.File]::WriteAllText(
        $badDigestManifestPath,
        (($badDigestManifest | ConvertTo-Json -Depth 10) + [Environment]::NewLine),
        [System.Text.UTF8Encoding]::new($false)
    )
    $badDigestStage = Join-Path $testRoot 'bad-digest-stage'
    $digestMismatch = Invoke-SeparatePowerShell -Arguments @(
        '-File', $stageScript,
        '-TargetTriple', 'x86_64-pc-windows-msvc',
        '-ManifestPath', $badDigestManifestPath,
        '-ArchivePath', $badArchivePath,
        '-OutputRoot', $badDigestStage
    )
    if ($digestMismatch.ExitCode -eq 0) { throw 'PDFium staging accepted an archive with the wrong SHA-256 digest.' }
    if (Test-Path -LiteralPath (Join-Path $badDigestStage 'pdfium.dll') -PathType Leaf) {
        throw 'PDFium staging retained a runtime from a digest-mismatched archive.'
    }

    $controlArtifact = Join-Path $testRoot 'control.exe'
    $candidateArtifact = Join-Path $testRoot 'candidate.exe'
    [System.IO.File]::WriteAllBytes($controlArtifact, [byte[]](1..10))
    [System.IO.File]::WriteAllBytes($candidateArtifact, [byte[]](1..20))
    $controlPayload = Join-Path $testRoot 'control-payload'
    $candidatePayload = Join-Path $testRoot 'candidate-payload'
    New-Item -ItemType Directory -Path $controlPayload, $candidatePayload | Out-Null
    [System.IO.File]::WriteAllBytes((Join-Path $controlPayload 'markion.exe'), [byte[]](1..100))
    [System.IO.File]::WriteAllBytes((Join-Path $candidatePayload 'markion.exe'), [byte[]](1..150))
    [System.IO.File]::WriteAllBytes((Join-Path $candidatePayload 'pdfium.dll'), [byte[]](1..50))
    $reportPath = Join-Path $testRoot 'passing-report.json'

    $passing = Invoke-SeparatePowerShell -Arguments @(
        '-File', $measureScript,
        '-Format', 'nsis',
        '-TargetTriple', 'x86_64-pc-windows-msvc',
        '-ControlArtifact', $controlArtifact,
        '-CandidateArtifact', $candidateArtifact,
        '-ControlPayloadRoot', $controlPayload,
        '-CandidatePayloadRoot', $candidatePayload,
        '-OutputPath', $reportPath,
        '-ManifestPath', $manifestPath
    )
    if ($passing.ExitCode -ne 0) { throw "Passing size comparison failed: $($passing.Output)" }
    $report = Get-Content -Raw -LiteralPath $reportPath | ConvertFrom-Json
    if (-not $report.passed -or $report.package.delta_bytes -ne 10 -or $report.installed_payload.delta_bytes -ne 100) {
        throw 'Size comparison report contains incorrect byte deltas.'
    }
    if (@($report.largest_changed_files | Where-Object { $_.path -eq 'pdfium.dll' }).Count -ne 1) {
        throw 'Size comparison report did not identify the added PDFium runtime.'
    }

    $missing = Invoke-SeparatePowerShell -Arguments @(
        '-File', $measureScript,
        '-Format', 'nsis',
        '-TargetTriple', 'x86_64-pc-windows-msvc',
        '-ControlArtifact', (Join-Path $testRoot 'missing.exe'),
        '-CandidateArtifact', $candidateArtifact,
        '-ControlPayloadRoot', $controlPayload,
        '-CandidatePayloadRoot', $candidatePayload,
        '-OutputPath', (Join-Path $testRoot 'missing-report.json'),
        '-ManifestPath', $manifestPath
    )
    if ($missing.ExitCode -eq 0) { throw 'Size comparison accepted a missing control artifact.' }

    $tooLargeArtifact = Join-Path $testRoot 'too-large.exe'
    $stream = [System.IO.File]::Create($tooLargeArtifact)
    try { $stream.SetLength(6291467) }
    finally { $stream.Dispose() }
    $failureReport = Join-Path $testRoot 'failure-report.json'
    $overBudget = Invoke-SeparatePowerShell -Arguments @(
        '-File', $measureScript,
        '-Format', 'nsis',
        '-TargetTriple', 'x86_64-pc-windows-msvc',
        '-ControlArtifact', $controlArtifact,
        '-CandidateArtifact', $tooLargeArtifact,
        '-ControlPayloadRoot', $controlPayload,
        '-CandidatePayloadRoot', $candidatePayload,
        '-OutputPath', $failureReport,
        '-ManifestPath', $manifestPath
    )
    if ($overBudget.ExitCode -eq 0) { throw 'Size comparison accepted an over-budget package delta.' }
    if (-not (Test-Path -LiteralPath $failureReport -PathType Leaf)) {
        throw 'Over-budget comparison did not retain its JSON report.'
    }

    Write-Host 'PDF runtime manifest and size measurement tests passed.'
}
finally {
    $resolvedTestRoot = [System.IO.Path]::GetFullPath($testRoot)
    if ($resolvedTestRoot.StartsWith($targetPrefix, [System.StringComparison]::OrdinalIgnoreCase) -and
        (Split-Path -Leaf $resolvedTestRoot).Length -eq 32) {
        Remove-Item -LiteralPath $resolvedTestRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
}

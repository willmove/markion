param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('nsis', 'dmg', 'deb', 'appimage')]
    [string]$Format,
    [Parameter(Mandatory = $true)][string]$ArtifactsRoot,
    [Parameter(Mandatory = $true)][string]$OutputRoot
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Find-UniqueArtifact {
    param([Parameter(Mandatory = $true)][string]$Pattern)
    $matches = @(Get-ChildItem -LiteralPath $resolvedArtifacts -Filter $Pattern -File -Recurse)
    if ($matches.Count -ne 1) {
        throw "Expected exactly one '$Pattern' artifact for $Format, found $($matches.Count)."
    }
    return $matches[0]
}

function Invoke-Native {
    param([Parameter(Mandatory = $true)][string]$Label, [Parameter(Mandatory = $true)][scriptblock]$Command)
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "$Label failed with exit code $LASTEXITCODE." }
}

$resolvedArtifacts = [System.IO.Path]::GetFullPath($ArtifactsRoot)
if (-not (Test-Path -LiteralPath $resolvedArtifacts -PathType Container)) {
    throw "Artifact directory is unavailable: $resolvedArtifacts"
}
$resolvedOutput = [System.IO.Path]::GetFullPath($OutputRoot)
if (Test-Path -LiteralPath $resolvedOutput) {
    $existing = @(Get-ChildItem -LiteralPath $resolvedOutput -Force)
    if ($existing.Count -ne 0) { throw "Payload output directory must be empty: $resolvedOutput" }
}
else {
    New-Item -ItemType Directory -Path $resolvedOutput | Out-Null
}

$mountedDmg = $null
try {
    switch ($Format) {
        'nsis' {
            $artifact = Find-UniqueArtifact -Pattern '*-setup.exe'
            $payloadRoot = Join-Path $resolvedOutput 'installed'
            New-Item -ItemType Directory -Path $payloadRoot | Out-Null
            Invoke-Native -Label 'NSIS payload extraction' -Command { 7z x -y "-o$payloadRoot" $artifact.FullName }
        }
        'dmg' {
            $artifact = Find-UniqueArtifact -Pattern '*.dmg'
            $mountedDmg = Join-Path $resolvedOutput 'mounted'
            New-Item -ItemType Directory -Path $mountedDmg | Out-Null
            Invoke-Native -Label 'DMG mount' -Command { hdiutil attach -readonly -nobrowse -mountpoint $mountedDmg $artifact.FullName }
            $apps = @(Get-ChildItem -LiteralPath $mountedDmg -Filter '*.app' -Directory)
            if ($apps.Count -ne 1) { throw "Expected exactly one app bundle in the DMG, found $($apps.Count)." }
            $payloadParent = Join-Path $resolvedOutput 'installed'
            New-Item -ItemType Directory -Path $payloadParent | Out-Null
            Copy-Item -LiteralPath $apps[0].FullName -Destination $payloadParent -Recurse
            $payloadRoot = Join-Path $payloadParent $apps[0].Name
        }
        'deb' {
            $artifact = Find-UniqueArtifact -Pattern '*.deb'
            $payloadRoot = Join-Path $resolvedOutput 'installed'
            New-Item -ItemType Directory -Path $payloadRoot | Out-Null
            Invoke-Native -Label 'DEB payload extraction' -Command { dpkg-deb -x $artifact.FullName $payloadRoot }
        }
        'appimage' {
            $artifact = Find-UniqueArtifact -Pattern '*.AppImage'
            Invoke-Native -Label 'AppImage executable permission' -Command { chmod +x $artifact.FullName }
            Push-Location $resolvedOutput
            try {
                Invoke-Native -Label 'AppImage payload extraction' -Command { & $artifact.FullName --appimage-extract }
            }
            finally {
                Pop-Location
            }
            $payloadRoot = Join-Path $resolvedOutput 'squashfs-root'
            if (-not (Test-Path -LiteralPath $payloadRoot -PathType Container)) {
                throw 'AppImage extraction did not produce squashfs-root.'
            }
        }
    }
}
finally {
    if ($null -ne $mountedDmg -and (Test-Path -LiteralPath $mountedDmg)) {
        & hdiutil detach $mountedDmg -quiet
    }
}

[ordered]@{
    schema_version = 1
    format = $Format
    artifact_path = $artifact.FullName
    payload_root = [System.IO.Path]::GetFullPath($payloadRoot)
} | ConvertTo-Json -Depth 4 -Compress

$ErrorActionPreference = "Stop"

function Invoke-Gate {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Command
    )

    Write-Host "==> $Name"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    Invoke-Gate "Rust formatting" { cargo fmt --all -- --check }
    Invoke-Gate "Cargo workspace tests" { cargo test --workspace }
    Invoke-Gate "Strict OpenSpec validation" {
        openspec validate --all --strict --no-interactive
    }
}
finally {
    Pop-Location
}

Write-Host "All repository quality gates passed."

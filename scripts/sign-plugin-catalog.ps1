param(
    [Parameter(Mandatory = $true)][string]$CatalogPath,
    [Parameter(Mandatory = $true)][string]$SecretKeyPath,
    [Parameter(Mandatory = $true)][string]$PublicKeyPath,
    [string]$SignaturePath = "$CatalogPath.minisig",
    [string]$CatalogToolPath = ''
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

$catalog = Resolve-RequiredFile -Path $CatalogPath -Label 'Plugin catalog'
$secretKey = Resolve-RequiredFile -Path $SecretKeyPath -Label 'Plugin signing secret key'
$publicKey = Resolve-RequiredFile -Path $PublicKeyPath -Label 'Plugin signing public key'
$signature = [System.IO.Path]::GetFullPath($SignaturePath)

if ([string]::IsNullOrWhiteSpace($CatalogToolPath)) {
    $suffix = if ($IsWindows) { '.exe' } else { '' }
    $CatalogToolPath = Join-Path $PWD "target/release/plugin-catalog-tool$suffix"
    Invoke-Native -Label 'Plugin catalog tool build' -Command {
        cargo build --release --locked -p markion-plugin-protocol --bin plugin-catalog-tool
    }
}
$catalogTool = Resolve-RequiredFile -Path $CatalogToolPath -Label 'Plugin catalog tool'
$temporary = "$($catalog.FullName).canonical-$PID-$([Guid]::NewGuid().ToString('N'))"

try {
    Invoke-Native -Label 'Plugin catalog canonicalization' -Command {
        & $catalogTool.FullName canonicalize $catalog.FullName $temporary
    }
    Move-Item -LiteralPath $temporary -Destination $catalog.FullName -Force
    Invoke-Native -Label 'Plugin catalog signing' -Command {
        minisign -S -s $secretKey.FullName -m $catalog.FullName -x $signature -t 'Markion official plugin catalog' -q
    }
    Invoke-Native -Label 'Plugin catalog signature verification' -Command {
        minisign -Vm $catalog.FullName -p $publicKey.FullName -x $signature -q
    }
    Invoke-Native -Label 'In-process plugin catalog signature verification' -Command {
        & $catalogTool.FullName verify $catalog.FullName $signature $publicKey.FullName
    }
}
finally {
    if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
}

[ordered]@{
    catalog_path = $catalog.FullName
    signature_path = $signature
    public_key_path = $publicKey.FullName
    catalog_bytes = (Get-Item -LiteralPath $catalog.FullName).Length
    signature_bytes = (Get-Item -LiteralPath $signature).Length
} | ConvertTo-Json -Compress

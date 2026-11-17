param(
    [string]$ExamplesRoot = (Join-Path (Split-Path -Parent $PSScriptRoot) "assets/examples"),
    [string]$PackagedRoot
)

$ErrorActionPreference = "Stop"
$examples = [System.IO.Path]::GetFullPath($ExamplesRoot)
if (-not (Test-Path -LiteralPath $examples -PathType Container)) {
    throw "Example collection not found: $examples"
}
$indexPath = Join-Path $examples "README.md"
if (-not (Test-Path -LiteralPath $indexPath -PathType Leaf)) {
    throw "Example index is missing: README.md"
}

$utf8 = [System.Text.UTF8Encoding]::new($false, $true)
$index = [System.IO.File]::ReadAllText($indexPath, $utf8)
$files = @(Get-ChildItem -LiteralPath $examples -File -Recurse)
$documents = @($files | Where-Object { $_.Extension -eq ".md" })
$templates = @($documents | Where-Object { $_.Name -match '^(1[0-9]|2[01])-' })
if ($documents.Count -lt 18 -or $templates.Count -lt 12) {
    throw "Example collection requires an index, five guides, and twelve templates"
}

function Assert-LocalReference([System.IO.FileInfo]$Document, [string]$Target) {
    $targetPath = $Target.Trim('<', '>')
    if ($targetPath -match '^[a-zA-Z][a-zA-Z0-9+.-]*:' -or $targetPath.StartsWith('//')) {
        return # Optional external references are not fetched by this offline check.
    }
    $targetPath = [System.Uri]::UnescapeDataString(($targetPath -split '[?#]', 2)[0])
    if (-not $targetPath) { return }
    $resolved = [System.IO.Path]::GetFullPath((Join-Path $Document.DirectoryName $targetPath))
    $relative = [System.IO.Path]::GetRelativePath($examples, $resolved)
    if ([System.IO.Path]::IsPathRooted($relative) -or $relative -match '^\.\.([/\\]|$)') {
        throw "Example reference escapes the portable collection: $($Document.Name) -> $Target"
    }
    if (-not (Test-Path -LiteralPath $resolved -PathType Leaf)) {
        throw "Broken example reference: $($Document.Name) -> $Target"
    }
}

# These patterns cover the ordinary inline/reference links and HTML images used
# by this collection. Ignore fenced and inline code because those teach syntax.
$inlineLink = @'
!?\[[^\]\r\n]*\]\(\s*(?<target><[^>]+>|[^\s)]+)(?:\s+"[^"]*")?\s*\)
'@
$referenceLink = '(?m)^\[(?!\^)[^\]\r\n]+\]:\s*(?<target>\S+)'
$htmlLink = @'
(?i)<(?:img|a)\b[^>]*?\b(?:src|href)\s*=\s*["'](?<target>[^"']+)["']
'@
foreach ($document in $documents) {
    $content = [System.IO.File]::ReadAllText($document.FullName, $utf8)
    if ($content.Length -lt 100 -or $content -notmatch '^#\s+\S') {
        throw "Example document is empty or lacks its title: $($document.Name)"
    }
    $relative = [System.IO.Path]::GetRelativePath($examples, $document.FullName).Replace('\', '/')
    if ($document.FullName -ne $indexPath -and -not $index.Contains("($relative)")) {
        throw "Example is not linked from README.md: $relative"
    }
    $prose = [regex]::Replace($content, '(?ms)^```[^\r\n]*\r?\n.*?^```\s*$', '')
    $prose = [regex]::Replace($prose, '`[^`\r\n]+`', '')
    foreach ($pattern in @($inlineLink, $referenceLink, $htmlLink)) {
        foreach ($link in [regex]::Matches($prose, $pattern)) {
            Assert-LocalReference $document $link.Groups['target'].Value
        }
    }
}

if ($PackagedRoot) {
    $packaged = [System.IO.Path]::GetFullPath($PackagedRoot)
    foreach ($file in $files) {
        $relative = [System.IO.Path]::GetRelativePath($examples, $file.FullName)
        $destination = Join-Path $packaged $relative
        if (-not (Test-Path -LiteralPath $destination -PathType Leaf)) {
            throw "Packaged example is missing: $relative"
        }
        if ((Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256).Hash -ne
            (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash) {
            throw "Packaged example differs from the source: $relative"
        }
    }
    Write-Host "Verified $($files.Count) packaged example files against $examples"
} else {
    Write-Host "Verified $($documents.Count) Markdown examples, index coverage, and local references in $examples"
}

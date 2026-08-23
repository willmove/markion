param(
    [string]$Source = "C:\Coding\EditorProjects\marknice",
    [switch]$RefreshThirdParty
)

$ErrorActionPreference = "Stop"
$expectedCommit = "c009c1ec7e7c92f89afa5a32edcb126b5296bda7"
$repoRoot = Split-Path -Parent $PSScriptRoot
$bundleRoot = Join-Path $repoRoot "assets\marknice-workspace"
$staticRoot = Join-Path $bundleRoot "static"
$vendorRoot = Join-Path $staticRoot "vendor"
$htmlDocxVersion = "0.3.1"
$mathjaxVersion = "3.2.2"

if ((git -C $Source rev-parse HEAD).Trim() -ne $expectedCommit) {
    throw "MarkNice source must be pinned at $expectedCommit"
}
if (-not (Test-Path -LiteralPath $bundleRoot -PathType Container)) {
    throw "Expected checked-in workspace shell at $bundleRoot"
}
New-Item -ItemType Directory -Force -Path $vendorRoot | Out-Null

function Get-UniqueSourceRegion {
    param(
        [Parameter(Mandatory = $true)][string]$Text,
        [Parameter(Mandatory = $true)][string]$StartMarker,
        [Parameter(Mandatory = $true)][string]$EndMarker
    )

    $start = $Text.IndexOf($StartMarker, [System.StringComparison]::Ordinal)
    if ($start -lt 0) { throw "MarkNice source marker not found: $StartMarker" }
    if ($Text.IndexOf($StartMarker, $start + $StartMarker.Length, [System.StringComparison]::Ordinal) -ge 0) {
        throw "MarkNice source marker is ambiguous: $StartMarker"
    }
    $end = $Text.IndexOf($EndMarker, $start + $StartMarker.Length, [System.StringComparison]::Ordinal)
    if ($end -lt 0) { throw "MarkNice source end marker not found: $EndMarker" }
    return $Text.Substring($start, $end - $start).TrimEnd()
}

function Replace-UniqueSourceText {
    param(
        [Parameter(Mandatory = $true)][string]$Text,
        [Parameter(Mandatory = $true)][string]$Search,
        [Parameter(Mandatory = $true)][AllowEmptyString()][string]$Replacement,
        [Parameter(Mandatory = $true)][string]$Description
    )

    $count = [regex]::Matches($Text, [regex]::Escape($Search)).Count
    if ($count -ne 1) {
        throw "Unexpected MarkNice $Description source shape: expected one match, found $count"
    }
    return $Text.Replace($Search, $Replacement)
}

$sourceText = [System.IO.File]::ReadAllText((Join-Path $Source "src\main.js"))
$runtime = "/* Generated from MarkNice $expectedCommit by scripts/sync-marknice-workspace.ps1. */`n" +
    (Get-UniqueSourceRegion -Text $sourceText -StartMarker "function tokenHeadingOptions(tagName) {" -EndMarker "function systemMode() {")
$runtime = [regex]::Replace(
    $runtime,
    "(?s)\s*statusEl\.textContent = md\.trim\(\)\s*\?.*?\s*:\s*'';",
    "`n  statusEl.textContent = md.trim() ? locale.opened + localImageStatusSuffix() : '';"
)
$runtime = $runtime.Replace(
    "    document.execCommand('copy');",
    "    if (!document.execCommand('copy')) throw new Error('clipboard denied');"
)
# MarkNice renders formulas with KaTeX (MathML + CSS-dependent HTML spans), which
# the WeChat editor strips into duplicated, broken output. Markion replaces the
# renderer with the curated static/math-runtime.js (MathJax tex-svg, matching the
# MarkNice Obsidian plugin) so formulas are self-contained inline SVG.
$runtime = [regex]::Replace(
    $runtime,
    "(?s)function renderMath\(container\) \{\r?\n  if \(typeof katex === 'undefined'\) return;\r?\n.*?\n\}",
    "function renderMath(container) {`n  if (window.MarkionMath && typeof MarkionMath.renderInto === 'function') MarkionMath.renderInto(container);`n}"
)
if ($runtime -notlike '*MarkionMath.renderInto*' -or $runtime -like '*katex.renderToString*') {
    throw "MarkNice math rendering patch failed: renderMath source shape changed"
}
Set-Content -LiteralPath (Join-Path $staticRoot "theme-runtime.js") -Value $runtime -Encoding utf8NoBOM
$formatRuntime = "/* Generated from MarkNice $expectedCommit by scripts/sync-marknice-workspace.ps1. */`n" +
    (Get-UniqueSourceRegion -Text $sourceText -StartMarker "const markdownFormatToolbar = document.querySelector('.markdown-format-toolbar');" -EndMarker "function localImageRecordsForCurrentPreview() {")
$formatRuntime = Replace-UniqueSourceText -Text $formatRuntime -Search "  if (message) statusEl.textContent = message;" -Replacement "  if (message) statusEl.textContent = markdownFormatText('applied');" -Description "format status"
$formatRuntime = Replace-UniqueSourceText -Text $formatRuntime -Search "  else if (action === 'imageUpload') imageFileInput?.click();`r`n" -Replacement "" -Description "image upload action"
$formatRuntime = Replace-UniqueSourceText -Text $formatRuntime -Search "  const table = '| 标题一 | 标题二 | 标题三 |\n| --- | --- | --- |\n| 内容 | 内容 | 内容 |';" -Replacement "  const table = markdownFormatText('tableTemplate');" -Description "table template"
$formatRuntime = Replace-UniqueSourceText -Text $formatRuntime -Search "    const inserted = '1. 列表项';" -Replacement "    const inserted = '1. ' + markdownFormatText('listPlaceholder');" -Description "ordered-list placeholder"
$formatPlaceholders = [ordered]@{
    "'标题'" = "markdownFormatText('headingPlaceholder')"
    "'列表项'" = "markdownFormatText('listPlaceholder')"
    "'代码'" = "markdownFormatText('codePlaceholder')"
    "'代码内容'" = "markdownFormatText('codeBlockPlaceholder')"
    "'链接文字'" = "markdownFormatText('linkPlaceholder')"
    "'图片描述'" = "markdownFormatText('imagePlaceholder')"
    "'图片地址'" = "markdownFormatText('imageUrlPlaceholder')"
    "'加粗文字'" = "markdownFormatText('boldPlaceholder')"
    "'斜体文字'" = "markdownFormatText('italicPlaceholder')"
    "'下划线文字'" = "markdownFormatText('underlinePlaceholder')"
    "'引用内容'" = "markdownFormatText('quotePlaceholder')"
}
foreach ($entry in $formatPlaceholders.GetEnumerator()) {
    $formatRuntime = $formatRuntime.Replace($entry.Key, $entry.Value)
}
$formatRuntime += "`nwindow.MarkionMarkdownFormat = Object.freeze({ runMarkdownAction, rememberMarkdownSelection });"
Set-Content -LiteralPath (Join-Path $staticRoot "marknice-format-runtime.js") -Value $formatRuntime -Encoding utf8NoBOM
$wordRuntime = "/* Generated from MarkNice $expectedCommit by scripts/sync-marknice-workspace.ps1. */`n" +
    (Get-UniqueSourceRegion -Text $sourceText -StartMarker "// ===== Word export helpers =====" -EndMarker "// ===== Save as Word =====")
Set-Content -LiteralPath (Join-Path $staticRoot "marknice-word-runtime.js") -Value $wordRuntime -Encoding utf8NoBOM
Copy-Item -LiteralPath (Join-Path $Source "LICENSE") -Destination (Join-Path $bundleRoot "LICENSE.marknice.txt") -Force

if ($RefreshThirdParty) {
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        throw "Refreshing pinned third-party packages requires npm; normal builds do not."
    }
    $refreshRoot = Join-Path ([System.IO.Path]::GetTempPath()) ("markion-marknice-refresh-" + [guid]::NewGuid().ToString("N"))
    New-Item -ItemType Directory -Path $refreshRoot | Out-Null
    try {
        $markedArchive = (& npm pack marked@15.0.12 --pack-destination $refreshRoot --silent | Select-Object -Last 1).Trim()
        $mathjaxArchive = (& npm pack "mathjax@$mathjaxVersion" --pack-destination $refreshRoot --silent | Select-Object -Last 1).Trim()
        $htmlDocxArchive = (& npm pack "html-docx-js@$htmlDocxVersion" --pack-destination $refreshRoot --silent | Select-Object -Last 1).Trim()
        if ($LASTEXITCODE -ne 0 -or -not $markedArchive -or -not $mathjaxArchive -or -not $htmlDocxArchive) {
            throw "npm could not fetch the pinned renderer packages"
        }
        $markedExtract = Join-Path $refreshRoot "marked"
        $mathjaxExtract = Join-Path $refreshRoot "mathjax"
        $htmlDocxExtract = Join-Path $refreshRoot "html-docx-js"
        New-Item -ItemType Directory -Path $markedExtract, $mathjaxExtract, $htmlDocxExtract | Out-Null
        tar -xf (Join-Path $refreshRoot $markedArchive) -C $markedExtract
        tar -xf (Join-Path $refreshRoot $mathjaxArchive) -C $mathjaxExtract
        tar -xf (Join-Path $refreshRoot $htmlDocxArchive) -C $htmlDocxExtract
        Copy-Item -LiteralPath (Join-Path $markedExtract "package\lib\marked.umd.js") -Destination (Join-Path $vendorRoot "marked.umd.js") -Force
        Copy-Item -LiteralPath (Join-Path $markedExtract "package\LICENSE.md") -Destination (Join-Path $bundleRoot "LICENSE.marked.txt") -Force
        Copy-Item -LiteralPath (Join-Path $mathjaxExtract "package\es5\tex-svg-full.js") -Destination (Join-Path $vendorRoot "tex-svg-full.js") -Force
        Copy-Item -LiteralPath (Join-Path $mathjaxExtract "package\LICENSE") -Destination (Join-Path $bundleRoot "LICENSE.mathjax.txt") -Force
        Copy-Item -LiteralPath (Join-Path $htmlDocxExtract "package\dist\html-docx.js") -Destination (Join-Path $vendorRoot "html-docx.js") -Force
        Copy-Item -LiteralPath (Join-Path $htmlDocxExtract "package\LICENSE") -Destination (Join-Path $bundleRoot "LICENSE.html-docx-js.txt") -Force
    } finally {
        $resolvedRefresh = [System.IO.Path]::GetFullPath($refreshRoot)
        $resolvedTemp = [System.IO.Path]::GetFullPath([System.IO.Path]::GetTempPath())
        if ($resolvedRefresh.StartsWith($resolvedTemp, [System.StringComparison]::OrdinalIgnoreCase) -and (Split-Path -Leaf $resolvedRefresh).StartsWith("markion-marknice-refresh-")) {
            Remove-Item -LiteralPath $resolvedRefresh -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

$requiredThirdParty = @("marked.umd.js", "tex-svg-full.js", "html-docx.js")
foreach ($file in $requiredThirdParty) {
    if (-not (Test-Path -LiteralPath (Join-Path $vendorRoot $file) -PathType Leaf)) {
        throw "Missing vendored dependency $file; rerun with -RefreshThirdParty"
    }
}

# html-docx-js writes synthetic `file:///C:/fake/...` MHT part locations. They
# are not user paths, but they would escape into the downloaded package and
# violate the workspace's no-filesystem-reference artifact boundary. MHT part
# locations accept absolute URIs, so use deterministic non-resolving URNs while
# preserving the converter's intra-package image matching behavior.
$htmlDocxPath = Join-Path $vendorRoot "html-docx.js"
$htmlDocxRuntime = [System.IO.File]::ReadAllText($htmlDocxPath)
$documentLocation = "file:///C:/fake/document.html"
$imageLocation = '"file:///C:/fake/image" + index + "." + extension'
$safeDocumentLocation = "urn:markion:document.html"
$safeImageLocation = '"urn:markion:image" + index + "." + extension'
$documentLocationCount = [regex]::Matches($htmlDocxRuntime, [regex]::Escape($documentLocation)).Count
$imageLocationCount = [regex]::Matches($htmlDocxRuntime, [regex]::Escape($imageLocation)).Count
$safeDocumentLocationCount = [regex]::Matches($htmlDocxRuntime, [regex]::Escape($safeDocumentLocation)).Count
$safeImageLocationCount = [regex]::Matches($htmlDocxRuntime, [regex]::Escape($safeImageLocation)).Count
if (($documentLocationCount -ne 1 -or $imageLocationCount -ne 1) -and
    ($safeDocumentLocationCount -ne 1 -or $safeImageLocationCount -ne 1)) {
    throw "Unexpected html-docx-js MHT location templates; review the pinned converter before refresh"
}
if ($documentLocationCount -eq 1) {
    $htmlDocxRuntime = $htmlDocxRuntime.Replace($documentLocation, $safeDocumentLocation)
    $htmlDocxRuntime = $htmlDocxRuntime.Replace($imageLocation, $safeImageLocation)
}
[System.IO.File]::WriteAllText($htmlDocxPath, $htmlDocxRuntime, [System.Text.UTF8Encoding]::new($false))

# Text files are hashed after normalizing CRLF and lone CR line endings to LF,
# matching the normalization applied by the Rust bundle verifier, so a workspace
# checked out with platform line endings verifies identically on every platform.
$script:textExtensions = @("html", "htm", "css", "js", "mjs", "json", "txt", "md", "map")

function Get-BundleFileSha256 {
    param([Parameter(Mandatory = $true)][string]$LiteralPath)
    $bytes = [System.IO.File]::ReadAllBytes($LiteralPath)
    $extension = [System.IO.Path]::GetExtension($LiteralPath).TrimStart('.').ToLowerInvariant()
    if ($script:textExtensions -contains $extension) {
        $normalized = [System.Collections.Generic.List[byte]]::new()
        for ($index = 0; $index -lt $bytes.Length; $index++) {
            if ($bytes[$index] -eq 13) {
                if (($index + 1) -lt $bytes.Length -and $bytes[$index + 1] -eq 10) { $index++ }
                $normalized.Add(10)
            }
            else {
                $normalized.Add($bytes[$index])
            }
        }
        $bytes = $normalized.ToArray()
    }
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        return (($sha.ComputeHash($bytes) | ForEach-Object { $_.ToString('x2') }) -join '')
    }
    finally {
        $sha.Dispose()
    }
}

$manifestPath = Join-Path $bundleRoot "bundle-manifest.json"
$files = Get-ChildItem -LiteralPath $bundleRoot -File -Recurse |
    Where-Object { $_.FullName -ne $manifestPath } |
    Sort-Object FullName |
    ForEach-Object {
        [ordered]@{
            path = $_.FullName.Substring($bundleRoot.Length + 1).Replace('\', '/')
            sha256 = Get-BundleFileSha256 -LiteralPath $_.FullName
        }
    }
$manifest = [ordered]@{
    import_format_version = 1
    source_repository = "https://github.com/willmove/marknice"
    source_commit = $expectedCommit
    third_party = @(
        [ordered]@{ name = "MarkNice"; version = $expectedCommit; license = "MIT"; license_file = "LICENSE.marknice.txt" }
        [ordered]@{ name = "marked"; version = "15.0.12"; license = "MIT"; license_file = "LICENSE.marked.txt" }
        [ordered]@{ name = "MathJax"; version = $mathjaxVersion; license = "Apache-2.0"; license_file = "LICENSE.mathjax.txt" }
        [ordered]@{ name = "html-docx-js"; version = $htmlDocxVersion; license = "MIT"; license_file = "LICENSE.html-docx-js.txt" }
    )
    files = @($files)
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
Write-Host "Synchronized $($files.Count) workspace files from MarkNice $expectedCommit"

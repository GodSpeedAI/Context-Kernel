# Build Windows packages for Context Kernel (x86_64-pc-windows-msvc).
# Produces a .zip and an Inno Setup installer plus SHA256SUMS.
# NOTE: the binaries and installer are UNSIGNED. Not executed/verified on Linux.
#
# Usage:
#   pwsh -File scripts/package/windows/build-windows.ps1 `
#     -Binary target\x86_64-pc-windows-msvc\release\context-kernal.exe `
#     -OutDir dist -Version 0.1.0
# Requires ISCC.exe (Inno Setup 6) on PATH or in a default install location.

param(
    [Parameter(Mandatory = $true)][string]$Binary,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [Parameter(Mandatory = $true)][string]$Version
)

$ErrorActionPreference = 'Stop'

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot  = (Resolve-Path (Join-Path $ScriptDir '..\..\..')).Path
$IssFile   = Join-Path $RepoRoot 'packaging\windows\context-kernel.iss'

function Require-File([string]$Path, [string]$What) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Missing input ($What): $Path"
    }
}

if ([string]::IsNullOrWhiteSpace($Version)) { throw 'Version is empty' }

$Docs = @{
    'README.md' = (Join-Path $RepoRoot 'README.md')
    'LICENSE'   = (Join-Path $RepoRoot 'LICENSE')
    'context-kernal.example.toml'     = (Join-Path $RepoRoot 'config\context-kernal.example.toml')
    'context-kernal.sse.example.toml' = (Join-Path $RepoRoot 'config\context-kernal.sse.example.toml')
}

Require-File $Binary 'binary'
Require-File $IssFile 'Inno Setup script'
foreach ($k in $Docs.Keys) { Require-File $Docs[$k] $k }

# Locate ISCC.exe
$Iscc = $null
$cmd = Get-Command 'ISCC.exe' -ErrorAction SilentlyContinue
if ($cmd) { $Iscc = $cmd.Source }
if (-not $Iscc) {
    $candidates = @(
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
        (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe'),
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe')
    )
    foreach ($c in $candidates) {
        if ($c -and (Test-Path -LiteralPath $c)) { $Iscc = $c; break }
    }
}
if (-not $Iscc) { throw 'ISCC.exe (Inno Setup 6) not found' }

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$OutDir = (Resolve-Path $OutDir).Path

$Stage = Join-Path ([System.IO.Path]::GetTempPath()) ("ck-stage-" + [guid]::NewGuid().ToString('N'))
try {
    $BinDir = Join-Path $Stage 'bin'
    $DocDir = Join-Path $Stage 'share\doc\context-kernel'
    $ExDir  = Join-Path $DocDir 'examples'
    New-Item -ItemType Directory -Force -Path $BinDir, $DocDir, $ExDir | Out-Null

    Copy-Item -LiteralPath $Binary -Destination (Join-Path $BinDir 'context-kernal.exe')
    # Alias is an identical copy, not a second compile.
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $BinDir 'context-kernel.exe')
    Copy-Item -LiteralPath $Docs['README.md'] -Destination $DocDir
    Copy-Item -LiteralPath $Docs['LICENSE'] -Destination $DocDir
    Copy-Item -LiteralPath $Docs['context-kernal.example.toml'] -Destination $ExDir
    Copy-Item -LiteralPath $Docs['context-kernal.sse.example.toml'] -Destination $ExDir

    $ZipName   = "context-kernel-$Version-windows-x86_64.zip"
    $SetupBase = "context-kernel-$Version-windows-x86_64-setup"
    $ZipPath   = Join-Path $OutDir $ZipName
    $SetupPath = Join-Path $OutDir ($SetupBase + '.exe')

    if (Test-Path -LiteralPath $ZipPath)   { Remove-Item -LiteralPath $ZipPath -Force }
    if (Test-Path -LiteralPath $SetupPath) { Remove-Item -LiteralPath $SetupPath -Force }

    # Zip: top-level bin\ and share\ (no extra wrapper directory).
    $items = Get-ChildItem -LiteralPath $Stage | ForEach-Object { $_.FullName }
    Compress-Archive -Path $items -DestinationPath $ZipPath -CompressionLevel Optimal

    & $Iscc "/DAppVersion=$Version" "/DStageDir=$Stage" "/DOutDir=$OutDir" "/DOutName=$SetupBase" $IssFile
    if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }

    Require-File $ZipPath 'zip output'
    Require-File $SetupPath 'installer output'

    $lines = @()
    foreach ($f in @($ZipPath, $SetupPath)) {
        $h = (Get-FileHash -LiteralPath $f -Algorithm SHA256).Hash.ToLower()
        $lines += ("{0}  {1}" -f $h, (Split-Path -Leaf $f))
    }
    $SumsPath = Join-Path $OutDir 'SHA256SUMS'
    # ASCII, LF endings, so sha256sum -c works.
    [System.IO.File]::WriteAllText($SumsPath, (($lines -join "`n") + "`n"), (New-Object System.Text.ASCIIEncoding))

    Write-Host "Wrote $ZipPath"
    Write-Host "Wrote $SetupPath"
    Write-Host "Wrote $SumsPath"
}
finally {
    if (Test-Path -LiteralPath $Stage) { Remove-Item -LiteralPath $Stage -Recurse -Force -ErrorAction SilentlyContinue }
}

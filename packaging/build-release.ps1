[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$REPO = Split-Path $PSScriptRoot
$DIST = Join-Path $REPO 'dist'
$BUILD_JOBS = 2
$INNO_COMPILER_PATHS = @(
    (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:ProgramFiles 'Inno Setup 6\ISCC.exe'),
    (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe')
)

function Get-CrateVersion {
    $match = Select-String -LiteralPath (Join-Path $REPO 'Cargo.toml') -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
    return $match.Matches[0].Groups[1].Value
}

function Build-Binary {
    cargo build --release -j $BUILD_JOBS --manifest-path (Join-Path $REPO 'Cargo.toml')
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed with exit code $LASTEXITCODE" }
}

function New-Stage([string] $Version) {
    $stage = Join-Path $DIST "Magpie-$Version"
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item -LiteralPath (Join-Path $REPO 'target\release\magpie.exe') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'install.ps1'), (Join-Path $PSScriptRoot 'uninstall.ps1') -Destination $stage
    Set-Content -LiteralPath (Join-Path $stage 'VERSION') -Value $Version -Encoding ASCII
    return $stage
}

function New-PortableZip([string] $Stage) {
    $zip = "$Stage-windows-x64.zip"
    Compress-Archive -Path (Join-Path $Stage '*') -DestinationPath $zip -Force
    return $zip
}

function New-PortableExe([string] $Version) {
    $exe = Join-Path $DIST "Magpie-$Version-windows-x64.exe"
    Copy-Item -LiteralPath (Join-Path $REPO 'target\release\magpie.exe') -Destination $exe -Force
    return $exe
}

function Write-Checksums([string[]] $Files) {
    $checksums = Join-Path $DIST 'checksums.txt'
    $lines = $Files | ForEach-Object { '{0}  {1}' -f (Get-FileHash -Algorithm SHA256 -LiteralPath $_).Hash.ToLowerInvariant(), (Split-Path $_ -Leaf) }
    Set-Content -LiteralPath $checksums -Value $lines -Encoding ASCII
    return $checksums
}

function Find-InnoCompiler {
    $onPath = Get-Command 'ISCC.exe' -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }
    return $INNO_COMPILER_PATHS | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}

function New-Installer([string] $Compiler, [string] $Stage, [string] $Version) {
    & $Compiler "/DAppVersion=$Version" "/DStageDir=$Stage" "/O$DIST" (Join-Path $PSScriptRoot 'magpie.iss') | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "ISCC failed with exit code $LASTEXITCODE" }
    return Join-Path $DIST "Magpie-$Version-setup.exe"
}

$version = Get-CrateVersion
Build-Binary
$stage = New-Stage -Version $version
$assets = @((New-PortableZip -Stage $stage), (New-PortableExe -Version $version))
$compiler = Find-InnoCompiler
if ($compiler) {
    $assets += New-Installer -Compiler $compiler -Stage $stage -Version $version
} else {
    Write-Host 'Inno Setup 6 not found, so no setup.exe was built. Install it from https://jrsoftware.org/isinfo.php to build one.'
}
$assets | ForEach-Object { Write-Host "Release asset: $_" }
Write-Host "Checksums: $(Write-Checksums -Files $assets)"

[CmdletBinding(SupportsShouldProcess)]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$MANIFEST = Join-Path $PSScriptRoot 'install.json'

function Read-Manifest {
    if (-not (Test-Path -LiteralPath $MANIFEST)) {
        throw "No install.json in $PSScriptRoot, so this is not a Magpie install folder."
    }
    return Get-Content -LiteralPath $MANIFEST -Raw | ConvertFrom-Json
}

function Remove-Existing([string] $Path) {
    if (Test-Path -LiteralPath $Path) { Remove-Item -LiteralPath $Path -Recurse -Force }
}

$manifest = Read-Manifest
@($manifest.Shortcuts) | ForEach-Object { Remove-Existing $_ }
Remove-Existing $manifest.UninstallKey
Set-Location -LiteralPath $env:TEMP
Remove-Existing $PSScriptRoot
Write-Host 'Magpie uninstalled. Your browsing data and settings were kept.'

param(
    [Parameter(Mandatory)] [int] $ProcessId,
    [Parameter(Mandatory)] [ValidateSet('Setup', 'Msi', 'Portable')] [string] $Kind,
    [Parameter(Mandatory)] [string] $Source,
    [Parameter(Mandatory)] [string] $Target,
    [Parameter(Mandatory)] [string] $FailureNote
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$ATTEMPTS = 40
$RETRY_MS = 250
$SUCCESS_CODES = 0, 3010
$SETUP_ARGUMENTS = '/SILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-'
$MSI_ARGUMENTS = '/passive', '/norestart'

function Wait-ForExit {
    $running = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
    if ($running) { $running.WaitForExit() }
}

function Invoke-Installer([string] $Name, [string] $FilePath, [string[]] $Arguments) {
    $installer = Start-Process -FilePath $FilePath -ArgumentList $Arguments -Wait -PassThru
    if ($installer.ExitCode -notin $SUCCESS_CODES) { throw "$Name stopped with exit code $($installer.ExitCode)" }
}

function Replace-Target([string] $Staged) {
    for ($attempt = 1; ; $attempt++) {
        try {
            [System.IO.File]::Replace($Staged, $Target, [NullString]::Value)
            return
        } catch {
            if ($attempt -ge $ATTEMPTS) { throw }
            Start-Sleep -Milliseconds $RETRY_MS
        }
    }
}

function Install-Portable {
    $staged = "$Target.new"
    Copy-Item -LiteralPath $Source -Destination $staged -Force
    try {
        Replace-Target -Staged $staged
    } finally {
        Remove-Item -LiteralPath $staged -ErrorAction SilentlyContinue
    }
}

function Install-Update {
    switch ($Kind) {
        'Setup' { Invoke-Installer -Name 'Setup' -FilePath $Source -Arguments $SETUP_ARGUMENTS }
        'Msi' { Invoke-Installer -Name 'Windows Installer' -FilePath 'msiexec.exe' -Arguments (@('/i', "`"$Source`"") + $MSI_ARGUMENTS) }
        'Portable' { Install-Portable }
    }
}

try {
    Wait-ForExit
    Install-Update
} catch {
    [System.IO.File]::WriteAllText($FailureNote, $_.Exception.Message)
}
Start-Process -FilePath $Target

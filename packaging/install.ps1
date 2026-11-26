[CmdletBinding(SupportsShouldProcess)]
param(
    [string] $InstallDir = (Join-Path $env:LOCALAPPDATA 'Programs\Magpie'),
    [string] $StartMenuDir = [Environment]::GetFolderPath('Programs'),
    [string] $DesktopDir = [Environment]::GetFolderPath('Desktop'),
    [string] $UninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Magpie',
    [string] $SoftwareKey = 'HKCU:\Software',
    [switch] $DesktopShortcut
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$PACKAGE_FILES = 'magpie.exe', 'uninstall.ps1'
$SHORTCUT_NAME = 'Magpie.lnk'
$APP_NAME = 'Magpie'
$APP_DESCRIPTION = 'Magpie web browser'
$PROG_ID = 'MagpieHTML'
$CLIENT_KEY = 'Clients\StartMenuInternet\Magpie'
$CAPABILITIES_KEY = "$CLIENT_KEY\Capabilities"
$URL_SCHEMES = 'http', 'https'
$FILE_EXTENSIONS = '.htm', '.html'

function Assert-Package {
    $missing = @($PACKAGE_FILES + 'VERSION' | Where-Object { -not (Test-Path (Join-Path $PSScriptRoot $_)) })
    if ($missing) {
        throw "Missing $($missing -join ', ') next to install.ps1. Run packaging\build-release.ps1, then the install.ps1 in dist\Magpie-<version>."
    }
}

function Assert-NotRunning([string] $Executable) {
    $running = @(Get-Process -Name 'magpie' -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $Executable })
    if ($running) {
        throw "Magpie is running from $InstallDir. Close it, then run install.ps1 again."
    }
}

function Copy-Package {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    $PACKAGE_FILES | ForEach-Object { Copy-Item -LiteralPath (Join-Path $PSScriptRoot $_) -Destination $InstallDir -Force }
}

function New-Shortcut([string] $Path, [string] $Target) {
    if (-not $PSCmdlet.ShouldProcess($Path, 'Create shortcut')) { return }
    New-Item -ItemType Directory -Path (Split-Path $Path) -Force | Out-Null
    $shortcut = (New-Object -ComObject WScript.Shell).CreateShortcut($Path)
    $shortcut.TargetPath = $Target
    $shortcut.WorkingDirectory = Split-Path $Target
    $shortcut.IconLocation = "$Target,0"
    $shortcut.Description = 'Magpie web browser'
    $shortcut.Save()
}

function New-Shortcuts([string] $Target) {
    $directories = @($StartMenuDir) + @($DesktopDir | Where-Object { $DesktopShortcut })
    $paths = @($directories | ForEach-Object { Join-Path $_ $SHORTCUT_NAME })
    $paths | ForEach-Object { New-Shortcut -Path $_ -Target $Target }
    return $paths
}

function Register-Uninstall([string] $Executable) {
    if (-not $PSCmdlet.ShouldProcess($UninstallKey, 'Register uninstall entry')) { return }
    $uninstallCommand = "powershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $InstallDir 'uninstall.ps1')`""
    $sizeKb = [int][Math]::Ceiling((Get-Item (Join-Path $PSScriptRoot 'magpie.exe')).Length / 1KB)
    $strings = @{
        DisplayName          = 'Magpie'
        DisplayVersion       = (Get-Content (Join-Path $PSScriptRoot 'VERSION') -Raw).Trim()
        Publisher            = 'Magpie'
        DisplayIcon          = "$Executable,0"
        InstallLocation      = $InstallDir
        InstallDate          = Get-Date -Format 'yyyyMMdd'
        UninstallString      = $uninstallCommand
        QuietUninstallString = $uninstallCommand
    }
    $dwords = @{ EstimatedSize = $sizeKb; NoModify = 1; NoRepair = 1 }

    New-Item -Path $UninstallKey -Force | Out-Null
    $strings.GetEnumerator() | ForEach-Object { New-ItemProperty -Path $UninstallKey -Name $_.Key -Value $_.Value -PropertyType String -Force | Out-Null }
    $dwords.GetEnumerator() | ForEach-Object { New-ItemProperty -Path $UninstallKey -Name $_.Key -Value $_.Value -PropertyType DWord -Force | Out-Null }
}

function Set-Values([string] $Key, [hashtable] $Values) {
    New-Item -Path $Key -Force | Out-Null
    $Values.GetEnumerator() | ForEach-Object { New-ItemProperty -Path $Key -Name $_.Key -Value $_.Value -PropertyType String -Force | Out-Null }
}

function Register-ProgId([string] $Executable) {
    $progIdKey = Join-Path $SoftwareKey "Classes\$PROG_ID"
    $icon = "$Executable,0"
    Set-Values $progIdKey @{ '(default)' = "$APP_NAME HTML Document"; 'URL Protocol' = '' }
    Set-Values "$progIdKey\DefaultIcon" @{ '(default)' = $icon }
    Set-Values "$progIdKey\Application" @{ ApplicationName = $APP_NAME; ApplicationDescription = $APP_DESCRIPTION; ApplicationIcon = $icon }
    Set-Values "$progIdKey\shell\open\command" @{ '(default)' = "`"$Executable`" `"%1`"" }
    $FILE_EXTENSIONS | ForEach-Object { Set-Values (Join-Path $SoftwareKey "Classes\$_\OpenWithProgids") @{ $PROG_ID = '' } }
}

function Register-Client([string] $Executable) {
    $clientKey = Join-Path $SoftwareKey $CLIENT_KEY
    $capabilitiesKey = Join-Path $SoftwareKey $CAPABILITIES_KEY
    $icon = "$Executable,0"
    Set-Values $clientKey @{ '(default)' = $APP_NAME }
    Set-Values "$clientKey\DefaultIcon" @{ '(default)' = $icon }
    Set-Values "$clientKey\shell\open\command" @{ '(default)' = "`"$Executable`"" }
    Set-Values $capabilitiesKey @{ ApplicationName = $APP_NAME; ApplicationDescription = $APP_DESCRIPTION; ApplicationIcon = $icon }
    Set-Values "$capabilitiesKey\StartMenu" @{ StartMenuInternet = $APP_NAME }
    $urls = @{}; $URL_SCHEMES | ForEach-Object { $urls[$_] = $PROG_ID }
    $files = @{}; $FILE_EXTENSIONS | ForEach-Object { $files[$_] = $PROG_ID }
    Set-Values "$capabilitiesKey\URLAssociations" $urls
    Set-Values "$capabilitiesKey\FileAssociations" $files
    Set-Values (Join-Path $SoftwareKey 'RegisteredApplications') @{ $APP_NAME = "Software\$CAPABILITIES_KEY" }
}

function Register-Browser([string] $Executable) {
    if (-not $PSCmdlet.ShouldProcess($APP_NAME, 'Register as a browser in Default Apps')) { return }
    Register-ProgId -Executable $Executable
    Register-Client -Executable $Executable
}

function Save-Manifest([string[]] $Shortcuts) {
    $manifest = [ordered]@{
        Shortcuts        = $Shortcuts
        UninstallKey     = $UninstallKey
        RegistryKeys     = @((Join-Path $SoftwareKey "Classes\$PROG_ID"), (Join-Path $SoftwareKey $CLIENT_KEY))
        RegistryValues   = @(@{ Key = (Join-Path $SoftwareKey 'RegisteredApplications'); Name = $APP_NAME }) +
            @($FILE_EXTENSIONS | ForEach-Object { @{ Key = (Join-Path $SoftwareKey "Classes\$_\OpenWithProgids"); Name = $PROG_ID } })
    }
    $manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $InstallDir 'install.json') -Encoding UTF8
}

Assert-Package
$executable = Join-Path $InstallDir 'magpie.exe'
Assert-NotRunning -Executable $executable
Copy-Package
$shortcuts = New-Shortcuts -Target $executable
Register-Uninstall -Executable $executable
Register-Browser -Executable $executable
Save-Manifest -Shortcuts $shortcuts
Write-Host "Magpie installed to $InstallDir"

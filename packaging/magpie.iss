#ifndef AppVersion
  #error Pass the version: ISCC /DAppVersion=<version> /DStageDir=<dist\Magpie-version> magpie.iss
#endif
#ifndef StageDir
  #error Pass the staged package folder: ISCC /DAppVersion=<version> /DStageDir=<dist\Magpie-version> magpie.iss
#endif

[Setup]
AppId={{3E84C269-EBDC-46FD-AEC0-B49F4DB27B6E}
AppName=Magpie
AppVersion={#AppVersion}
AppPublisher=Magpie
AppPublisherURL=https://github.com/vaakx-dev/magpie
AppSupportURL=https://github.com/vaakx-dev/magpie/issues
AppUpdatesURL=https://github.com/vaakx-dev/magpie/releases
AppComments=Magpie web browser
DefaultDirName={userpf}\Magpie
DisableProgramGroupPage=yes
DisableDirPage=auto
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
SetupIconFile=..\assets\magpie.ico
UninstallDisplayIcon={app}\magpie.exe
UninstallDisplayName=Magpie
OutputBaseFilename=Magpie-{#AppVersion}-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; Flags: unchecked

[Files]
Source: "{#StageDir}\magpie.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\Magpie"; Filename: "{app}\magpie.exe"; Comment: "Magpie web browser"
Name: "{userdesktop}\Magpie"; Filename: "{app}\magpie.exe"; Comment: "Magpie web browser"; Tasks: desktopicon

[Run]
Filename: "{app}\magpie.exe"; Description: "Launch Magpie"; Flags: nowait postinstall skipifsilent

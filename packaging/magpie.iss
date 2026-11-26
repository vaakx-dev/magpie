#ifndef AppVersion
  #error Pass the version: ISCC /DAppVersion=<version> /DStageDir=<dist\Magpie-version> magpie.iss
#endif
#ifndef AssetLabel
  #define AssetLabel AppVersion
#endif
#ifndef StageDir
  #error Pass the staged package folder: ISCC /DAppVersion=<version> /DStageDir=<dist\Magpie-version> magpie.iss
#endif

#define ProgId "MagpieHTML"
#define ClientKey "Software\Clients\StartMenuInternet\Magpie"
#define ProgIdKey "Software\Classes\" + ProgId
#define Icon "{app}\magpie.exe,0"

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
OutputBaseFilename=Magpie-{#AssetLabel}-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
AppMutex=Magpie.Instance
CloseApplications=yes
RestartApplications=no
ChangesAssociations=yes

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop shortcut"; Flags: unchecked

[Files]
Source: "{#StageDir}\magpie.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{userprograms}\Magpie"; Filename: "{app}\magpie.exe"; Comment: "Magpie web browser"
Name: "{userdesktop}\Magpie"; Filename: "{app}\magpie.exe"; Comment: "Magpie web browser"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "{#ProgIdKey}"; ValueType: string; ValueData: "Magpie HTML Document"; Flags: uninsdeletekey
Root: HKCU; Subkey: "{#ProgIdKey}"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "{#ProgIdKey}\DefaultIcon"; ValueType: string; ValueData: "{#Icon}"
Root: HKCU; Subkey: "{#ProgIdKey}\Application"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Magpie"
Root: HKCU; Subkey: "{#ProgIdKey}\Application"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Magpie web browser"
Root: HKCU; Subkey: "{#ProgIdKey}\Application"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{#Icon}"
Root: HKCU; Subkey: "{#ProgIdKey}\shell\open\command"; ValueType: string; ValueData: """{app}\magpie.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\.htm\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.html\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "{#ClientKey}"; ValueType: string; ValueData: "Magpie"; Flags: uninsdeletekey
Root: HKCU; Subkey: "{#ClientKey}\DefaultIcon"; ValueType: string; ValueData: "{#Icon}"
Root: HKCU; Subkey: "{#ClientKey}\shell\open\command"; ValueType: string; ValueData: """{app}\magpie.exe"""
Root: HKCU; Subkey: "{#ClientKey}\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "Magpie"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "Magpie web browser"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: "{#Icon}"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities\StartMenu"; ValueType: string; ValueName: "StartMenuInternet"; ValueData: "Magpie"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities\URLAssociations"; ValueType: string; ValueName: "http"; ValueData: "{#ProgId}"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities\URLAssociations"; ValueType: string; ValueName: "https"; ValueData: "{#ProgId}"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".htm"; ValueData: "{#ProgId}"
Root: HKCU; Subkey: "{#ClientKey}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".html"; ValueData: "{#ProgId}"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "Magpie"; ValueData: "{#ClientKey}\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\magpie.exe"; Description: "Launch Magpie"; Flags: nowait postinstall skipifsilent

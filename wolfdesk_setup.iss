; Script de Instalacion Inno Setup para WolfDesk Pro
#define MyAppName "WolfDesk Pro"
#define MyAppVersion "1.2.0"
#define MyAppPublisher "WolfDesk Software"
#define MyAppURL "https://github.com/wolfdesk"
#define MyAppExeName "wolfdesk.exe"

[Setup]
AppId={{5A37F1CF-9514-75AA-A889-7A1ED3701C38}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\WolfDesk
DisableProgramGroupPage=yes
OutputDir=.
OutputBaseFilename=WolfDesk_Setup_v1.2.0
SetupIconFile=assets\wolfdesk.png
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "wolfdesk.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "WolfDesk_Certificate.cer"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: filesandordirs; Name: "{commonappdata}\WolfDesk"
Type: filesandordirs; Name: "{userappdata}\WolfDesk"

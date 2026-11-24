; Inno Setup 6. Build: iscc installer\gravi-island.iss  (setelah cargo build --release)
#define AppName "Gravi Island"
#define AppVer "1.0.0"

[Setup]
AppId={{6C1D2F6E-8B1A-4E1B-9D55-1A2B3C4D5E6F}
AppName={#AppName}
AppVersion={#AppVer}
AppPublisher=avoocreator
DefaultDirName={autopf}\GraviIsland
DefaultGroupName={#AppName}
OutputDir=..\dist
OutputBaseFilename=GraviIsland-Setup-{#AppVer}
Compression=lzma2
SolidCompression=yes
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=lowest
SetupIconFile=..\assets\gravi.ico
UninstallDisplayIcon={app}\gravi-island.exe
CloseApplications=yes

[Tasks]
Name: "autostart"; Description: "Jalankan otomatis saat Windows menyala"; Flags: unchecked
Name: "desktopicon"; Description: "Buat ikon di Desktop"; Flags: unchecked

[Files]
Source: "..\target\release\gravi-island.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\gravi-hook.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\gravi-island.exe"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\gravi-island.exe"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "GraviIsland"; ValueData: """{app}\gravi-island.exe"""; Flags: uninsdeletevalue; Tasks: autostart

[Run]
Filename: "{app}\gravi-island.exe"; Description: "Jalankan {#AppName}"; Flags: nowait postinstall skipifsilent

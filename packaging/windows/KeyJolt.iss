#ifndef AppVersion
  #error AppVersion must be passed by build.ps1.
#endif
#ifndef AppPublisher
  #error AppPublisher must be passed by build.ps1.
#endif
#ifndef AppBinary
  #error AppBinary must be passed by build.ps1.
#endif

#define AppName "KeyJolt"
#define AppExeName "key-jolt.exe"

[Setup]
AppId=LucasErrNotFound.KeyJolt
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppPublisher}
AppPublisherURL=https://github.com/LucasErrNotFound
AppSupportURL=https://github.com/LucasErrNotFound/key-jolt/issues
AppUpdatesURL=https://github.com/LucasErrNotFound/key-jolt/releases
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableDirPage=no
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
SourceDir=..\..
LicenseFile=LICENSE
OutputBaseFilename=KeyJolt-{#AppVersion}-windows-x64-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
SetupIconFile=assets\icons\key-jolt.ico
UninstallDisplayIcon={app}\{#AppExeName}
CloseApplications=yes
RestartApplications=no
VersionInfoVersion={#AppVersion}.0
VersionInfoCompany={#AppPublisher}
VersionInfoDescription={#AppName} Installer
VersionInfoProductName={#AppName}
VersionInfoProductVersion={#AppVersion}.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#AppBinary}"; DestDir: "{app}"; DestName: "{#AppExeName}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#AppName}"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExeName}"; WorkingDir: "{app}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent unchecked

[Code]
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  StartupCommand: String;
  InstalledCommand: String;
begin
  if CurUninstallStep = usUninstall then
  begin
    InstalledCommand := '"' + ExpandConstant('{app}\{#AppExeName}') + '" --startup';
    if RegQueryStringValue(HKCU64, 'Software\Microsoft\Windows\CurrentVersion\Run',
      'KeyJolt', StartupCommand) then
    begin
      if CompareText(StartupCommand, InstalledCommand) = 0 then
        RegDeleteValue(HKCU64, 'Software\Microsoft\Windows\CurrentVersion\Run', 'KeyJolt');
    end;
  end;
end;

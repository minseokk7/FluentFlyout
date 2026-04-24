; Inno Setup Script for FluentFlyout
#define MyAppName "FluentFlyout"
#define MyAppVersion "3.1.3"
#define MyAppPublisher "minseokk7"
#define MyAppURL "https://github.com/minseokk7/FluentFlyout"
#define MyAppExeName "FluentFlyout.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application. Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{D3D8B0A1-6F5C-4FBC-B0D1-1C5E2E2F2E2F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
; Remove the following line to run in administrative install mode (install for all users.)
PrivilegesRequired=lowest
OutputBaseFilename=FluentFlyout-v{#MyAppVersion}-Installer
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "korean"; MessagesFile: "compiler:Languages\Korean.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startup"; Description: "Windows 시작 시 자동 실행"; GroupDescription: "기타:"; Flags: checkedonce

[Files]
Source: "C:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\publish_output\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\publish_output\fluent_flyout_core.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\publish_output\*.dll"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\publish_output\NLog.config"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\Users\minse\.gemini\antigravity\playground\spatial-kilonova\FluentFlyout\publish_output\Resources\*"; DestDir: "{app}\Resources"; Flags: ignoreversion recursesubdirs createallsubdirs
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startup

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

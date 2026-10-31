; Context Kernel - Inno Setup 6 script (per-user install).
;
; The binaries and this installer are UNSIGNED. Windows SmartScreen may warn.
;
; Required /D defines (passed by scripts/package/windows/build-windows.ps1):
;   /DAppVersion=0.1.0  /DStageDir=<abs path to stage root>  /DOutDir=<abs dir>
;   /DOutName=<installer base name without .exe>  /DRepoRoot=<unused, optional>
;
; Uninstall removes only files installed here and the PATH entry added by the
; "addtopath" task. It never touches %LOCALAPPDATA%\Context Kernel (user data).

#ifndef AppVersion
  #error AppVersion is required (/DAppVersion=x.y.z)
#endif
#ifndef StageDir
  #error StageDir is required (/DStageDir=path)
#endif
#ifndef OutDir
  #error OutDir is required (/DOutDir=path)
#endif
#ifndef OutName
  #define OutName "context-kernel-" + AppVersion + "-windows-x86_64-setup"
#endif

[Setup]
AppId={{6F1D3C52-8B7A-4E0B-9C4A-5D2E7A1B3F90}
AppName=Context Kernel
AppVersion={#AppVersion}
AppPublisher=GodSpeedAI
DefaultDirName={localappdata}\Programs\Context Kernel
DefaultGroupName=Context Kernel
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#OutDir}
OutputBaseFilename={#OutName}
Compression=lzma2
SolidCompression=yes
LicenseFile={#StageDir}\share\doc\context-kernel\LICENSE
UninstallDisplayName=Context Kernel
ChangesEnvironment=yes

[Tasks]
Name: "addtopath"; Description: "Add install directory to the user PATH"; GroupDescription: "Environment:"; Flags: unchecked

[Files]
Source: "{#StageDir}\bin\context-kernal.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "{#StageDir}\bin\context-kernel.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "{#StageDir}\share\doc\context-kernel\README.md"; DestDir: "{app}\share\doc\context-kernel"; Flags: ignoreversion
Source: "{#StageDir}\share\doc\context-kernel\LICENSE"; DestDir: "{app}\share\doc\context-kernel"; Flags: ignoreversion
Source: "{#StageDir}\share\doc\context-kernel\examples\context-kernal.example.toml"; DestDir: "{app}\share\doc\context-kernel\examples"; Flags: ignoreversion
Source: "{#StageDir}\share\doc\context-kernel\examples\context-kernal.sse.example.toml"; DestDir: "{app}\share\doc\context-kernel\examples"; Flags: ignoreversion

[Code]
const
  EnvKey = 'Environment';

function BinDir(): string;
begin
  Result := ExpandConstant('{app}\bin');
end;

function PathContains(const Paths, Dir: string): Boolean;
begin
  Result := Pos(';' + Lowercase(Dir) + ';', ';' + Lowercase(Paths) + ';') > 0;
end;

procedure AddToUserPath();
var
  Paths: string;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', Paths) then
    Paths := '';
  if PathContains(Paths, BinDir()) then
    Exit;
  if (Paths <> '') and (Copy(Paths, Length(Paths), 1) <> ';') then
    Paths := Paths + ';';
  RegWriteExpandStringValue(HKCU, EnvKey, 'Path', Paths + BinDir());
end;

procedure RemoveFromUserPath();
var
  Paths, Needle: string;
  P: Integer;
begin
  if not RegQueryStringValue(HKCU, EnvKey, 'Path', Paths) then
    Exit;
  Needle := BinDir();
  // Wrap in separators so only whole entries match.
  Paths := ';' + Paths + ';';
  P := Pos(';' + Lowercase(Needle) + ';', Lowercase(Paths));
  if P = 0 then
    Exit;
  Delete(Paths, P, Length(Needle) + 1);
  // Trim the wrapping separators.
  if Copy(Paths, 1, 1) = ';' then
    Delete(Paths, 1, 1);
  if Copy(Paths, Length(Paths), 1) = ';' then
    Delete(Paths, Length(Paths), 1);
  RegWriteExpandStringValue(HKCU, EnvKey, 'Path', Paths);
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    AddToUserPath();
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    RemoveFromUserPath();
end;

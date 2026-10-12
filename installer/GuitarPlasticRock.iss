; Instalador de Guitar Plastic Rock (Inno Setup 6).
; Compilar desde la raíz del repositorio, después de `cargo build --release`:
;   iscc /DAppVersion=1.0.0 installer\GuitarPlasticRock.iss
; Resultado: target\installer\GuitarPlasticRock-Setup-<versión>.exe
;
; Instalación por usuario (sin administrador). ViGEmBus, si falta, lo instala el propio
; GuitarPlasticRock.exe con el instalador oficial que lleva dentro (`--install-vigem`): es el único
; paso que pide permiso de administrador.

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#define AppName "Guitar Plastic Rock"
#define AppId "GuitarPlasticRock"
#define AppExe "GuitarPlasticRock.exe"

[Setup]
AppId={{F2746305-F00D-4271-8708-3790C81458EC}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher={#AppName}
DefaultDirName={autopf}\{#AppId}
DisableProgramGroupPage=yes
DisableReadyPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\target\installer
OutputBaseFilename={#AppId}-Setup-{#AppVersion}
SetupIconFile=..\assets\brand\icon.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
CloseApplications=yes
RestartApplications=no
#if FileExists(AddBackslash(SourcePath) + "..\LICENSE")
LicenseFile=..\LICENSE
#endif

; Los mismos 9 idiomas que la app (crates/gpr-i18n). El asistente elige el de Windows.
[Languages]
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ru"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "pl"; MessagesFile: "compiler:Languages\Polish.isl"
Name: "uk"; MessagesFile: "compiler:Languages\Ukrainian.isl"

[CustomMessages]
es.Autostart=Iniciar con Windows (driver en segundo plano)
en.Autostart=Start with Windows (background driver)
ru.Autostart=Запускать вместе с Windows (драйвер в фоне)
fr.Autostart=Lancer avec Windows (pilote en arrière-plan)
de.Autostart=Mit Windows starten (Treiber im Hintergrund)
it.Autostart=Avvia con Windows (driver in background)
ptbr.Autostart=Iniciar com o Windows (driver em segundo plano)
pl.Autostart=Uruchamiaj z Windows (sterownik w tle)
uk.Autostart=Запускати разом із Windows (драйвер у фоні)
es.InstallingViGEm=Instalando ViGEmBus, necesario para los mandos Xbox virtuales...
en.InstallingViGEm=Installing ViGEmBus, required for the virtual Xbox controllers...
ru.InstallingViGEm=Установка ViGEmBus, необходимого для виртуальных геймпадов Xbox...
fr.InstallingViGEm=Installation de ViGEmBus, nécessaire aux manettes Xbox virtuelles...
de.InstallingViGEm=ViGEmBus wird installiert, nötig für die virtuellen Xbox-Controller...
it.InstallingViGEm=Installazione di ViGEmBus, necessario per i controller Xbox virtuali...
ptbr.InstallingViGEm=Instalando o ViGEmBus, necessário para os controles Xbox virtuais...
pl.InstallingViGEm=Instalowanie ViGEmBus, wymaganego dla wirtualnych kontrolerów Xbox...
uk.InstallingViGEm=Встановлення ViGEmBus, потрібного для віртуальних геймпадів Xbox...
es.Launch=Abrir {#AppName}
en.Launch=Open {#AppName}
ru.Launch=Открыть {#AppName}
fr.Launch=Ouvrir {#AppName}
de.Launch={#AppName} öffnen
it.Launch=Apri {#AppName}
ptbr.Launch=Abrir {#AppName}
pl.Launch=Otwórz {#AppName}
uk.Launch=Відкрити {#AppName}

[Tasks]
Name: "autostart"; Description: "{cm:Autostart}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\target\release\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
#if FileExists(AddBackslash(SourcePath) + "..\LICENSE")
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
#endif

[Registry]
; Mismo valor que crea la casilla "Iniciar con Windows" de la app
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#AppId}"; ValueData: """{app}\{#AppExe}"" --background"; Tasks: autostart; Flags: uninsdeletevalue

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#AppExe}"; Parameters: "--install-vigem"; StatusMsg: "{cm:InstallingViGEm}"; Check: ViGEmMissing; Flags: waituntilterminated
Filename: "{app}\{#AppExe}"; Parameters: "--background"; Flags: nowait
Filename: "{app}\{#AppExe}"; Description: "{cm:Launch}"; Flags: postinstall nowait skipifsilent

[UninstallRun]
Filename: "{app}\{#AppExe}"; Parameters: "--stop"; RunOnceId: "StopDriver"; Flags: runhidden waituntilterminated

[UninstallDelete]
; estado y registro del driver
Type: filesandordirs; Name: "{localappdata}\{#AppId}"

[Code]
const
  RunKey = 'Software\Microsoft\Windows\CurrentVersion\Run';
  { La app se llamaba GuitarBridge en sus primeras versiones }
  LegacyUninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{45DCCC7E-E327-44FC-A4B7-7C9DE7DCA950}_is1';
  LegacyRunValue = 'GuitarBridge';

function ViGEmMissing: Boolean;
begin
  Result := not RegKeyExists(HKLM, 'SYSTEM\CurrentControlSet\Services\ViGEmBus');
end;

{ Ruta del .exe en un valor de la clave Run ("C:\...\x.exe" --background). }
function ExeFromRunValue(Value: String): String;
begin
  Result := Value;
  if (Length(Result) > 0) and (Result[1] = '"') then
  begin
    Delete(Result, 1, 1);
    Result := Copy(Result, 1, Pos('"', Result) - 1);
  end;
end;

{ GuitarBridge (nombre anterior): detener su driver, para que no haya dos creando mandos para la
  misma guitarra, y desinstalarlo si se instaló con su Setup. }
procedure RemoveLegacyGuitarBridge;
var
  Value, Exe: String;
  Code: Integer;
begin
  if RegQueryStringValue(HKCU, RunKey, LegacyRunValue, Value) then
  begin
    Exe := ExeFromRunValue(Value);
    if FileExists(Exe) then
      Exec(Exe, '--stop', '', SW_HIDE, ewWaitUntilTerminated, Code);
    RegDeleteValue(HKCU, RunKey, LegacyRunValue);
  end;
  if RegQueryStringValue(HKCU, LegacyUninstallKey, 'UninstallString', Value) then
  begin
    Exe := RemoveQuotes(Value);
    if FileExists(Exe) then
      Exec(Exe, '/VERYSILENT /SUPPRESSMSGBOXES /NORESTART', '', SW_HIDE, ewWaitUntilTerminated, Code);
  end;
end;

{ Antes de sustituir archivos, detener el driver (también si es una copia portable): así una
  actualización nunca choca con el .exe en uso. }
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  RemoveLegacyGuitarBridge;
  if FileExists(ExpandConstant('{app}\{#AppExe}')) then
    Exec(ExpandConstant('{app}\{#AppExe}'), '--stop', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := '';
end;

//! Inicio con Windows (valor de la clave Run) y desinstalador del Setup.

use std::path::{Path, PathBuf};

use gpr_core::brand::APP_ID;
use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
use winreg::RegKey;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// Ruta del .exe que arranca con Windows, si lo hay.
pub fn autostart_path() -> Option<PathBuf> {
    let value: String = RegKey::predef(HKEY_CURRENT_USER).open_subkey(RUN_KEY).ok()?.get_value(APP_ID).ok()?;
    let path = match value.strip_prefix('"') {
        Some(rest) => rest.split('"').next()?.to_string(),
        None => value.split_whitespace().next()?.to_string(),
    };
    Some(PathBuf::from(path))
}

/// ¿El inicio automático apunta a este mismo .exe?
pub fn autostart_is_this_exe() -> bool {
    let (Some(target), Ok(me)) = (autostart_path(), std::env::current_exe()) else { return false };
    let canon = |p: &Path| std::fs::canonicalize(p).ok();
    canon(&target).is_some() && canon(&target) == canon(&me)
}

/// Activa (apuntando a este .exe) o desactiva el arranque del driver al iniciar sesión.
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if enabled {
        let me = std::env::current_exe().map_err(|e| e.to_string())?;
        let (run, _) = hkcu.create_subkey(RUN_KEY).map_err(|e| e.to_string())?;
        run.set_value(APP_ID, &format!("\"{}\" {}", me.display(), gpr_core::cli::BACKGROUND)).map_err(|e| e.to_string())
    } else {
        match hkcu.open_subkey_with_flags(RUN_KEY, KEY_SET_VALUE) {
            Ok(run) => match run.delete_value(APP_ID) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.to_string()),
                _ => Ok(()),
            },
            Err(_) => Ok(()),
        }
    }
}

/// Desinstalador de Inno Setup junto al .exe: solo existe si se instaló con el Setup.
pub fn uninstaller() -> Option<PathBuf> {
    let unins = std::env::current_exe().ok()?.parent()?.join("unins000.exe");
    unins.exists().then_some(unins)
}

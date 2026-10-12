//! Procesos y diálogos del sistema.

use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

use windows::core::HSTRING;

const DETACHED: u32 = 0x0000_0008 | 0x0000_0200; // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP

/// Lanza un proceso independiente (sigue vivo al cerrar este).
pub fn spawn_detached(exe: &Path, args: &[&str]) -> bool {
    Command::new(exe).args(args).creation_flags(DETACHED).spawn().is_ok()
}

/// Arranca el driver con este mismo .exe.
pub fn start_driver() -> bool {
    std::env::current_exe().map(|me| spawn_detached(&me, &[gpr_core::cli::BACKGROUND])).unwrap_or(false)
}

/// Abre la app (ventana) con este mismo .exe.
pub fn open_app() -> bool {
    std::env::current_exe().map(|me| spawn_detached(&me, &[])).unwrap_or(false)
}

pub fn error_box(title: &str, text: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    unsafe {
        MessageBoxW(None, &HSTRING::from(text), &HSTRING::from(title), MB_OK | MB_ICONERROR);
    }
}

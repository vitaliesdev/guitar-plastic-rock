//! ViGEmBus: el driver de Nefarius que crea los mandos Xbox virtuales, y su instalador incluido.

use gpr_core::brand::APP_ID;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Foundation::CloseHandle;

pub fn available() -> bool {
    vigem_client::Client::connect().is_ok()
}

/// Instalador oficial de ViGEmBus 1.22.0 (BSD-3-Clause, firmado por Nefarius Software Solutions;
/// SHA-256 89220a7865076b342892f98865f3499fb7c4cfd673159e89d352c360fd014c6a).
const VIGEM_SETUP: &[u8] = include_bytes!("../../../third_party/vigembus/ViGEmBus_1.22.0_x64_x86_arm64.exe");

/// Por qué no se pudo instalar ViGEmBus (la interfaz lo traduce).
#[derive(Debug)]
pub enum InstallError {
    /// No se pudo escribir el instalador en la carpeta temporal
    Extract(String),
    /// El usuario rechazó el permiso de administrador
    Cancelled,
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstallError::Extract(e) => write!(f, "could not extract the installer: {e}"),
            InstallError::Cancelled => write!(f, "installation cancelled"),
        }
    }
}

/// Extrae el instalador de ViGEmBus, lo ejecuta como administrador (Windows pide confirmación)
/// y espera a que termine. Devuelve el código de salida del instalador.
pub fn install() -> Result<u32, InstallError> {
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let path = std::env::temp_dir().join(format!("{APP_ID}-ViGEmBus_1.22.0_x64_x86_arm64.exe"));
    std::fs::write(&path, VIGEM_SETUP).map_err(|e| InstallError::Extract(e.to_string()))?;
    let file = HSTRING::from(path.as_os_str());
    let verb = HSTRING::from("runas");
    let mut info = SHELLEXECUTEINFOW {
        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as u32,
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    let result = unsafe { ShellExecuteExW(&mut info) }.map_err(|_| InstallError::Cancelled).map(|_| {
        let mut code = 1u32;
        unsafe {
            WaitForSingleObject(info.hProcess, INFINITE);
            let _ = GetExitCodeProcess(info.hProcess, &mut code);
            let _ = CloseHandle(info.hProcess);
        }
        code
    });
    let _ = std::fs::remove_file(&path);
    result
}

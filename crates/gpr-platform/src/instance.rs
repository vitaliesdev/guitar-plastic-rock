//! Instancia única del driver (mutex con nombre) y eventos con nombre para controlarlo desde la
//! app o el Setup: detener el driver y cerrar las ventanas abiertas.

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use gpr_core::brand::APP_ID;
use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, OpenEventW, OpenMutexW, ResetEvent, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE,
    SYNCHRONIZATION_SYNCHRONIZE,
};

/// Nombre de un objeto en el espacio de la sesión: `Local\GuitarPlasticRock.<sufijo>`.
fn name(suffix: &str) -> HSTRING {
    HSTRING::from(format!("Local\\{APP_ID}.{suffix}"))
}

fn driver_mutex() -> &'static HSTRING {
    static N: OnceLock<HSTRING> = OnceLock::new();
    N.get_or_init(|| name("Driver"))
}

fn stop_event() -> &'static HSTRING {
    static N: OnceLock<HSTRING> = OnceLock::new();
    N.get_or_init(|| name("Stop"))
}

fn quit_app_event() -> &'static HSTRING {
    static N: OnceLock<HSTRING> = OnceLock::new();
    N.get_or_init(|| name("QuitApp"))
}

/// Handle de Windows que se cierra solo.
pub struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

impl Handle {
    pub fn raw(&self) -> HANDLE {
        self.0
    }

    /// ¿Está activo el evento? (sin esperar)
    pub fn is_set(&self) -> bool {
        unsafe { WaitForSingleObject(self.0, 0) == WAIT_OBJECT_0 }
    }
}

// ---------- driver ----------

/// Reserva la instancia única del driver. None si ya hay otro en marcha.
pub fn acquire_driver() -> Option<Handle> {
    unsafe {
        let h = CreateMutexW(None, true, driver_mutex()).ok()?;
        if GetLastError() == ERROR_ALREADY_EXISTS {
            let _ = CloseHandle(h);
            return None;
        }
        Some(Handle(h))
    }
}

/// Evento que el driver vigila: cuando se activa, se cierra.
pub fn create_stop_event() -> Option<Handle> {
    unsafe { CreateEventW(None, true, false, stop_event()).ok().map(Handle) }
}

pub fn driver_running() -> bool {
    unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, false, driver_mutex()).map(Handle).is_ok() }
}

/// Pide al driver que se cierre. Devuelve false si no estaba en marcha.
pub fn stop_driver() -> bool {
    set_event(stop_event())
}

/// Espera (hasta `timeout`) a que el driver termine de cerrarse.
pub fn wait_driver_stopped(timeout: Duration) -> bool {
    let end = Instant::now() + timeout;
    while Instant::now() < end {
        if !driver_running() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    !driver_running()
}

// ---------- ventanas de la app ----------

/// Evento que vigila cada ventana de la app: el Setup (vía `--stop`) lo activa para cerrarlas antes
/// de sustituir archivos. No se puede confiar en el Restart Manager de Windows: winit no atiende
/// su petición de cierre.
pub fn create_quit_app_event() -> Option<Handle> {
    unsafe { CreateEventW(None, true, false, quit_app_event()).ok().map(Handle) }
}

/// Cierra todas las ventanas abiertas de la app (de esta copia o de cualquier otra).
pub fn close_app_windows() {
    unsafe {
        if let Ok(h) = OpenEventW(EVENT_MODIFY_STATE, false, quit_app_event()) {
            let h = Handle(h);
            let _ = SetEvent(h.raw());
            // las ventanas lo comprueban en cada fotograma; después se desactiva para que una
            // ventana abierta más tarde no se cierre sola
            std::thread::sleep(Duration::from_millis(1500));
            let _ = ResetEvent(h.raw());
        }
    }
}

fn set_event(name: &HSTRING) -> bool {
    unsafe {
        match OpenEventW(EVENT_MODIFY_STATE, false, name) {
            Ok(h) => {
                let h = Handle(h);
                SetEvent(h.raw()).is_ok()
            }
            Err(_) => false,
        }
    }
}

//! Guitar Plastic Rock: usa guitarras de Guitar Hero Live (dongle PS4) y PDP Riffmaster
//! (PlayStation) en Windows. Un único ejecutable, sin dependencias, que hace de:
//!
//! - app (sin argumentos): probador visual y control del driver; arranca el driver si no está en marcha
//! - driver (`--background`): los mandos Xbox virtuales, en segundo plano
//! - utilidades del Setup: `--stop` y `--install-vigem`
//!
//! Este crate solo reparte el trabajo; la arquitectura está en docs/ARCHITECTURE.md.

#![cfg_attr(not(test), windows_subsystem = "windows")]

use std::time::Duration;

use gpr_core::cli;
use gpr_platform::{instance, vigem};

fn main() {
    let arg = std::env::args().nth(1);
    let code = match arg.as_deref() {
        Some(cli::BACKGROUND) => gpr_backend::service::run(),
        Some(cli::STOP) => stop(),
        Some(cli::INSTALL_VIGEM) => install_vigem(),
        _ => gpr_frontend::run(),
    };
    std::process::exit(code);
}

/// Lo usa el Setup antes de sustituir archivos: cierra las ventanas y detiene el driver.
fn stop() -> i32 {
    instance::close_app_windows();
    instance::stop_driver();
    instance::wait_driver_stopped(Duration::from_secs(5));
    0
}

/// Lo usa el Setup si falta ViGEmBus: 0 si queda instalado.
fn install_vigem() -> i32 {
    match vigem::install() {
        Ok(_) if vigem::available() => 0,
        Ok(code) => code.max(1) as i32,
        Err(e) => {
            gpr_platform::log(&format!("ViGEmBus: {e}"));
            1
        }
    }
}

//! Rutas de la aplicación.

use std::path::PathBuf;

use gpr_core::brand::APP_ID;

/// Carpeta de datos (estado y registro del driver), se use el Setup o la versión portable.
pub fn data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join(APP_ID)
}

pub fn status_file() -> PathBuf {
    data_dir().join("status.json")
}

pub fn log_file() -> PathBuf {
    data_dir().join("driver.log")
}

pub fn settings_file() -> PathBuf {
    data_dir().join("settings.json")
}

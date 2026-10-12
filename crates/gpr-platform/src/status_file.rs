//! El estado que publica el driver, en disco (lo lee la app).

use gpr_core::status::DriverStatus;

use crate::paths::{data_dir, status_file};

pub fn write(st: &DriverStatus) -> bool {
    let _ = std::fs::create_dir_all(data_dir());
    let path = status_file();
    let tmp = path.with_extension("tmp");
    let Ok(json) = serde_json::to_string(st) else { return false };
    // escritura atómica; si el cambio de nombre falla (archivo abierto por otro), escritura directa
    (std::fs::write(&tmp, &json).is_ok() && std::fs::rename(&tmp, &path).is_ok()) || std::fs::write(&path, &json).is_ok()
}

pub fn read() -> Option<DriverStatus> {
    serde_json::from_str(&std::fs::read_to_string(status_file()).ok()?).ok()
}

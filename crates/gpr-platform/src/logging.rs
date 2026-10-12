//! Registro de actividad del driver (driver.log, rota a los 256 KB).

use std::io::Write;

use crate::paths::{data_dir, log_file};

const MAX_LOG_BYTES: u64 = 256 * 1024;

pub fn log(msg: &str) {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = LOCK.lock();
    let _ = std::fs::create_dir_all(data_dir());
    let path = log_file();
    if path.metadata().map(|m| m.len() > MAX_LOG_BYTES).unwrap_or(false) {
        let _ = std::fs::rename(&path, path.with_extension("log.1"));
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
        let _ = writeln!(
            f,
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}  {msg}",
            t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond
        );
    }
}

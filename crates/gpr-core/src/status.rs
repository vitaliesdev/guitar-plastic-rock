//! Estado que el driver publica cada segundo para la app (status.json en la carpeta de datos).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Segundos sin publicar a partir de los cuales el driver se considera colgado.
pub const STALE_AFTER_SECS: u64 = 5;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub name: String,
    /// "buscando" | "conectado" | "desconectado" | "detenido"
    pub status: String,
    pub usb_pid: Option<u16>,
    pub xinput_slot: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DriverStatus {
    pub version: String,
    /// Segundos Unix de la última publicación: si se queda atrás, el driver no responde.
    #[serde(default)]
    pub updated: u64,
    pub pid: u32,
    pub vigem: bool,
    pub error: Option<String>,
    /// Por clave de guitarra (`Kind::key`)
    pub devices: BTreeMap<String, DeviceStatus>,
}

impl DriverStatus {
    pub fn is_stale(&self, now: u64) -> bool {
        now.saturating_sub(self.updated) > STALE_AFTER_SECS
    }
}

pub fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

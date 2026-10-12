//! Preferencias del usuario (settings.json en la carpeta de datos) e idioma de Windows.

use gpr_i18n::Lang;
use serde::{Deserialize, Serialize};

use crate::paths::{data_dir, settings_file};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Código de idioma elegido (`Lang::code`); `None` = el de Windows
    #[serde(default)]
    pub language: Option<String>,
}

impl Settings {
    pub fn load() -> Settings {
        std::fs::read_to_string(settings_file()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        std::fs::create_dir_all(data_dir()).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(settings_file(), json).map_err(|e| e.to_string())
    }

    /// Idioma elegido, si el usuario eligió uno.
    pub fn chosen_language(&self) -> Option<Lang> {
        self.language.as_deref().and_then(Lang::from_code)
    }
}

/// Idioma de la interfaz de Windows, si está entre los traducidos.
pub fn windows_language() -> Option<Lang> {
    use windows::Win32::Globalization::GetUserDefaultUILanguage;
    let id = unsafe { GetUserDefaultUILanguage() };
    // idioma principal = 10 bits bajos del LANGID
    let code = match id & 0x3ff {
        0x0a => "es",
        0x09 => "en",
        0x19 => "ru",
        0x0c => "fr",
        0x07 => "de",
        0x10 => "it",
        0x16 => "pt",
        0x15 => "pl",
        0x22 => "uk",
        _ => return None,
    };
    Lang::from_code(code)
}

/// Idioma automático: el de Windows o, si no está traducido, inglés.
pub fn auto_language() -> Lang {
    windows_language().unwrap_or(Lang::FALLBACK)
}

/// Idioma en uso: el elegido en Ajustes o el automático.
pub fn language() -> Lang {
    Settings::load().chosen_language().unwrap_or_else(auto_language)
}

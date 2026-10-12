//! Textos de la interfaz en 9 idiomas (los mismos que Deadhunt Remastered).
//!
//! Cada idioma es una constante [`Texts`] en su propio archivo: si falta un texto, no compila.
//! Los huecos se escriben `{nombre}` y se rellenan con [`fill`]. Para los números se usa siempre la
//! forma «etiqueta: {n}», que evita las reglas de plural (ruso, polaco, ucraniano…).
//!
//! Para añadir un idioma: un archivo nuevo copiando `en.rs`, una variante en [`Lang`] y sus brazos
//! en `code`, `texts` y `from_code`; en el instalador, su línea en `[Languages]` y sus mensajes.

mod de;
mod en;
mod es;
mod fr;
mod it;
mod pl;
mod pt_br;
mod ru;
mod uk;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lang {
    Es,
    En,
    Ru,
    Fr,
    De,
    It,
    PtBr,
    Pl,
    Uk,
}

impl Lang {
    pub const ALL: [Lang; 9] = [Lang::Es, Lang::En, Lang::Ru, Lang::Fr, Lang::De, Lang::It, Lang::PtBr, Lang::Pl, Lang::Uk];

    /// Idioma por defecto si el de Windows no está traducido.
    pub const FALLBACK: Lang = Lang::En;

    /// Código guardado en los ajustes.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
            Lang::Ru => "ru",
            Lang::Fr => "fr",
            Lang::De => "de",
            Lang::It => "it",
            Lang::PtBr => "pt-BR",
            Lang::Pl => "pl",
            Lang::Uk => "uk",
        }
    }

    /// Acepta el código propio o uno de idioma de Windows ("es-ES", "pt", "uk-UA"…).
    pub fn from_code(code: &str) -> Option<Lang> {
        let primary = code.split(['-', '_']).next()?.to_ascii_lowercase();
        Lang::ALL.into_iter().find(|l| l.code().split('-').next() == Some(primary.as_str()))
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::Es => &es::TEXTS,
            Lang::En => &en::TEXTS,
            Lang::Ru => &ru::TEXTS,
            Lang::Fr => &fr::TEXTS,
            Lang::De => &de::TEXTS,
            Lang::It => &it::TEXTS,
            Lang::PtBr => &pt_br::TEXTS,
            Lang::Pl => &pl::TEXTS,
            Lang::Uk => &uk::TEXTS,
        }
    }

    /// Nombre del idioma en ese idioma.
    pub fn native_name(self) -> &'static str {
        self.texts().lang_name
    }
}

/// Rellena los huecos `{clave}` de un texto.
pub fn fill(text: &str, args: &[(&str, &dyn std::fmt::Display)]) -> String {
    let mut out = text.to_string();
    for (key, value) in args {
        out = out.replace(&format!("{{{key}}}"), &value.to_string());
    }
    out
}

/// Todos los textos de la interfaz. Los marcados con huecos indican cuáles admiten.
pub struct Texts {
    /// Nombre del idioma en ese idioma
    pub lang_name: &'static str,

    // ---- selector de guitarra ----
    pub not_detected: &'static str,
    pub guitar_off: &'static str,
    pub connected: &'static str,
    /// {m} = PS4 | PS5
    pub mode: &'static str,
    /// {n} = número de mando
    pub pad: &'static str,

    // ---- avisos ----
    pub vigem_missing_title: &'static str,
    pub vigem_missing_text: &'static str,
    pub vigem_installing_title: &'static str,
    pub vigem_installing_text: &'static str,
    pub driver_stopped_title: &'static str,
    pub driver_stopped_text: &'static str,
    pub driver_stale_title: &'static str,
    pub driver_stale_text: &'static str,
    pub install: &'static str,
    pub start: &'static str,
    pub restart: &'static str,

    // ---- guitarra ----
    pub connect_title: &'static str,
    pub connect_text: &'static str,
    pub power_on_title: &'static str,
    pub power_on_text: &'static str,
    /// {n} = trastes del acorde
    pub chord: &'static str,

    // ---- medidores ----
    pub whammy: &'static str,
    pub tilt: &'static str,
    pub pickup: &'static str,

    // ---- barra de comandos ----
    pub status_stopped: &'static str,
    pub status_stale: &'static str,
    pub status_no_vigem: &'static str,
    /// {n} = mandos Xbox creados
    pub status_active: &'static str,
    pub autostart: &'static str,
    pub autostart_elsewhere: &'static str,

    // ---- ajustes ----
    pub settings: &'static str,
    pub driver: &'static str,
    pub driver_state_stopped: &'static str,
    pub driver_state_stale: &'static str,
    /// {n} = mandos virtuales
    pub driver_state_running: &'static str,
    pub vigem_state_installing: &'static str,
    pub vigem_state_installed: &'static str,
    pub vigem_state_missing: &'static str,
    pub language: &'static str,
    /// {lang} = idioma detectado
    pub language_auto: &'static str,
    pub tech: &'static str,
    pub tech_text: &'static str,
    /// {name} = guitarra, {n} = número de mando
    pub tech_pad: &'static str,
    /// {name} = guitarra
    pub tech_no_pad: &'static str,
    /// {path} = ruta del registro
    pub tech_log: &'static str,
    pub uninstall: &'static str,
    /// {app} = nombre de la aplicación
    pub uninstall_text: &'static str,
    pub uninstall_button: &'static str,
    pub portable: &'static str,
    pub portable_text: &'static str,

    // ---- errores ----
    pub err_driver_start: &'static str,
    pub err_vigem_cancelled: &'static str,
    /// {e} = detalle del sistema
    pub err_vigem_extract: &'static str,
    /// {code} = código de salida del instalador
    pub err_vigem_failed: &'static str,
    /// {e} = detalle del sistema
    pub err_generic: &'static str,
    pub err_no_renderer: &'static str,

    // ---- bandeja del sistema ----
    /// {app} = nombre de la aplicación
    pub tray_open: &'static str,
    pub tray_stop: &'static str,
    pub tray_no_guitar: &'static str,

    // ---- controles (zonas de la foto) ----
    pub white: &'static str,
    pub black: &'static str,
    pub green: &'static str,
    pub red: &'static str,
    pub yellow: &'static str,
    pub blue: &'static str,
    pub orange: &'static str,
    /// {fret} = color del traste
    pub solo: &'static str,
    pub strum: &'static str,
    pub pause: &'static str,
    pub dpad: &'static str,
    pub joystick: &'static str,
    pub left_stick: &'static str,
    pub right_stick: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_round_trip_and_windows_codes_map() {
        for l in Lang::ALL {
            assert_eq!(Lang::from_code(l.code()), Some(l));
        }
        assert_eq!(Lang::from_code("es-MX"), Some(Lang::Es));
        assert_eq!(Lang::from_code("pt-PT"), Some(Lang::PtBr));
        assert_eq!(Lang::from_code("uk-UA"), Some(Lang::Uk));
        assert_eq!(Lang::from_code("ja-JP"), None);
    }

    /// Los huecos de cada texto deben ser los mismos en todos los idiomas.
    #[test]
    fn placeholders_match_english() {
        fn holes(s: &str) -> Vec<String> {
            let mut v: Vec<String> = s.split('{').skip(1).filter_map(|p| p.split('}').next()).map(str::to_string).collect();
            v.sort();
            v
        }
        let en = Lang::En.texts();
        for l in Lang::ALL {
            let t = l.texts();
            let pairs = [
                (en.mode, t.mode),
                (en.pad, t.pad),
                (en.chord, t.chord),
                (en.status_active, t.status_active),
                (en.driver_state_running, t.driver_state_running),
                (en.language_auto, t.language_auto),
                (en.tech_pad, t.tech_pad),
                (en.tech_no_pad, t.tech_no_pad),
                (en.tech_log, t.tech_log),
                (en.uninstall_text, t.uninstall_text),
                (en.err_vigem_extract, t.err_vigem_extract),
                (en.err_vigem_failed, t.err_vigem_failed),
                (en.err_generic, t.err_generic),
                (en.tray_open, t.tray_open),
                (en.solo, t.solo),
            ];
            for (a, b) in pairs {
                assert_eq!(holes(a), holes(b), "{:?}: «{b}»", l);
            }
        }
    }
}

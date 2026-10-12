//! Tema de Windows 11: modo claro/oscuro y color de acento del sistema, con los colores de las
//! superficies Fluent. También pone la barra de título en modo oscuro y con material Mica.
//! Las fuentes del sistema (Segoe UI Variable, Segoe Fluent Icons) están en [`fonts`].

pub mod fonts;

use eframe::egui::Color32;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

#[derive(Clone, Copy, PartialEq)]
pub struct Palette {
    pub dark: bool,
    /// Fondo de la ventana (color sólido de Mica)
    pub bg: Color32,
    /// Capa de tarjetas y barra de comandos
    pub layer: Color32,
    /// Tarjetas destacadas y botones
    pub card: Color32,
    pub card_hover: Color32,
    pub stroke: Color32,
    pub text: Color32,
    pub text2: Color32,
    pub text3: Color32,
    pub track: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub warn_bg: Color32,
    pub warn_fg: Color32,
    pub ok: Color32,
    pub off: Color32,
    pub shadow: Color32,
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

fn system_dark() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
        .and_then(|k| k.get_value::<u32, _>("AppsUseLightTheme"))
        .map(|v| v == 0)
        .unwrap_or(true)
}

/// Color de acento como lo usa Windows 11: un tono claro de la paleta en modo oscuro (Light2)
/// y uno oscuro en modo claro (Dark1).
fn system_accent(dark: bool) -> Color32 {
    let palette: Option<Vec<u8>> = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent")
        .and_then(|k| k.get_raw_value("AccentPalette"))
        .ok()
        .map(|v| v.bytes.into_owned());
    let index = if dark { 1 } else { 4 };
    match palette {
        Some(b) if b.len() >= 32 => Color32::from_rgb(b[index * 4], b[index * 4 + 1], b[index * 4 + 2]),
        _ if dark => rgb(0x60cdff),
        _ => rgb(0x005fb8),
    }
}

pub fn current() -> Palette {
    let dark = system_dark();
    let accent = system_accent(dark);
    // texto sobre el acento: negro o blanco según su luminosidad
    let lum = 0.299 * accent.r() as f32 + 0.587 * accent.g() as f32 + 0.114 * accent.b() as f32;
    let on_accent = if lum > 150.0 { Color32::BLACK } else { Color32::WHITE };
    if dark {
        Palette {
            dark,
            bg: rgb(0x202020),
            layer: Color32::from_rgba_unmultiplied(255, 255, 255, 9),
            card: Color32::from_rgba_unmultiplied(255, 255, 255, 15),
            card_hover: Color32::from_rgba_unmultiplied(255, 255, 255, 24),
            stroke: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
            text: Color32::WHITE,
            text2: Color32::from_rgba_unmultiplied(255, 255, 255, 200),
            text3: Color32::from_rgba_unmultiplied(255, 255, 255, 120),
            track: Color32::from_rgba_unmultiplied(255, 255, 255, 30),
            accent,
            on_accent,
            warn_bg: rgb(0x433519),
            warn_fg: rgb(0xfce100),
            ok: rgb(0x6ccb5f),
            off: rgb(0x8a8a8a),
            shadow: Color32::from_black_alpha(110),
        }
    } else {
        Palette {
            dark,
            bg: rgb(0xf3f3f3),
            layer: Color32::from_rgba_unmultiplied(255, 255, 255, 160),
            card: rgb(0xfbfbfb),
            card_hover: rgb(0xf6f6f6),
            stroke: Color32::from_rgba_unmultiplied(0, 0, 0, 18),
            text: rgb(0x1a1a1a),
            text2: rgb(0x5d5d5d),
            text3: rgb(0x8a8a8a),
            track: Color32::from_rgba_unmultiplied(0, 0, 0, 34),
            accent,
            on_accent,
            warn_bg: rgb(0xfff4ce),
            warn_fg: rgb(0x9d5d00),
            ok: rgb(0x0f7b0f),
            off: rgb(0x8a8a8a),
            shadow: Color32::from_black_alpha(40),
        }
    }
}

/// Barra de título nativa en el modo del sistema y con material Mica (Windows 11).
pub fn apply_titlebar(hwnd: isize, dark: bool) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMSBT_MAINWINDOW, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
    };
    let hwnd = HWND(hwnd as *mut core::ffi::c_void);
    let dark_flag: i32 = dark.into();
    let backdrop = DWMSBT_MAINWINDOW;
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_flag as *const i32 as *const core::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_SYSTEMBACKDROP_TYPE,
            &backdrop as *const _ as *const core::ffi::c_void,
            std::mem::size_of_val(&backdrop) as u32,
        );
    }
}

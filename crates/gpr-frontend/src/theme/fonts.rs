//! Fuentes del sistema (Segoe UI Variable, Segoe UI Semibold, Segoe Fluent Icons, Cascadia Mono)
//! y glifos de iconos. Se cargan de C:\Windows\Fonts: no se incrustan en el .exe.

use std::sync::Arc;

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId};

/// Glifos de Segoe Fluent Icons / Segoe MDL2 Assets.
pub mod glyph {
    pub const GEAR: &str = "\u{E713}";
    pub const CLOSE: &str = "\u{E8BB}";
    pub const WARN: &str = "\u{E7BA}";
    pub const CHECK: &str = "\u{E73E}";
    pub const DOWN: &str = "\u{E70D}";
    pub const UP: &str = "\u{E70E}";
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into());
    let load = |name: &str| std::fs::read(format!(r"{windir}\Fonts\{name}")).ok();
    let mut add = |name: &str, data: Vec<u8>, family: FontFamily, first: bool| {
        fonts.font_data.insert(name.into(), Arc::new(FontData::from_owned(data)));
        let fam = fonts.families.entry(family).or_default();
        if first {
            fam.insert(0, name.into());
        } else {
            fam.push(name.into());
        }
    };
    if let Some(regular) = load("SegUIVar.ttf").or_else(|| load("segoeui.ttf")) {
        add("segoe", regular, FontFamily::Proportional, true);
    }
    if let Some(b) = load("seguisb.ttf").or_else(|| load("segoeuib.ttf")) {
        add("segoe-semibold", b, FontFamily::Name("bold".into()), true);
    }
    if let Some(icons) = load("SegoeIcons.ttf").or_else(|| load("segmdl2.ttf")) {
        add("icons", icons, FontFamily::Name("icons".into()), true);
    }
    if let Some(sym) = load("seguisym.ttf") {
        add("symbols", sym, FontFamily::Proportional, false);
    }
    if let Some(mono) = load("CascadiaMono.ttf").or_else(|| load("consola.ttf")) {
        add("mono", mono, FontFamily::Monospace, true);
    }
    // familias propias con las de serie como respaldo
    let proportional = fonts.families[&FontFamily::Proportional].clone();
    for fam in ["bold", "icons"] {
        fonts.families.entry(FontFamily::Name(fam.into())).or_default().extend(proportional.iter().cloned());
    }
    ctx.set_fonts(fonts);
}

pub fn body(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("bold".into()))
}

pub fn icon(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("icons".into()))
}

pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

//! Frontend: la ventana de la app, con el lenguaje de Windows 11 (Fluent).
//!
//! - [`app`]: estado de la app y composición de la ventana
//! - `model`: estado en directo de cada guitarra y del driver
//! - `guitars`: lo específico de cada guitarra (foto, zonas iluminables, colores, demo)
//! - `views`: cada zona de la ventana (selector, aviso, guitarra, medidores, barra, ajustes)
//! - `widgets`, `theme`, `images`: piezas de interfaz, colores y fuentes del sistema, imágenes

mod app;
mod demo;
mod guitars;
mod images;
mod model;
mod theme;
mod views;
mod widgets;

use gpr_core::brand::{APP_ID, DISPLAY_NAME};

/// Tamaño de la ventana, en puntos.
pub const WINDOW: [f32; 2] = [880.0, 560.0];

/// Abre la ventana y la mantiene hasta que se cierra. Devuelve el código de salida.
pub fn run() -> i32 {
    // Direct3D/Vulkan primero (en Windows sin driver gráfico hay adaptador por software);
    // si no hay ninguno, OpenGL; si tampoco, se avisa en vez de cerrarse sin decir nada
    let mut errors = Vec::new();
    for renderer in [eframe::Renderer::Wgpu, eframe::Renderer::Glow] {
        let options = eframe::NativeOptions {
            renderer,
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title(DISPLAY_NAME)
                .with_inner_size(WINDOW)
                .with_resizable(false)
                .with_maximize_button(false)
                .with_icon(images::window_icon()),
            ..Default::default()
        };
        match eframe::run_native(APP_ID, options, Box::new(|cc| Ok(Box::new(app::App::new(cc))))) {
            Ok(()) => return 0,
            Err(e) => {
                gpr_platform::log(&format!("no se pudo abrir la ventana con {renderer:?}: {e}"));
                errors.push(format!("{renderer:?}: {e}"));
            }
        }
    }
    gpr_platform::process::error_box(
        DISPLAY_NAME,
        &format!("{}\n\n{}", gpr_platform::settings::language().texts().err_no_renderer, errors.join("\n")),
    );
    1
}

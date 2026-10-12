//! Decodificación de imágenes incrustadas: fotos de las guitarras, miniaturas e icono de ventana.

use eframe::egui;

/// Decodifica un PNG a RGBA (ancho, alto, píxeles).
pub fn rgba(png: &[u8]) -> (u32, u32, Vec<u8>) {
    let img = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .expect("PNG incrustado válido")
        .to_rgba8();
    (img.width(), img.height(), img.into_raw())
}

/// Miniatura de buena calidad que cabe en `max_w`×`max_h` píxeles reales: Lanczos sobre alfa
/// premultiplicado (sin halos oscuros en el contorno). Devuelve RGBA premultiplicado.
/// `rim`: opacidad (0-255) de un contorno blanco de 1 px, para que una guitarra negra se
/// distinga sobre fondo oscuro (0 = sin contorno).
pub fn thumbnail(png: &[u8], max_w: u32, max_h: u32, rim: u8) -> (u32, u32, Vec<u8>) {
    let mut img = image::load_from_memory_with_format(png, image::ImageFormat::Png)
        .expect("PNG incrustado válido")
        .to_rgba8();
    for p in img.pixels_mut() {
        let a = p[3] as u32;
        for c in 0..3 {
            p[c] = ((p[c] as u32 * a + 127) / 255) as u8;
        }
    }
    let img = image::DynamicImage::ImageRgba8(img)
        .resize(max_w.max(1), max_h.max(1), image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (w, h) = (img.width() as i32, img.height() as i32);
    let src = img.into_raw();
    if rim == 0 {
        return (w as u32, h as u32, src);
    }
    // contorno: alfa máximo de los 8 vecinos, compuesto por debajo de la guitarra
    let alpha = |x: i32, y: i32| if x < 0 || y < 0 || x >= w || y >= h { 0 } else { src[((y * w + x) * 4 + 3) as usize] as u32 };
    let mut out = src.clone();
    for y in 0..h {
        for x in 0..w {
            let mut a = 0;
            for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
                a = a.max(alpha(x + dx, y + dy));
            }
            let under = a * rim as u32 / 255; // blanco premultiplicado: r = g = b = a
            let i = ((y * w + x) * 4) as usize;
            let top = src[i + 3] as u32;
            for c in 0..4 {
                out[i + c] = (src[i + c] as u32 + under * (255 - top) / 255).min(255) as u8;
            }
        }
    }
    (w as u32, h as u32, out)
}

pub fn window_icon() -> egui::IconData {
    let (width, height, rgba) = rgba(gpr_core::brand::ICON_PNG);
    egui::IconData { rgba, width, height }
}

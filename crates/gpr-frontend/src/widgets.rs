//! Controles Fluent dibujados a mano (botón, interruptor, botón de icono, medidor, sombra).

use eframe::egui::{self, pos2, Align2, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Ui};

use crate::theme::fonts::{body, icon};
use crate::theme::Palette;

pub fn text_width(p: &egui::Painter, text: &str, font: FontId) -> f32 {
    p.layout_no_wrap(text.to_owned(), font, Color32::WHITE).size().x
}

/// Ancho de un botón para su texto (los textos cambian de largo con el idioma).
pub fn button_width(p: &egui::Painter, label: &str, min: f32) -> f32 {
    (text_width(p, label, body(13.0)) + 28.0).max(min)
}

/// Botón estándar o de acento.
pub fn button(ui: &mut Ui, pal: &Palette, r: Rect, label: &str, accent: bool, id: &str) -> bool {
    let resp = ui.interact(r, ui.id().with(id), Sense::click());
    let p = ui.painter();
    let (mut fill, text, stroke) = if accent {
        (pal.accent, pal.on_accent, Stroke::NONE)
    } else {
        (pal.card, pal.text, Stroke::new(1.0, pal.stroke))
    };
    if resp.hovered() {
        fill = if accent { fill.gamma_multiply(0.9) } else { pal.card_hover };
    }
    if resp.is_pointer_button_down_on() {
        fill = fill.gamma_multiply(0.82);
    }
    p.rect(r, CornerRadius::same(5), fill, stroke, StrokeKind::Inside);
    p.text(r.center(), Align2::CENTER_CENTER, label, body(13.0), text);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Interruptor.
pub fn toggle(ui: &mut Ui, pal: &Palette, r: Rect, on: bool, id: &str) -> bool {
    let resp = ui.interact(r, ui.id().with(id), Sense::click());
    let t = ui.ctx().animate_bool_with_time(ui.id().with((id, "anim")), on, 0.12);
    let p = ui.painter();
    let rad = r.height() / 2.0;
    if on {
        p.rect_filled(r, CornerRadius::same(rad as u8), pal.accent);
    } else {
        p.rect_stroke(r, CornerRadius::same(rad as u8), Stroke::new(1.0, pal.text2), StrokeKind::Inside);
    }
    let knob_r = if resp.hovered() { 7.0 } else { 6.0 };
    let x = egui::lerp((r.left() + rad)..=(r.right() - rad), t);
    p.circle_filled(pos2(x, r.center().y), knob_r, if on { pal.on_accent } else { pal.text2 });
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Botón con solo un icono (sin fondo hasta pasar el ratón).
pub fn icon_button(ui: &mut Ui, pal: &Palette, r: Rect, glyph: &str, id: &str) -> bool {
    let resp = ui.interact(r, ui.id().with(id), Sense::click());
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(r, CornerRadius::same(5), pal.card_hover);
    }
    p.text(r.center(), Align2::CENTER_CENTER, glyph, icon(16.0), if resp.hovered() { pal.text } else { pal.text2 });
    resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked()
}

/// Barra de progreso de `from` a `to` (0..1); `centered` marca el centro.
pub fn meter(p: &egui::Painter, pal: &Palette, r: Rect, from: f32, to: f32, centered: bool) {
    p.rect_filled(r, CornerRadius::same(2), pal.track);
    let (a, b) = (from.min(to).clamp(0.0, 1.0), from.max(to).clamp(0.0, 1.0));
    if b > a {
        let fill = Rect::from_min_max(pos2(r.left() + r.width() * a, r.top()), pos2(r.left() + r.width() * b, r.bottom()));
        p.rect_filled(fill, CornerRadius::same(2), pal.accent);
    }
    if centered {
        let x = r.center().x;
        p.line_segment([pos2(x, r.top() - 3.0), pos2(x, r.bottom() + 3.0)], Stroke::new(1.0, pal.text3));
    }
}

/// Sombra suave bajo una superficie flotante.
pub fn shadow(p: &egui::Painter, pal: &Palette, r: Rect, radius: u8) {
    for (i, a) in [(10.0, 0.18), (6.0, 0.28), (3.0, 0.4)] {
        p.rect_filled(r.expand(i), CornerRadius::same(radius + i as u8), pal.shadow.gamma_multiply(a));
    }
}

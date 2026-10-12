//! Selector de guitarra: miniatura, nombre y estado de cada una, con indicador animado.

use eframe::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, Id, Rect, Sense, Ui};

use gpr_i18n::fill;

use crate::app::App;
use crate::guitars;
use crate::model::THUMB;
use crate::theme::fonts::body;
use crate::theme::Palette;
use crate::widgets::text_width;

impl App {
    pub(crate) fn draw_selector(&mut self, ui: &mut Ui, pal: &Palette, full: Rect, y: f32) {
        let ctx = ui.ctx().clone();
        let t = self.t();
        let mut x = full.left() + 14.0;
        for i in 0..self.guitars.len() {
            let (kind, thumb_tex) = {
                let g = &mut self.guitars[i];
                (g.kind, g.thumb(&ctx, pal.dark).clone())
            };
            let (connected, detail) = {
                let l = self.guitars[i].live.lock().unwrap();
                let ro = guitars::readout(&l.state);
                let slot = self.driver.slot(kind).map(|s| fill(t.pad, &[("n", &(s + 1))]));
                let mode = ro.mode.map(|m| fill(t.mode, &[("m", &m)]));
                let detail = if !l.connected() {
                    t.not_detected.to_string()
                } else if !ro.powered {
                    t.guitar_off.to_string()
                } else {
                    match (mode, slot) {
                        (Some(m), Some(s)) => format!("{m} · {s}"),
                        (Some(m), None) => m,
                        (None, Some(s)) => s,
                        (None, None) => t.connected.to_string(),
                    }
                };
                (l.connected() && ro.powered, detail)
            };
            let p = ui.painter().clone();
            let name_w = text_width(&p, kind.name(), body(14.0)).max(text_width(&p, &detail, body(11.5)) + 12.0);
            let r = Rect::from_min_size(pos2(x, y), vec2(THUMB.x + 10.0 + name_w + 28.0, 54.0));
            let resp = ui.interact(r, ui.id().with(("sel", i)), Sense::click());
            if resp.clicked() {
                self.select(i, true);
            }
            let selected = i == self.selected;
            if resp.hovered() {
                p.rect_filled(r, CornerRadius::same(6), pal.layer);
            }
            let thumb = Rect::from_min_size(pos2(r.left() + 12.0, r.top() + 14.0), THUMB);
            // 1 texel = 1 píxel de pantalla: tamaño en píxeles reales y esquina alineada a la rejilla
            let ppp = ctx.pixels_per_point();
            let [tw, th] = thumb_tex.size();
            let size = vec2(tw as f32, th as f32) / ppp;
            let min = ((thumb.center() - size / 2.0) * ppp).round() / ppp;
            let img = Rect::from_min_size(min, size);
            let tint = if connected { Color32::WHITE } else { Color32::from_white_alpha(120) };
            p.image(thumb_tex.id(), img, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
            let tx = thumb.right() + 10.0;
            p.text(pos2(tx, r.top() + 9.0), Align2::LEFT_TOP, kind.name(), body(14.0), if selected { pal.text } else { pal.text2 });
            p.circle_filled(pos2(tx + 3.0, r.top() + 36.0), 3.0, if connected { pal.ok } else { pal.off });
            p.text(pos2(tx + 11.0, r.top() + 29.0), Align2::LEFT_TOP, &detail, body(11.5), pal.text3);
            let w = ctx.animate_value_with_time(Id::new(("ind", i)), if selected { 18.0 } else { 0.0 }, 0.18);
            if w > 0.5 {
                let ind = Rect::from_center_size(pos2(r.center().x, r.bottom() - 2.0), vec2(w, 3.0));
                p.rect_filled(ind, CornerRadius::same(2), pal.accent);
            }
            resp.on_hover_cursor(egui::CursorIcon::PointingHand);
            x = r.right() + 4.0;
        }
    }
}

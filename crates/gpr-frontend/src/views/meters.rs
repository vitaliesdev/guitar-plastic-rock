//! Medidores: whammy, inclinación (con el aviso de Hero Power / Overdrive) y pastilla.

use eframe::egui::{pos2, vec2, Align2, CornerRadius, Id, Rect, Stroke, Ui};
use gpr_core::instruments::State;

use crate::app::App;
use crate::guitars;
use crate::theme::fonts::body;
use crate::theme::Palette;
use crate::widgets::{meter, text_width};

/// Umbrales del aviso de inclinación (histéresis para que no parpadee).
const TILT_ON: f32 = 0.6;
const TILT_OFF: f32 = 0.45;

impl App {
    pub(crate) fn draw_meters(&mut self, ui: &mut Ui, pal: &Palette, r: Rect, state: &State, connected: bool) {
        let p = ui.painter().clone();
        let t = self.t();
        let ro = guitars::readout(state);
        let label = guitars::visual(state.kind()).tilt_label;
        let live = connected && ro.powered;
        // la inclinación se suaviza (el sensor de la Riffmaster va por escalones)
        self.tilt_smooth += ((if live { ro.tilt } else { 0.0 }) - self.tilt_smooth) * 0.35;
        self.tilted = live && self.tilt_smooth > if self.tilted { TILT_OFF } else { TILT_ON };

        let pk_w = if ro.pickup.is_some() { 96.0 } else { 0.0 };
        let gap = 28.0;
        let col_w = (r.width() - pk_w - gap * if ro.pickup.is_some() { 2.0 } else { 1.0 }) / 2.0;
        let top = r.top() + 4.0;
        let track_y = r.bottom() - 6.0;
        let fmt = |v: f32, signed: bool| {
            if !live {
                "—".to_string()
            } else if signed {
                format!("{:+.0} %", v * 100.0)
            } else {
                format!("{:.0} %", v * 100.0)
            }
        };

        // whammy
        let x0 = r.left();
        p.text(pos2(x0, top), Align2::LEFT_TOP, t.whammy, body(12.0), pal.text2);
        p.text(pos2(x0 + col_w, top), Align2::RIGHT_TOP, fmt(ro.whammy, false), body(12.0), pal.text);
        meter(&p, pal, Rect::from_min_size(pos2(x0, track_y), vec2(col_w, 4.0)), 0.0, if live { ro.whammy } else { 0.0 }, false);

        // inclinación
        let x1 = x0 + col_w + gap;
        p.text(pos2(x1, top), Align2::LEFT_TOP, t.tilt, body(12.0), pal.text2);
        let pill_a = ui.ctx().animate_bool_with_time(Id::new("pill"), self.tilted, 0.15);
        if pill_a > 0.01 {
            let pw = text_width(&p, label, body(10.5)) + 14.0;
            let px = x1 + text_width(&p, t.tilt, body(12.0)) + 10.0;
            let pr = Rect::from_min_size(pos2(px, top - 1.0), vec2(pw, 17.0));
            p.rect_filled(pr, CornerRadius::same(8), pal.accent.gamma_multiply(pill_a));
            p.text(pr.center(), Align2::CENTER_CENTER, label, body(10.5), pal.on_accent.gamma_multiply(pill_a));
        }
        p.text(pos2(x1 + col_w, top), Align2::RIGHT_TOP, fmt(ro.tilt, ro.tilt_centered), body(12.0), pal.text);
        let tr = Rect::from_min_size(pos2(x1, track_y), vec2(col_w, 4.0));
        if ro.tilt_centered {
            meter(&p, pal, tr, 0.5, 0.5 + self.tilt_smooth / 2.0, true);
        } else {
            meter(&p, pal, tr, 0.0, self.tilt_smooth, false);
        }

        // pastilla (solo si la guitarra la tiene)
        if let Some((pk, available)) = ro.pickup {
            let x2 = x1 + col_w + gap;
            p.text(pos2(x2, top), Align2::LEFT_TOP, t.pickup, body(12.0), pal.text2);
            for i in 0..5u8 {
                let c = pos2(x2 + 5.0 + i as f32 * 16.0, track_y + 1.0);
                if live && available && i == pk {
                    p.circle_filled(c, 5.0, pal.accent);
                } else {
                    p.circle_stroke(c, 4.5, Stroke::new(1.0, pal.text3));
                }
            }
        }
    }
}

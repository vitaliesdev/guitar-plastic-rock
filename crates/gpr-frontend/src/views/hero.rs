//! La foto de la guitarra con cada control iluminándose al pulsarlo, el aviso de acorde y la
//! asignación Xbox del control bajo el ratón.

use eframe::egui::{pos2, vec2, Align2, Color32, CornerRadius, Id, LayerId, Order, Rect, Sense, Stroke, StrokeKind, Ui};
use gpr_core::instruments::State;

use crate::app::App;
use crate::guitars::{self, Shape};
use crate::theme::fonts::{body, glyph, icon, semibold};
use crate::theme::Palette;
use crate::widgets::{shadow, text_width};

/// Duración del aviso de acorde, en segundos.
const TOAST_SECS: f32 = 1.8;

impl App {
    pub(crate) fn draw_hero(&mut self, ui: &mut Ui, pal: &Palette, area: Rect, state: &State, connected: bool) {
        let ctx = ui.ctx().clone();
        let t = self.t();
        let g = &self.guitars[self.selected];
        let visual = guitars::visual(g.kind);
        let (iw, ih) = visual.size;
        let f = (area.width() * 0.96 / iw).min(area.height() / ih);
        let img = Rect::from_center_size(area.center(), vec2(iw * f, ih * f));
        let live = connected && guitars::readout(state).powered;
        let tint = if live { Color32::WHITE } else { Color32::from_white_alpha(if pal.dark { 30 } else { 55 }) };
        let p = ui.painter().clone();
        p.image(g.photo.id(), img, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
        let map = |x: f32, y: f32| img.min + vec2(x * f, y * f);

        if !live {
            let (title, sub) = if connected { (t.power_on_title, t.power_on_text) } else { (t.connect_title, t.connect_text) };
            p.text(area.center() - vec2(0.0, 12.0), Align2::CENTER_CENTER, title, semibold(18.0), pal.text);
            p.text(area.center() + vec2(0.0, 14.0), Align2::CENTER_CENTER, sub, body(13.0), pal.text2);
            return;
        }

        let mut hovered: Option<(guitars::Ctl, &str)> = None;
        for (idx, &(ctl, shape, mapping)) in visual.zones.iter().enumerate() {
            let (on, color, level) = guitars::control(state, ctl, pal);
            let hit = match shape {
                Shape::Rect(cx, cy, w, h) => Rect::from_min_max(map(cx - w / 2.0, cy - h / 2.0), map(cx + w / 2.0, cy + h / 2.0)),
                Shape::Circle(cx, cy, r) => Rect::from_center_size(map(cx, cy), vec2(2.0 * r * f, 2.0 * r * f)),
                Shape::Line(ax, ay, bx, by) => Rect::from_two_pos(map(ax, ay), map(bx, by)).expand(4.0),
            };
            if ui.interact(hit, ui.id().with(("zone", self.selected, idx)), Sense::hover()).hovered() {
                hovered = Some((ctl, mapping));
            }
            if !on {
                continue;
            }
            // un control negro necesita halo y borde claros para verse sobre la guitarra
            let dark_color = color.r() < 40 && color.g() < 40 && color.b() < 40;
            let glow = if dark_color { Color32::from_white_alpha(170) } else { color };
            match shape {
                Shape::Rect(..) => {
                    for (e, a) in [(9.0, 0.10), (6.0, 0.16), (3.0, 0.26)] {
                        p.rect_filled(hit.expand(e), CornerRadius::same((5.0 + e) as u8), glow.gamma_multiply(a));
                    }
                    let stroke = if dark_color { Stroke::new(1.5, Color32::from_white_alpha(220)) } else { Stroke::NONE };
                    p.rect(hit, CornerRadius::same(4), color, stroke, StrokeKind::Inside);
                }
                Shape::Circle(cx, cy, r) => {
                    let c = map(cx, cy);
                    for (e, a) in [(8.0, 0.10), (5.0, 0.18), (2.5, 0.28)] {
                        p.circle_filled(c, r * f + e, glow.gamma_multiply(a));
                    }
                    p.circle_filled(c, r * f, color);
                }
                Shape::Line(ax, ay, bx, by) => {
                    let (a, b) = (map(ax, ay), map(bx, by));
                    for (w, al) in [(16.0, 0.10), (10.0, 0.18), (6.0, 0.3)] {
                        p.line_segment([a, b], Stroke::new(w, glow.gamma_multiply(al * level)));
                    }
                    p.line_segment([a, b], Stroke::new(3.5, color.gamma_multiply(0.4 + 0.6 * level)));
                }
            }
        }
        guitars::draw_extras(&p, state, &map, pal);

        // aviso de acorde
        if let Some((text, t0)) = &self.toast {
            let age = t0.elapsed().as_secs_f32();
            if age > TOAST_SECS {
                self.toast = None;
            } else {
                let alpha = (age / 0.15).min(1.0).min((TOAST_SECS - age) / 0.3);
                let w = text_width(&p, text, body(13.0)) + 52.0;
                let r = Rect::from_center_size(pos2(area.center().x, area.top() + 16.0 + (1.0 - alpha) * -6.0), vec2(w, 34.0));
                let tp = ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("toast")));
                shadow(&tp, pal, r, 8);
                tp.rect(r, CornerRadius::same(8), pal.bg.gamma_multiply(alpha), Stroke::new(1.0, pal.stroke.gamma_multiply(alpha)), StrokeKind::Inside);
                tp.text(pos2(r.left() + 16.0, r.center().y), Align2::LEFT_CENTER, glyph::CHECK, icon(14.0), pal.ok.gamma_multiply(alpha));
                tp.text(pos2(r.left() + 38.0, r.center().y), Align2::LEFT_CENTER, text, body(13.0), pal.text.gamma_multiply(alpha));
            }
        }

        // asignación Xbox del control bajo el ratón
        if let (Some((ctl, mapping)), Some(pos)) = (hovered, ctx.pointer_hover_pos()) {
            if !self.settings_open {
                let tp = ctx.layer_painter(LayerId::new(Order::Tooltip, Id::new("zone-tip")));
                let a = format!("{}  →  ", guitars::ctl_name(ctl, t));
                let mapping = guitars::mapping(mapping, t);
                let mapping = mapping.as_str();
                let wa = text_width(&tp, &a, body(12.5));
                let wb = text_width(&tp, mapping, semibold(12.5));
                let r = Rect::from_min_size(pos + vec2(14.0, -34.0), vec2(wa + wb + 20.0, 26.0));
                shadow(&tp, pal, r, 5);
                tp.rect(r, CornerRadius::same(5), pal.bg, Stroke::new(1.0, pal.stroke), StrokeKind::Inside);
                tp.text(pos2(r.left() + 10.0, r.center().y), Align2::LEFT_CENTER, &a, body(12.5), pal.text2);
                tp.text(pos2(r.left() + 10.0 + wa, r.center().y), Align2::LEFT_CENTER, mapping, semibold(12.5), pal.accent);
            }
        }
    }
}

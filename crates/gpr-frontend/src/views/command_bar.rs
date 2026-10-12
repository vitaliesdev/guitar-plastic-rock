//! Barra de comandos: estado del driver, "Iniciar con Windows" y botón de ajustes.

use eframe::egui::{pos2, vec2, Align2, CornerRadius, Rect, Stroke, StrokeKind, Ui};

use gpr_i18n::fill;

use crate::app::App;
use crate::theme::fonts::{body, glyph};
use crate::theme::Palette;
use crate::widgets::{icon_button, toggle};

impl App {
    pub(crate) fn draw_command_bar(&mut self, ui: &mut Ui, pal: &Palette, r: Rect) {
        let ctx = ui.ctx().clone();
        let t = self.t();
        let p = ui.painter().clone();
        p.rect(r, CornerRadius::same(8), pal.layer, Stroke::new(1.0, pal.stroke), StrokeKind::Inside);
        let cy = r.center().y;
        let (dot, text) = if self.vigem_setup.is_some() {
            (pal.warn_fg, t.vigem_installing_title.to_string())
        } else if !self.driver.running {
            (pal.off, t.status_stopped.to_string())
        } else if self.driver.stale() {
            (pal.warn_fg, t.status_stale.to_string())
        } else if !self.vigem {
            (pal.warn_fg, t.status_no_vigem.to_string())
        } else {
            (pal.ok, fill(t.status_active, &[("n", &self.driver.pads())]))
        };
        p.circle_filled(pos2(r.left() + 18.0, cy), 3.5, dot);
        let msg = self.message.clone().unwrap_or(text);
        p.text(pos2(r.left() + 30.0, cy), Align2::LEFT_CENTER, msg, body(12.5), pal.text2);

        let gear = Rect::from_center_size(pos2(r.right() - 24.0, cy), vec2(32.0, 30.0));
        if icon_button(ui, pal, gear, glyph::GEAR, "gear") {
            self.settings_open = true;
            self.message = None;
        }
        let tg = Rect::from_center_size(pos2(gear.left() - 34.0, cy), vec2(40.0, 20.0));
        let elsewhere = self.driver.autostart.is_some() && !self.driver.autostart_here;
        if toggle(ui, pal, tg, self.driver.autostart_here, "autostart") {
            self.toggle_autostart(&ctx);
        }
        let label = if elsewhere { t.autostart_elsewhere } else { t.autostart };
        p.text(pos2(tg.left() - 10.0, cy), Align2::RIGHT_CENTER, label, body(12.5), if elsewhere { pal.warn_fg } else { pal.text2 });
    }
}

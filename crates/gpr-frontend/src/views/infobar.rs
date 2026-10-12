//! InfoBar: aviso persistente (falta ViGEmBus, driver detenido o colgado) con su acción.

use eframe::egui::{pos2, vec2, Align2, CornerRadius, Rect, Ui};

use crate::app::App;
use crate::model::Notice;
use crate::theme::fonts::{body, glyph, icon, semibold};
use crate::theme::Palette;
use crate::widgets::{button, button_width, text_width};

impl App {
    pub(crate) fn draw_infobar(&mut self, ui: &mut Ui, pal: &Palette, r: Rect, n: Notice) {
        let ctx = ui.ctx().clone();
        let t = self.t();
        let p = ui.painter().with_clip_rect(r);
        p.rect_filled(r, CornerRadius::same(6), pal.warn_bg);
        p.text(pos2(r.left() + 14.0, r.top() + 21.0), Align2::LEFT_CENTER, glyph::WARN, icon(16.0), pal.warn_fg);
        let (title, text, action) = match n {
            Notice::VigemMissing => (t.vigem_missing_title, t.vigem_missing_text, Some(t.install)),
            Notice::VigemInstalling => (t.vigem_installing_title, t.vigem_installing_text, None),
            Notice::DriverStopped => (t.driver_stopped_title, t.driver_stopped_text, Some(t.start)),
            Notice::DriverStale => (t.driver_stale_title, t.driver_stale_text, Some(t.restart)),
        };
        let bw = action.map(|a| button_width(&p, a, 88.0)).unwrap_or(0.0);
        // el texto se recorta antes del botón (algunos idiomas son más largos)
        let tp = p.with_clip_rect(Rect::from_min_max(r.min, pos2(r.right() - bw - 20.0, r.bottom())));
        let tx = r.left() + 40.0;
        let tw = text_width(&p, title, semibold(13.0));
        tp.text(pos2(tx, r.top() + 21.0), Align2::LEFT_CENTER, title, semibold(13.0), pal.text);
        tp.text(pos2(tx + tw + 6.0, r.top() + 21.0), Align2::LEFT_CENTER, text, body(13.0), pal.text);
        let Some(label) = action else { return };
        if r.height() <= 36.0 {
            return; // aún desplegándose
        }
        let b = Rect::from_min_size(pos2(r.right() - bw - 12.0, r.top() + 7.0), vec2(bw, 28.0));
        if button(ui, pal, b, label, true, "infobar-action") {
            match n {
                Notice::VigemMissing => self.install_vigem(&ctx),
                Notice::DriverStopped | Notice::DriverStale => self.restart_driver(&ctx),
                Notice::VigemInstalling => {}
            }
        }
    }
}

//! Panel de ajustes (se desliza desde la derecha): driver, ViGEmBus, idioma, detalles técnicos y
//! desinstalar.

use eframe::egui::{self, pos2, vec2, Align2, Color32, CornerRadius, Id, Order, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};
use gpr_core::brand::{DISPLAY_NAME, VERSION};
use gpr_core::gamepad::pressed_names;
use gpr_i18n::{fill, Lang};
use gpr_platform::{paths, process, settings, xinput};

use crate::app::App;
use crate::theme::fonts::{body, glyph, icon, mono, semibold};
use crate::theme::Palette;
use crate::widgets::{button, button_width, icon_button, shadow};

const PANE_W: f32 = 330.0;
const MENU_ROW_H: f32 = 28.0;

impl App {
    pub(crate) fn draw_settings(&mut self, ui: &mut Ui, pal: &Palette, full: Rect) {
        let ctx = ui.ctx().clone();
        let t = self.t();
        let anim = ctx.animate_bool_with_time(Id::new("settings"), self.settings_open, 0.22);
        if anim <= 0.001 {
            return;
        }
        // velo sobre el contenido: un clic fuera del panel lo cierra
        let scrim = ui.interact(full, ui.id().with("scrim"), Sense::click());
        let p = ui.painter().clone();
        p.rect_filled(full, CornerRadius::ZERO, Color32::from_black_alpha((if pal.dark { 90.0 } else { 40.0 } * anim) as u8));
        let w = PANE_W;
        let ease = 1.0 - (1.0 - anim).powi(3);
        let pane = Rect::from_min_max(pos2(full.right() - w * ease, full.top()), pos2(full.right() - w * ease + w, full.bottom()));
        if scrim.clicked() && !pane.contains(scrim.interact_pointer_pos().unwrap_or(Pos2::ZERO)) {
            self.close_settings();
        }
        for (e, a) in [(14.0, 0.12), (8.0, 0.2), (3.0, 0.3)] {
            p.rect_filled(Rect::from_min_max(pos2(pane.left() - e, pane.top()), pane.max), CornerRadius::ZERO, pal.shadow.gamma_multiply(a * anim));
        }
        p.rect_filled(pane, CornerRadius::ZERO, pal.bg);
        p.line_segment([pane.left_top(), pane.left_bottom()], Stroke::new(1.0, pal.stroke));
        // el panel captura los clics para que no lleguen al velo
        let _ = ui.interact(pane, ui.id().with("pane"), Sense::click());

        let x = pane.left() + 20.0;
        let cw = w - 40.0;
        p.text(pos2(x, pane.top() + 20.0), Align2::LEFT_TOP, t.settings, semibold(20.0), pal.text);
        let close = Rect::from_min_size(pos2(pane.right() - 48.0, pane.top() + 16.0), vec2(32.0, 30.0));
        if icon_button(ui, pal, close, glyph::CLOSE, "close-settings") {
            self.close_settings();
        }

        let mut y = pane.top() + 66.0;
        let card = |p: &egui::Painter, y: f32, h: f32| {
            let r = Rect::from_min_size(pos2(x, y), vec2(cw, h));
            p.rect(r, CornerRadius::same(6), pal.card, Stroke::new(1.0, pal.stroke), StrokeKind::Inside);
            r
        };
        // títulos de una tarjeta, recortados para no pisar su botón (`right` = espacio reservado)
        let titles = |p: &egui::Painter, r: Rect, right: f32, title: &str, sub: &str| {
            let clip = p.with_clip_rect(Rect::from_min_max(r.min, pos2(r.right() - right, r.bottom())));
            clip.text(pos2(r.left() + 14.0, r.top() + 11.0), Align2::LEFT_TOP, title, body(14.0), pal.text);
            clip.text(pos2(r.left() + 14.0, r.top() + 31.0), Align2::LEFT_TOP, sub, body(12.0), pal.text2);
        };
        let card_button = |r: Rect, bw: f32| Rect::from_min_size(pos2(r.right() - bw - 12.0, r.top() + 15.0), vec2(bw, 30.0));

        // driver
        let r = card(&p, y, 60.0);
        let sub = if !self.driver.running {
            t.driver_state_stopped.to_string()
        } else if self.driver.stale() {
            t.driver_state_stale.to_string()
        } else {
            fill(t.driver_state_running, &[("n", &self.driver.pads())])
        };
        let label = if self.driver.running { t.restart } else { t.start };
        let bw = button_width(&p, label, 84.0);
        titles(&p, r, bw + 20.0, t.driver, &sub);
        if button(ui, pal, card_button(r, bw), label, false, "set-restart") {
            self.restart_driver(&ctx);
        }
        y += 64.0;

        // ViGEmBus
        let r = card(&p, y, 60.0);
        let sub = if self.vigem_setup.is_some() {
            t.vigem_state_installing
        } else if self.vigem {
            t.vigem_state_installed
        } else {
            t.vigem_state_missing
        };
        let show_install = !self.vigem && self.vigem_setup.is_none();
        let bw = if show_install { button_width(&p, t.install, 84.0) } else { 0.0 };
        titles(&p, r, bw + 20.0, "ViGEmBus", sub);
        if show_install && button(ui, pal, card_button(r, bw), t.install, true, "set-vigem") {
            self.install_vigem(&ctx);
        }
        y += 64.0;

        // idioma (menú desplegable)
        let lang_card = card(&p, y, 60.0);
        let current = match self.lang_choice {
            Some(l) => l.native_name().to_string(),
            None => fill(t.language_auto, &[("lang", &settings::auto_language().native_name())]),
        };
        titles(&p, lang_card, 40.0, t.language, &current);
        if ui.interact(lang_card, ui.id().with("lang"), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            self.lang_open = !self.lang_open;
        }
        let chevron = if self.lang_open { glyph::UP } else { glyph::DOWN };
        p.text(pos2(lang_card.right() - 22.0, lang_card.center().y), Align2::CENTER_CENTER, chevron, icon(12.0), pal.text2);
        y += 64.0;

        // detalles técnicos (desplegable)
        let r = card(&p, y, 60.0);
        titles(&p, r, 40.0, t.tech, t.tech_text);
        if ui.interact(r, ui.id().with("tech"), Sense::click()).clicked() {
            self.tech_open = !self.tech_open;
        }
        p.text(pos2(r.right() - 22.0, r.center().y), Align2::CENTER_CENTER, if self.tech_open { glyph::UP } else { glyph::DOWN }, icon(12.0), pal.text2);
        y += 60.0;
        if self.tech_open {
            let lines = self.tech_lines();
            let h = lines.len() as f32 * 17.0 + 16.0;
            let r = Rect::from_min_size(pos2(x, y - 2.0), vec2(cw, h));
            p.rect(r, CornerRadius::same(6), pal.layer, Stroke::new(1.0, pal.stroke), StrokeKind::Inside);
            let clip = p.with_clip_rect(r.shrink(2.0));
            for (i, l) in lines.iter().enumerate() {
                clip.text(pos2(r.left() + 12.0, r.top() + 8.0 + i as f32 * 17.0), Align2::LEFT_TOP, l, mono(11.0), pal.text2);
            }
            y += h + 2.0;
        }
        y += 4.0;

        // desinstalar o versión portable
        let r = card(&p, y, 60.0);
        if let Some(unins) = self.driver.uninstaller.clone() {
            let bw = button_width(&p, t.uninstall_button, 96.0);
            titles(&p, r, bw + 20.0, t.uninstall, &fill(t.uninstall_text, &[("app", &DISPLAY_NAME)]));
            if button(ui, pal, card_button(r, bw), t.uninstall_button, false, "set-uninstall") && process::spawn_detached(&unins, &[]) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else {
            titles(&p, r, 0.0, t.portable, t.portable_text);
        }

        p.text(pos2(x, pane.bottom() - 20.0), Align2::LEFT_BOTTOM, format!("{DISPLAY_NAME} {VERSION}"), body(12.0), pal.text3);

        if self.lang_open {
            self.draw_language_menu(&ctx, pal, lang_card);
        }
    }

    fn close_settings(&mut self) {
        self.settings_open = false;
        self.lang_open = false;
    }

    /// Lista de idiomas bajo su tarjeta, por encima del resto del panel.
    fn draw_language_menu(&mut self, ctx: &egui::Context, pal: &Palette, anchor: Rect) {
        let t = self.t();
        let mut options: Vec<(Option<Lang>, String)> =
            vec![(None, fill(t.language_auto, &[("lang", &settings::auto_language().native_name())]))];
        options.extend(Lang::ALL.iter().map(|&l| (Some(l), l.native_name().to_string())));
        let menu = Rect::from_min_size(
            pos2(anchor.left(), anchor.bottom() + 4.0),
            vec2(anchor.width(), MENU_ROW_H * options.len() as f32 + 8.0),
        );
        let mut picked = None;
        egui::Area::new(Id::new("lang-menu")).order(Order::Foreground).fixed_pos(menu.min).show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(menu.size(), Sense::click());
            let p = ui.painter();
            shadow(p, pal, rect, 8);
            p.rect(rect, CornerRadius::same(8), pal.bg, Stroke::new(1.0, pal.stroke), StrokeKind::Inside);
            for (i, (choice, label)) in options.iter().enumerate() {
                let row = Rect::from_min_size(pos2(rect.left() + 4.0, rect.top() + 4.0 + i as f32 * MENU_ROW_H), vec2(rect.width() - 8.0, MENU_ROW_H));
                let resp = ui.interact(row, ui.id().with(("lang-opt", i)), Sense::click());
                if resp.hovered() {
                    p.rect_filled(row, CornerRadius::same(4), pal.card_hover);
                }
                if *choice == self.lang_choice {
                    let bar = Rect::from_center_size(pos2(row.left() + 3.0, row.center().y), vec2(3.0, 14.0));
                    p.rect_filled(bar, CornerRadius::same(2), pal.accent);
                }
                p.text(pos2(row.left() + 14.0, row.center().y), Align2::LEFT_CENTER, label, body(13.0), pal.text);
                if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                    picked = Some(*choice);
                }
            }
        });
        if let Some(choice) = picked {
            self.set_language(choice);
            self.lang_open = false;
        } else if ctx.input(|i| i.pointer.any_click()) {
            // un clic fuera de la lista (y de su tarjeta) la cierra
            let pos = ctx.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
            if !menu.contains(pos) && !anchor.contains(pos) {
                self.lang_open = false;
            }
        }
    }

    /// Lo que Windows ve en cada mando virtual, y dónde está el registro.
    fn tech_lines(&self) -> Vec<String> {
        let t = self.t();
        let mut lines = Vec::new();
        for g in &self.guitars {
            let name = g.kind.name();
            match self.driver.slot(g.kind).and_then(|s| xinput::state(s).map(|x| (s, x))) {
                Some((s, x)) => {
                    let pressed = pressed_names(x.wButtons.0);
                    lines.push(fill(t.tech_pad, &[("name", &name), ("n", &(s + 1))]));
                    lines.push(format!("  {}", if pressed.is_empty() { "—".to_string() } else { pressed.join(" ") }));
                    lines.push(format!("  LX {:+6} LY {:+6}  LT {:3}", x.sThumbLX, x.sThumbLY, x.bLeftTrigger));
                    lines.push(format!("  RX {:+6} RY {:+6}  RT {:3}", x.sThumbRX, x.sThumbRY, x.bRightTrigger));
                }
                None => lines.push(fill(t.tech_no_pad, &[("name", &name)])),
            }
        }
        lines.push(fill(t.tech_log, &[("path", &paths::log_file().display())]));
        lines
    }
}

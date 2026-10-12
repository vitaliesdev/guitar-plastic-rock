//! Estado de la app, acciones sobre el driver y composición de la ventana. Cada zona se dibuja en
//! su vista (`views`).

use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

use eframe::egui::{self, pos2, vec2, CornerRadius, Id, Rect, Ui};
use gpr_core::cli;
use gpr_core::instruments::Kind;
use gpr_i18n::{fill, Lang, Texts};
use gpr_platform::instance::{self, Handle};
use gpr_platform::settings::{self, Settings};
use gpr_platform::vigem::{self, InstallError};
use gpr_platform::{installation, process, status_file};

use crate::model::{DriverInfo, Guitar, Notice};
use crate::theme::{self, Palette};

/// Opciones de `--demo` para revisar la interfaz y hacer capturas.
struct DemoArgs {
    on: bool,
    /// empezar con la Riffmaster seleccionada
    riff: bool,
    /// simular un equipo sin ViGEmBus
    no_vigem: bool,
    /// abrir el panel de ajustes
    settings: bool,
    /// abrir los ajustes con el menú de idiomas desplegado
    lang_menu: bool,
}

impl DemoArgs {
    fn parse() -> DemoArgs {
        let args: Vec<String> = std::env::args().collect();
        let has = |a: &str| args.iter().any(|x| x == a);
        let on = has(cli::DEMO);
        DemoArgs {
            on,
            riff: on && has("riff"),
            no_vigem: on && has("novigem"),
            settings: on && (has("settings") || has("langmenu")),
            lang_menu: on && has("langmenu"),
        }
    }
}

pub struct App {
    /// `--demo`: guitarras y driver simulados
    demo: bool,
    pub(crate) guitars: Vec<Guitar>,
    pub(crate) selected: usize,
    manual_select: bool,
    pub(crate) tilt_smooth: f32,
    pub(crate) tilted: bool,
    pub(crate) driver: DriverInfo,
    pub(crate) vigem: bool,
    pub(crate) vigem_setup: Option<Receiver<Result<u32, InstallError>>>,
    quit_event: Option<Handle>,
    pub(crate) pal: Palette,
    pal_checked: Instant,
    hwnd: Option<isize>,
    pub(crate) settings_open: bool,
    pub(crate) tech_open: bool,
    pub(crate) toast: Option<(String, Instant)>,
    pub(crate) message: Option<String>,
    /// idioma en uso, y el elegido en Ajustes (None = el de Windows)
    pub(crate) lang: Lang,
    pub(crate) lang_choice: Option<Lang>,
    pub(crate) lang_open: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> App {
        let ctx = &cc.egui_ctx;
        theme::fonts::install(ctx);
        let pal = theme::current();
        ctx.set_visuals(visuals(&pal));
        let hwnd = window_handle(cc);
        if let Some(h) = hwnd {
            theme::apply_titlebar(h, pal.dark);
        }

        let lang_choice = Settings::load().chosen_language();
        let demo = DemoArgs::parse();
        let guitars: Vec<Guitar> = Kind::ALL.iter().map(|&kind| Guitar::new(ctx, kind, !demo.on)).collect();
        if demo.on {
            crate::demo::spawn(guitars.iter().map(|g| (g.kind, g.live.clone())).collect(), ctx.clone());
        } else if !instance::driver_running() {
            // abrir la app basta para que la guitarra funcione como mando: el driver sigue en la
            // bandeja al cerrar la ventana
            process::start_driver();
        }
        App {
            demo: demo.on,
            guitars,
            selected: usize::from(demo.riff),
            manual_select: demo.riff,
            tilt_smooth: 0.0,
            tilted: false,
            driver: DriverInfo::default(),
            vigem: vigem::available() && !demo.no_vigem,
            vigem_setup: None,
            quit_event: instance::create_quit_app_event(),
            pal,
            pal_checked: Instant::now(),
            hwnd,
            settings_open: demo.settings,
            tech_open: false,
            toast: None,
            message: None,
            lang: lang_choice.unwrap_or_else(settings::auto_language),
            lang_choice,
            lang_open: demo.lang_menu,
        }
    }

    /// Textos en el idioma de la app.
    pub(crate) fn t(&self) -> &'static Texts {
        self.lang.texts()
    }

    /// Cambia el idioma (None = el de Windows) y lo guarda.
    pub(crate) fn set_language(&mut self, choice: Option<Lang>) {
        self.lang_choice = choice;
        self.lang = choice.unwrap_or_else(settings::auto_language);
        let saved = Settings { language: choice.map(|l| l.code().to_string()) }.save();
        self.message = saved.err().map(|e| fill(self.t().err_generic, &[("e", &e)]));
    }

    /// Tema del sistema y estado del driver (como mucho cada medio segundo, salvo `force`).
    pub(crate) fn refresh(&mut self, ctx: &egui::Context, force: bool) {
        if self.pal_checked.elapsed() > Duration::from_secs(2) {
            self.pal_checked = Instant::now();
            let pal = theme::current();
            if pal != self.pal {
                self.pal = pal;
                ctx.set_visuals(visuals(&pal));
                if let Some(h) = self.hwnd {
                    theme::apply_titlebar(h, pal.dark);
                }
            }
        }
        if !force && self.driver.checked.is_some_and(|t| t.elapsed() < Duration::from_millis(500)) {
            return;
        }
        self.driver.running = instance::driver_running();
        self.driver.status = if self.driver.running { status_file::read() } else { None };
        self.driver.autostart = installation::autostart_path();
        self.driver.autostart_here = installation::autostart_is_this_exe();
        self.driver.uninstaller = installation::uninstaller();
        self.driver.checked = Some(Instant::now());
        if self.demo {
            self.driver.simulate();
        }
        if let Some(rx) = &self.vigem_setup {
            if let Ok(result) = rx.try_recv() {
                self.vigem_setup = None;
                self.vigem = vigem::available();
                let t = self.lang.texts();
                self.message = match (result, self.vigem) {
                    (_, true) => None,
                    (Err(InstallError::Cancelled), false) => Some(t.err_vigem_cancelled.to_string()),
                    (Err(InstallError::Extract(e)), false) => Some(fill(t.err_vigem_extract, &[("e", &e)])),
                    (Ok(code), false) => Some(fill(t.err_vigem_failed, &[("code", &code)])),
                };
            }
        }
    }

    pub(crate) fn select(&mut self, i: usize, manual: bool) {
        if self.selected != i {
            self.selected = i;
            self.tilt_smooth = 0.0;
            self.tilted = false;
        }
        self.manual_select |= manual;
    }

    pub(crate) fn notice(&self) -> Option<Notice> {
        if self.vigem_setup.is_some() {
            Some(Notice::VigemInstalling)
        } else if !self.vigem {
            Some(Notice::VigemMissing)
        } else if !self.driver.running {
            Some(Notice::DriverStopped)
        } else if self.driver.stale() {
            Some(Notice::DriverStale)
        } else {
            None
        }
    }

    // ---------- acciones ----------

    pub(crate) fn install_vigem(&mut self, ctx: &egui::Context) {
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(vigem::install());
            ctx.request_repaint();
        });
        self.vigem_setup = Some(rx);
        self.message = None;
    }

    pub(crate) fn restart_driver(&mut self, ctx: &egui::Context) {
        if self.driver.running {
            instance::stop_driver();
            instance::wait_driver_stopped(Duration::from_secs(5));
        }
        if !process::start_driver() {
            self.message = Some(self.t().err_driver_start.to_string());
        }
        std::thread::sleep(Duration::from_millis(300));
        self.refresh(ctx, true);
    }

    pub(crate) fn toggle_autostart(&mut self, ctx: &egui::Context) {
        let elsewhere = self.driver.autostart.is_some() && !self.driver.autostart_here;
        let enable = !self.driver.autostart_here || elsewhere;
        self.message = installation::set_autostart(enable).err().map(|e| fill(self.t().err_generic, &[("e", &e)]));
        self.refresh(ctx, true);
    }

    /// Selección automática de la guitarra conectada (hasta que el usuario elige una).
    fn auto_select(&mut self) {
        if self.manual_select {
            return;
        }
        let conn: Vec<bool> = self.guitars.iter().map(|g| g.live.lock().unwrap().connected()).collect();
        if !conn[self.selected] {
            if let Some(i) = conn.iter().position(|&c| c) {
                self.select(i, false);
            }
        }
    }

    /// Aviso cuando se registra un acorde nuevo.
    fn check_chords(&mut self) {
        let (chords, last) = {
            let l = self.guitars[self.selected].live.lock().unwrap();
            (l.chords, l.last_chord)
        };
        let g = &mut self.guitars[self.selected];
        if chords != g.seen_chords {
            if chords > g.seen_chords {
                self.toast = Some((fill(self.lang.texts().chord, &[("n", &last)]), Instant::now()));
            }
            g.seen_chords = chords;
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        ctx.request_repaint_after(Duration::from_millis(16));
        if self.quit_event.as_ref().is_some_and(Handle::is_set) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close); // el Setup va a actualizar la app
        }
        self.refresh(&ctx, false);
        self.auto_select();
        self.check_chords();

        let pal = self.pal;
        let full = ui.max_rect();
        ui.painter().rect_filled(full, CornerRadius::ZERO, pal.bg);

        // de arriba abajo: selector, aviso (animado), guitarra, medidores y barra de comandos
        let mut y = full.top() + 10.0;
        self.draw_selector(ui, &pal, full, y);
        y += 62.0;

        let notice = self.notice();
        let h = ctx.animate_value_with_time(Id::new("infobar"), if notice.is_some() { 42.0 } else { 0.0 }, 0.2);
        if h > 1.0 {
            let r = Rect::from_min_size(pos2(full.left() + 16.0, y + 4.0), vec2(full.width() - 32.0, h));
            if let Some(n) = notice {
                self.draw_infobar(ui, &pal, r, n);
            }
            y += h + 8.0;
        }

        let cmd = Rect::from_min_max(pos2(full.left() + 12.0, full.bottom() - 56.0), pos2(full.right() - 12.0, full.bottom() - 12.0));
        let meters = Rect::from_min_max(pos2(full.left() + 36.0, cmd.top() - 58.0), pos2(full.right() - 36.0, cmd.top() - 18.0));
        let hero = Rect::from_min_max(pos2(full.left() + 24.0, y + 4.0), pos2(full.right() - 24.0, meters.top() - 10.0));

        let (state, connected) = {
            let l = self.guitars[self.selected].live.lock().unwrap();
            (l.state.clone(), l.connected())
        };
        self.draw_hero(ui, &pal, hero, &state, connected);
        self.draw_meters(ui, &pal, meters, &state, connected);
        self.draw_command_bar(ui, &pal, cmd);
        self.draw_settings(ui, &pal, full);
    }
}

fn visuals(pal: &Palette) -> egui::Visuals {
    let mut v = if pal.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = pal.bg;
    v.window_fill = pal.bg;
    v.selection.bg_fill = pal.accent;
    v
}

fn window_handle(cc: &eframe::CreationContext) -> Option<isize> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    cc.window_handle().ok().and_then(|h| match h.as_raw() {
        RawWindowHandle::Win32(w) => Some(w.hwnd.get()),
        _ => None,
    })
}

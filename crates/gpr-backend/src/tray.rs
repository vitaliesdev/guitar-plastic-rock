//! Icono del driver en la bandeja del sistema: menú, doble clic y tooltip (en el idioma de la app,
//! leído al arrancar el driver).

use gpr_core::brand::{DISPLAY_NAME, ICON_PNG};
use gpr_i18n::{fill, Texts};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub enum TrayAction {
    OpenApp,
    StopDriver,
}

pub struct Tray {
    icon: Option<TrayIcon>,
    open_item: MenuItem,
    quit_item: MenuItem,
    t: &'static Texts,
}

fn icon() -> tray_icon::Icon {
    let img = image::load_from_memory_with_format(ICON_PNG, image::ImageFormat::Png)
        .expect("icono incrustado válido")
        .resize(32, 32, image::imageops::FilterType::Lanczos3)
        .to_rgba8();
    let (w, h) = (img.width(), img.height());
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).expect("icono de bandeja")
}

impl Tray {
    /// Necesita un bucle de mensajes de Windows en el mismo hilo.
    pub fn new() -> Tray {
        let t = gpr_platform::settings::language().texts();
        let menu = Menu::new();
        let open_item = MenuItem::new(fill(t.tray_open, &[("app", &DISPLAY_NAME)]), true, None);
        let quit_item = MenuItem::new(t.tray_stop, true, None);
        let _ = menu.append_items(&[&open_item, &PredefinedMenuItem::separator(), &quit_item]);
        let icon = TrayIconBuilder::new()
            .with_tooltip(DISPLAY_NAME)
            .with_icon(icon())
            .with_menu(Box::new(menu))
            .build()
            .ok();
        Tray { icon, open_item, quit_item, t }
    }

    /// Acciones pendientes del usuario (menú o doble clic).
    pub fn actions(&self) -> Vec<TrayAction> {
        let mut out = Vec::new();
        while let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == self.open_item.id() {
                out.push(TrayAction::OpenApp);
            } else if ev.id == self.quit_item.id() {
                out.push(TrayAction::StopDriver);
            }
        }
        while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } = ev {
                out.push(TrayAction::OpenApp);
            }
        }
        out
    }

    /// Resume las guitarras conectadas en el tooltip.
    pub fn set_connected(&self, names: &[&str]) {
        if let Some(icon) = &self.icon {
            let tip = if names.is_empty() {
                format!("{DISPLAY_NAME} · {}", self.t.tray_no_guitar)
            } else {
                format!("{DISPLAY_NAME} · {}", names.join(", "))
            };
            let _ = icon.set_tooltip(Some(tip));
        }
    }
}

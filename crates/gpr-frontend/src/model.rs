//! Estado en directo que dibuja la app: cada guitarra (alimentada por su hilo lector) y el driver.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui::{self, TextureHandle};
use gpr_backend::input::{Event, Reader, Status};
use gpr_core::instruments::{Kind, State};
use gpr_core::status::{unix_now, DeviceStatus, DriverStatus};

use crate::guitars;
use crate::images;

/// Una guitarra sin reportes durante este tiempo se considera desconectada.
const LIVE_TIMEOUT: Duration = Duration::from_secs(1);

/// Lo último que ha llegado de una guitarra (lo escribe su hilo lector).
pub struct Live {
    pub state: State,
    pub status: Status,
    last: Option<Instant>,
    prev_strum: bool,
    /// acordes registrados (2 o más trastes al rasguear) y tamaño del último
    pub chords: u32,
    pub last_chord: usize,
}

impl Live {
    pub fn new(kind: Kind) -> Live {
        Live { state: State::empty(kind), status: Status::Searching, last: None, prev_strum: false, chords: 0, last_chord: 0 }
    }

    pub fn connected(&self) -> bool {
        matches!(self.status, Status::Connected { .. }) && self.last.is_some_and(|t| t.elapsed() < LIVE_TIMEOUT)
    }

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Status(st) => self.status = st,
            Event::State(s) => {
                let strum = s.strum();
                if strum && !self.prev_strum {
                    let n = s.frets_held();
                    if n >= 2 {
                        self.chords += 1;
                        self.last_chord = n;
                    }
                }
                self.prev_strum = strum;
                self.state = s;
                self.last = Some(Instant::now());
            }
        }
    }
}

/// Caja de la miniatura del selector, en puntos.
pub const THUMB: egui::Vec2 = egui::vec2(64.0, 22.0);

/// Una guitarra en la app: su lector HID, su estado y sus texturas.
pub struct Guitar {
    pub kind: Kind,
    pub live: Arc<Mutex<Live>>,
    _reader: Option<Reader>,
    pub photo: TextureHandle,
    /// miniatura del selector, generada para (escala de pantalla, modo oscuro)
    thumb: Option<((f32, bool), TextureHandle)>,
    /// acordes ya avisados
    pub seen_chords: u32,
}

impl Guitar {
    /// `read_hid`: false en la demo (el estado lo simula `demo`).
    pub fn new(ctx: &egui::Context, kind: Kind, read_hid: bool) -> Guitar {
        let live = Arc::new(Mutex::new(Live::new(kind)));
        let reader = read_hid.then(|| {
            let live = live.clone();
            let ctx = ctx.clone();
            Reader::start(kind, move |ev| {
                if let Ok(mut l) = live.lock() {
                    l.on_event(ev);
                }
                ctx.request_repaint();
            })
        });
        let (w, h, rgba) = images::rgba(guitars::visual(kind).png);
        let image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        let photo = ctx.load_texture(kind.key(), image, egui::TextureOptions::LINEAR);
        Guitar { kind, live, _reader: reader, photo, thumb: None, seen_chords: 0 }
    }

    /// Miniatura a resolución real de pantalla; se regenera si cambia la escala (otro monitor) o
    /// el modo claro/oscuro (el contorno solo hace falta sobre fondo oscuro).
    pub fn thumb(&mut self, ctx: &egui::Context, dark: bool) -> &TextureHandle {
        let key = (ctx.pixels_per_point(), dark);
        if self.thumb.as_ref().is_none_or(|(k, _)| *k != key) {
            let ppp = key.0;
            let (w, h, rgba) = images::thumbnail(
                guitars::visual(self.kind).png,
                (THUMB.x * ppp).round() as u32,
                (THUMB.y * ppp).round() as u32,
                if dark { 80 } else { 0 },
            );
            let image = egui::ColorImage::from_rgba_premultiplied([w as usize, h as usize], &rgba);
            let tex = ctx.load_texture(format!("{}-thumb", self.kind.key()), image, egui::TextureOptions::LINEAR);
            self.thumb = Some((key, tex));
        }
        &self.thumb.as_ref().unwrap().1
    }
}

/// Lo que la app sabe del driver (se refresca cada medio segundo).
#[derive(Default)]
pub struct DriverInfo {
    pub running: bool,
    pub status: Option<DriverStatus>,
    pub autostart: Option<PathBuf>,
    pub autostart_here: bool,
    pub uninstaller: Option<PathBuf>,
    pub checked: Option<Instant>,
}

impl DriverInfo {
    /// En marcha pero sin publicar su estado desde hace rato.
    pub fn stale(&self) -> bool {
        self.running && self.status.as_ref().is_none_or(|s| s.is_stale(unix_now()))
    }

    /// Mando Xbox (0..3) que el driver ha creado para esta guitarra.
    pub fn slot(&self, kind: Kind) -> Option<u32> {
        self.status.as_ref()?.devices.get(kind.key())?.xinput_slot.filter(|_| self.running)
    }

    pub fn pads(&self) -> usize {
        Kind::ALL.iter().filter(|&&k| self.slot(k).is_some()).count()
    }

    /// Para `--demo` (capturas): un driver en marcha con un mando por guitarra.
    pub fn simulate(&mut self) {
        let devices = Kind::ALL
            .iter()
            .enumerate()
            .map(|(i, k)| {
                let device = DeviceStatus {
                    name: k.name().into(),
                    status: "conectado".into(),
                    usb_pid: None,
                    xinput_slot: Some(i as u32),
                };
                (k.key().to_string(), device)
            })
            .collect();
        self.running = true;
        self.status = Some(DriverStatus { updated: unix_now(), vigem: true, devices, ..Default::default() });
        self.autostart_here = true;
    }
}

/// Aviso de la InfoBar (y su acción).
#[derive(Clone, Copy, PartialEq)]
pub enum Notice {
    VigemMissing,
    VigemInstalling,
    DriverStopped,
    DriverStale,
}

//! Lo que la app necesita saber de cada guitarra para dibujarla: su foto, las zonas que se
//! iluminan al pulsar cada control, una lectura común para medidores y selector, y una demo.
//! Las vistas solo usan las funciones de este módulo; no distinguen entre guitarras.

mod ghl;
mod riffmaster;

use eframe::egui::{Color32, Painter, Pos2};
use gpr_core::instruments::{Kind, State};
use gpr_i18n::{fill, Texts};

use crate::theme::Palette;

/// Forma de una zona iluminable, en píxeles de la foto (assets/guitars/<guitarra>.png).
#[derive(Clone, Copy)]
pub enum Shape {
    /// centro x, centro y, ancho, alto
    Rect(f32, f32, f32, f32),
    /// centro x, centro y, radio
    Circle(f32, f32, f32),
    /// de (x, y) a (x, y)
    Line(f32, f32, f32, f32),
}

/// Controles que puede iluminar la foto.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ctl {
    White(u8),
    Black(u8),
    Fret(u8),
    Solo(u8),
    Strum,
    Whammy,
    HeroPower,
    Pause,
    Dpad,
    Ghtv,
    Start,
    Select,
    Ps,
    P1,
    Joystick,
}

/// (control, forma, a qué del mando Xbox va). La asignación admite `{dpad}`, `{ls}` y `{rs}`
/// (cruceta y sticks, que se traducen); los nombres de botón (A, LB, Start…) son universales.
pub type Zone = (Ctl, Shape, &'static str);

/// Cómo se ve una guitarra en la app.
pub struct Visual {
    pub png: &'static [u8],
    /// tamaño de la foto en píxeles
    pub size: (f32, f32),
    pub zones: &'static [Zone],
    /// cómo llama el juego a la acción de inclinar la guitarra
    pub tilt_label: &'static str,
}

/// Lectura común de cualquier guitarra (medidores y selector).
pub struct Readout {
    /// false si el receptor está conectado pero la guitarra apagada
    pub powered: bool,
    /// 0..1
    pub whammy: f32,
    /// 0..1, o -1..1 si `tilt_centered`
    pub tilt: f32,
    pub tilt_centered: bool,
    /// posición del selector de pastilla y si la guitarra la envía en este modo
    pub pickup: Option<(u8, bool)>,
    /// modo en que se presenta la guitarra ("PS4", "PS5"…)
    pub mode: Option<&'static str>,
}

/// Cómo se ilumina un control: (pulsado, color, nivel 0..1 para los analógicos).
pub type Lit = (bool, Color32, f32);

/// Nombre del control en el idioma de la app.
pub fn ctl_name(ctl: Ctl, t: &Texts) -> String {
    let colors = [t.green, t.red, t.yellow, t.blue, t.orange];
    match ctl {
        Ctl::White(i) => format!("{} {}", t.white, i + 1),
        Ctl::Black(i) => format!("{} {}", t.black, i + 1),
        Ctl::Fret(i) => colors[i as usize].to_string(),
        Ctl::Solo(i) => fill(t.solo, &[("fret", &colors[i as usize])]),
        Ctl::Strum => t.strum.to_string(),
        Ctl::Whammy => t.whammy.to_string(),
        Ctl::HeroPower => "Hero Power".to_string(),
        Ctl::Pause => t.pause.to_string(),
        Ctl::Dpad => t.dpad.to_string(),
        Ctl::Ghtv => "GHTV".to_string(),
        Ctl::Start => "Start".to_string(),
        Ctl::Select => "Select".to_string(),
        Ctl::Ps => "PS".to_string(),
        Ctl::P1 => "P1".to_string(),
        Ctl::Joystick => t.joystick.to_string(),
    }
}

/// Asignación Xbox de una zona, traducida.
pub fn mapping(m: &str, t: &Texts) -> String {
    fill(m, &[("dpad", &t.dpad), ("ls", &t.left_stick), ("rs", &t.right_stick)])
}

pub fn visual(kind: Kind) -> &'static Visual {
    match kind {
        Kind::Ghl => &ghl::VISUAL,
        Kind::Riffmaster => &riffmaster::VISUAL,
    }
}

pub fn readout(state: &State) -> Readout {
    match state {
        State::Ghl(s) => ghl::readout(s),
        State::Riffmaster(s) => riffmaster::readout(s),
    }
}

pub fn control(state: &State, ctl: Ctl, pal: &Palette) -> Lit {
    match state {
        State::Ghl(s) => ghl::control(s, ctl, pal),
        State::Riffmaster(s) => riffmaster::control(s, ctl, pal),
    }
}

/// Detalles propios de cada guitarra sobre la foto (p. ej. el joystick de la Riffmaster).
/// `map` convierte píxeles de la foto a coordenadas de pantalla.
pub fn draw_extras(p: &Painter, state: &State, map: &dyn Fn(f32, f32) -> Pos2, pal: &Palette) {
    match state {
        State::Ghl(_) => {}
        State::Riffmaster(s) => riffmaster::draw_extras(p, s, map, pal),
    }
}

/// Estado simulado en el instante `t` (segundos), para `--demo`.
pub fn demo(kind: Kind, t: f32) -> State {
    match kind {
        Kind::Ghl => State::Ghl(ghl::demo(t)),
        Kind::Riffmaster => State::Riffmaster(riffmaster::demo(t)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photos_match_their_size_and_zones_fit_inside() {
        for kind in Kind::ALL {
            let v = visual(kind);
            let (w, h, _) = crate::images::rgba(v.png);
            assert_eq!((w as f32, h as f32), v.size, "{}: tamaño de la foto", kind.name());
            for &(ctl, shape, _) in v.zones {
                let name = ctl_name(ctl, gpr_i18n::Lang::Es.texts());
                let (x0, y0, x1, y1) = match shape {
                    Shape::Rect(cx, cy, zw, zh) => (cx - zw / 2.0, cy - zh / 2.0, cx + zw / 2.0, cy + zh / 2.0),
                    Shape::Circle(cx, cy, r) => (cx - r, cy - r, cx + r, cy + r),
                    Shape::Line(ax, ay, bx, by) => (ax.min(bx), ay.min(by), ax.max(bx), ay.max(by)),
                };
                assert!(x0 >= 0.0 && y0 >= 0.0 && x1 <= v.size.0 && y1 <= v.size.1, "{}: zona «{name}» fuera de la foto", kind.name());
            }
        }
    }

    #[test]
    fn demo_produces_a_state_of_each_kind() {
        for kind in Kind::ALL {
            assert_eq!(demo(kind, 1.0).kind(), kind);
        }
    }
}

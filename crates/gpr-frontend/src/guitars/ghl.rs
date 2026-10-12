//! Guitar Hero Live: foto, zonas y colores (los trastes se iluminan en blanco y negro, como en el juego).

use eframe::egui::Color32;
use gpr_core::instruments::ghl::GhlState;

use super::Shape::*;
use super::{Ctl, Lit, Readout, Visual, Zone};
use crate::theme::Palette;

pub const WHITE: Color32 = Color32::from_rgb(0xff, 0xff, 0xff);
pub const BLACK: Color32 = Color32::from_rgb(0x0c, 0x0c, 0x0c);

const ZONES: &[Zone] = &[
    (Ctl::White(0), Rect(921.0, 165.0, 32.0, 28.0), "X"),
    (Ctl::White(1), Rect(885.0, 165.0, 32.0, 28.0), "LB"),
    (Ctl::White(2), Rect(850.0, 165.0, 32.0, 28.0), "RB"),
    (Ctl::Black(0), Rect(921.0, 196.0, 32.0, 28.0), "A"),
    (Ctl::Black(1), Rect(885.0, 196.0, 32.0, 28.0), "B"),
    (Ctl::Black(2), Rect(850.0, 196.0, 32.0, 28.0), "Y"),
    (Ctl::Strum, Rect(265.0, 178.0, 140.0, 34.0), "{dpad} ↑ ↓"),
    (Ctl::HeroPower, Rect(143.0, 183.0, 34.0, 124.0), "Back"),
    (Ctl::Whammy, Line(232.0, 287.0, 362.0, 232.0), "{rs} Y"),
    (Ctl::Pause, Circle(138.0, 297.0, 19.0), "Start"),
    (Ctl::Dpad, Circle(195.0, 290.0, 19.0), "{dpad}"),
    (Ctl::Ghtv, Circle(972.0, 180.0, 15.0), "LS"),
];

pub static VISUAL: Visual = Visual {
    png: include_bytes!("../../../../assets/guitars/ghl.png"),
    size: (1144.0, 359.0),
    zones: ZONES,
    tilt_label: "Hero Power",
};

pub fn readout(s: &GhlState) -> Readout {
    Readout { powered: s.guitar_on, whammy: s.whammy, tilt: s.tilt, tilt_centered: true, pickup: None, mode: None }
}

pub fn control(s: &GhlState, ctl: Ctl, pal: &Palette) -> Lit {
    let (on, color) = match ctl {
        Ctl::White(i) => ([s.white1, s.white2, s.white3][i as usize], WHITE),
        Ctl::Black(i) => ([s.black1, s.black2, s.black3][i as usize], BLACK),
        Ctl::Strum => (s.strum_up || s.strum_down, pal.accent),
        Ctl::Whammy => (s.whammy > 0.05, pal.accent),
        Ctl::HeroPower => (s.hero_power, pal.accent),
        Ctl::Pause => (s.pause, pal.accent),
        Ctl::Dpad => (s.dpad_up || s.dpad_down || s.dpad_left || s.dpad_right || s.dpad_center, pal.accent),
        Ctl::Ghtv => (s.ghtv, pal.accent),
        _ => (false, pal.accent),
    };
    (on, color, if ctl == Ctl::Whammy { s.whammy } else { 1.0 })
}

pub fn demo(t: f32) -> GhlState {
    let k = (t * 2.0) as u32 % 8;
    GhlState {
        black1: k < 2,
        black2: k == 1 || k == 2,
        white1: k == 3,
        white2: k == 4 || k == 5,
        white3: k == 6,
        black3: k == 7,
        strum_down: ((t * 4.0) as u32).is_multiple_of(3),
        whammy: (t * 0.8).sin().max(0.0),
        tilt: (t / 2.0).sin(),
        pause: k == 5,
        dpad_up: k == 2,
        guitar_on: true,
        ..Default::default()
    }
}

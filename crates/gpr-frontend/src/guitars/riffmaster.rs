//! PDP Riffmaster: foto, zonas y colores de Rock Band, más el joystick que se mueve.

use eframe::egui::{Color32, Painter, Pos2};
use gpr_core::instruments::riffmaster::{Mode, RiffState};

use super::Shape::*;
use super::{Ctl, Lit, Readout, Visual, Zone};
use crate::theme::Palette;

/// Verde, rojo, amarillo, azul, naranja.
const RB_COLORS: [Color32; 5] = [
    Color32::from_rgb(0x6c, 0xcb, 0x5f),
    Color32::from_rgb(0xff, 0x6b, 0x6b),
    Color32::from_rgb(0xfc, 0xe1, 0x00),
    Color32::from_rgb(0x4c, 0xc2, 0xff),
    Color32::from_rgb(0xff, 0x8c, 0x00),
];

const JOYSTICK: (f32, f32, f32) = (1000.0, 214.0, 15.0);

const ZONES: &[Zone] = &[
    (Ctl::Fret(0), Rect(905.0, 193.0, 28.0, 54.0), "A"),
    (Ctl::Fret(1), Rect(873.0, 193.0, 30.0, 54.0), "B"),
    (Ctl::Fret(2), Rect(841.0, 193.0, 30.0, 54.0), "Y"),
    (Ctl::Fret(3), Rect(809.0, 193.0, 30.0, 54.0), "X"),
    (Ctl::Fret(4), Rect(777.0, 193.0, 29.0, 54.0), "LB"),
    (Ctl::Solo(0), Rect(470.0, 193.0, 15.0, 54.0), "A + LS"),
    (Ctl::Solo(1), Rect(455.0, 193.0, 15.0, 54.0), "B + LS"),
    (Ctl::Solo(2), Rect(440.0, 193.0, 15.0, 54.0), "Y + LS"),
    (Ctl::Solo(3), Rect(424.0, 193.0, 16.0, 54.0), "X + LS"),
    (Ctl::Solo(4), Rect(406.0, 193.0, 19.0, 54.0), "LB + LS"),
    (Ctl::Strum, Rect(280.0, 190.0, 96.0, 34.0), "{dpad} ↑ ↓"),
    (Ctl::Whammy, Line(152.0, 212.0, 318.0, 252.0), "{rs} X"),
    (Ctl::Ps, Circle(172.0, 247.0, 9.0), "Guide"),
    (Ctl::Select, Circle(157.0, 268.0, 7.0), "Back"),
    (Ctl::Start, Circle(190.0, 268.0, 7.0), "Start"),
    (Ctl::P1, Circle(362.0, 302.0, 10.0), "RB"),
    (Ctl::Joystick, Circle(JOYSTICK.0, JOYSTICK.1, JOYSTICK.2), "{ls}"),
];

pub static VISUAL: Visual = Visual {
    png: include_bytes!("../../../../assets/guitars/riffmaster.png"),
    size: (1144.0, 391.0),
    zones: ZONES,
    tilt_label: "Overdrive",
};

pub fn readout(s: &RiffState) -> Readout {
    let ps4 = s.mode == Mode::Ps4;
    Readout {
        powered: true,
        whammy: s.whammy,
        tilt: s.tilt,
        tilt_centered: false,
        // en modo PS5 la guitarra no envía la pastilla
        pickup: Some((s.pickup, ps4)),
        mode: Some(if ps4 { "PS4" } else { "PS5" }),
    }
}

pub fn control(s: &RiffState, ctl: Ctl, pal: &Palette) -> Lit {
    let (on, color) = match ctl {
        Ctl::Fret(i) => (s.frets()[i as usize], RB_COLORS[i as usize]),
        Ctl::Solo(i) => (s.solo_frets()[i as usize], RB_COLORS[i as usize]),
        Ctl::Strum => (s.strum_up || s.strum_down, pal.accent),
        Ctl::Whammy => (s.whammy > 0.05, pal.accent),
        Ctl::Start => (s.start, pal.accent),
        Ctl::Select => (s.select, pal.accent),
        Ctl::Ps => (s.ps, pal.accent),
        Ctl::P1 => (s.p1, pal.accent),
        Ctl::Joystick => (s.joy_click || s.dpad_left || s.dpad_right, pal.accent),
        _ => (false, pal.accent),
    };
    (on, color, if ctl == Ctl::Whammy { s.whammy } else { 1.0 })
}

/// El joystick: un punto que se mueve dentro de su zona.
pub fn draw_extras(p: &Painter, s: &RiffState, map: &dyn Fn(f32, f32) -> Pos2, pal: &Palette) {
    let (jx, jy, jr) = JOYSTICK;
    let c = map(jx + s.joy_x * jr * 0.55, jy + s.joy_y * jr * 0.55);
    p.circle_filled(c, 3.5, if s.joy_click { pal.accent } else { Color32::from_white_alpha(170) });
}

pub fn demo(t: f32) -> RiffState {
    let k = (t * 2.0) as u32 % 8;
    RiffState {
        green: k < 2,
        red: k == 1 || k == 2,
        yellow: k == 3,
        blue: k == 4,
        orange: k == 5,
        solo_green: k == 6,
        solo_blue: k == 7,
        strum_down: ((t * 4.0) as u32).is_multiple_of(3),
        whammy: (t * 0.8).sin().max(0.0),
        tilt: ((t / 2.0).sin() + 1.0) / 2.0,
        pickup: (t / 2.0) as u8 % 5,
        joy_x: t.cos(),
        joy_y: t.sin(),
        start: k == 5,
        p1: k == 2,
        ..Default::default()
    }
}

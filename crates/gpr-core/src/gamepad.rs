//! El mando Xbox 360 virtual: el formato común al que se traduce cualquier guitarra.

/// Reporte de mando Xbox 360 (mismo orden que XINPUT_GAMEPAD).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct XReport {
    pub buttons: u16,
    pub lt: u8,
    pub rt: u8,
    pub lx: i16,
    pub ly: i16,
    pub rx: i16,
    pub ry: i16,
}

/// Bits de `XReport::buttons` (los de XINPUT_GAMEPAD).
pub mod xbtn {
    pub const DPAD_UP: u16 = 0x0001;
    pub const DPAD_DOWN: u16 = 0x0002;
    pub const DPAD_LEFT: u16 = 0x0004;
    pub const DPAD_RIGHT: u16 = 0x0008;
    pub const START: u16 = 0x0010;
    pub const BACK: u16 = 0x0020;
    pub const LEFT_THUMB: u16 = 0x0040;
    pub const RIGHT_THUMB: u16 = 0x0080;
    pub const LEFT_SHOULDER: u16 = 0x0100;
    pub const RIGHT_SHOULDER: u16 = 0x0200;
    pub const GUIDE: u16 = 0x0400;
    pub const A: u16 = 0x1000;
    pub const B: u16 = 0x2000;
    pub const X: u16 = 0x4000;
    pub const Y: u16 = 0x8000;
}

/// Nombre corto de cada botón, en el orden en que se muestran.
pub const BUTTON_NAMES: [(u16, &str); 15] = [
    (xbtn::A, "A"),
    (xbtn::B, "B"),
    (xbtn::Y, "Y"),
    (xbtn::X, "X"),
    (xbtn::LEFT_SHOULDER, "LB"),
    (xbtn::RIGHT_SHOULDER, "RB"),
    (xbtn::DPAD_UP, "↑"),
    (xbtn::DPAD_DOWN, "↓"),
    (xbtn::DPAD_LEFT, "←"),
    (xbtn::DPAD_RIGHT, "→"),
    (xbtn::START, "Start"),
    (xbtn::BACK, "Back"),
    (xbtn::LEFT_THUMB, "LS"),
    (xbtn::RIGHT_THUMB, "RS"),
    (xbtn::GUIDE, "Guide"),
];

/// Nombres de los botones pulsados en `buttons`.
pub fn pressed_names(buttons: u16) -> Vec<&'static str> {
    BUTTON_NAMES.iter().filter(|(bit, _)| buttons & bit != 0).map(|(_, n)| *n).collect()
}

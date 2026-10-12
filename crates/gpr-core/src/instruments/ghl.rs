//! Guitar Hero Live con dongle de PS4 (USB 1430:07BB).
//! PlasticBand: Docs/Instruments/6-Fret Guitar/PS4.md. El dongle solo informa de un botón a la vez
//! si no recibe el keep-alive (report 0x30) al menos cada 8 segundos.

use super::report::{bits, hat};
use super::{InstrumentSpec, InstrumentState};
use crate::gamepad::{xbtn, XReport};

pub const VID: u16 = 0x1430;
pub const PID: u16 = 0x07BB;
pub const KEEPALIVE: [u8; 9] = [0x30, 0x02, 0x08, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00];

pub const SPEC: InstrumentSpec =
    InstrumentSpec { key: "ghl", name: "Guitar Hero Live", ids: &[(VID, PID)], keepalive: Some(&KEEPALIVE) };

/// RPCS3 (Emu/Io/GHLtar.cpp) calcula el whammy como `~valor + 1`: con el stick justo al tope el
/// valor es 0 y da la vuelta a 0x00 (= suelto). Nos quedamos un poco por debajo.
pub const WHAMMY_MAX: f32 = 32000.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GhlState {
    pub black1: bool,
    pub black2: bool,
    pub black3: bool,
    pub white1: bool,
    pub white2: bool,
    pub white3: bool,
    pub strum_up: bool,
    pub strum_down: bool,
    /// 0..1
    pub whammy: f32,
    /// -1 (abajo) .. 1 (arriba)
    pub tilt: f32,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub pause: bool,
    pub hero_power: bool,
    pub ghtv: bool,
    pub dpad_center: bool,
    pub guitar_on: bool,
}

impl GhlState {
    pub fn frets(&self) -> [bool; 6] {
        [self.black1, self.black2, self.black3, self.white1, self.white2, self.white3]
    }

}

impl InstrumentState for GhlState {
    /// Distribución de la guitarra GHL oficial de Xbox 360 (PlasticBand, 6-Fret Guitar/Xbox 360.md).
    fn to_xreport(&self) -> XReport {
        use xbtn::*;
        XReport {
            buttons: bits(&[
                (self.black1, A),
                (self.black2, B),
                (self.black3, Y),
                (self.white1, X),
                (self.white2, LEFT_SHOULDER),
                (self.white3, RIGHT_SHOULDER),
                (self.pause, START),
                (self.hero_power, BACK),
                (self.ghtv, LEFT_THUMB),
                (self.dpad_center, GUIDE),
                (self.dpad_up || self.strum_up, DPAD_UP),
                (self.dpad_down || self.strum_down, DPAD_DOWN),
                (self.dpad_left, DPAD_LEFT),
                (self.dpad_right, DPAD_RIGHT),
            ]),
            ly: if self.strum_up { 32767 } else if self.strum_down { -32768 } else { 0 },
            rx: (self.tilt * 32767.0).round() as i16,
            ry: (self.whammy * WHAMMY_MAX).round() as i16,
            ..Default::default()
        }
    }

    fn strum(&self) -> bool {
        self.strum_up || self.strum_down
    }

    fn frets_held(&self) -> usize {
        self.frets().iter().filter(|&&f| f).count()
    }
}

pub fn parse(r: &[u8]) -> Option<GhlState> {
    if r.len() < 27 || r[0] != 0x01 {
        return None;
    }
    let (b5, b6, b7) = (r[5], r[6], r[7]);
    let (up, down, left, right) = hat(b5);
    Some(GhlState {
        white1: b5 & 0x10 != 0,
        black1: b5 & 0x20 != 0,
        black2: b5 & 0x40 != 0,
        black3: b5 & 0x80 != 0,
        white2: b6 & 0x01 != 0,
        white3: b6 & 0x02 != 0,
        pause: b6 & 0x20 != 0,
        ghtv: b6 & 0x40 != 0,
        hero_power: b6 & 0x80 != 0,
        dpad_center: b7 & 0x01 != 0,
        strum_up: r[2] < 0x40,
        strum_down: r[2] > 0xC0,
        whammy: ((r[4] as f32 - 128.0) / 127.0).clamp(0.0, 1.0),
        tilt: ((r[3] as f32 - 128.0) / 127.0).clamp(-1.0, 1.0),
        dpad_up: up,
        dpad_down: down,
        dpad_left: left,
        dpad_right: right,
        // byte 25 = "guitarra conectada"; el acelerómetro (19..24) también es != 0 con la guitarra viva
        guitar_on: r[25] != 0 || r[19..25].iter().any(|&b| b != 0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(set: &[(usize, u8)]) -> Vec<u8> {
        let mut r = vec![0u8; 64];
        r[..6].copy_from_slice(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x08]);
        for &(i, v) in set {
            r[i] = v;
        }
        r
    }

    #[test]
    fn idle_report_from_real_dongle() {
        let s = parse(&report(&[])).unwrap();
        assert!(!s.frets().iter().any(|&f| f));
        assert!(!s.strum_up && !s.strum_down);
        assert_eq!((s.whammy, s.tilt), (0.0, 0.0));
    }

    #[test]
    fn frets_strum_and_axes() {
        let s = parse(&report(&[(5, 0x08 | 0x20 | 0x10), (6, 0x01), (2, 0x00), (4, 0xFF), (3, 0xFF)])).unwrap();
        assert!(s.black1 && s.white1 && s.white2);
        assert!(s.strum_up);
        assert_eq!((s.whammy, s.tilt), (1.0, 1.0));
    }

    #[test]
    fn xbox_mapping() {
        let x = parse(&report(&[(5, 0x08 | 0x20), (2, 0xFF), (4, 0xFF)])).unwrap().to_xreport();
        assert_eq!(x.buttons, xbtn::A | xbtn::DPAD_DOWN);
        assert_eq!(x.ly, -32768);
        assert_eq!(x.ry, WHAMMY_MAX as i16); // nunca 32767: evita el desbordamiento de RPCS3
    }
}

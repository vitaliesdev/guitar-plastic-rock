//! PDP Riffmaster (versión PlayStation) en modo PS4 (0E6F:024A) o PS5 (0E6F:0249).
//! PlasticBand: Docs/Instruments/5-Fret Guitar/Rock Band/PS4.md y PS5.md. En PC los juegos leen
//! trastes y púa, pero el whammy y la inclinación van en bytes que no leen; aquí se traducen al
//! formato de las guitarras Rock Band de Xbox 360, que Clone Hero y YARG reconocen.

use super::report::{axis, bits, hat};
use super::{InstrumentSpec, InstrumentState};
use crate::gamepad::{xbtn, XReport};

pub const VID: u16 = 0x0E6F;
pub const PID_PS4: u16 = 0x024A;
pub const PID_PS5: u16 = 0x0249;

pub const SPEC: InstrumentSpec = InstrumentSpec {
    key: "riffmaster",
    name: "PDP Riffmaster",
    ids: &[(VID, PID_PS5), (VID, PID_PS4)],
    keepalive: None,
};

/// El sensor de inclinación va por escalones y en vertical se queda en ~233 (medido), no en 255.
pub const TILT_MAX: f32 = 230.0;

/// Desplazamientos de cada modo: (botones, whammy, inclinación, trastes, trastes solo, pastilla).
fn layout(pid: u16) -> Option<(usize, usize, usize, usize, usize, Option<usize>)> {
    match pid {
        PID_PS4 => Some((5, 44, 45, 46, 47, Some(43))),
        PID_PS5 => Some((8, 41, 42, 43, 44, None)),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Ps4,
    Ps5,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiffState {
    pub green: bool,
    pub red: bool,
    pub yellow: bool,
    pub blue: bool,
    pub orange: bool,
    pub solo_green: bool,
    pub solo_red: bool,
    pub solo_yellow: bool,
    pub solo_blue: bool,
    pub solo_orange: bool,
    pub strum_up: bool,
    pub strum_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    /// 0..1
    pub whammy: f32,
    /// 0 (horizontal) .. 1 (vertical)
    pub tilt: f32,
    /// 0..4 (solo en modo PS4)
    pub pickup: u8,
    /// -1..1
    pub joy_x: f32,
    /// -1 (arriba) .. 1 (abajo)
    pub joy_y: f32,
    pub joy_click: bool,
    pub p1: bool,
    pub start: bool,
    pub select: bool,
    pub ps: bool,
    pub mode: Mode,
}

impl RiffState {
    pub fn frets(&self) -> [bool; 5] {
        [self.green, self.red, self.yellow, self.blue, self.orange]
    }

    pub fn solo_frets(&self) -> [bool; 5] {
        [self.solo_green, self.solo_red, self.solo_yellow, self.solo_blue, self.solo_orange]
    }

}

impl InstrumentState for RiffState {
    /// Guitarra Rock Band de Xbox 360 (PlasticBand, Rock Band/Xbox 360.md). Extras: el joystick va al
    /// stick izquierdo, su clic a RS, P1 a RB y la inclinación también a RT (umbral propio en RPCS3).
    fn to_xreport(&self) -> XReport {
        use xbtn::*;
        let [g, r, y, b, o] = self.frets();
        let [sg, sr, sy, sb, so] = self.solo_frets();
        XReport {
            buttons: bits(&[
                (g || sg, A),
                (r || sr, B),
                (y || sy, Y),
                (b || sb, X),
                (o || so, LEFT_SHOULDER),
                (sg || sr || sy || sb || so, LEFT_THUMB),
                (self.strum_up, DPAD_UP),
                (self.strum_down, DPAD_DOWN),
                (self.dpad_left, DPAD_LEFT),
                (self.dpad_right, DPAD_RIGHT),
                (self.start, START),
                (self.select, BACK),
                (self.ps, GUIDE),
                (self.joy_click, RIGHT_THUMB),
                (self.p1, RIGHT_SHOULDER),
            ]),
            lt: self.pickup * 51 + 25, // centro de cada muesca (General Notes.md)
            rt: (self.tilt * 255.0).round() as u8,
            lx: (self.joy_x * 32767.0).round() as i16,
            ly: (-self.joy_y * 32767.0).round() as i16,
            // 0 (suelto) .. 32767: las guitarras RB de 360 usan -32768 en reposo, pero RPCS3 solo lee
            // la mitad positiva del eje y Clone Hero/YARG calibran el rango igualmente
            rx: (self.whammy * 32767.0).round() as i16,
            ry: (self.tilt * 32767.0).round() as i16,
        }
    }

    fn strum(&self) -> bool {
        self.strum_up || self.strum_down
    }

    fn frets_held(&self) -> usize {
        self.frets().iter().chain(self.solo_frets().iter()).filter(|&&f| f).count()
    }
}

pub fn parse(r: &[u8], pid: u16) -> Option<RiffState> {
    let (b, w, t, f, sf, pk) = layout(pid)?;
    if r.len() < 48 || r[0] != 0x01 {
        return None;
    }
    let (up, down, left, right) = hat(r[b]);
    let (upper, lower) = (r[f], r[sf]);
    let solo_flag = r[b + 1] & 0x40 != 0;
    Some(RiffState {
        green: upper & 0x01 != 0,
        red: upper & 0x02 != 0,
        yellow: upper & 0x04 != 0,
        blue: upper & 0x08 != 0,
        orange: upper & 0x10 != 0,
        solo_green: lower & 0x01 != 0,
        solo_red: lower & 0x02 != 0,
        solo_yellow: lower & 0x04 != 0,
        solo_blue: lower & 0x08 != 0,
        solo_orange: lower & 0x10 != 0,
        strum_up: up,
        strum_down: down,
        dpad_left: left,
        dpad_right: right,
        whammy: r[w] as f32 / 255.0,
        tilt: (r[t] as f32 / TILT_MAX).min(1.0),
        pickup: pk.map(|i| r[i].min(4)).unwrap_or(0),
        joy_x: axis(r[1]),
        joy_y: axis(r[2]),
        // el clic del joystick comparte bit (L3) con el indicador de solo
        joy_click: solo_flag && lower & 0x1F == 0,
        p1: r[b + 1] & 0x80 != 0,
        select: r[b + 1] & 0x10 != 0,
        start: r[b + 1] & 0x20 != 0,
        ps: r[b + 2] & 0x01 != 0,
        mode: if pid == PID_PS4 { Mode::Ps4 } else { Mode::Ps5 },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct R {
        hat: Option<u8>,
        flags2: u8,
        whammy: u8,
        tilt: u8,
        upper: u8,
        lower: u8,
        pickup: u8,
    }

    fn report(pid: u16, o: R) -> Vec<u8> {
        let (b, w, t, f, sf, pk) = layout(pid).unwrap();
        let mut r = vec![0u8; 64];
        r[0] = 0x01;
        r[1] = 0x80;
        r[2] = 0x80;
        r[b] = o.hat.unwrap_or(0x0F); // cruceta suelta (los dos modos usan 15)
        r[b + 1] = o.flags2;
        r[w] = o.whammy;
        r[t] = o.tilt;
        r[f] = o.upper;
        r[sf] = o.lower;
        if let Some(i) = pk {
            r[i] = o.pickup;
        }
        r
    }

    #[test]
    fn idle_both_modes() {
        for pid in [PID_PS4, PID_PS5] {
            let s = parse(&report(pid, R::default()), pid).unwrap();
            assert!(!s.frets().iter().chain(s.solo_frets().iter()).any(|&f| f));
            assert!(!s.strum_up && !s.strum_down && !s.joy_click);
        }
    }

    #[test]
    fn real_idle_report_ps4() {
        // reporte real capturado de la Riffmaster en modo PS4, en reposo
        let mut r = vec![0u8; 64];
        r[..6].copy_from_slice(&[0x01, 0x80, 0x80, 0x80, 0x80, 0x0F]);
        r[25] = 0x01;
        r[35] = 0x80;
        r[39] = 0x80;
        let s = parse(&r, PID_PS4).unwrap();
        assert_eq!(s, RiffState { mode: Mode::Ps4, ..Default::default() });
    }

    #[test]
    fn frets_whammy_tilt_pickup_ps4() {
        let r = report(PID_PS4, R { hat: Some(4), upper: 0x01 | 0x10, whammy: 0xFF, tilt: 0xE9, pickup: 3, ..R::default() });
        let s = parse(&r, PID_PS4).unwrap();
        assert!(s.green && s.orange && !s.red);
        assert!(s.strum_down);
        assert_eq!((s.whammy, s.tilt, s.pickup), (1.0, 1.0, 3));
    }

    #[test]
    fn solo_frets_set_flag_but_not_joystick_click() {
        let s = parse(&report(PID_PS5, R { flags2: 0x40, lower: 0x04, ..R::default() }), PID_PS5).unwrap();
        assert!(s.solo_yellow && !s.joy_click);
        let s = parse(&report(PID_PS5, R { flags2: 0x40, ..R::default() }), PID_PS5).unwrap();
        assert!(s.joy_click);
    }

    #[test]
    fn xbox_mapping_rock_band_360_layout() {
        let r = report(PID_PS5, R { hat: Some(0), upper: 0x04 | 0x08, lower: 0x01, tilt: 0xFF, ..R::default() });
        let x = parse(&r, PID_PS5).unwrap().to_xreport();
        use xbtn::*;
        assert_eq!(x.buttons, Y | X | A | LEFT_THUMB | DPAD_UP);
        assert_eq!((x.rx, x.ry, x.rt), (0, 32767, 255));
    }
}

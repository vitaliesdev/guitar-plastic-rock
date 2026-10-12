//! Guitarras compatibles. Cada una vive en su propio módulo con:
//! - `SPEC`: cómo se reconoce (VID/PID) y si necesita keep-alive
//! - su estado (botones y ejes ya decodificados) y `parse`, el decodificador de reportes HID
//! - su traducción a mando Xbox 360 (`InstrumentState::to_xreport`)
//!
//! Formatos según <https://github.com/TheNathannator/PlasticBand>.
//! Para añadir una guitarra: docs/ADDING_A_GUITAR.md

pub mod ghl;
mod report;
pub mod riffmaster;

use crate::gamepad::XReport;

/// Cómo se reconoce una guitarra en el bus USB.
pub struct InstrumentSpec {
    /// Identificador estable (status.json, texturas…)
    pub key: &'static str,
    pub name: &'static str,
    /// (vid, pid) aceptados
    pub ids: &'static [(u16, u16)],
    /// Paquete que hay que enviar periódicamente para que el dispositivo informe de todo
    pub keepalive: Option<&'static [u8]>,
}

/// Lo que cualquier guitarra sabe hacer con su estado.
pub trait InstrumentState {
    /// Traducción al mando Xbox 360 virtual
    fn to_xreport(&self) -> XReport;
    /// ¿Púa accionada (arriba o abajo)?
    fn strum(&self) -> bool;
    /// Trastes pulsados a la vez (para detectar acordes)
    fn frets_held(&self) -> usize;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Ghl,
    Riffmaster,
}

impl Kind {
    pub const ALL: [Kind; 2] = [Kind::Ghl, Kind::Riffmaster];

    pub fn spec(self) -> &'static InstrumentSpec {
        match self {
            Kind::Ghl => &ghl::SPEC,
            Kind::Riffmaster => &riffmaster::SPEC,
        }
    }

    pub fn key(self) -> &'static str {
        self.spec().key
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub fn ids(self) -> &'static [(u16, u16)] {
        self.spec().ids
    }

    pub fn keepalive(self) -> Option<&'static [u8]> {
        self.spec().keepalive
    }

    /// Decodifica un reporte HID de entrada (`pid`: el modo en que se presenta el dispositivo).
    pub fn parse(self, report: &[u8], pid: u16) -> Option<State> {
        match self {
            Kind::Ghl => ghl::parse(report).map(State::Ghl),
            Kind::Riffmaster => riffmaster::parse(report, pid).map(State::Riffmaster),
        }
    }
}

/// Estado decodificado de cualquier guitarra.
#[derive(Clone, Debug)]
pub enum State {
    Ghl(ghl::GhlState),
    Riffmaster(riffmaster::RiffState),
}

impl State {
    pub fn empty(kind: Kind) -> State {
        match kind {
            Kind::Ghl => State::Ghl(Default::default()),
            Kind::Riffmaster => State::Riffmaster(Default::default()),
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            State::Ghl(_) => Kind::Ghl,
            State::Riffmaster(_) => Kind::Riffmaster,
        }
    }

    fn inner(&self) -> &dyn InstrumentState {
        match self {
            State::Ghl(s) => s,
            State::Riffmaster(s) => s,
        }
    }

    pub fn to_xreport(&self) -> XReport {
        self.inner().to_xreport()
    }

    pub fn strum(&self) -> bool {
        self.inner().strum()
    }

    pub fn frets_held(&self) -> usize {
        self.inner().frets_held()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_unique_key_and_ids() {
        let keys: std::collections::HashSet<_> = Kind::ALL.iter().map(|k| k.key()).collect();
        assert_eq!(keys.len(), Kind::ALL.len());
        for k in Kind::ALL {
            assert!(!k.ids().is_empty());
            assert_eq!(State::empty(k).kind(), k);
        }
    }
}

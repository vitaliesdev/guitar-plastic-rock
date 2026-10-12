//! Mando Xbox 360 virtual de una guitarra (ViGEmBus): se crea al conectarla, recibe cada estado
//! traducido y se retira al desconectarla.

use std::sync::Arc;
use std::time::Duration;

use gpr_core::gamepad::XReport;
use gpr_core::instruments::Kind;
use gpr_core::status::DeviceStatus;
use gpr_platform::log;
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

use crate::input::{Event, Status};

type Pad = Xbox360Wired<Arc<Client>>;

fn to_gamepad(x: &XReport) -> XGamepad {
    XGamepad {
        buttons: XButtons { raw: x.buttons },
        left_trigger: x.lt,
        right_trigger: x.rt,
        thumb_lx: x.lx,
        thumb_ly: x.ly,
        thumb_rx: x.rx,
        thumb_ry: x.ry,
    }
}

/// Una guitarra unida a su mando virtual.
pub struct Bridge {
    kind: Kind,
    client: Option<Arc<Client>>,
    pad: Option<Pad>,
    last: Option<XReport>,
    status: Status,
    error: Option<String>,
}

impl Bridge {
    pub fn new(kind: Kind, client: Option<Arc<Client>>) -> Bridge {
        Bridge { kind, client, pad: None, last: None, status: Status::Searching, error: None }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// ViGEmBus pasó a estar disponible con el driver ya en marcha: crear el mando si hace falta.
    pub fn set_client(&mut self, client: Arc<Client>) {
        self.client = Some(client);
        if matches!(self.status, Status::Connected { .. }) {
            self.plug();
        }
    }

    pub fn on_event(&mut self, ev: Event) {
        match ev {
            Event::Status(st) => {
                self.status = st;
                let text = match st {
                    Status::Searching => "buscando".to_string(),
                    Status::Connected { pid } => format!("conectada (USB {:04X}:{pid:04X})", self.kind.ids()[0].0),
                    Status::Disconnected => "desconectada".to_string(),
                };
                log(&format!("{}: {text}", self.kind.name()));
                match st {
                    Status::Connected { .. } => self.plug(),
                    _ => self.unplug(),
                }
            }
            Event::State(state) => {
                let x = state.to_xreport();
                if self.last != Some(x) {
                    if let Some(pad) = &mut self.pad {
                        if pad.update(&to_gamepad(&x)).is_ok() {
                            self.last = Some(x);
                        }
                    }
                }
            }
        }
    }

    fn plug(&mut self) {
        let Some(client) = self.client.clone() else { return };
        if self.pad.is_some() {
            return;
        }
        let mut pad = Xbox360Wired::new(client, TargetId::XBOX360_WIRED);
        let result = pad.plugin().and_then(|_| pad.wait_ready());
        match result {
            Ok(()) => {
                // ViGEmBus ignora reportes repetidos y Windows descarta los anteriores a terminar de
                // enumerar el mando: uno distinto y después el neutro garantizan un estado limpio
                let _ = pad.update(&XGamepad { thumb_lx: 1, ..Default::default() });
                std::thread::sleep(Duration::from_millis(20));
                let _ = pad.update(&XGamepad::default());
                self.last = None;
                self.pad = Some(pad);
                self.error = None;
                log(&format!("{}: mando Xbox 360 virtual conectado", self.kind.name()));
            }
            Err(e) => {
                self.error = Some(format!("no se pudo crear el mando virtual: {e:?}"));
                log(&format!("{}: {:?}", self.kind.name(), self.error));
            }
        }
    }

    pub fn unplug(&mut self) {
        if let Some(mut pad) = self.pad.take() {
            let _ = pad.unplug();
            log(&format!("{}: mando Xbox 360 virtual retirado", self.kind.name()));
        }
    }

    pub fn device_status(&mut self) -> DeviceStatus {
        let (status, usb_pid) = match self.status {
            Status::Searching => ("buscando", None),
            Status::Connected { pid } => ("conectado", Some(pid)),
            Status::Disconnected => ("desconectado", None),
        };
        DeviceStatus {
            name: self.kind.name().into(),
            status: status.into(),
            usb_pid,
            xinput_slot: self.pad.as_mut().and_then(|p| p.get_user_index().ok()),
        }
    }
}

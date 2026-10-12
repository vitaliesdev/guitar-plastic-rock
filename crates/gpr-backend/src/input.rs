//! Lectura HID: un hilo por tipo de guitarra que la busca, la abre, la mantiene activa
//! (keep-alive) y entrega cada estado decodificado. No sabe nada de mandos virtuales ni de la
//! interfaz: quien lo usa decide qué hacer con cada `Event`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use hidapi::HidApi;

use gpr_core::instruments::{Kind, State};

const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(4); // el dongle GHL exige < 8 s
const READ_TIMEOUT_MS: i32 = 250;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Searching,
    Connected { pid: u16 },
    Disconnected,
}

pub enum Event {
    Status(Status),
    State(State),
}

/// hidapi solo admite una instancia por proceso: se comparte entre los hilos.
fn hidapi() -> Option<&'static Mutex<HidApi>> {
    static API: OnceLock<Option<Mutex<HidApi>>> = OnceLock::new();
    API.get_or_init(|| HidApi::new().ok().map(Mutex::new)).as_ref()
}

/// Busca la colección de mando (Generic Desktop, Joystick/Gamepad) de la guitarra.
fn find(kind: Kind) -> Option<(std::ffi::CString, u16)> {
    let mut api = hidapi()?.lock().ok()?;
    api.refresh_devices().ok()?;
    let found = api
        .device_list()
        .find(|d| {
            kind.ids().contains(&(d.vendor_id(), d.product_id()))
                && d.usage_page() == 0x01
                && matches!(d.usage(), 0x04 | 0x05)
        })
        .map(|d| (d.path().to_owned(), d.product_id()));
    found
}

pub struct Reader {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Reader {
    pub fn start(kind: Kind, mut on_event: impl FnMut(Event) + Send + 'static) -> Reader {
        let stop = Arc::new(AtomicBool::new(false));
        let stop2 = stop.clone();
        let thread = thread::Builder::new()
            .name(format!("reader-{}", kind.key()))
            .spawn(move || run(kind, &stop2, &mut on_event))
            .expect("hilo lector");
        Reader { stop, thread: Some(thread) }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn sleep_unless(stop: &AtomicBool, d: Duration) {
    let end = Instant::now() + d;
    while Instant::now() < end && !stop.load(Ordering::Relaxed) {
        thread::sleep(Duration::from_millis(50));
    }
}

fn run(kind: Kind, stop: &AtomicBool, on_event: &mut dyn FnMut(Event)) {
    on_event(Event::Status(Status::Searching));
    while !stop.load(Ordering::Relaxed) {
        let Some((path, pid)) = find(kind) else {
            sleep_unless(stop, Duration::from_secs(1));
            continue;
        };
        let opened = hidapi().and_then(|api| {
            let api = api.lock().ok()?;
            let rd = api.open_path(&path).ok()?;
            // segundo handle para escribir: una lectura bloqueada no debe frenar el keep-alive
            let wr = match kind.keepalive() {
                Some(_) => Some(api.open_path(&path).ok()?),
                None => None,
            };
            Some((rd, wr))
        });
        let Some((rd, wr)) = opened else {
            sleep_unless(stop, Duration::from_secs(1));
            continue;
        };
        on_event(Event::Status(Status::Connected { pid }));

        let mut buf = [0u8; 128];
        let mut last_poke: Option<Instant> = None;
        loop {
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if let (Some(wr), Some(packet)) = (&wr, kind.keepalive()) {
                if last_poke.is_none_or(|t| t.elapsed() >= KEEPALIVE_INTERVAL) {
                    if wr.write(packet).is_err() {
                        let _ = wr.send_output_report(packet);
                    }
                    last_poke = Some(Instant::now());
                }
            }
            match rd.read_timeout(&mut buf, READ_TIMEOUT_MS) {
                Ok(0) => {}
                Ok(n) => {
                    if let Some(state) = kind.parse(&buf[..n], pid) {
                        on_event(Event::State(state));
                    }
                }
                Err(_) => break, // desenchufado
            }
        }
        drop((rd, wr));
        if stop.load(Ordering::Relaxed) {
            break;
        }
        on_event(Event::Status(Status::Disconnected));
        sleep_unless(stop, Duration::from_secs(1));
    }
}

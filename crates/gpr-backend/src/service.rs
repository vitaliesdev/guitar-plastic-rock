//! Driver en segundo plano (`--background`): un mando Xbox 360 virtual por cada guitarra
//! conectada, icono en la bandeja y estado publicado cada segundo para la app.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use gpr_core::brand::VERSION;
use gpr_core::instruments::Kind;
use gpr_core::status::{unix_now, DriverStatus};
use gpr_platform::{instance, log, process, status_file};
use vigem_client::Client;
use windows::Win32::Foundation::WAIT_OBJECT_0;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, TranslateMessage, MSG, PM_REMOVE, QS_ALLINPUT,
};

use crate::input::Reader;
use crate::tray::{Tray, TrayAction};
use crate::virtual_pad::Bridge;

const VIGEM_RETRY: Duration = Duration::from_secs(5);
const PUBLISH_EVERY: Duration = Duration::from_secs(1);
const LOOP_TICK_MS: u32 = 250;

/// Ejecuta el driver hasta que alguien active su evento de parada. Devuelve el código de salida.
pub fn run() -> i32 {
    let Some(_instance) = instance::acquire_driver() else {
        return 0; // ya hay un driver en marcha
    };
    let Some(stop) = instance::create_stop_event() else {
        return 1;
    };
    log(&format!("driver {VERSION} iniciado (pid {})", std::process::id()));

    let mut client = Client::connect().ok().map(Arc::new);
    if client.is_none() {
        log("ViGEmBus no disponible: se reintentará cada 5 s");
    }
    let mut last_vigem_try = Instant::now();

    let bridges: Vec<Arc<Mutex<Bridge>>> =
        Kind::ALL.iter().map(|&kind| Arc::new(Mutex::new(Bridge::new(kind, client.clone())))).collect();
    let readers: Vec<Reader> = bridges
        .iter()
        .map(|b| {
            let kind = b.lock().unwrap().kind();
            let b = b.clone();
            Reader::start(kind, move |ev| {
                if let Ok(mut b) = b.lock() {
                    b.on_event(ev);
                }
            })
        })
        .collect();

    let tray = Tray::new();
    let mut last_tick = Instant::now() - PUBLISH_EVERY;
    let mut last_published: Option<DriverStatus> = None;
    let mut status_failed = false;
    loop {
        let wait = unsafe { MsgWaitForMultipleObjects(Some(&[stop.raw()]), false, LOOP_TICK_MS, QS_ALLINPUT) };
        if wait == WAIT_OBJECT_0 {
            break;
        }
        pump_messages();
        for action in tray.actions() {
            match action {
                TrayAction::OpenApp => {
                    process::open_app();
                }
                TrayAction::StopDriver => {
                    instance::stop_driver();
                }
            }
        }
        // ViGEmBus instalado con el driver ya en marcha: conectar y crear los mandos sin reiniciar
        if client.is_none() && last_vigem_try.elapsed() >= VIGEM_RETRY {
            last_vigem_try = Instant::now();
            if let Ok(c) = Client::connect() {
                log("ViGEmBus disponible");
                let c = Arc::new(c);
                for b in &bridges {
                    b.lock().unwrap().set_client(c.clone());
                }
                client = Some(c);
            }
        }
        if last_tick.elapsed() >= PUBLISH_EVERY {
            last_tick = Instant::now();
            let (status, connected) = snapshot(&bridges, client.is_some(), last_published.as_ref());
            if !status_file::write(&status) && !status_failed {
                status_failed = true;
                log("no se pudo escribir status.json");
            }
            tray.set_connected(&connected);
            last_published = Some(status);
        }
    }

    log("deteniendo");
    drop(readers);
    for b in &bridges {
        b.lock().unwrap().unplug();
    }
    let mut status = status_file::read().unwrap_or_default();
    for d in status.devices.values_mut() {
        d.status = "detenido".into();
        d.xinput_slot = None;
    }
    status_file::write(&status);
    0
}

fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Estado de todas las guitarras y nombres de las conectadas.
fn snapshot(
    bridges: &[Arc<Mutex<Bridge>>],
    vigem: bool,
    previous: Option<&DriverStatus>,
) -> (DriverStatus, Vec<&'static str>) {
    let mut status = DriverStatus {
        version: VERSION.into(),
        updated: unix_now(),
        pid: std::process::id(),
        vigem,
        error: (!vigem).then(|| "ViGEmBus no está instalado: solo se mantiene activa la guitarra".to_string()),
        devices: Default::default(),
    };
    let mut connected = Vec::new();
    for b in bridges {
        // sin bloquearse: si un hilo de guitarra está ocupado (p. ej. creando su mando), se
        // publica su último estado conocido y el bucle sigue atendiendo la parada
        let Ok(mut b) = b.try_lock() else {
            if let Some(prev) = previous {
                for (k, v) in &prev.devices {
                    status.devices.entry(k.clone()).or_insert_with(|| v.clone());
                }
            }
            continue;
        };
        if status.error.is_none() {
            status.error = b.error().map(str::to_string);
        }
        let ds = b.device_status();
        if ds.status == "conectado" {
            connected.push(b.kind().name());
        }
        status.devices.insert(b.kind().key().into(), ds);
    }
    (status, connected)
}

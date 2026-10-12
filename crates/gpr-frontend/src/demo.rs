//! `--demo`: pulsaciones simuladas en todas las guitarras, para revisar la interfaz sin hardware.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use eframe::egui;
use gpr_backend::input::{Event, Status};
use gpr_core::instruments::Kind;

use crate::guitars;
use crate::model::Live;

pub fn spawn(lives: Vec<(Kind, Arc<Mutex<Live>>)>, ctx: egui::Context) {
    std::thread::spawn(move || {
        let t0 = Instant::now();
        loop {
            let t = t0.elapsed().as_secs_f32();
            for (kind, live) in &lives {
                let mut l = live.lock().unwrap();
                l.status = Status::Connected { pid: 0 };
                l.on_event(Event::State(guitars::demo(*kind, t)));
            }
            ctx.request_repaint();
            std::thread::sleep(Duration::from_millis(16));
        }
    });
}

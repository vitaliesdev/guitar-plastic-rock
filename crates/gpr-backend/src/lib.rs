//! Backend: habla con el hardware y con ViGEmBus.
//!
//! - [`input`]: un hilo por guitarra que la busca, la abre, la mantiene activa y entrega su estado
//!   (lo usan el driver y también la app, para mostrar las pulsaciones en directo)
//! - [`virtual_pad`]: el mando Xbox 360 virtual de cada guitarra
//! - [`service`]: el driver en segundo plano (`--background`) que une las dos cosas
//! - [`tray`]: su icono en la bandeja del sistema

pub mod input;
pub mod service;
mod tray;
pub mod virtual_pad;

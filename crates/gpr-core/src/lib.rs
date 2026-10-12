//! Núcleo de Guitar Plastic Rock: el dominio, sin Windows ni entrada/salida (todo se prueba con
//! `cargo test` en cualquier sistema).
//!
//! - [`instruments`]: guitarras compatibles, cómo se reconocen y cómo se decodifican sus reportes HID
//! - [`gamepad`]: el mando Xbox 360 al que se traduce cada guitarra
//! - [`status`]: el estado que el driver publica para la app
//! - [`brand`] y [`cli`]: identidad de la aplicación y argumentos de línea de comandos

pub mod brand;
pub mod cli;
pub mod gamepad;
pub mod instruments;
pub mod status;

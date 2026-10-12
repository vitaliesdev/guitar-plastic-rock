//! Todo lo que depende de Windows, detrás de funciones sencillas para el backend y el frontend.
//!
//! - [`paths`] y [`logging`]: carpeta de datos y registro de actividad
//! - [`instance`]: instancia única del driver y eventos con nombre (parar el driver, cerrar ventanas)
//! - [`process`]: lanzar procesos y diálogos del sistema
//! - [`status_file`]: el estado que publica el driver, en disco
//! - [`installation`]: inicio con Windows y desinstalador del Setup
//! - [`settings`]: preferencias del usuario (idioma) e idioma de Windows
//! - [`vigem`]: ViGEmBus (mandos virtuales) y su instalador incluido
//! - [`xinput`]: lo que ve Windows en cada mando Xbox

pub mod installation;
pub mod instance;
pub mod logging;
pub mod paths;
pub mod process;
pub mod settings;
pub mod status_file;
pub mod vigem;
pub mod xinput;

pub use logging::log;

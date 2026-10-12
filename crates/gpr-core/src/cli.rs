//! Argumentos de línea de comandos del ejecutable. El mismo .exe es la app, el driver y las
//! utilidades que usa el Setup; aquí están los nombres para que nadie los escriba a mano.

/// El driver en segundo plano (el que arranca con Windows).
pub const BACKGROUND: &str = "--background";
/// Detiene el driver y cierra las ventanas abiertas de la app (lo usa el Setup).
pub const STOP: &str = "--stop";
/// Instala ViGEmBus con el instalador oficial incluido (lo usa el Setup).
pub const INSTALL_VIGEM: &str = "--install-vigem";
/// La app con pulsaciones simuladas, sin guitarras. Admite `riff`, `novigem`, `settings` y `langmenu` detrás.
pub const DEMO: &str = "--demo";

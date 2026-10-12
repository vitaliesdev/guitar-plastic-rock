//! Identidad de la aplicación: un único sitio para el nombre, la versión y el icono.

/// Nombre que ve el usuario.
pub const DISPLAY_NAME: &str = "Guitar Plastic Rock";

/// Identificador sin espacios: carpeta de datos, objetos con nombre de Windows y valor de inicio
/// automático. Cambiarlo rompe la comunicación con versiones anteriores ya instaladas.
pub const APP_ID: &str = "GuitarPlasticRock";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Icono de la aplicación (ventana y bandeja). Generado por tools/make_icon.py.
pub const ICON_PNG: &[u8] = include_bytes!("../../../assets/brand/icon.png");

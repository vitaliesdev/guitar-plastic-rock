# Arquitectura

Guitar Plastic Rock es un único ejecutable (`GuitarPlasticRock.exe`) que hace tres papeles según cómo
se arranque:

| Papel | Arranque | Qué hace |
|---|---|---|
| **App** | sin argumentos | Ventana: probador visual de las guitarras y control del driver. Arranca el driver si no está en marcha. |
| **Driver** | `--background` | Sin ventana: un mando Xbox 360 virtual por guitarra, icono en la bandeja. Es lo que arranca con Windows. |
| **Utilidades del Setup** | `--stop`, `--install-vigem` | Detener el driver y cerrar las ventanas antes de actualizar; instalar ViGEmBus. |

El código es un *workspace* de Rust dividido en capas. Cada capa solo conoce las de debajo:

```
                 ┌──────────────┐
                 │   gpr-app    │  main.rs: reparte según los argumentos
                 └──────┬───────┘
          ┌─────────────┴─────────────┐
   ┌──────▼───────┐            ┌──────▼───────┐
   │ gpr-frontend │──────────▶ │ gpr-backend  │  la app usa también el lector HID
   │   (la app)   │            │ (el driver)  │  para mostrar las pulsaciones en directo
   └──────┬───────┘            └──────┬───────┘
          └─────────────┬─────────────┘
                 ┌──────▼───────┐
                 │ gpr-platform │  todo lo que depende de Windows (+ gpr-i18n: textos)
                 └──────┬───────┘
                 ┌──────▼───────┐
                 │   gpr-core   │  dominio puro: sin Windows ni E/S, todo con pruebas
                 └──────────────┘
```

## Crates

### `gpr-core`: dominio
Sin dependencias de sistema; se prueba con `cargo test` en cualquier plataforma.

| Módulo | Contenido |
|---|---|
| `instruments/` | Una guitarra por archivo: `SPEC` (VID/PID, keep-alive), su estado, `parse` (reporte HID → estado) y `to_xreport` (estado → mando Xbox). `mod.rs` las registra en `Kind` y `State`. |
| `gamepad` | `XReport`, el reporte de mando Xbox 360, y los nombres de sus botones. |
| `status` | `DriverStatus`, lo que el driver publica para la app (y cuándo se considera colgado). |
| `brand`, `cli` | Nombre, identificador, versión e icono; argumentos de línea de comandos. |

### `gpr-i18n`: idiomas
Los textos de la interfaz en 9 idiomas (es, en, ru, fr, de, it, pt-BR, pl, uk), uno por archivo. Cada
idioma es una constante `Texts`: si falta un texto, no compila, y una prueba comprueba que los huecos
(`{n}`, `{app}`…) coinciden con el inglés. Los números siempre van como «etiqueta: {n}», sin reglas
de plural. La app usa el idioma de Windows o el elegido en Ajustes (`settings.json`); la bandeja del
driver lo lee al arrancar. El instalador tiene los mismos 9 idiomas.

### `gpr-platform`: Windows
Funciones pequeñas que esconden la API de Windows.

| Módulo | Contenido |
|---|---|
| `paths`, `logging` | `%LOCALAPPDATA%\GuitarPlasticRock`, `driver.log` con rotación. |
| `instance` | Mutex de instancia única del driver y eventos con nombre: parar el driver, cerrar ventanas. |
| `status_file` | Lectura y escritura atómica de `status.json`. |
| `installation` | Inicio con Windows (clave `Run`) y desinstalador de Inno Setup. |
| `settings` | Preferencias del usuario (`settings.json`) e idioma de Windows. |
| `vigem` | ¿Está ViGEmBus? Su instalador oficial va incrustado y se ejecuta con UAC. |
| `process`, `xinput` | Lanzar procesos y diálogos; leer un mando XInput. |

### `gpr-backend`: servicio
| Módulo | Contenido |
|---|---|
| `input` | `Reader`: un hilo por guitarra que la busca, la abre, envía el keep-alive y entrega `Event`s. |
| `virtual_pad` | `Bridge`: el mando Xbox virtual de una guitarra (crearlo, actualizarlo, retirarlo). |
| `service` | `run()`: el bucle del driver; une lectores y mandos, atiende la bandeja y publica el estado. |
| `tray` | Icono, menú y tooltip de la bandeja. |

### `gpr-frontend`: app
egui con el lenguaje visual de Windows 11 (Fluent).

| Módulo | Contenido |
|---|---|
| `app` | Estado de la app, acciones (reiniciar driver, instalar ViGEmBus…) y composición de la ventana. |
| `model` | Estado en directo de cada guitarra (`Live`, `Guitar`) y del driver (`DriverInfo`). |
| `guitars/` | Lo visual de cada guitarra: foto, zonas iluminables, colores, lectura común (`Readout`), demo. |
| `views/` | Una zona de la ventana por archivo: selector, InfoBar, guitarra, medidores, barra de comandos, ajustes. |
| `widgets`, `theme/`, `images` | Controles Fluent, paleta y fuentes del sistema, decodificación de imágenes. |

Las vistas no distinguen entre guitarras: preguntan a `guitars` (`visual`, `readout`, `control`).

### `gpr-app`: ejecutable
`main.rs` reparte según el argumento; `build.rs` incrusta el icono y los datos de versión.

## Comunicación entre la app y el driver

Son procesos distintos (el driver sigue en marcha al cerrar la ventana):

- **¿Hay driver?** Mutex con nombre `Local\GuitarPlasticRock.Driver`.
- **Parar el driver:** evento `Local\GuitarPlasticRock.Stop`.
- **Cerrar las ventanas** (lo hace el Setup antes de actualizar): evento `Local\GuitarPlasticRock.QuitApp`.
- **Estado:** el driver escribe `status.json` cada segundo (guitarras, mando Xbox asignado, errores). Si
  deja de actualizarse más de 5 s, la app avisa de que no responde.

## Distribución

- `installer/GuitarPlasticRock.iss` (Inno Setup): instalación por usuario, sin administrador. Al
  instalar detiene el driver anterior (y desinstala la versión antigua *GuitarBridge* si existe).
- El mismo `.exe` sirve como versión portable.
- GitHub Actions (`.github/workflows/build.yml`): clippy, pruebas, compilación, Setup y portable; al
  etiquetar `v*` los publica como release.

## Ampliar

- **Una guitarra nueva:** [ADDING_A_GUITAR.md](ADDING_A_GUITAR.md).
- **Otra función del driver** (p. ej. reasignar botones): en `gpr-core` la lógica pura con pruebas;
  en `gpr-backend` dónde se aplica; si necesita configuración, guardarla en `gpr-platform::paths::data_dir()`.
- **Otra pantalla o ajuste:** una vista nueva en `gpr-frontend/src/views/` y su estado en `App`.

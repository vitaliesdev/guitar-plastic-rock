# Añadir una guitarra

Ejemplo: una guitarra *Xplorer*. Hacen falta cuatro piezas. El compilador avisa de cualquier `match`
que se quede sin la guitarra nueva, así que no se puede olvidar ninguna.

## 1. Protocolo: `crates/gpr-core/src/instruments/xplorer.rs`

Copia `riffmaster.rs` como plantilla:

- `VID`, `PID` y `SPEC` (clave, nombre, ids aceptados, keep-alive si lo necesita).
- `XplorerState`: botones y ejes ya normalizados (`bool`, `0..1`, `-1..1`).
- `parse(report) -> Option<XplorerState>`: decodifica el reporte HID. La referencia de formatos es
  [PlasticBand](https://github.com/TheNathannator/PlasticBand).
- `impl InstrumentState`: `to_xreport` (la distribución de Xbox 360 que reconocen Clone Hero y YARG),
  `strum` y `frets_held`.
- Pruebas con **reportes reales capturados** (en reposo y con botones pulsados).

Regístrala en `instruments/mod.rs`: `pub mod xplorer;`, una variante en `Kind` (y en `Kind::ALL`),
otra en `State`, y sus brazos en `spec`, `parse`, `empty`, `kind` e `inner`.

## 2. Foto: `assets/guitars/xplorer.png`

Foto recortada con fondo transparente. `tools/build_assets.py` la genera a partir de la foto
original, que se guarda en `assets/source/`.

## 3. Parte visual: `crates/gpr-frontend/src/guitars/xplorer.rs`

Copia `riffmaster.rs` como plantilla:

- `VISUAL`: la foto (`include_bytes!`), su tamaño en píxeles, las zonas iluminables y cómo se llama la
  inclinación en su juego.
- Las zonas, en píxeles de la foto: `(control, forma, nombre, botón Xbox)`. Si hace falta un control
  nuevo, añádelo a `Ctl` en `guitars/mod.rs`.
- `readout`, `control` (color de cada control al pulsarlo) y `demo`.

Regístrala en `guitars/mod.rs` (`visual`, `readout`, `control`, `draw_extras`, `demo`).

## 4. Comprobar

```bash
cargo test --workspace     # decodificación, registro sin claves repetidas, zonas dentro de la foto
cargo run --release -- --demo
```

Y con la guitarra real: que cada control se ilumine en su sitio y que el mando Xbox reciba lo
esperado (Ajustes → Detalles técnicos).

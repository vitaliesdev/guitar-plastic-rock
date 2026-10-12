# Avisos de terceros

Guitar Plastic Rock se distribuye bajo la licencia MIT (ver `LICENSE`). Incluye o depende de:

| Componente | Licencia | Uso |
|---|---|---|
| [egui / eframe](https://github.com/emilk/egui) | MIT OR Apache-2.0 | Interfaz |
| [hidapi (crate)](https://github.com/ruabmbua/hidapi-rs) | MIT | Lectura de los receptores USB |
| [vigem-client](https://github.com/CasualX/vigem-client) | MIT | Mandos Xbox 360 virtuales |
| [tray-icon](https://github.com/tauri-apps/tray-icon) | MIT OR Apache-2.0 | Icono de la bandeja |
| [windows-rs](https://github.com/microsoft/windows-rs) | MIT OR Apache-2.0 | API de Windows |
| [winreg](https://github.com/gentoo90/winreg-rs) | MIT | Registro de Windows |
| [image](https://github.com/image-rs/image) | MIT OR Apache-2.0 | Imágenes incrustadas |
| [serde / serde_json](https://github.com/serde-rs) | MIT OR Apache-2.0 | Estado del driver |

El resto de dependencias transitivas usan licencias compatibles (MIT, Apache-2.0, BSD, Zlib); `cargo tree` muestra la lista completa.

**Inno Setup** ([jrsoftware.org](https://jrsoftware.org/isinfo.php)) genera el instalador (`installer/GuitarPlasticRock.iss`); su licencia permite distribuir los instaladores creados con él.

**ViGEmBus** ([nefarius/ViGEmBus](https://github.com/nefarius/ViGEmBus)): el ejecutable (y por tanto el instalador y la versión portable) incluye, sin modificar, su instalador oficial v1.22.0 (`third_party/vigembus/ViGEmBus_1.22.0_x64_x86_arm64.exe`, firmado por Nefarius Software Solutions e.U.), que solo se ejecuta si falta ViGEmBus y el usuario lo acepta. Se redistribuye bajo su licencia BSD-3-Clause:

```
Copyright (c) 2016-2020, Nefarius Software Solutions e.U.
All rights reserved.
```

El texto completo de la licencia está en `third_party/vigembus/LICENSE.txt`.

**Formatos de los dispositivos**: documentación de [PlasticBand](https://github.com/TheNathannator/PlasticBand) y el driver de Linux `drivers/hid/hid-sony.c`. Se usan como referencia técnica (datos de interoperabilidad); no se copia su código.

**Fuentes**: la app usa en tiempo de ejecución las fuentes de Windows (Segoe UI, Cascadia Mono); no se incluyen en el ejecutable.

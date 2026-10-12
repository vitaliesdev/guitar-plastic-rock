//! Recursos de Windows del .exe: icono y datos de versión (Propiedades > Detalles).

const ICON: &str = "../../assets/brand/icon.ico";

fn main() {
    println!("cargo:rerun-if-changed={ICON}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ICON)
            .set("ProductName", "Guitar Plastic Rock")
            .set("FileDescription", "Guitar Plastic Rock: guitarras de Guitar Hero y Rock Band en PC")
            .set("OriginalFilename", "GuitarPlasticRock.exe")
            .set("LegalCopyright", "MIT");
        res.compile().expect("recursos de Windows (icono)");
    }
}

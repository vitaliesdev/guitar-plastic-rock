//! Utilidades comunes para decodificar reportes HID.

/// Hat de cruceta PS (0 = arriba, en sentido horario; cualquier otro valor = suelto) -> (arriba, abajo, izq, der).
pub(crate) fn hat(v: u8) -> (bool, bool, bool, bool) {
    match v & 0x0F {
        0 => (true, false, false, false),
        1 => (true, false, false, true),
        2 => (false, false, false, true),
        3 => (false, true, false, true),
        4 => (false, true, false, false),
        5 => (false, true, true, false),
        6 => (false, false, true, false),
        7 => (true, false, true, false),
        _ => (false, false, false, false),
    }
}

/// OR de los bits cuyo booleano está activo.
pub(crate) fn bits(pairs: &[(bool, u16)]) -> u16 {
    pairs.iter().filter(|(on, _)| *on).fold(0, |acc, (_, bit)| acc | bit)
}

/// Eje de 8 bits centrado en 0x80 -> -1..1.
pub(crate) fn axis(v: u8) -> f32 {
    ((v as f32 - 128.0) / 127.0).clamp(-1.0, 1.0)
}

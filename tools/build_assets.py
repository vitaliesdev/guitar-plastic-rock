"""Adapta las fotos de las guitarras a la interfaz del probador (requiere Pillow).

Quita el fondo con un relleno que sigue el degradado desde los bordes, recorta, escala a un
ancho común y añade un contorno claro para que la guitarra negra se lea sobre el panel oscuro.
Uso:  python tools/build_assets.py [carpeta_con_las_fotos]
       (por defecto, assets/source: las fotos originales ghl1.png y rif1.png)
"""
import sys
from collections import deque
from pathlib import Path

from PIL import Image, ImageChops, ImageFilter

REPO = Path(__file__).resolve().parent.parent
ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else REPO / "assets" / "source"  # fotos originales
OUT = REPO / "assets" / "guitars"
WIDTH = 1120  # 2x del ancho de 560 px que ocupa en el probador
RIM = (122, 132, 150)  # contorno: gris azulado de la interfaz

SOURCES = {
    # archivo, tolerancia del relleno, píxeles a recortar del borde (restos del fondo)
    # (las fotos con transparencia ignoran la tolerancia: se usa su canal alfa)
    "ghl": ("ghl1.png", 7, 0),
    "riffmaster": ("rif1.png", 10, 0),
}
SHADOW_ALPHA = 120  # la sombra de las fotos es negra con alfa ~87: todo lo menos opaco se descarta


def background_mask(img: Image.Image, step_tol: int) -> Image.Image:
    """Crece desde los bordes mientras el color cambie poco entre vecinos (fondo liso o degradado)."""
    rgb = img.convert("RGB")
    w, h = rgb.size
    px = rgb.load()
    seen = bytearray(w * h)
    q = deque()
    for x in range(w):
        q.append((x, 0))
        q.append((x, h - 1))
    for y in range(h):
        q.append((0, y))
        q.append((w - 1, y))
    for x, y in q:
        seen[y * w + x] = 1
    while q:
        x, y = q.popleft()
        c = px[x, y]
        for nx, ny in ((x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)):
            if 0 <= nx < w and 0 <= ny < h and not seen[ny * w + nx]:
                n = px[nx, ny]
                if max(abs(n[0] - c[0]), abs(n[1] - c[1]), abs(n[2] - c[2])) <= step_tol and \
                        max(n) - min(n) < 40:  # el fondo es gris (poco saturado)
                    seen[ny * w + nx] = 1
                    q.append((nx, ny))
    mask = Image.frombytes("L", (w, h), bytes(0 if s else 255 for s in seen))
    # cierra huecos de un píxel y suaviza el borde
    return mask.filter(ImageFilter.MaxFilter(3)).filter(ImageFilter.MinFilter(3))


def build(key, filename, tol, erode):
    img = Image.open(ROOT / filename)
    if "A" in img.getbands():
        mask = img.getchannel("A").point(lambda a: 255 if a >= SHADOW_ALPHA else 0)
        mask = mask.filter(ImageFilter.MinFilter(3)).filter(ImageFilter.MaxFilter(3))  # quita motas sueltas
    else:
        mask = background_mask(img, tol)
    src = img.convert("RGB")
    if erode:
        mask = mask.filter(ImageFilter.MinFilter(2 * erode + 1))
    bbox = mask.getbbox()
    src, mask = src.crop(bbox), mask.crop(bbox)
    scale = WIDTH / src.width
    size = (WIDTH, round(src.height * scale))
    src = src.resize(size, Image.LANCZOS)
    mask = mask.resize(size, Image.LANCZOS).filter(ImageFilter.GaussianBlur(1.2))

    pad = 12
    canvas = Image.new("RGBA", (size[0] + 2 * pad, size[1] + 2 * pad), (0, 0, 0, 0))
    full = Image.new("L", canvas.size, 0)
    full.paste(mask, (pad, pad))
    # contorno: máscara dilatada menos la original
    rim_alpha = ImageChops.subtract(full.filter(ImageFilter.MaxFilter(5)), full).filter(ImageFilter.GaussianBlur(1))
    rim = Image.new("RGBA", canvas.size, RIM + (0,))
    rim.putalpha(rim_alpha.point(lambda a: a * 0.85))
    canvas.alpha_composite(rim)
    body = Image.new("RGBA", canvas.size, (0, 0, 0, 0))
    body.paste(src, (pad, pad))
    body.putalpha(full)
    canvas.alpha_composite(body)
    OUT.mkdir(parents=True, exist_ok=True)
    canvas.save(OUT / f"{key}.png", optimize=True)
    print(f"{key}.png {canvas.size}")


if __name__ == "__main__":
    for key, (filename, tol, erode) in SOURCES.items():
        build(key, filename, tol, erode)

"""Genera el icono de la app (púa naranja con trastes) en assets/brand/icon.png y assets/brand/icon.ico.
Uso:  python tools/make_icon.py   (requiere Pillow)
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parent.parent / "assets" / "brand"
ACCENT = (255, 122, 47, 255)
DARK = (15, 17, 21, 255)


def convex_hull(points):
    pts = sorted(set(points))

    def cross(o, a, b):
        return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0])

    lower, upper = [], []
    for p in pts:
        while len(lower) >= 2 and cross(lower[-2], lower[-1], p) <= 0:
            lower.pop()
        lower.append(p)
    for p in reversed(pts):
        while len(upper) >= 2 and cross(upper[-2], upper[-1], p) <= 0:
            upper.pop()
        upper.append(p)
    return lower[:-1] + upper[:-1]


def pick(size: int) -> Image.Image:
    s = size * 4  # supermuestreo para bordes suaves
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # púa: envolvente convexa de dos círculos grandes arriba y uno pequeño abajo
    circles = [(s * 0.30, s * 0.30, s * 0.20), (s * 0.70, s * 0.30, s * 0.20), (s * 0.50, s * 0.84, s * 0.09)]
    pts = [(cx + r * math.cos(a / 90 * math.pi), cy + r * math.sin(a / 90 * math.pi))
           for cx, cy, r in circles for a in range(180)]
    d.polygon(convex_hull(pts), fill=ACCENT)
    # tres "trastes" oscuros
    w, h = s * 0.12, s * 0.10
    for i in range(3):
        x = s * 0.5 + (i - 1) * s * 0.16 - w / 2
        y = s * 0.36
        d.rounded_rectangle([x, y, x + w, y + h], radius=s * 0.025, fill=DARK)
    d.rounded_rectangle([s * 0.5 - w / 2, s * 0.52, s * 0.5 + w / 2, s * 0.52 + h], radius=s * 0.025, fill=DARK)
    return img.resize((size, size), Image.LANCZOS)


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    big = pick(256)
    big.save(OUT / "icon.png")
    big.save(OUT / "icon.ico", sizes=[(16, 16), (20, 20), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
    print("icon.png / icon.ico")

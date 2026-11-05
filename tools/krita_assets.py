"""Generate every raster asset for SPY HUNTER: INTERCEPTOR with Krita 6.

    ~/.agents/skills/krita/scripts/kritarun.sh tools/krita_assets.py

All artwork is original 1x pixel art drawn with QImage pixel writes and
QPainter primitives, pushed through Krita layers and exported from the merged
projection. Source `.kra` boards land in assets/art/ for hand editing.

No sprite, screenshot, or tile from the 1983 arcade ROM is used.
"""

import ksketch as ks

ks.started()

import math
import os
import random
import sys
import traceback

from PyQt6.QtCore import Qt
from PyQt6.QtGui import QColor, QImage, QPainter

import pixelfont

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SPRITES = os.path.join(ROOT, "assets", "sprites")
BACKGROUNDS = os.path.join(ROOT, "assets", "backgrounds")
ART = os.path.join(ROOT, "assets", "art")

for d in (SPRITES, BACKGROUNDS, ART):
    os.makedirs(d, exist_ok=True)

TRANSPARENT = QColor(0, 0, 0, 0)


def C(hexstr, a=255):
    hexstr = hexstr.lstrip("#")
    return QColor(int(hexstr[0:2], 16), int(hexstr[2:4], 16), int(hexstr[4:6], 16), a)


# ---------------------------------------------------------------------------
# Palette
# ---------------------------------------------------------------------------
INK = C("14161c")
ASPHALT = C("3a3d44")
ASPHALT_DARK = C("2c2f35")
LINE = C("e7e2d4")
LANE = C("d8c56a")
GRASS = C("2c6338")
GRASS_DARK = C("24522f")
SNOW = C("dfe9f2")
SNOW_DARK = C("b9cee0")
WATER = C("1c5d86")
WATER_DARK = C("164a6c")
WATER_LIGHT = C("2f86b4")
WHITE = C("f3f1e7")
SILVER = C("c9cbd0")
STEEL = C("7e848d")
RED = C("cf3a3a")
RED_DARK = C("8e2229")
BLUE = C("355fa8")
BLUE_DARK = C("243f70")
TEAL = C("3f8f8a")
TEAL_DARK = C("2a6663")
GOLD = C("e5c47c")
AMBER = C("f0a63a")
ORANGE = C("ef7a2a")
YELLOW = C("ffe066")
CYAN = C("7fd4e6")
GLASS = C("1b2f3d")
GLASS_HI = C("4f97ad")
TIRE = C("16181d")
HUB = C("8d939c")
SMOKE = C("9aa0a8")
GREEN_DARK = C("1d4427")
NIGHT = C("101522")
NIGHT_2 = C("1a2233")


class Pix:
    def __init__(self, w, h):
        self.w, self.h = w, h
        self.img = QImage(w, h, QImage.Format.Format_RGBA8888)
        self.img.fill(TRANSPARENT)

    def px(self, x, y, c):
        x, y = int(x), int(y)
        if 0 <= x < self.w and 0 <= y < self.h:
            self.img.setPixelColor(x, y, c)

    def rect(self, x, y, w, h, c):
        for yy in range(int(y), int(y + h)):
            for xx in range(int(x), int(x + w)):
                self.px(xx, yy, c)

    def frame(self, x, y, w, h, c):
        self.hline(x, y, w, c)
        self.hline(x, y + h - 1, w, c)
        self.vline(x, y, h, c)
        self.vline(x + w - 1, y, h, c)

    def hline(self, x, y, w, c):
        for xx in range(int(x), int(x + w)):
            self.px(xx, y, c)

    def vline(self, x, y, h, c):
        for yy in range(int(y), int(y + h)):
            self.px(x, yy, c)

    def line(self, x0, y0, x1, y1, c):
        x0, y0, x1, y1 = int(x0), int(y0), int(x1), int(y1)
        dx, dy = abs(x1 - x0), -abs(y1 - y0)
        sx = 1 if x0 < x1 else -1
        sy = 1 if y0 < y1 else -1
        err = dx + dy
        while True:
            self.px(x0, y0, c)
            if x0 == x1 and y0 == y1:
                break
            e2 = 2 * err
            if e2 >= dy:
                err += dy
                x0 += sx
            if e2 <= dx:
                err += dx
                y0 += sy

    def disc(self, cx, cy, r, c):
        r = int(r)
        for yy in range(-r, r + 1):
            span = int((r * r - yy * yy) ** 0.5)
            self.hline(cx - span, cy + yy, span * 2 + 1, c)

    def ellipse(self, cx, cy, rx, ry, c):
        for yy in range(-int(ry), int(ry) + 1):
            v = 1.0 - (yy * yy) / float(max(1, ry * ry))
            if v < 0:
                continue
            span = int(rx * math.sqrt(v))
            self.hline(cx - span, cy + yy, span * 2 + 1, c)


def clear_background(doc):
    """Krita creates an opaque white Background layer; hide it so exports keep
    real alpha instead of a white card behind every sprite."""
    for node in doc.rootNode().childNodes():
        if node.name() == "Background":
            node.setVisible(False)


def export_sprite(name, pix, keep_source=False):
    doc = ks.new_doc(pix.w, pix.h, name)
    clear_background(doc)
    lay = ks.new_layer(doc, "art")
    ks.write_pixels(lay, pix.img)
    ok = ks.export_png(doc, os.path.join(SPRITES, name + ".png"))
    if not ok:
        raise RuntimeError("export failed for " + name)
    if keep_source:
        ks.save_kra(doc, os.path.join(ART, name + ".kra"))
    ks.log("sprite", name, pix.w, pix.h)


def body_span(y, profile):
    for (y0, y1, x0, x1) in profile:
        if y0 <= y < y1:
            return x0, x1
    return None


CAR_PROFILE = [
    (0, 5, 18, 30),
    (5, 10, 15, 33),
    (10, 18, 11, 37),
    (18, 58, 8, 40),
    (58, 66, 11, 37),
    (66, 72, 15, 33),
]


def draw_car(pix, body, body_dark, accent, glass, headlight=YELLOW, taillight=RED,
             gun_pods=False, civilian=False):
    # Tires first, so the body overlaps their inner edge.
    for wy in (13, 47):
        pix.rect(3, wy, 8, 17, TIRE)
        pix.rect(37, wy, 8, 17, TIRE)
        pix.rect(5, wy + 4, 4, 9, HUB)
        pix.rect(39, wy + 4, 4, 9, HUB)

    for y in range(72):
        span = body_span(y, CAR_PROFILE)
        if not span:
            continue
        x0, x1 = span
        pix.hline(x0, y, x1 - x0, INK)
        pix.hline(x0 + 2, y, x1 - x0 - 4, body)
        pix.px(x0 + 1, y, body_dark)
        pix.px(x1 - 2, y, body_dark)

    # Long accent rails.
    pix.vline(11, 12, 48, accent)
    pix.vline(36, 12, 48, accent)
    pix.vline(12, 14, 44, body_dark)
    pix.vline(35, 14, 44, body_dark)

    # Windshield.
    for i in range(11):
        y = 17 + i
        inset = 3 - i // 4
        pix.hline(13 + inset, y, 22 - inset * 2, glass)
    pix.line(16, 18, 32, 27, GLASS_HI)
    pix.line(18, 18, 33, 26, GLASS_HI)

    # Roof / deck.
    pix.rect(14, 29, 20, 17, body)
    pix.rect(15, 30, 18, 2, body_dark)
    pix.rect(16, 32, 16, 12, body_dark if civilian else body)
    pix.rect(22, 31, 4, 13, accent if not civilian else body)
    pix.line(14, 29, 33, 29, INK)

    # Rear window and deck.
    for i in range(9):
        y = 46 + i
        inset = i // 3
        pix.hline(15 + inset, y, 18 - inset * 2, glass)
    pix.rect(13, 56, 22, 8, body)
    pix.rect(13, 56, 22, 1, INK)
    pix.rect(16, 58, 16, 3, accent)

    # Nose and tail details.
    pix.rect(19, 2, 4, 4, body_dark)
    pix.rect(25, 2, 4, 4, body_dark)
    pix.rect(16, 5, 6, 3, headlight)
    pix.rect(26, 5, 6, 3, headlight)
    pix.rect(13, 63, 7, 3, taillight)
    pix.rect(28, 63, 7, 3, taillight)
    pix.rect(17, 67, 14, 2, INK)

    # Mirrors.
    pix.rect(6, 21, 3, 2, body)
    pix.rect(39, 21, 3, 2, body)

    if gun_pods:
        pix.rect(6, 26, 4, 12, INK)
        pix.rect(7, 27, 2, 10, STEEL)
        pix.rect(38, 26, 4, 12, INK)
        pix.rect(39, 27, 2, 10, STEEL)
        pix.px(8, 24, RED)
        pix.px(39, 24, RED)

    if civilian:
        pix.rect(21, 36, 6, 4, WHITE)
        pix.rect(22, 37, 4, 2, TEAL)


def draw_boat(pix, hull, hull_dark, accent, glass):
    profile = [
        (0, 6, 20, 28),
        (6, 16, 14, 34),
        (16, 52, 9, 39),
        (52, 62, 11, 37),
        (62, 72, 15, 33),
    ]
    for y in range(72):
        span = body_span(y, profile)
        if not span:
            continue
        x0, x1 = span
        pix.hline(x0, y, x1 - x0, INK)
        pix.hline(x0 + 2, y, x1 - x0 - 4, hull)
        pix.px(x0 + 1, y, hull_dark)
        pix.px(x1 - 2, y, hull_dark)

    # Deck stripe and cockpit.
    pix.rect(10, 14, 28, 3, accent)
    pix.rect(11, 20, 26, 14, glass)
    pix.rect(12, 21, 24, 2, GLASS_HI)
    pix.rect(16, 18, 16, 2, hull_dark)
    pix.rect(13, 36, 22, 12, hull_dark)
    pix.rect(17, 38, 14, 8, hull)
    pix.rect(22, 40, 4, 5, accent)

    # Engine block and wake vents.
    pix.rect(16, 57, 16, 8, INK)
    pix.rect(18, 58, 12, 5, STEEL)
    pix.rect(21, 59, 2, 3, RED)
    pix.rect(25, 59, 2, 3, RED)
    pix.rect(14, 67, 20, 3, hull_dark)
    pix.rect(8, 18, 3, 30, accent)
    pix.rect(37, 18, 3, 30, accent)


def draw_helicopter():
    p = Pix(96, 72)
    # Rotor blur.
    p.ellipse(48, 34, 34, 26, C("9aa0a8", 45))
    p.ellipse(48, 34, 30, 22, C("c9cbd0", 30))
    p.line(14, 34, 82, 34, C("d9dde3", 120))
    p.line(48, 8, 48, 60, C("d9dde3", 100))
    p.line(26, 17, 70, 51, C("d9dde3", 60))
    p.line(70, 17, 26, 51, C("d9dde3", 60))
    # Tail boom and fin.
    p.rect(45, 0, 6, 22, INK)
    p.rect(46, 2, 4, 20, STEEL)
    p.rect(41, 5, 14, 4, INK)
    p.rect(42, 6, 12, 2, STEEL)
    p.rect(45, 0, 6, 3, RED_DARK)
    # Fuselage.
    p.ellipse(48, 38, 13, 22, INK)
    p.ellipse(48, 38, 11, 20, C("3f4a3d"))
    p.ellipse(48, 36, 9, 14, C("53614f"))
    # Cockpit facing down.
    p.ellipse(48, 50, 8, 7, GLASS)
    p.ellipse(48, 50, 6, 5, GLASS_HI)
    p.rect(44, 34, 8, 3, INK)
    # Skids.
    p.vline(30, 30, 24, INK)
    p.vline(66, 30, 24, INK)
    p.hline(28, 36, 8, INK)
    p.hline(60, 36, 8, INK)
    # Weapon pods.
    p.rect(28, 42, 10, 4, INK)
    p.rect(58, 42, 10, 4, INK)
    p.rect(30, 43, 6, 2, RED)
    p.rect(60, 43, 6, 2, RED)
    # Lights.
    p.px(48, 12, RED)
    p.px(48, 61, YELLOW)
    return p


def draw_van():
    p = Pix(64, 96)
    # Tires.
    for wy in (12, 66):
        p.rect(2, wy, 9, 20, TIRE)
        p.rect(53, wy, 9, 20, TIRE)
        p.rect(4, wy + 5, 5, 10, HUB)
        p.rect(55, wy + 5, 5, 10, HUB)
    # Boxy body.
    p.rect(6, 2, 52, 92, INK)
    p.rect(8, 4, 48, 88, WHITE)
    p.rect(8, 4, 48, 6, SILVER)
    p.rect(8, 88, 48, 4, STEEL)
    # Cab windshield at nose.
    p.rect(14, 8, 36, 10, GLASS)
    p.rect(16, 9, 32, 2, GLASS_HI)
    # Blue command stripe.
    p.rect(8, 24, 48, 9, BLUE)
    p.rect(8, 36, 48, 3, RED)
    # Roof hatch and weapons logo.
    p.rect(22, 48, 20, 20, ASPHALT_DARK)
    p.rect(24, 50, 16, 16, BLUE_DARK)
    p.rect(28, 54, 8, 8, GOLD)
    p.rect(30, 56, 4, 4, INK)
    # Rear doors.
    p.rect(12, 74, 40, 14, SILVER)
    p.vline(32, 74, 14, STEEL)
    p.rect(16, 78, 4, 3, RED)
    p.rect(44, 78, 4, 3, AMBER)
    # Side lights.
    p.rect(9, 44, 3, 4, AMBER)
    p.rect(52, 44, 3, 4, AMBER)
    # Headlights.
    p.rect(12, 3, 8, 3, YELLOW)
    p.rect(44, 3, 8, 3, YELLOW)
    return p


def draw_tree(snow=False):
    p = Pix(48, 72)
    p.ellipse(24, 67, 16, 4, C("15251a", 110))
    p.rect(21, 52, 6, 16, C("3a2418"))
    p.rect(22, 52, 2, 16, C("6b4a2f"))
    layers = [(4, 26, 22), (20, 24, 22), (36, 21, 20)]
    for (base_y, half, h) in layers:
        for yy in range(h):
            w = int(half * (yy + 2) / h)
            y = base_y + yy
            if y >= 70:
                break
            p.hline(24 - w, y, w * 2, C("14301b"))
            p.hline(24 - w + 2, y, max(0, w * 2 - 4), C("2f7040"))
            p.hline(24 - w + 4, y, max(0, w * 2 - 8), C("3f8a4c"))
            if snow and yy < 3:
                p.hline(24 - w + 1, y, max(0, w * 2 - 2), C("e7eff7"))
    if snow:
        p.rect(19, 5, 10, 2, SNOW)
        p.rect(15, 16, 18, 2, SNOW)
        p.rect(11, 29, 26, 2, SNOW)
    p.rect(18, 58, 12, 2, C("6b4a2f"))
    return p


def draw_rock():
    p = Pix(48, 32)
    p.ellipse(24, 22, 20, 9, INK)
    p.ellipse(24, 20, 19, 8, C("6b7078"))
    p.ellipse(22, 18, 14, 6, C("868c94"))
    p.line(12, 17, 24, 12, C("a8aeb6"))
    p.line(24, 22, 36, 18, C("4e535a"))
    return p


def draw_building():
    p = Pix(64, 96)
    p.rect(4, 8, 56, 88, INK)
    p.rect(6, 10, 52, 86, NIGHT_2)
    p.rect(6, 10, 52, 4, C("253047"))
    rnd = random.Random(7)
    for yy in range(18, 88, 8):
        for xx in range(10, 54, 8):
            lit = rnd.random() < 0.55
            p.rect(xx, yy, 5, 4, C("e5c47c") if lit else C("232c3e"))
            if lit and rnd.random() < 0.3:
                p.px(xx + 1, yy + 1, WHITE)
    p.rect(28, 92, 8, 4, C("0b0e16"))
    return p


def draw_oil():
    p = Pix(56, 24)
    p.ellipse(28, 14, 26, 9, C("0a0b0e"))
    p.ellipse(24, 12, 16, 5, C("171a20"))
    p.hline(10, 10, 20, C("3a2a4a"))
    p.hline(16, 8, 14, C("2a3a4a"))
    p.hline(26, 16, 10, C("4a2a3a"))
    p.px(12, 14, C("6a4a7a"))
    p.px(40, 12, C("3a6a5a"))
    return p


def draw_smoke():
    p = Pix(64, 64)
    rnd = random.Random(11)
    for i in range(9):
        cx = 18 + rnd.randint(0, 28)
        cy = 18 + rnd.randint(0, 28)
        r = 8 + rnd.randint(0, 8)
        p.disc(cx, cy, r, C("9aa0a8", 70))
        p.disc(cx - 2, cy - 2, r - 3, C("c9cbd0", 80))
    return p


def draw_explosion(frame):
    p = Pix(96, 96)
    rnd = random.Random(21 + frame)
    outer = [38, 42, 46, 44][frame]
    mid = [26, 31, 36, 30][frame]
    inner = [14, 18, 22, 12][frame]
    # Irregular fireball.
    for i in range(11):
        ang = (i / 11.0) * math.tau
        rr = outer + rnd.randint(-5, 5)
        cx = 48 + int(math.cos(ang) * 10)
        cy = 48 + int(math.sin(ang) * 10)
        p.disc(cx, cy, max(4, rr // 3), C("b83a1e", 210))
    p.disc(48, 48, outer, C("ef7a2a", 235))
    p.disc(48, 48, mid, C("f0a63a"))
    p.disc(48, 48, inner, YELLOW)
    if frame >= 2:
        p.disc(48, 48, max(4, inner - 8), WHITE)
    # Debris.
    for i in range(10):
        ang = rnd.random() * math.tau
        dist = rnd.randint(outer, 46)
        x = 48 + int(math.cos(ang) * dist)
        y = 48 + int(math.sin(ang) * dist)
        p.rect(x, y, 2 + rnd.randint(0, 2), 2, YELLOW if i % 2 else ORANGE)
    return p


def draw_crater():
    p = Pix(72, 48)
    p.ellipse(36, 26, 34, 18, C("22242a"))
    p.ellipse(36, 26, 30, 15, C("0e0f12"))
    p.ellipse(34, 24, 20, 9, C("050507"))
    p.line(8, 16, 20, 22, C("5a4a3a"))
    p.line(52, 14, 44, 21, C("5a4a3a"))
    p.line(12, 36, 24, 31, C("5a4a3a"))
    p.line(60, 36, 48, 31, C("5a4a3a"))
    p.hline(28, 22, 14, C("3a2a1a"))
    return p


def draw_icon(kind):
    p = Pix(32, 32)
    p.rect(1, 1, 30, 30, C("0d131c", 210))
    p.frame(1, 1, 30, 30, C("e5c47c", 180))
    if kind == "gun":
        p.rect(7, 10, 18, 5, STEEL)
        p.rect(7, 12, 12, 3, SILVER)
        p.rect(11, 15, 5, 10, STEEL)
        p.rect(17, 15, 9, 3, STEEL)
        p.rect(22, 8, 3, 4, AMBER)
    elif kind == "oil":
        p.rect(9, 12, 15, 13, C("22252c"))
        p.rect(11, 10, 11, 3, STEEL)
        p.rect(14, 6, 5, 4, STEEL)
        p.hline(12, 17, 8, C("4a2a5a"))
        p.px(24, 24, C("6a4a7a"))
    elif kind == "smoke":
        p.disc(12, 20, 7, SMOKE)
        p.disc(19, 15, 8, C("c9cbd0"))
        p.disc(23, 22, 6, C("8a9098"))
        p.px(10, 12, WHITE)
    elif kind == "missile":
        p.rect(13, 8, 6, 16, WHITE)
        p.rect(13, 8, 6, 4, RED)
        p.rect(10, 20, 12, 4, STEEL)
        p.rect(15, 24, 2, 5, ORANGE)
        p.px(15, 29, YELLOW)
    elif kind == "van":
        p.rect(6, 8, 20, 17, WHITE)
        p.rect(6, 8, 20, 4, BLUE)
        p.rect(9, 14, 14, 6, GLASS)
        p.rect(8, 25, 4, 3, TIRE)
        p.rect(20, 25, 4, 3, TIRE)
    return p


def draw_life_icon():
    p = Pix(22, 34)
    p.rect(5, 2, 12, 30, INK)
    p.rect(6, 3, 10, 28, WHITE)
    p.rect(7, 7, 8, 5, GLASS)
    p.rect(7, 24, 8, 3, RED)
    p.rect(3, 6, 3, 7, TIRE)
    p.rect(16, 6, 3, 7, TIRE)
    p.rect(3, 21, 3, 8, TIRE)
    p.rect(16, 21, 3, 8, TIRE)
    return p


def sprite_strip(name, frames, gap=2):
    w = sum(f.w for f in frames) + gap * (len(frames) - 1)
    h = max(f.h for f in frames)
    p = Pix(w, h)
    x = 0
    for f in frames:
        for yy in range(f.h):
            for xx in range(f.w):
                c = f.img.pixelColor(xx, yy)
                if c.alpha() > 0:
                    p.px(x + xx, yy, c)
        x += f.w + gap
    export_sprite(name, p)


# ---------------------------------------------------------------------------
# Backgrounds (QPainter for speed)
# ---------------------------------------------------------------------------


def background(name, base, mode):
    doc = ks.new_doc(960, 720, name)
    clear_background(doc)
    img, p = ks.blank_pixels(960, 720, base)
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)
    rnd = random.Random(hash(name) & 0xFFFF)
    if mode == "grass":
        for y in range(0, 720, 4):
            shade = C("2f6b3c") if (y // 4) % 2 == 0 else C("2a6035")
            p.fillRect(0, y, 960, 2, shade)
        for _ in range(1400):
            x, y = rnd.randrange(960), rnd.randrange(720)
            c = C("3a7d49") if rnd.random() < 0.6 else C("22512c")
            p.fillRect(x, y, 2, 3, c)
    elif mode == "snow":
        for y in range(0, 720, 5):
            p.fillRect(0, y, 960, 2, C("e7eff7"))
        for _ in range(900):
            x, y = rnd.randrange(960), rnd.randrange(720)
            c = C("c7d9e8") if rnd.random() < 0.5 else C("f7fbff")
            p.fillRect(x, y, 4, 2, c)
    elif mode == "water":
        for y in range(0, 720, 6):
            c = C("1c5d86") if (y // 6) % 2 == 0 else C("19537a")
            p.fillRect(0, y, 960, 3, c)
        for _ in range(320):
            x, y = rnd.randrange(960), rnd.randrange(720)
            w = rnd.randint(8, 42)
            p.fillRect(x, y, w, 2, C("2f86b4"))
            p.fillRect(x + 4, y + 2, max(4, w // 2), 1, C("57a8cc"))
    else:  # night city ground
        p.fillRect(0, 0, 960, 720, NIGHT)
        for y in range(0, 720, 24):
            p.fillRect(0, y, 960, 1, C("151b2a"))
        for _ in range(240):
            x, y = rnd.randrange(960), rnd.randrange(720)
            p.fillRect(x, y, 2, 2, C("202a40") if rnd.random() < 0.7 else C("3a4a66"))
    p.end()
    lay = ks.new_layer(doc, "terrain")
    ks.write_pixels(lay, img)
    if not ks.export_png(doc, os.path.join(BACKGROUNDS, name + ".png")):
        raise RuntimeError("background export failed " + name)
    ks.save_kra(doc, os.path.join(ART, name + ".kra"))
    ks.log("background", name)


# ---------------------------------------------------------------------------
# Logo and banner
# ---------------------------------------------------------------------------


def pixel_text(pix, text, x, y, scale, color_fn):
    def paint(px, py, _color):
        pix.rect(px, py, scale, scale, color_fn(py))

    pixelfont.draw_text(paint, text, x, y, scale, (255, 255, 255, 255))
    return x + pixelfont.text_width(text, scale)


def chrome(py, top, bottom):
    return top if py < 0 else bottom


def make_logo():
    p = Pix(800, 260)
    # Radar sweep ornament.
    p.disc(60, 60, 34, C("7fd4e6", 20))
    for r in (18, 27, 34):
        for a in range(0, 360, 6):
            p.px(60 + int(math.cos(math.radians(a)) * r),
                 60 + int(math.sin(math.radians(a)) * r), C("7fd4e6", 110))
    p.disc(60, 60, 3, CYAN)
    p.line(60, 60, 88, 36, C("7fd4e6", 160))

    # SPY HUNTER, two-tone, with a hard drop shadow.
    def spy_color(py):
        return WHITE if (py // 12) % 2 == 0 else SILVER

    def hunter_color(py):
        return C("ff5a4a") if (py // 12) % 2 == 0 else RED

    # Shadow.
    def shadow(_py):
        return C("05070b")

    pixel_text(p, "SPY HUNTER", 82 + 6, 52 + 6, 12, shadow)
    x_end = pixel_text(p, "SPY", 82, 52, 12, spy_color)
    pixel_text(p, "HUNTER", x_end + 12 * 6, 52, 12, hunter_color)

    # Underline and subtitle.
    p.rect(84, 158, 640, 4, GOLD)
    p.rect(84, 158, 220, 4, CYAN)
    pixel_text(p, "G-6155 INTERCEPTOR", 86, 178, 4,
               lambda _py: C("c9cbd0"))
    pixel_text(p, "ARCADE TRIBUTE", 540, 178, 3, lambda _py: GOLD)
    # Crosshair corners.
    for cx, cy in ((28, 28), (772, 28), (28, 232), (772, 232)):
        p.hline(cx - 10, cy, 20, C("7fd4e6", 150))
        p.vline(cx, cy - 10, 20, C("7fd4e6", 150))
    return p


def make_banner():
    """1200x750 landing / social image composed from the same pixel art."""
    doc = ks.new_doc(1200, 750, "spy_hunter_banner")
    clear_background(doc)
    img, p = ks.blank_pixels(1200, 750)
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)

    # Sky gradient.
    for y in range(0, 750):
        t = y / 750.0
        r = int(8 + 10 * t)
        g = int(12 + 18 * t)
        b = int(24 + 34 * t)
        p.fillRect(0, y, 1200, 1, QColor(r, g, b))
    # Distant city.
    rnd = random.Random(4)
    for i in range(26):
        w = rnd.randint(30, 90)
        h = rnd.randint(50, 190)
        x = rnd.randint(-20, 1200)
        y = 300 - h
        p.fillRect(x, y, w, h, C("111827"))
        p.fillRect(x, y, w, 2, C("1d2a44"))
        for wy in range(y + 12, y + h - 8, 16):
            for wx in range(x + 8, x + w - 8, 14):
                if rnd.random() < 0.4:
                    p.fillRect(wx, wy, 5, 6, C("e5c47c") if rnd.random() < 0.8 else CYAN)
    # Ground.
    p.fillRect(0, 300, 1200, 450, C("16251c"))
    # Road in perspective.
    p.setRenderHint(QPainter.RenderHint.Antialiasing, True)
    from PyQt6.QtGui import QPolygonF
    from PyQt6.QtCore import QPointF
    road = QPolygonF([QPointF(500, 300), QPointF(700, 300),
                      QPointF(1010, 750), QPointF(190, 750)])
    p.setBrush(C("33363d"))
    p.setPen(Qt.PenStyle.NoPen)
    p.drawPolygon(road)
    p.setBrush(C("e7e2d4"))
    p.drawPolygon(QPolygonF([QPointF(500, 300), QPointF(512, 300),
                             QPointF(214, 750), QPointF(190, 750)]))
    p.drawPolygon(QPolygonF([QPointF(688, 300), QPointF(700, 300),
                             QPointF(1010, 750), QPointF(986, 750)]))
    p.setBrush(LANE)
    for i in range(8):
        t = i / 8.0
        y = 320 + t * t * 430
        w = 5 + int(20 * t)
        h = 12 + int(30 * t)
        x = 600 - w // 2 + int((y - 300) * 0.02)
        p.drawRect(int(x), int(y), int(w), int(h))
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)

    # Paste player car and enemies large, nearest-neighbour.
    def paste(sprite, x, y, scale):
        big = sprite.img.scaled(sprite.w * scale, sprite.h * scale,
                                Qt.AspectRatioMode.IgnoreAspectRatio,
                                Qt.TransformationMode.FastTransformation)
        p.drawImage(x, y, big)

    player = Pix(48, 72)
    draw_car(player, WHITE, SILVER, RED, GLASS)
    enemy = Pix(48, 72)
    draw_car(enemy, BLUE, BLUE_DARK, CYAN, GLASS)
    civ = Pix(48, 72)
    draw_car(civ, TEAL, TEAL_DARK, GOLD, GLASS, civilian=True)
    heli = draw_helicopter()
    paste(player, 528, 470, 3)
    paste(civ, 292, 398, 3)
    paste(enemy, 782, 378, 3)
    paste(heli, 1040, 300, 1)

    # Explosion accent.
    boom = draw_explosion(2)
    paste(boom, 850, 500, 2)

    # Logo and tagline.
    logo = make_logo()
    paste(logo, 200, 40, 1)
    p.setFont(ks.font("DejaVu Sans", 22, bold=True))
    p.setPen(QColor(229, 196, 124))
    from PyQt6.QtCore import QRectF
    p.drawText(QRectF(300, 310, 600, 40),
               int(Qt.AlignmentFlag.AlignCenter),
               "RUST + BEVY 0.20 · ORIGINAL PIXEL ART")
    p.end()

    lay = ks.new_layer(doc, "banner")
    ks.write_pixels(lay, img)
    ks.export_png(doc, os.path.join(ART, "banner.png"))
    ks.export_png(doc, os.path.join(ART, "spy-hunter-thumb.png"))
    ks.save_kra(doc, os.path.join(ART, "spy_hunter_banner.kra"))
    ks.log("banner done")


# ---------------------------------------------------------------------------
# Build
# ---------------------------------------------------------------------------


def build_sprites():
    p = Pix(48, 72); draw_car(p, WHITE, SILVER, RED, GLASS); export_sprite("player_car", p)
    p = Pix(48, 72); draw_car(p, RED, RED_DARK, C("f2b09a"), GLASS); export_sprite("enemy_red", p)
    p = Pix(48, 72); draw_car(p, BLUE, BLUE_DARK, CYAN, GLASS); export_sprite("enemy_blue", p)
    p = Pix(48, 72); draw_car(p, C("262830"), C("17181d"), RED, C("101820"), gun_pods=True); export_sprite("enemy_black", p)
    p = Pix(48, 72); draw_car(p, TEAL, TEAL_DARK, GOLD, GLASS, civilian=True); export_sprite("civilian", p)
    p = Pix(48, 72); draw_boat(p, WHITE, SILVER, RED, GLASS); export_sprite("player_boat", p)
    p = Pix(48, 72); draw_boat(p, C("3a3d44"), C("22242a"), RED, GLASS); export_sprite("enemy_boat", p)
    export_sprite("weapons_van", draw_van())
    export_sprite("helicopter", draw_helicopter())
    export_sprite("tree", draw_tree(False))
    export_sprite("snow_pine", draw_tree(True))
    export_sprite("rock", draw_rock())
    export_sprite("building", draw_building())
    export_sprite("oil_slick", draw_oil())
    export_sprite("smoke", draw_smoke())
    export_sprite("crater", draw_crater())
    export_sprite("life_icon", draw_life_icon())

    boom = [draw_explosion(i) for i in range(4)]
    for i, f in enumerate(boom):
        export_sprite(f"explosion_{i}", f)

    p = Pix(6, 18); p.rect(2, 0, 2, 18, YELLOW); p.rect(1, 2, 4, 12, AMBER); p.rect(2, 14, 2, 4, WHITE)
    export_sprite("bullet", p)
    p = Pix(8, 16); p.rect(2, 0, 4, 16, ORANGE); p.rect(3, 0, 2, 10, RED); p.rect(2, 12, 4, 4, YELLOW)
    export_sprite("enemy_bullet", p)
    p = Pix(8, 24); p.rect(3, 0, 2, 18, WHITE); p.rect(2, 2, 4, 4, SILVER); p.rect(1, 16, 6, 3, STEEL)
    p.rect(3, 19, 2, 3, ORANGE); p.rect(3, 22, 2, 2, YELLOW); p.px(3, 0, RED); p.px(4, 0, RED)
    export_sprite("missile", p)
    p = Pix(14, 22); p.rect(4, 0, 6, 16, C("3a3d44")); p.rect(5, 1, 4, 14, C("5a5f68"))
    p.rect(2, 15, 10, 3, INK); p.rect(6, 18, 2, 3, RED)
    export_sprite("bomb", p)
    p = Pix(10, 10); p.hline(0, 4, 10, YELLOW); p.vline(4, 0, 10, YELLOW); p.rect(3, 3, 4, 4, WHITE)
    export_sprite("spark", p)
    p = Pix(24, 32); p.rect(8, 18, 8, 12, C("1c3a52")); p.rect(6, 4, 12, 16, RED)
    p.rect(6, 12, 12, 5, WHITE); p.rect(10, 2, 4, 4, AMBER); p.rect(9, 28, 6, 3, INK)
    export_sprite("buoy", p)
    p = Pix(32, 32); p.rect(14, 4, 4, 24, C("c9cbd0")); p.rect(10, 26, 12, 4, C("7e848d"))
    p.rect(9, 8, 14, 3, C("e5c47c")); p.rect(9, 18, 14, 3, C("e5c47c"))
    export_sprite("sign_post", p)

    for kind in ("gun", "oil", "smoke", "missile", "van"):
        export_sprite("icon_" + kind, draw_icon(kind))

    sprite_strip("explosion_sheet", boom)
    ks.log("sprites complete")


def build_board():
    """A hand-editable contact sheet of every sprite."""
    names = [
        ("player_car", 48, 72), ("player_boat", 48, 72), ("enemy_red", 48, 72),
        ("enemy_blue", 48, 72), ("enemy_black", 48, 72), ("civilian", 48, 72),
        ("enemy_boat", 48, 72), ("weapons_van", 64, 96), ("helicopter", 96, 72),
        ("tree", 48, 64), ("snow_pine", 48, 64), ("rock", 48, 32),
        ("building", 64, 96), ("oil_slick", 56, 24), ("smoke", 64, 64),
        ("crater", 72, 48), ("life_icon", 22, 34), ("explosion_0", 96, 96),
        ("explosion_1", 96, 96), ("explosion_2", 96, 96), ("explosion_3", 96, 96),
    ]
    doc = ks.new_doc(1024, 900, "sprite_board")
    clear_background(doc)
    img, p = ks.blank_pixels(1024, 900, C("11151d"))
    p.setRenderHint(QPainter.RenderHint.Antialiasing, False)
    p.fillRect(0, 0, 1024, 54, C("0b0e14"))
    p.setFont(ks.font("DejaVu Sans", 20, bold=True))
    p.setPen(GOLD)
    p.drawText(24, 36, "SPY HUNTER: INTERCEPTOR - ORIGINAL SPRITE BOARD")
    p.setFont(ks.font("DejaVu Sans", 9))
    x, y, row_h = 22, 76, 0
    for name, w, h in names:
        path = os.path.join(SPRITES, name + ".png")
        if not os.path.exists(path):
            continue
        im = QImage(path)
        if x + w > 1002:
            x = 22
            y += row_h + 28
            row_h = 0
        p.drawImage(x, y, im)
        p.setPen(C("7fd4e6"))
        p.drawText(x, y + h + 14, name)
        x += w + 26
        row_h = max(row_h, h)
    p.end()
    lay = ks.new_layer(doc, "sprites")
    ks.write_pixels(lay, img)
    ks.export_png(doc, os.path.join(ART, "sprite_contact.png"))
    ks.save_kra(doc, os.path.join(ART, "sprite_board.kra"))
    ks.log("board complete")


def __main__(args):
    try:
        build_sprites()
        background("terrain_grass", GRASS, "grass")
        background("terrain_snow", SNOW, "snow")
        background("terrain_water", WATER, "water")
        background("terrain_night", NIGHT, "night")
        logo = make_logo()
        export_sprite("logo", logo)
        make_banner()
        build_board()
        ks.done()
    except BaseException:
        traceback.print_exc(file=sys.stderr)

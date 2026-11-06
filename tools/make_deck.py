#!/usr/bin/env python3
"""Build the SPY HUNTER: INTERCEPTOR mission-debrief deck with LibreOffice Impress.

Run against the UNO listener on 127.0.0.1:2002 (see the libreoffice skill).
Outputs:
  docs/spy-hunter-debrief.odp
  docs/spy-hunter-debrief.pdf
  docs/spy-hunter-debrief.pptx
"""

from pathlib import Path

import uno
from PIL import Image

FILL_NONE = 0
FILL_SOLID = 1
LINE_NONE = 0
LINE_SOLID = 1

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs"
DECK = DOCS / "deck"

W, H = 33867, 19050  # 16:9, 1/100 mm
BG = 0x060A10
PANEL = 0x0B1420
PANEL_2 = 0x101B28
LINE = 0x223244
INK = 0xECE6D6
MUTED = 0x93A0AB
GOLD = 0xC9A45C
GOLD_B = 0xE5C47C
RED = 0xD93442
RED_B = 0xFF5A4A
ICE = 0x7FD4E6
GREEN = 0x5FD08A

BOLD = 150
NORMAL = 100
LEFT = 0
CENTER = 3

FONT = "Liberation Sans"
MONO = "JetBrains Mono"


def connect():
    local = uno.getComponentContext()
    resolver = local.ServiceManager.createInstanceWithContext(
        "com.sun.star.bridge.UnoUrlResolver", local
    )
    ctx = resolver.resolve(
        "uno:socket,host=127.0.0.1,port=2002;urp;StarOffice.ComponentContext"
    )
    smgr = ctx.ServiceManager
    desktop = smgr.createInstanceWithContext("com.sun.star.frame.Desktop", ctx)
    return desktop


def pv(name, value):
    prop = uno.createUnoStruct("com.sun.star.beans.PropertyValue")
    prop.Name = name
    prop.Value = value
    return prop


def pt(x, y):
    p = uno.createUnoStruct("com.sun.star.awt.Point")
    p.X, p.Y = int(x), int(y)
    return p


def sz(w, h):
    s = uno.createUnoStruct("com.sun.star.awt.Size")
    s.Width, s.Height = int(w), int(h)
    return s


class Deck:
    def __init__(self, desktop):
        self.doc = desktop.loadComponentFromURL(
            "private:factory/simpress", "_blank", 0, (pv("Hidden", True),)
        )
        self.pages = self.doc.DrawPages
        self.created = 0

    def new_slide(self, bg=BG):
        if self.created == 0:
            slide = self.pages.getByIndex(0)
            for i in reversed(range(slide.getCount())):
                slide.remove(slide.getByIndex(i))
        else:
            slide = self.pages.insertNewByIndex(self.pages.getCount())
        self.created += 1
        slide.Width, slide.Height = W, H
        self.rect(slide, 0, 0, W, H, bg)
        return slide

    def rect(self, slide, x, y, w, h, color, alpha=0, line=None, line_w=1):
        shape = self.doc.createInstance("com.sun.star.drawing.RectangleShape")
        shape.Position = pt(x, y)
        shape.Size = sz(w, h)
        shape.FillStyle = FILL_SOLID
        shape.FillColor = color
        shape.FillTransparence = int(alpha)
        if line is None:
            shape.LineStyle = LINE_NONE
        else:
            shape.LineStyle = LINE_SOLID
            shape.LineColor = line
            shape.LineWidth = line_w
        slide.add(shape)
        return shape

    def text(self, slide, x, y, w, h, content, size=16, color=INK,
             bold=False, font=FONT, align=LEFT, spacing=None, alpha=100,
             auto_grow=False):
        shape = self.doc.createInstance("com.sun.star.drawing.TextShape")
        shape.Position = pt(x, y)
        shape.Size = sz(w, h)
        shape.FillStyle = FILL_NONE
        shape.LineStyle = LINE_NONE
        shape.TextAutoGrowHeight = auto_grow
        shape.TextWordWrap = True
        try:
            shape.TextLeftDistance = 0
            shape.TextRightDistance = 0
            shape.TextUpperDistance = 0
            shape.TextLowerDistance = 0
            shape.TextVerticalAdjust = 0
        except Exception:
            pass
        shape.ParaAdjust = align
        shape.CharFontName = font
        shape.CharHeight = float(size)
        shape.CharColor = color
        shape.CharWeight = BOLD if bold else NORMAL
        shape.CharTransparence = 100 - alpha
        if spacing:
            shape.CharKerning = int(spacing * 100)
        slide.add(shape)
        cursor = shape.Text.createTextCursor()
        for i, line in enumerate(content.split("\n")):
            if i:
                shape.Text.insertControlCharacter(cursor, 0, False)
            shape.Text.insertString(cursor, line, False)
        cursor.gotoStart(False)
        cursor.gotoEnd(True)
        for prop, value in (
            ("CharFontName", font),
            ("CharHeight", float(size)),
            ("CharColor", color),
            ("CharWeight", BOLD if bold else NORMAL),
            ("CharTransparence", 100 - alpha),
        ):
            try:
                cursor.setPropertyValue(prop, value)
            except Exception:
                pass
        if spacing:
            try:
                cursor.setPropertyValue("CharKerning", int(spacing * 100))
            except Exception:
                pass
        return shape

    def image(self, slide, path, x, y, w=None, h=None):
        path = Path(path)
        with Image.open(path) as im:
            iw, ih = im.size
        if w is None and h is not None:
            w = h * iw / ih
        if h is None and w is not None:
            h = w * ih / iw
        shape = self.doc.createInstance("com.sun.star.drawing.GraphicObjectShape")
        shape.GraphicURL = uno.systemPathToFileUrl(str(path))
        shape.Position = pt(x, y)
        shape.Size = sz(w, h)
        slide.add(shape)
        return shape

    def kicker(self, slide, text, x=1700, y=760, color=ICE):
        self.text(slide, x, y, 24000, 500, text, size=10.5, color=color,
                  font=MONO, spacing=1.8)

    def title(self, slide, text, x=1650, y=1180, w=30000, h=2000, size=31,
              color=INK):
        self.text(slide, x, y, w, h, text, size=size, color=color, bold=True)

    def footer(self, slide, n, label="SPY HUNTER: INTERCEPTOR  /  MISSION DEBRIEF"):
        self.rect(slide, 1650, 17920, 30500, 1, LINE)
        self.text(slide, 1650, 18100, 24000, 400, label, size=8.5,
                  color=0x6F7A84, font=MONO, spacing=1.2)
        self.text(slide, 30700, 18070, 1700, 500, f"{n:02d}", size=10,
                  color=GOLD, font=MONO, align=CENTER)

    def accent(self, slide, x=1650, y=1080, w=1100):
        self.rect(slide, x, y, w, 14, RED)
        self.rect(slide, x + w + 22, y, 190, 14, GOLD)

    def card(self, slide, x, y, w, h, title, body, number=None, accent=GOLD):
        self.rect(slide, x, y, w, h, PANEL, line=LINE)
        self.rect(slide, x, y, 8, h, accent)
        if number:
            self.text(slide, x + 440, y + 340, 4000, 400, number, size=9,
                      color=accent, font=MONO, spacing=1.4)
            ty = y + 760
        else:
            ty = y + 440
        self.text(slide, x + 440, ty, w - 800, 900, title, size=17, bold=True)
        self.text(slide, x + 440, ty + 980, w - 800, h - 1500, body, size=12.5,
                  color=0xB3BCC5)

    def stat(self, slide, x, y, w, h, value, label, color=GOLD_B):
        self.rect(slide, x, y, w, h, PANEL_2, line=LINE)
        self.text(slide, x, y + 380, w, 1100, value, size=34, color=color,
                  bold=True, align=CENTER)
        self.text(slide, x, y + 1600, w, 500, label, size=9.5, color=MUTED,
                  font=MONO, align=CENTER, spacing=1.4)

    def save(self, name):
        odt = DOCS / f"{name}.odp"
        self.doc.storeToURL(
            uno.systemPathToFileUrl(str(odt)), (pv("FilterName", "impress8"),)
        )
        self.doc.close(False)
        return odt



def build():
    desktop = connect()
    d = Deck(desktop)

    # ---------------------------------------------------------------- cover
    s = d.new_slide()
    d.image(s, ROOT / "assets/art/banner.png", 0, 0, W, H)
    d.rect(s, 0, 16300, W, 2750, 0x04070C, alpha=25)
    d.rect(s, 1650, 16520, 1300, 10, RED_B)
    d.text(s, 1650, 16720, 20000, 700, "MISSION DEBRIEF · NOVEMBER 2026",
           size=13, color=GOLD_B, font=MONO, spacing=1.6)
    d.text(s, 1650, 17420, 21000, 700,
           "A playable Rust + Bevy tribute, built and shipped in one session.",
           size=15, color=INK)
    d.text(s, 24000, 16720, 8200, 700,
           "orbitmechanic.github.io/spy-hunter-web", size=10.5, color=ICE,
           font=MONO, align=CENTER, spacing=0.4)

    # ---------------------------------------------------------------- brief
    s = d.new_slide()
    d.kicker(s, "01 / BRIEF")
    d.accent(s)
    d.title(s, "From a 1983 arcade memory\nto a shipped browser game")
    d.text(s, 1700, 3950, 15800, 1400,
           "The assignment was deliberately broad: research the original, build an "
           "original game in the same spirit, generate the art and audio, test it, "
           "publish it, and dock it on the station.", size=13.5, color=0xB3BCC5)
    rows = [
        ("RESEARCH", "Deconstruct Spy Hunter's top-down chase, weapons van, civilian rule, car-to-boat switch, and Peter Gunn-era sound."),
        ("FOUNDATION", "New project at games/bevy/spy-hunter, Cargo + Bevy 0.20.1, public GitHub repo from the first commit."),
        ("ASSETS", "Original Krita pixel art, a 5×7 pixel TTF, a 23.7 s chiptune loop, and ten SFX — no ripped assets."),
        ("SHIP", "Native + wasm builds, screenshot harness, GitHub Pages deployment, landing-page dock, and this deck."),
    ]
    y = 5700
    for i, (head, body) in enumerate(rows):
        d.rect(s, 1700, y, 120, 1150, GOLD if i % 2 == 0 else RED)
        d.text(s, 2050, y + 40, 4200, 400, head, size=10, color=GOLD_B,
               font=MONO, spacing=1.5)
        d.text(s, 2050, y + 500, 13800, 700, body, size=12.5, color=0xC0C7CF)
        y += 2550
    d.stat(s, 23000, 4550, 8800, 2700, "4", "TERRAINS", ICE)
    d.stat(s, 23000, 7550, 8800, 2700, "23.7 s", "ORIGINAL LOOP", GOLD_B)
    d.stat(s, 23000, 10550, 8800, 2700, "6 / 6", "SIM TESTS", GREEN)
    d.stat(s, 23000, 13550, 8800, 2700, "0", "RIPPED ASSETS", RED_B)
    d.footer(s, 2)

    # ---------------------------------------------------------------- research
    s = d.new_slide()
    d.kicker(s, "02 / RESEARCH")
    d.accent(s)
    d.title(s, "The target: Spy Hunter, Bally Midway, 1983", size=29)
    d.text(s, 1700, 3350, 14500, 2200,
           "A vertical-scrolling vehicular-combat game. The player drives the G-6155 "
           "Interceptor, destroys enemy traffic, protects civilians, and collects "
           "special weapons from a van. The road forks into new weather and terrain; "
           "at water, the car becomes a speedboat.", size=13, color=0xB3BCC5)
    findings = [
        ("GAMEPLAY", "Top-down, 4-lane road, machine gun, oil / smoke / missiles, helicopter bombs, off-road and civilian penalties."),
        ("LOOK", "Chunky sprite cars, hard lane lines, bold roadside colour blocks, a compact dashboard HUD."),
        ("SOUND", "An electronic arrangement of Henry Mancini's Peter Gunn theme; engine and weapon cues under it."),
        ("RULES", "Score accrues while moving on-road. Civilian hits freeze scoring. Extra cars at score thresholds. No ending."),
    ]
    y = 5700
    for head, body in findings:
        d.text(s, 1700, y, 4200, 400, head, size=9.5, color=ICE,
               font=MONO, spacing=1.4)
        d.text(s, 1700, y + 430, 13800, 900, body, size=11.5, color=0xB3BCC5)
        y += 2400
    d.image(s, DOCS / "screenshots/gameplay.png", 17300, 3450, 14400, 10800)
    d.rect(s, 17300, 14400, 14400, 1200, PANEL, line=LINE)
    d.text(s, 17700, 14620, 13600, 800,
           "The Peter Gunn theme is copyrighted. This tribute writes an original "
           "spy-surf cue instead of reproducing it.", size=10.5, color=MUTED)
    d.footer(s, 3)

    # ---------------------------------------------------------------- features
    s = d.new_slide()
    d.kicker(s, "03 / WHAT SHIPPED")
    d.accent(s)
    d.title(s, "One Interceptor, four terrains, endless pursuit", size=29)
    cards = [
        ("DRIVE & SURVIVE", "Curving four-lane highway, coast / snow / water / night cycle, off-road timer, craters, helicopter bombs, extra lives."),
        ("FIGHT", "Hold-fire machine gun, oil slick, smoke screen, homing missile, weapons van, switchblade cars, pursuing boats."),
        ("ARCADE SHELL", "Title, HUD, pause veil, game-over stats, hi-score, touch controls, and a browser-first canvas."),
    ]
    x = 1700
    for i, (head, body) in enumerate(cards):
        d.card(s, x, 3350, 9800, 5200, head, body, number=f"0{i+1}",
               accent=[GOLD, RED, ICE][i])
        x += 10200
    d.image(s, DOCS / "screenshots/water.png", 1700, 8850, 14400, 8100)
    d.image(s, DOCS / "screenshots/gameover.png", 17300, 8850, 14400, 8100)
    d.text(s, 17700, 17100, 14000, 500,
           "Delta Waterway: car becomes boat, oil becomes flamethrower.",
           size=9.5, color=ICE, font=MONO)
    d.footer(s, 4)

    # ---------------------------------------------------------------- architecture
    s = d.new_slide()
    d.kicker(s, "04 / UNDER THE HOOD")
    d.accent(s)
    d.title(s, "A small ECS with a very clear loop", size=30)
    d.text(s, 1700, 3250, 30000, 500,
           "Bevy 0.20.1 · edition 2024 · fixed 960 × 720 logical canvas · one 2D camera",
           size=10.5, color=MUTED, font=MONO)
    states = [("TITLE", 1900), ("IN GAME", 7900), ("GAME OVER", 13900)]
    for name, x in states:
        d.rect(s, x, 4050, 5000, 1000, PANEL_2, line=GOLD)
        d.text(s, x, 4300, 5000, 500, name, size=14, color=GOLD_B, bold=True,
               align=CENTER)
    d.text(s, 6700, 4300, 1200, 500, "→", size=20, color=ICE, align=CENTER)
    d.text(s, 12700, 4300, 1200, 500, "→", size=20, color=ICE, align=CENTER)
    d.text(s, 1700, 5250, 30400, 700,
           "Enter starts a run; death with zero cars ends it; Enter redeploys.",
           size=11.5, color=0xB3BCC5)
    steps = ["input", "road", "spawn", "move", "fire", "collide", "hud"]
    x = 1850
    for i, step in enumerate(steps):
        d.rect(s, x, 6500, 3950, 900, PANEL, line=LINE)
        d.text(s, x, 6700, 3950, 500, step.upper(), size=11, color=ICE
               if i % 2 == 0 else GOLD_B, font=MONO, align=CENTER, spacing=1.2)
        x += 4300
    d.text(s, 1850, 7700, 30000, 500,
           "Update systems run in a chain; gameplay pauses with one resource flag.",
           size=10, color=MUTED, font=MONO)
    res = [("Game", "score · lives · terrain · timers"), ("Weapons", "oil · smoke · missiles · van"),
           ("Road", "scroll distance"), ("Rng", "deterministic xorshift"),
           ("Fx", "shake + particles")]
    x = 1700
    for name, body in res:
        d.rect(s, x, 8800, 5750, 1850, PANEL_2, line=LINE)
        d.text(s, x + 400, 9050, 5000, 500, name, size=15, color=GOLD_B, bold=True)
        d.text(s, x + 400, 9600, 5000, 800, body, size=10.5, color=0xAAB4BE)
        x += 6050
    d.text(s, 1700, 11350, 9000, 500, "CONTENT PIPELINE", size=10, color=RED_B,
           font=MONO, spacing=1.4)
    pipes = [("Krita 6", "PNG sprites\n+ terrain"), ("make_font.py", "5×7 pixel\nTTF"),
             ("make_audio.py", "WAV music\n+ 10 SFX"), ("wasm-bindgen", "WebGL 2\nbundle")]
    x = 1700
    for i, (head, body) in enumerate(pipes):
        d.rect(s, x, 11900, 6500, 3300, PANEL, line=LINE)
        d.text(s, x + 500, 12200, 5500, 500, head, size=15, color=ICE, bold=True)
        d.text(s, x + 500, 13000, 5500, 1600, body, size=11.5, color=0xB3BCC5)
        if i < 3:
            d.text(s, x + 6650, 13100, 900, 500, "→", size=18, color=GOLD, align=CENTER)
        x += 7700
    d.footer(s, 5)

    # ---------------------------------------------------------------- art
    s = d.new_slide()
    d.kicker(s, "05 / ART PIPELINE")
    d.accent(s)
    d.title(s, "Original pixel art, drawn through Krita's Python API", size=27)
    left = [
        ("PALETTE", "Twelve fixed colours: ink, asphalt, lane cream, grass, snow, water, body, enemy red, blue, civilian teal, gold, ice."),
        ("SPRITES", "Cars, boats, helicopter, van, pickups, explosions, trees, rocks, buoys, buildings, icons, and a 21-sprite editable board."),
        ("TERRAIN", "Four 960×720 fields: grass, snow, water, night city. The four-lane road itself is built from recycled Bevy sprites."),
        ("FIX", "Krita creates an opaque white Background layer. The export script hides it before writing pixels so alpha survives."),
    ]
    y = 3200
    for head, body in left:
        d.text(s, 1700, y, 4200, 400, head, size=9.5, color=GOLD_B,
               font=MONO, spacing=1.4)
        d.text(s, 1700, y + 400, 13600, 900, body, size=11.5, color=0xB3BCC5)
        y += 2500
    d.image(s, DOCS / "deck/sprites.png", 16800, 3150, 15000, 6550)
    d.image(s, DOCS / "deck/palette.png", 16800, 10200, 15000, 2375)
    d.image(s, ROOT / "assets/art/banner.png", 20500, 13100, 7680, 4800)
    d.footer(s, 6)

    # ---------------------------------------------------------------- audio
    s = d.new_slide()
    d.kicker(s, "06 / SOUND")
    d.accent(s)
    d.title(s, "An original spy-surf cue, synthesized from scratch", size=28)
    d.image(s, DOCS / "deck/waveform.png", 1700, 3250, 30000, 7500)
    rows = [
        ("FORM", "8-bar loop at 240 BPM in E minor. Walking square bass, tremolo square lead, triangle pad, noise drums."),
        ("SFX", "Ten cues: machine gun, enemy shot, explosion, missile, pickup, civilian penalty, death, extra life, terrain, UI."),
        ("FORMAT", "22.05 kHz mono 16-bit WAV; one 1.0 MB music loop, 1.2 MB total audio. No samples from the arcade machine."),
        ("WHY ORIGINAL", "Peter Gunn is owned by its composers/publishers. A bespoke cue keeps the tribute legally clean and gives the game its own identity."),
    ]
    y = 11300
    for i, (head, body) in enumerate(rows):
        x = 1700 + (i % 2) * 15300
        yy = y + (i // 2) * 2900
        d.text(s, x, yy, 4200, 400, head, size=9.5, color=ICE, font=MONO, spacing=1.4)
        d.text(s, x, yy + 430, 14000, 1700, body, size=11, color=0xB3BCC5)
    d.footer(s, 7)

    # ---------------------------------------------------------------- verification
    s = d.new_slide()
    d.kicker(s, "07 / VERIFICATION")
    d.accent(s)
    d.title(s, "Compiling is not proof. Every claim has a check.", size=27)
    shots = [
        ("TITLE", DOCS / "screenshots/title.png", 1700, 3250),
        ("HIGHWAY", DOCS / "screenshots/gameplay.png", 16950, 3250),
        ("ALPINE PASS", DOCS / "screenshots/snow.png", 1700, 10300),
        ("GAME OVER", DOCS / "screenshots/gameover.png", 16950, 10300),
    ]
    for label, path, x, y in shots:
        d.rect(s, x, y, 15250, 6500, PANEL, line=LINE)
        d.image(s, path, x + 250, y + 250, 14750, 5000)
        d.text(s, x + 400, y + 5450, 14000, 500, label, size=9.5,
               color=ICE, font=MONO, spacing=1.4)
    d.rect(s, 1700, 16900, 30400, 850, PANEL_2, line=GOLD)
    d.text(s, 2050, 17040, 30000, 500,
           "cargo test: 6/6 pass   ·   cargo check: clean   ·   native capture at frames 120–520   ·   web smoke test in headless Chromium",
           size=10, color=GOLD_B, font=MONO)
    d.footer(s, 8)

    # ---------------------------------------------------------------- challenges
    s = d.new_slide()
    d.kicker(s, "08 / CHALLENGES & FIXES")
    d.accent(s)
    d.title(s, "Four bugs worth remembering", size=31)
    issues = [
        ("SCORE STUCK AT 4", "Score was added as (speed × dt × 0.11) as u64. At 60 fps the fraction truncated to zero every frame. Fix: keep a fractional accumulator and add whole points only.", GOLD),
        ("BEVY B0001", "Two queries touched Transform — one mutably — without disjoint filters, so the app panicked on the first frame. Fix: Without<Player/Enemy/Missile> filters.", RED),
        ("WHITE SPRITE CARDS", "Every Krita export had an opaque white background because createDocument adds a Background layer. Fix: hide that layer before merge-exporting.", ICE),
        ("WEB AUDIO + SIZE", "Browsers suspend AudioContext until a trusted gesture, and the raw wasm was over 100 MB. Fix: capture/resume contexts in the shell, wasm-bindgen + wasm-opt -Oz.", GREEN),
    ]
    y = 3200
    for i, (head, body, color) in enumerate(issues):
        d.rect(s, 1700, y, 30400, 3150, PANEL, line=LINE)
        d.rect(s, 1700, y, 10, 3150, color)
        d.text(s, 2100, y + 350, 8000, 500, head, size=11, color=color,
               font=MONO, spacing=1.5)
        d.text(s, 2100, y + 1050, 29200, 1700, body, size=12.5, color=0xB3BCC5)
        y += 3400
    d.footer(s, 9)

    # ---------------------------------------------------------------- deploy
    s = d.new_slide()
    d.kicker(s, "09 / DEPLOYMENT")
    d.accent(s)
    d.title(s, "Source, static build, Pages, and a new docking bay", size=28)
    chain = [
        ("SOURCE", "github.com/orbitmechanic/\nspy-hunter", GOLD),
        ("BUILD", "tools/build_web.sh\nwasm-bindgen + wasm-opt", ICE),
        ("WEB REPO", "github.com/orbitmechanic/\nspy-hunter-web", GOLD),
        ("PAGES", "orbitmechanic.github.io/\nspy-hunter-web", ICE),
        ("DOCK", "Gallifrey Station\nBay 08", RED_B),
    ]
    x = 1200
    for i, (head, body, color) in enumerate(chain):
        d.rect(s, x, 4200, 5700, 3300, PANEL, line=color)
        d.text(s, x + 300, 4500, 5100, 500, head, size=10, color=color,
               font=MONO, spacing=1.4, align=CENTER)
        d.text(s, x + 300, 5300, 5100, 1700, body, size=11.5, color=INK,
               font=MONO, align=CENTER)
        if i < 4:
            d.text(s, x + 5800, 5450, 1000, 600, "→", size=20, color=GOLD, align=CENTER)
        x += 6400
    d.text(s, 1700, 8200, 30400, 600,
           "The game repo carries source, art boards, tests, and the build/deploy scripts.",
           size=12, color=0xB3BCC5)
    d.text(s, 1700, 8900, 30400, 600,
           "The web repo carries only the staged runtime: index.html, CSS, JS glue, wasm, assets, .nojekyll.",
           size=12, color=0xB3BCC5)
    d.rect(s, 1700, 10100, 30400, 5600, PANEL_2, line=LINE)
    d.text(s, 2200, 10400, 29200, 600, "LIVE URLS", size=10, color=ICE,
           font=MONO, spacing=1.5)
    d.text(s, 2200, 11100, 29200, 4200,
           "PLAY      https://orbitmechanic.github.io/spy-hunter-web/\n"
           "SOURCE    https://github.com/orbitmechanic/spy-hunter\n"
           "BUILD     https://github.com/orbitmechanic/spy-hunter-web\n"
           "STATION   https://orbitmechanic.github.io/  (Bay 08)",
           size=15, color=GOLD_B, font=MONO)
    d.footer(s, 10)

    # ---------------------------------------------------------------- next
    s = d.new_slide()
    d.kicker(s, "10 / RECOMMENDED NEXT STEPS")
    d.accent(s)
    d.title(s, "What I would build next, in order", size=31)
    steps = [
        ("01", "GAMEPAD + MOBILE POLISH", "Wire gamepad axes and tune the touch hit targets. Add haptics and a rotate-to-landscape hint."),
        ("02", "MORE ROAD STORIES", "Branching forks, a boathouse choice, motorcycles, limousines, and a second helicopter pattern."),
        ("03", "RENDER BUDGET", "Bake the sprite set into an atlas and batch the 100+ road parts. Target a stable 60 fps on integrated GPUs."),
        ("04", "PERSISTENCE", "Local high-score table and per-terrain bests; optional serverless leaderboard."),
        ("05", "AUDIO MIX", "Duck the music under explosions, add engine pitch tied to speed, and a mute toggle on the title screen."),
        ("06", "ACCESSIBILITY", "Remap keys, reduce screen shake, high-contrast HUD, and a slower starting difficulty."),
    ]
    y = 3150
    for i, (num, head, body) in enumerate(steps):
        x = 1700 + (i % 2) * 15400
        yy = y + (i // 2) * 4200
        d.text(s, x, yy, 900, 700, num, size=22, color=RED_B, bold=True)
        d.text(s, x + 1100, yy, 13500, 500, head, size=11, color=GOLD_B,
               font=MONO, spacing=1.3)
        d.text(s, x + 1100, yy + 650, 13500, 2100, body, size=11.5, color=0xB3BCC5)
    d.footer(s, 11)

    # ---------------------------------------------------------------- close
    s = d.new_slide()
    d.kicker(s, "MISSION COMPLETE")
    d.accent(s)
    d.title(s, "A new arcade cabinet\nis docked at the station.", size=42)
    d.text(s, 1700, 5200, 18000, 900,
           "Play it: orbitmechanic.github.io/spy-hunter-web\n"
           "Source: github.com/orbitmechanic/spy-hunter",
           size=15, color=GOLD_B, font=MONO)
    d.text(s, 1700, 6800, 18000, 1200,
           "Original pixel art · original chiptune · Rust + Bevy 0.20.1 · GitHub Pages",
           size=12, color=0xC0C7CF)
    d.image(s, ROOT / "assets/art/banner.png", 1700, 9000, 14000, 8500)
    d.image(s, DOCS / "screenshots/gameplay.png", 18500, 4200, 13000, 9750)
    d.rect(s, 18500, 14400, 13000, 1500, PANEL, line=LINE)
    d.text(s, 18900, 14650, 12200, 900,
           "Original work. No ROM, cabinet, or arcade asset is included.",
           size=10.5, color=MUTED, font=MONO)
    d.footer(s, 12)

    odt = d.save("spy-hunter-debrief")
    print(f"wrote {odt}")
    print("convert to PDF/PPTX with soffice if needed:") 
    print(f"  soffice --headless --convert-to pdf --outdir {DOCS} {odt}")


if __name__ == "__main__":
    build()

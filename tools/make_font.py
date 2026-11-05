#!/usr/bin/env python3
"""Build assets/fonts/Interceptor.ttf from the shared 5x7 pixel glyph table."""

from pathlib import Path

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen

import pixelfont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "fonts" / "Interceptor.ttf"

UPEM = 1000
PX = 100
BASELINE_ROWS = 7
ASCENT = 800
DESCENT = -200


def glyph_name(ch: str) -> str:
    names = {
        " ": "space", ".": "period", ",": "comma", ":": "colon",
        ";": "semicolon", "-": "hyphen", "_": "underscore", "+": "plus",
        "=": "equal", "!": "exclam", "?": "question", "'": "quotesingle",
        '"': "quotedbl", "/": "slash", "\\": "backslash", "(": "parenleft",
        ")": "parenright", "[": "bracketleft", "]": "bracketright",
        "<": "less", ">": "greater", "*": "asterisk", "%": "percent",
        "@": "at", "#": "numbersign", "&": "ampersand",
        "\u00b7": "middot", "\u2190": "arrowleft", "\u2192": "arrowright",
        "\u2191": "arrowup", "\u2193": "arrowdown",
    }
    if ch in names:
        return names[ch]
    return "uni%04X" % ord(ch)


def build():
    chars = sorted(pixelfont.ALL.keys())
    order = [".notdef"] + [glyph_name(c) for c in chars]
    seen = set()
    order = [n for n in order if not (n in seen or seen.add(n))]

    fb = FontBuilder(UPEM, isTTF=True)
    fb.setupGlyphOrder(order)

    cmap = {32: "space"}
    for c in chars:
        cmap[ord(c)] = glyph_name(c)
    fb.setupCharacterMap(cmap)

    glyphs = {}
    metrics = {}

    notdef = TTGlyphPen(None)
    notdef.moveTo((50, 0))
    notdef.lineTo((50, 700))
    notdef.lineTo((550, 700))
    notdef.lineTo((550, 0))
    notdef.closePath()
    glyphs[".notdef"] = notdef.glyph()
    metrics[".notdef"] = (pixelfont.ADVANCE * PX, 50)

    for ch, rows in pixelfont.ALL.items():
        name = glyph_name(ch)
        if name in glyphs:
            continue
        pen = TTGlyphPen(None)
        for row, line in enumerate(rows):
            y_top = (BASELINE_ROWS - row) * PX
            y_bottom = y_top - PX
            col = 0
            while col < pixelfont.W:
                if line[col] == "1":
                    run = 1
                    while col + run < pixelfont.W and line[col + run] == "1":
                        run += 1
                    x0 = col * PX
                    x1 = (col + run) * PX
                    pen.moveTo((x0, y_bottom))
                    pen.lineTo((x0, y_top))
                    pen.lineTo((x1, y_top))
                    pen.lineTo((x1, y_bottom))
                    pen.closePath()
                    col += run
                else:
                    col += 1
        glyphs[name] = pen.glyph()
        metrics[name] = (pixelfont.ADVANCE * PX, 0)

    fb.setupGlyf(glyphs)
    fb.setupHorizontalMetrics(metrics)
    fb.setupHorizontalHeader(ascent=ASCENT, descent=DESCENT, lineGap=100)
    fb.setupNameTable(
        {
            "familyName": "Interceptor 17",
            "styleName": "Regular",
            "uniqueFontIdentifier": "Interceptor17-Regular-1.0",
            "fullName": "Interceptor 17 Regular",
            "psName": "Interceptor17-Regular",
            "version": "Version 1.000",
        }
    )
    fb.setupOS2(
        sTypoAscender=ASCENT,
        sTypoDescender=DESCENT,
        sTypoLineGap=100,
        usWinAscent=ASCENT,
        usWinDescent=-DESCENT,
        sxHeight=500,
        sCapHeight=700,
    )
    fb.setupPost()
    OUT.parent.mkdir(parents=True, exist_ok=True)
    fb.save(OUT)
    print(f"wrote {OUT} ({OUT.stat().st_size} bytes, {len(glyphs)} glyphs)")


if __name__ == "__main__":
    build()

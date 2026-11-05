#!/usr/bin/env python3
"""Generate original chiptune music and SFX for SPY HUNTER: INTERCEPTOR.

Output: 22.05 kHz mono 16-bit WAV under assets/audio/.

The loop is an original spy-surf riff (E minor, walking bass, tremolo lead,
brushed snare). It deliberately evokes the arcade era without quoting the
Peter Gunn theme or any other copyrighted melody.
"""

import math
import random
import struct
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "assets" / "audio"
SR = 22050


def env(i, n, attack=0.004, release=0.18):
    a = max(1, int(n * attack))
    r = max(1, int(n * release))
    if i < a:
        return i / a
    if i > n - r:
        return max(0.0, (n - i) / r)
    return 1.0


def square(freq, dur, vol=0.22, duty=0.5, slide=None, attack=0.004, release=0.18):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        f = freq if slide is None else freq + (slide - freq) * (i / max(1, n - 1))
        phase += f / SR
        s = vol if (phase % 1.0) < duty else -vol
        out.append(s * env(i, n, attack, release))
    return out


def triangle(freq, dur, vol=0.22, slide=None, attack=0.004, release=0.18):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        f = freq if slide is None else freq + (slide - freq) * (i / max(1, n - 1))
        phase += f / SR
        p = phase % 1.0
        out.append((4 * abs(p - 0.5) - 1) * vol * env(i, n, attack, release))
    return out


def saw(freq, dur, vol=0.18, attack=0.003, release=0.12):
    n = int(SR * dur)
    out = []
    phase = 0.0
    for i in range(n):
        phase += freq / SR
        out.append((2 * (phase % 1.0) - 1) * vol * env(i, n, attack, release))
    return out


def noise(dur, vol=0.2, decay=3.0, lowpass=0.5, seed=1):
    n = int(SR * dur)
    out = []
    rnd = random.Random(seed)
    prev = 0.0
    for i in range(n):
        raw = rnd.uniform(-1, 1)
        prev = prev * lowpass + raw * (1 - lowpass)
        out.append(prev * vol * math.exp(-decay * i / n))
    return out


def mix(*tracks):
    n = max(len(t) for t in tracks)
    out = [0.0] * n
    for t in tracks:
        for i, v in enumerate(t):
            out[i] += v
    return out


def seq(*parts):
    out = []
    for p in parts:
        out.extend(p)
    return out


def rest(dur):
    return [0.0] * int(SR * dur)


def write(name, samples, gain=0.8):
    OUT.mkdir(parents=True, exist_ok=True)
    peak = max(1e-6, max(abs(s) for s in samples))
    scale = min(gain, 0.95) / peak
    data = b"".join(
        struct.pack("<h", int(max(-1, min(1, s * scale)) * 32767)) for s in samples
    )
    path = OUT / name
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes(data)
    print(f"{name}: {len(samples) / SR:.2f}s {len(data) // 1024} KiB")


# Note helpers ---------------------------------------------------------------

def hz(midi):
    return 440.0 * 2 ** ((midi - 69) / 12.0)


def note(name):
    names = {"C": 0, "C#": 1, "D": 2, "D#": 3, "E": 4, "F": 5, "F#": 6,
             "G": 7, "G#": 8, "A": 9, "A#": 10, "B": 11}
    pitch = name[:-1]
    octave = int(name[-1])
    return 12 * (octave + 1) + names[pitch]


def sfx():
    write("shot.wav", mix(
        square(1100, 0.055, 0.14, 0.25, slide=180, release=0.5),
        noise(0.06, 0.12, 7.0, 0.25, 2),
    ), gain=0.55)
    write("enemy_shot.wav", mix(
        square(520, 0.09, 0.12, 0.4, slide=140, release=0.6),
        noise(0.09, 0.08, 5.0, 0.35, 4),
    ), gain=0.5)
    write("explosion.wav", mix(
        noise(0.55, 0.55, 4.2, 0.82, 6),
        triangle(90, 0.42, 0.22, slide=30, release=0.8),
        noise(0.22, 0.22, 8.0, 0.5, 9),
    ), gain=0.7)
    write("missile.wav", mix(
        square(280, 0.32, 0.13, 0.35, slide=1400, release=0.5),
        noise(0.32, 0.14, 2.5, 0.6, 12),
    ), gain=0.55)
    write("pickup.wav", seq(
        square(hz(note("C5")), 0.06, 0.16, 0.5, release=0.3),
        square(hz(note("E5")), 0.06, 0.16, 0.5, release=0.3),
        square(hz(note("G5")), 0.10, 0.18, 0.5, release=0.5),
        square(hz(note("C6")), 0.14, 0.16, 0.5, release=0.6),
    ), gain=0.55)
    write("penalty.wav", seq(
        square(hz(note("A3")), 0.14, 0.16, 0.5, release=0.4),
        square(hz(note("D#3")), 0.22, 0.16, 0.5, release=0.6),
    ), gain=0.5)
    write("death.wav", mix(
        noise(0.9, 0.42, 3.0, 0.86, 17),
        seq(
            square(hz(note("E4")), 0.12, 0.15, 0.5, slide=hz(note("C4"))),
            square(hz(note("C4")), 0.12, 0.15, 0.5, slide=hz(note("A3"))),
            square(hz(note("A3")), 0.20, 0.15, 0.5, slide=hz(note("E3"))),
            square(hz(note("E2")), 0.34, 0.15, 0.5, slide=hz(note("A1"))),
        ),
    ), gain=0.65)
    write("extra.wav", seq(
        square(hz(note("G4")), 0.08, 0.16, 0.5, release=0.3),
        square(hz(note("B4")), 0.08, 0.16, 0.5, release=0.3),
        square(hz(note("D5")), 0.08, 0.16, 0.5, release=0.3),
        square(hz(note("G5")), 0.20, 0.18, 0.5, release=0.5),
    ), gain=0.55)
    write("terrain.wav", mix(
        triangle(hz(note("D5")), 0.18, 0.14, release=0.4),
        triangle(hz(note("A5")), 0.24, 0.12, release=0.6),
    ), gain=0.5)
    write("ui.wav", square(hz(note("A4")), 0.05, 0.12, 0.25, release=0.5), gain=0.45)


# Music ---------------------------------------------------------------------

BEAT = 0.125  # 240 BPM eighth note, surf-punk pace
BAR = BEAT * 8


def bass_line():
    # E minor walking riff, 8 bars.
    roots = ["E2", "E2", "G2", "A2", "E2", "C3", "B2", "B2"]
    out = []
    for root in roots:
        base = note(root)
        pattern = [0, 7, 0, 12, 0, 10, 7, 3]
        for step, interval in enumerate(pattern):
            dur = BEAT * 0.82
            out.extend(square(hz(base + interval), dur, 0.20, 0.38,
                              release=0.25))
            out.extend(rest(BEAT * 0.18))
    return out


def lead_line():
    # Original tremolo spy lead over the same changes.
    melody = [
        "E4", "G4", "A4", "B4", "D5", "B4", "A4", "G4",
        "E4", "G4", "A4", "C5", "B4", "A4", "G4", "E4",
        "G4", "A4", "B4", "D5", "E5", "D5", "B4", "A4",
        "G4", "E4", "D4", "E4", "G4", "A4", "B4", "D5",
        "E5", "D5", "B4", "A4", "G4", "A4", "B4", "G4",
        "E4", "G4", "A4", "B4", "A4", "G4", "E4", "D4",
        "E4", "G4", "B4", "D5", "B4", "A4", "G4", "E4",
        "D4", "E4", "G4", "A4", "B4", "A4", "G4", "E4",
    ]
    out = []
    for i, name in enumerate(melody):
        # Tremolo: two fast attacks per eighth.
        out.extend(square(hz(note(name)), BEAT * 0.42, 0.09, 0.14, release=0.4))
        out.extend(square(hz(note(name)), BEAT * 0.34, 0.06, 0.14, release=0.5))
        out.extend(rest(BEAT * 0.24))
    return out


def pad_line():
    chords = [("E3", "B3", "G4"), ("E3", "B3", "G4"), ("G3", "D4", "B4"),
              ("A3", "E4", "C5"), ("E3", "B3", "G4"), ("C3", "G3", "E4"),
              ("B2", "F#3", "D4"), ("B2", "F#3", "D4")]
    out = []
    for chord in chords:
        for name in chord:
            out.extend(triangle(hz(note(name)), BAR * 0.96, 0.035, release=0.3))
        out.extend(rest(BAR * 0.04))
    return out


def drums():
    total = int(SR * BAR * 8)
    buf = [0.0] * total

    def add(samples, at):
        start = int(at * SR)
        for i, v in enumerate(samples):
            if start + i < total:
                buf[start + i] += v

    for bar_i in range(8):
        for step in range(8):
            t = BAR * bar_i + BEAT * step
            # Kick on 1 and 3-and.
            if step in (0, 3, 4):
                add(mix(
                    triangle(70, 0.09, 0.32, slide=38, release=0.7),
                    noise(0.05, 0.10, 7.0, 0.2, 30 + step),
                ), t)
            # Snare on 2 and 4 with a brush tail.
            if step in (2, 6):
                add(mix(
                    noise(0.10, 0.20, 5.0, 0.55, 50 + bar_i * 8 + step),
                    triangle(180, 0.05, 0.08, slide=120, release=0.5),
                ), t)
            # Closed hat on every offbeat.
            if step % 2 == 1:
                add(noise(0.035, 0.07, 9.0, 0.15, 90 + step), t)
    return buf


def music():
    bass = bass_line()
    lead = lead_line()
    pad = pad_line()
    drum = drums()
    # Align all parts to the longest (they should already match).
    total = max(len(bass), len(lead), len(pad), len(drum))
    body = [0.0] * total
    for part, gain in ((bass, 0.9), (lead, 0.8), (pad, 0.7), (drum, 0.65)):
        for i, v in enumerate(part):
            body[i] += v * gain
    # A short reverse-cymbal style pickup into the loop point.
    intro = noise(0.35, 0.10, 3.0, 0.3, 77)
    loop = intro + body
    write("music_interceptor.wav", loop, gain=0.72)


if __name__ == "__main__":
    sfx()
    music()

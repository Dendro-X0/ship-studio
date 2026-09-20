#!/usr/bin/env python3
"""Generate stylized silent GIFs for /demo (v0.1.0). Writes docs + public copies."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT_DIRS = [
    ROOT / "docs/assets/demo/v0.1.0",
    ROOT / "apps/website/public/demo/v0.1.0",
]

W, H = 720, 420
BG = (7, 9, 8)
PANEL = (14, 18, 16)
LINE = (36, 48, 42)
INK = (232, 238, 233)
MUTED = (139, 154, 145)
ACCENT = (94, 207, 148)
ACCENT_DIM = (22, 53, 40)
WARN = (212, 162, 76)


def font(size: int) -> ImageFont.ImageFont:
    for name in (
        "C:/Windows/Fonts/segoeui.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/consola.ttf",
    ):
        try:
            return ImageFont.truetype(name, size)
        except OSError:
            continue
    return ImageFont.load_default()


F_UI = font(18)
F_SM = font(14)
F_MONO = font(13)
F_BRAND = font(22)


def base() -> tuple[Image.Image, ImageDraw.ImageDraw]:
    im = Image.new("RGB", (W, H), BG)
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((28, 36, W - 28, H - 28), radius=14, fill=PANEL, outline=LINE, width=2)
    d.ellipse((48, 52, 62, 66), fill=(90, 60, 60))
    d.ellipse((70, 52, 84, 66), fill=(120, 100, 50))
    d.ellipse((92, 52, 106, 66), fill=(50, 100, 70))
    d.text((130, 50), "Ship Studio", fill=INK, font=F_BRAND)
    d.text((W - 200, 54), "local app · recorded", fill=MUTED, font=F_SM)
    d.rectangle((28, 88, 200, H - 28), fill=(10, 13, 11))
    d.line((200, 88, 200, H - 28), fill=LINE, width=1)
    for i, label in enumerate(["Pulse", "Publish", "Launch", "Portal"]):
        y = 110 + i * 36
        active = label == "Publish"
        if active:
            d.rounded_rectangle((40, y - 6, 186, y + 22), radius=6, fill=ACCENT_DIM)
            d.text((52, y), label, fill=ACCENT, font=F_UI)
        else:
            d.text((52, y), label, fill=MUTED, font=F_UI)
    return im, d


def draw_btn(d: ImageDraw.ImageDraw, xy: tuple[int, int, int, int], label: str, primary: bool = False) -> None:
    x0, y0, x1, y1 = xy
    fill = ACCENT if primary else (20, 28, 24)
    outline = ACCENT if primary else LINE
    ink = BG if primary else INK
    d.rounded_rectangle(xy, radius=8, fill=fill, outline=outline, width=1)
    tw = d.textlength(label, font=F_SM)
    d.text((x0 + (x1 - x0 - tw) / 2, y0 + 8), label, fill=ink, font=F_SM)


def save_gif(name: str, frames: list[Image.Image], duration: int = 280) -> None:
    for out in OUT_DIRS:
        out.mkdir(parents=True, exist_ok=True)
        path = out / name
        frames[0].save(
            path,
            save_all=True,
            append_images=frames[1:],
            duration=duration,
            loop=0,
            optimize=True,
        )
        print("wrote", path, path.stat().st_size)


def gif_bind() -> None:
    frames: list[Image.Image] = []
    for t in range(8):
        im, d = base()
        d.text((230, 110), "Bind project", fill=INK, font=F_UI)
        d.text((230, 140), "Pick a local folder for this session.", fill=MUTED, font=F_SM)
        d.rounded_rectangle(
            (230, 180, 660, 240),
            radius=8,
            fill=(10, 14, 12),
            outline=ACCENT if t > 2 else LINE,
            width=2,
        )
        path_txt = "E:/Web Projects/ship-studio"[: max(1, (t + 1) * 3)]
        d.text((248, 200), path_txt, fill=ACCENT if t > 2 else MUTED, font=F_MONO)
        draw_btn(d, (230, 270, 340, 304), "Browse", primary=False)
        draw_btn(d, (356, 270, 470, 304), "Bind", primary=t > 4)
        if t > 5:
            d.text((230, 330), "Bound · ready for Publish", fill=ACCENT, font=F_SM)
        frames.append(im)
    save_gif("01-bind.gif", frames)


def gif_open() -> None:
    frames: list[Image.Image] = []
    for t in range(8):
        im, d = base()
        d.text((230, 110), "Publish · Open", fill=INK, font=F_UI)
        d.text((230, 140), "Open the vendor door. Studio does not OAuth for you.", fill=MUTED, font=F_SM)
        for i, s in enumerate(["sign", "release", "deploy"]):
            x = 230 + i * 140
            on = t >= i * 2
            d.rounded_rectangle(
                (x, 190, x + 120, 250),
                radius=8,
                fill=ACCENT_DIM if on else (12, 16, 14),
                outline=ACCENT if on else LINE,
            )
            d.text((x + 28, 210), s, fill=ACCENT if on else MUTED, font=F_UI)
        draw_btn(d, (230, 290, 360, 324), "Open", primary=t >= 3)
        if t >= 5:
            d.text((380, 298), "browser · vendor console", fill=WARN, font=F_SM)
        frames.append(im)
    save_gif("02-open.gif", frames)


def gif_confirm() -> None:
    frames: list[Image.Image] = []
    for t in range(10):
        im, d = base()
        d.text((230, 110), "Confirm → Next", fill=INK, font=F_UI)
        d.text((230, 140), "You confirm each gate. Studio advances the spine.", fill=MUTED, font=F_SM)
        for i, g in enumerate(["signed", "tagged", "deployed"]):
            y = 185 + i * 42
            done = t > i * 3
            d.ellipse((240, y + 4, 256, y + 20), fill=ACCENT if done else LINE)
            d.text((270, y), g, fill=INK if done else MUTED, font=F_UI)
            if done:
                d.text((400, y), "ok", fill=ACCENT, font=F_MONO)
        phase = min(t // 3, 2)
        draw_btn(d, (230, 330, 340, 364), "Confirm", primary=(t % 3) == 1)
        draw_btn(d, (356, 330, 450, 364), "Next", primary=(t % 3) == 2 and t > 1)
        d.text((470, 338), f"gate {phase + 1}/3", fill=MUTED, font=F_SM)
        frames.append(im)
    save_gif("03-confirm-next.gif", frames, duration=240)


def gif_preview() -> None:
    frames: list[Image.Image] = []
    for t in range(8):
        im, d = base()
        d.text((230, 110), "Output Preview", fill=INK, font=F_UI)
        d.text((230, 140), "Inspect local artifacts before the next Confirm.", fill=MUTED, font=F_SM)
        d.rounded_rectangle((230, 175, 660, 320), radius=10, fill=(8, 11, 9), outline=LINE, width=1)
        for i, line in enumerate(["dist/shipctl.exe", "bundle/app.dmg", "notes/RELEASE.md"]):
            y = 195 + i * 28
            hi = (t % 3) == i
            if hi:
                d.rounded_rectangle((244, y - 4, 640, y + 22), radius=4, fill=ACCENT_DIM)
            d.text((256, y), line, fill=ACCENT if hi else MUTED, font=F_MONO)
        draw_btn(d, (230, 340, 400, 374), "Open preview", primary=t > 2)
        frames.append(im)
    save_gif("04-output-preview.gif", frames)


def main() -> None:
    gif_bind()
    gif_open()
    gif_confirm()
    gif_preview()
    print("done")


if __name__ == "__main__":
    main()

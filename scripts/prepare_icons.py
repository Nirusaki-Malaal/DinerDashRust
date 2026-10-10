#!/usr/bin/env python3
"""Derive platform icons from the original Windows ICO without redrawing it."""
from pathlib import Path
from PIL import Image
ROOT = Path(__file__).resolve().parents[1]
source = Image.open(ROOT / 'app_icon.ico').convert('RGBA')
for size in (16, 32, 64):
    (ROOT / f'packaging/icon{size}.rgba').write_bytes(source.resize((size, size), Image.Resampling.LANCZOS).tobytes())
for density, size in [('mdpi',48),('hdpi',72),('xhdpi',96),('xxhdpi',144),('xxxhdpi',192)]:
    dest = ROOT / 'packaging/android/res' / f'mipmap-{density}'
    dest.mkdir(parents=True, exist_ok=True)
    source.resize((size,size), Image.Resampling.LANCZOS).save(dest/'ic_launcher.png')
source.resize((256,256), Image.Resampling.LANCZOS).save(ROOT/'packaging/diner-dash.png')

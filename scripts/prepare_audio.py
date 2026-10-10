#!/usr/bin/env python3
"""Resample all game audio to 44100 Hz 16-bit stereo for clean quad-snd playback."""
import subprocess
import wave
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIO_DIR = ROOT / "assets/audio"

def resample_all():
    print("Resampling audio to 44100 Hz stereo...")
    for wav_file in sorted(AUDIO_DIR.glob("*.wav")):
        try:
            with wave.open(str(wav_file), "rb") as w:
                rate = w.getframerate()
                ch = w.getnchannels()
            if rate == 44100 and ch == 2:
                continue
            tmp = wav_file.with_suffix(".tmp.wav")
            subprocess.run(
                ["ffmpeg", "-y", "-i", str(wav_file), "-ar", "44100", "-ac", "2", str(tmp)],
                check=True,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            tmp.replace(wav_file)
            print(f"Resampled {wav_file.name} ({rate}Hz/{ch}ch -> 44100Hz/2ch)")
        except Exception as e:
            print(f"Error processing {wav_file}: {e}")

if __name__ == "__main__":
    resample_all()


"""Draws Dropwheel's icon and tray icon as SVG, then renders them with the Tauri CLI.

Run from the workspace root after `npm --prefix ui install`:

    python scripts/make-icons.py
"""

import math
import pathlib
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
ASSETS = ROOT / "ui" / "src" / "assets"
ICONS = ROOT / "src-tauri" / "icons"
TAURI = ROOT / "ui" / "node_modules" / ".bin" / "tauri"


def wedge(cx, cy, r0, r1, a0, a1):
    """An annular sector; angles in degrees, clockwise from twelve o'clock."""

    def pt(r, a):
        t = math.radians(a - 90)
        return f"{cx + r * math.cos(t):.1f} {cy + r * math.sin(t):.1f}"

    large = 1 if a1 - a0 > 180 else 0
    return (
        f"M{pt(r1, a0)} A{r1} {r1} 0 {large} 1 {pt(r1, a1)} "
        f"L{pt(r0, a1)} A{r0} {r0} 0 {large} 0 {pt(r0, a0)} Z"
    )


def icon(wedges, r0, r1, gap, hub, arrow, corner, size=1024):
    c = size / 2
    step = 360 / wedges
    parts = []
    for i in range(wedges):
        a0 = i * step - step / 2 + gap / 2
        a1 = (i + 1) * step - step / 2 - gap / 2
        if i == 1:  # the picked action
            parts.append(f'<path d="{wedge(c, c, r0, r1, a0, a1)}" fill="url(#hot)"/>')
        else:
            parts.append(
                f'<path d="{wedge(c, c, r0, r1, a0, a1)}" fill="#CFC8FF" fill-opacity="0.34"/>'
            )
    shape = [(-0.18, -0.62), (0.18, -0.62), (0.18, 0.02), (0.52, 0.02), (0, 0.62), (-0.52, 0.02), (-0.18, 0.02)]
    points = " ".join(f"{c + x * arrow:.1f},{c + y * arrow:.1f}" for x, y in shape)
    body = "\n  ".join(parts)
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#2B2456"/>
      <stop offset="1" stop-color="#15122B"/>
    </linearGradient>
    <linearGradient id="hot" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#FFB86B"/>
      <stop offset="1" stop-color="#FF6B6B"/>
    </linearGradient>
  </defs>
  <rect width="{size}" height="{size}" rx="{corner}" fill="url(#bg)"/>
  {body}
  <circle cx="{c}" cy="{c}" r="{hub}" fill="#F5F3FF"/>
  <polygon points="{points}" fill="#2B2456"/>
</svg>
"""


def main():
    app_svg = ASSETS / "icon.svg"
    tray_svg = ASSETS / "tray.svg"
    app_svg.write_text(icon(8, 262, 420, 7, 196, 150, corner=232))
    # Fewer, chunkier wedges stay legible at 16-32 px.
    tray_svg.write_text(icon(6, 250, 440, 12, 200, 168, corner=200))

    subprocess.run([str(TAURI), "icon", str(app_svg), "-o", str(ICONS)], check=True)
    for extra in ["android", "ios", "icon.icns", "StoreLogo.png", *ICONS.glob("Square*Logo.png")]:
        target = ICONS / extra
        shutil.rmtree(target) if target.is_dir() else target.unlink(missing_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run([str(TAURI), "icon", str(tray_svg), "-o", tmp, "-p", "32"], check=True)
        shutil.copy(pathlib.Path(tmp) / "32x32.png", ICONS / "tray.png")


if __name__ == "__main__":
    main()

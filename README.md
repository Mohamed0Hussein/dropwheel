# Dropwheel

A Windows tray utility for files. Drop files on a small floating shelf, and a radial menu
opens where you dropped them. Pick a wedge to convert, compress, crop, annotate or redact.

> **Dropwheel is a working name.** It stands in for the final product name. See
> [Renaming](#renaming).

Targets Windows 10 and 11, x64.

## Status

**Phases 1 to 4 of 6 are done.** The shelf, tray and radial menu work, and every image
action is real: convert, compress, crop, edit, annotate, redact, metadata and add
background. The PDF, DOCX and TXT work (phase 5) still reports "not available yet".

| Feature                                                                  | Status          |
| ------------------------------------------------------------------------ | --------------- |
| Tray icon with Show/Hide shelf, Settings and Quit                        | Done            |
| Floating shelf: frameless, transparent, always on top, light/dark theme  | Done            |
| Drag the shelf to move it; position remembered, reset if off-screen      | Done            |
| Shelf and menu kept out of the taskbar and Alt+Tab                       | Done            |
| Single instance (launching again shows the shelf)                        | Done            |
| Settings in `%APPDATA%\Dropwheel\settings.json`                          | Done (read-only window) |
| Drag and drop from Explorer onto the shelf                               | Done            |
| Radial menu at the drop point: Convert ring and Tools ring, only the actions that fit | Done |
| Hover or keyboard focus shows the action in the centre; Esc or a click outside closes | Done |
| Open and close animation (scale and fade, compositor only)               | Done            |
| Several files at once: the menu offers what fits all of them             | Done            |
| Background jobs with progress and a result on the shelf                  | Done            |
| Images: convert between JPG, PNG, WEBP, BMP, TIFF, AVIF (and HEIC, see below) | Done   |
| Compress by quality or to a target size, with a live size estimate      | Done            |
| Crop with handles, aspect chips (Free, 1:1, 4:3, 3:4, 16:9, 9:16) and pixel inputs | Done |
| Edit: rotate, flip, grayscale, brightness, contrast, with a live preview | Done            |
| Metadata: EXIF, location, PNG text, comments and XMP listed; remove all  | Done            |
| Add a solid background behind transparency                               | Done            |
| Annotate: boxes, arrows and text (Arabic too), with move, delete and undo | Done           |
| Redact: solid black boxes; the copy carries no metadata                  | Done for images; PDF in phase 5 |
| PDF, DOCX and TXT conversions; PDF compression                           | Phase 5         |
| Editable settings, toasts, start with Windows                            | Phase 6         |
| NSIS and MSI installers, portable .exe                                   | Builds now; polished in phase 6 |

### Which actions appear

| Dropped file                     | Convert ring                              | Tools ring |
| -------------------------------- | ----------------------------------------- | ---------- |
| JPG, PNG, WEBP, BMP, TIFF, GIF, ICO, AVIF, HEIC | PDF and the other image formats | Crop, Edit, Annotate and Redact (all but ICO); Compress (JPG, PNG, WEBP, AVIF, TIFF); Metadata (JPG, PNG, WEBP, TIFF, AVIF, HEIC); Add background (PNG, WEBP, AVIF, TIFF, GIF) |
| PDF                              | DOCX, TXT, JPG, PNG, WEBP, BMP, TIFF      | Compress, Crop, Redact, Metadata |
| DOCX                             | PDF, TXT                                  | Metadata |
| TXT                              | DOCX, PDF                                 | none |
| Anything else, or a folder       | none                                      | none |

A file's type comes from its content, so a PNG renamed to `.jpg` is treated as a PNG.
When several files are dropped, the menu shows only the actions that apply to all of
them.

### Image formats

| Format | Read | Write | Notes |
| ------ | ---- | ----- | ----- |
| JPG    | Yes  | Yes   | Progressive, optimised. EXIF orientation is applied, so photos come out upright. |
| PNG    | Yes  | Yes   | Compress below 90% reduces colours to a palette. |
| WEBP   | Yes  | Yes   | Lossy from photos, lossless from graphics. Animated files: first frame. |
| BMP    | Yes  | Yes   | |
| TIFF   | Yes  | Yes   | Written losslessly with Deflate. Multi-page files: first page. |
| GIF    | Yes  | Yes   | Animated files: first frame. |
| ICO    | Yes  | No    | Edits of an icon are saved as PNG. |
| AVIF   | Windows, with the free "AV1 Video Extension" | Yes | Encoding is slow on large images. |
| HEIC   | Windows, with the free "HEIF Image Extensions" | With Microsoft's paid "HEVC Video Extensions" | Without the encoder, the HEIC wedge is hidden and HEIC edits are saved as JPG. |

Results are written next to the original as `name_converted.png`, `name_compressed.jpg`,
`name_cropped.jpg`, `name_edited.jpg`, `name_annotated.jpg`, `name_redacted.jpg`,
`name_clean.jpg` (metadata removed) or `name_background.png`. Nothing is ever overwritten: a second run gives
`name_compressed (2).jpg`.

## Building

### Prerequisites (Windows)

- [Rust](https://rustup.rs) stable, 1.92 or newer, with the MSVC toolchain. Install the
  Visual Studio Build Tools with the "Desktop development with C++" workload.
- [Node.js](https://nodejs.org) 20.19+ or 22.12+ (check with `node --version`; Vite needs
  it). `winget install OpenJS.NodeJS.LTS` installs the current LTS.
- WebView2 Runtime. Windows 11 ships with it, and most Windows 10 machines have it
  through Edge.

### Run in development

Quit any installed copy first (tray icon → Quit). Only one Dropwheel runs at a time, so
a second one just shows the first one's shelf. Then, from the `ui` folder:

```powershell
cd ui
npm install
npm run tauri dev
```

If you have the Cargo subcommand (`cargo install tauri-cli --version "^2" --locked`),
`cargo tauri dev` from `dropwheel/` does the same as the last line. Either way, the Vite
dev server starts on port 1420 and the app reloads when Rust files change. The first run
compiles for several minutes.

To work on the menu without dragging files each time, list them in
`DROPWHEEL_DEV_DROP`, separated by `;`. Development builds then open the menu for them in
the middle of the screen at startup. Release builds ignore the variable.

```powershell
$env:DROPWHEEL_DEV_DROP = "C:\Users\me\Pictures\photo.jpg;C:\Users\me\report.pdf"
npm run tauri dev
```

### Release build

From the `ui` folder:

```powershell
npm run tauri build
```

| Output                                                  | What it is                             |
| ------------------------------------------------------- | -------------------------------------- |
| `target\release\dropwheel.exe`                          | Portable executable                    |
| `target\release\bundle\nsis\Dropwheel_0.1.0_x64-setup.exe` | Installer, per user, no admin rights |
| `target\release\bundle\msi\Dropwheel_0.1.0_x64_en-US.msi`  | Installer, per machine               |

Paths are relative to `dropwheel/`. The first release build downloads the NSIS and WiX
toolsets.

### Checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd ui
npm run typecheck
npm test
```

CI runs all of these on `windows-latest`, then builds both installers and the portable
exe and uploads them as the `dropwheel-windows-x64` artifact. See
[`.github/workflows/dropwheel.yml`](../.github/workflows/dropwheel.yml).

## Layout

```
dropwheel/
├─ crates/core/        Pure Rust file operations: no UI, no Tauri. Unit tested.
├─ src-tauri/          The Tauri 2 app: tray, windows, settings, IPC commands.
│  ├─ capabilities/    What each window may call.
│  └─ icons/           Generated by scripts/make-icons.py.
├─ ui/                 Vanilla TypeScript and Vite. One HTML page per window:
│  │                   shelf, radial (the menu) and settings.
│  ├─ src/lib/         Shared code: IPC types, wedge geometry, icons.
│  └─ src/assets/      Icon sources (SVG).
├─ scripts/            Icon generator.
├─ DECISIONS.md        Why things are the way they are.
└─ README.md
```

Each action in `crates/core` implements one trait:

```rust
pub trait Action: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn group(&self) -> ActionGroup;
    fn applicable(&self, kind: FileKind) -> bool;
    fn run(&self, input: &Path, opts: &Options, out: &Path) -> Result<PathBuf>;
}
```

`out` is the output folder. `run` writes through `output::write_new`, which never touches
the input or replaces an existing file.

## Files Dropwheel writes

| Path                                                    | Contents                       |
| ------------------------------------------------------- | ------------------------------ |
| `%APPDATA%\Dropwheel\settings.json`                     | Settings, including shelf position |
| `%APPDATA%\Dropwheel\settings.json.bad`                 | A settings file that could not be read, set aside |
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\logs\` | Log files                    |
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\EBWebView\` | WebView2 data          |

## Known limitations

- Windows only. The code also compiles on Linux so development checks can run there, but
  that is not a supported platform.
- The portable exe needs the WebView2 Runtime and keeps its settings in `%APPDATA%`, the
  same as the installed app.
- Every PDF, DOCX and TXT action (phase 5) still reports "not available yet".
- Annotation text uses the bundled Noto Sans and Noto Sans Arabic. Other scripts use
  fonts that come with Windows (Segoe UI, Microsoft YaHei, Malgun Gothic, Nirmala UI and
  others); emoji are drawn as plain glyphs where those fonts have them.
- HEIC and AVIF are read through Windows' own codecs, which come from Microsoft Store
  extensions. Without them, opening those files fails with a message naming the
  extension to install. Writing HEIC needs the paid "HEVC Video Extensions".
- Conversions, compression and edits keep the picture and its colour profile (JPG and
  PNG) but not its EXIF data, since orientation is applied to the pixels. Remove metadata
  is the action for stripping deliberately.
- Images with 16 bits per channel are written with 8.
- Animated GIF and WebP, and multi-page TIFF, are reduced to their first frame or page.
- Showing the shelf from the tray gives it keyboard focus.
- If you hide the shelf, it still appears for an instant when Dropwheel starts. This
  works around a Windows WebView bug that blocks file drops on windows created hidden.
- Some Windows behaviour has been built and reasoned about but not yet checked by hand on
  Windows 10 and 11: Alt+Tab exclusion, position restore across monitors, and HEIC and
  AVIF through Windows' codecs. Please report anything that looks off.

## Credits

This software is based in part on the work of the Independent JPEG Group (through the
`jpeg-encoder` crate). WebP is written with Google's libwebp (BSD licence). The Noto Sans
and Noto Sans Arabic fonts are © The Noto Project Authors, under the SIL Open Font
License 1.1 (`crates/core/fonts/OFL-*.txt`). Everything else is under permissive
licences (MIT, Apache-2.0, BSD); `cargo tree` lists them.

## Renaming

The product name appears in these places:

- `src-tauri/tauri.conf.json`: `productName` (drives window titles, the tray tooltip and the
  `%APPDATA%` folder) and `identifier`.
- `ui/*.html`: page titles.
- Crate and package names (`dropwheel`, `dropwheel-core`, `dropwheel-ui`) and the folder
  name. Changing these is optional.

`git grep -i dropwheel` lists every occurrence.

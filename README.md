# Dropwheel

A Windows tray utility for files. Drop files on a small floating shelf, and a radial menu
opens where you dropped them. Pick a wedge to convert, compress, crop, annotate or redact.

> **Dropwheel is a working name.** It stands in for the final product name. See
> [Renaming](#renaming).

Targets Windows 10 and 11, x64.

## Status

**Phases 1 and 2 of 6 are done.** The shelf and tray work. Dropping files opens the radial
menu, and choosing an action runs it in the background. The actions themselves are
placeholders that report "not available yet" until phases 3 to 5 fill them in.

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
| Images: convert, compress, crop, edit, metadata, add background          | Phase 3         |
| Annotate, redact                                                         | Phase 4         |
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

## Building

### Prerequisites (Windows)

- [Rust](https://rustup.rs) stable, 1.90 or newer, with the MSVC toolchain. Install the
  Visual Studio Build Tools with the "Desktop development with C++" workload.
- [Node.js](https://nodejs.org) 20.19+ or 22.12+.
- WebView2 Runtime. Windows 11 ships with it, and most Windows 10 machines have it
  through Edge.

### Run in development

From this folder (`dropwheel/`):

```powershell
npm --prefix ui install
npm --prefix ui run tauri dev
```

If you have the Cargo subcommand (`cargo install tauri-cli --version "^2" --locked`), use
`cargo tauri dev` instead of the second line. Either way, the Vite dev server starts on
port 1420 and the app reloads when Rust files change.

To work on the menu without dragging files each time, list them in
`DROPWHEEL_DEV_DROP`, separated by `;`. Development builds then open the menu for them in
the middle of the screen at startup. Release builds ignore the variable.

```powershell
$env:DROPWHEEL_DEV_DROP = "C:\Users\me\Pictures\photo.jpg;C:\Users\me\report.pdf"
npm --prefix ui run tauri dev
```

### Release build

```powershell
npm --prefix ui run tauri build
```

| Output                                                  | What it is                             |
| ------------------------------------------------------- | -------------------------------------- |
| `target\release\dropwheel.exe`                          | Portable executable                    |
| `target\release\bundle\nsis\Dropwheel_0.1.0_x64-setup.exe` | Installer, per user, no admin rights |
| `target\release\bundle\msi\Dropwheel_0.1.0_x64_en-US.msi`  | Installer, per machine               |

The first release build downloads the NSIS and WiX toolsets.

### Checks

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm --prefix ui run typecheck
npm --prefix ui test
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
- Every action is a placeholder until phases 3 to 5 and reports "not available yet".
- HEIC appears as a Convert target, but writing HEIC needs an HEVC encoder. Windows only
  has one through Microsoft's paid "HEVC Video Extensions", and the open-source x265
  encoder is GPL-licensed. See DECISIONS.md.
- Showing the shelf from the tray gives it keyboard focus.
- Some Windows behaviour has been built and reasoned about but not yet checked by hand on
  Windows 10 and 11: Alt+Tab exclusion, position restore across monitors, and the menu
  taking focus after a drop from Explorer. Please report anything that looks off.

## Renaming

The product name appears in these places:

- `src-tauri/tauri.conf.json`: `productName` (drives window titles, the tray tooltip and the
  `%APPDATA%` folder) and `identifier`.
- `ui/*.html`: page titles.
- Crate and package names (`dropwheel`, `dropwheel-core`, `dropwheel-ui`) and the folder
  name. Changing these is optional.

`git grep -i dropwheel` lists every occurrence.

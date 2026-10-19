# Dropwheel

A Windows tray utility for files. Drop files on a small floating shelf, and a radial menu
opens where you dropped them. Pick a wedge to convert, compress, crop, annotate or redact.

> **Dropwheel is a working name.** It stands in for the final product name. See
> [Renaming](#renaming).

Targets Windows 10 and 11, x64.

## Status

**All six phases are done.** Every action in the menu works, for images, PDF, DOCX and
TXT. Settings, notifications, starting with Windows and the installers are in place.

| Feature                                                                  | Status          |
| ------------------------------------------------------------------------ | --------------- |
| Tray icon with Show/Hide shelf, Settings and Quit                        | Done            |
| Floating shelf: frameless, transparent, always on top, light/dark theme  | Done            |
| Drag the shelf to move it; position remembered, reset if off-screen      | Done            |
| Shelf and menu kept out of the taskbar and Alt+Tab                       | Done            |
| Single instance (launching again shows the shelf)                        | Done            |
| Settings in `%APPDATA%\Dropwheel\settings.json`, edited in a Settings window | Done      |
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
| Redact: solid black boxes; the copy carries no metadata; PDF pages too   | Done            |
| Images to PDF; PDF pages to JPG, PNG, WEBP, BMP, TIFF                    | Done            |
| PDF to DOCX and TXT, Arabic included; DOCX and TXT to each other and to PDF | Done         |
| PDF compression (pictures recompressed), cropping (every page) and metadata | Done         |
| DOCX metadata: properties, custom properties, thumbnail, reviewer names  | Done            |
| Results next to the original or all in one chosen folder                 | Done            |
| Windows notification when a job ends, with Open and Show in Explorer     | Done            |
| Start with Windows (per user, no admin rights)                           | Done            |
| NSIS and MSI installers, portable .exe                                   | Done            |
| Third-party licences, opened from Settings                               | Done            |

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

### Documents

| From | To | How |
| ---- | -- | --- |
| Image | PDF | One A4 page, turned to suit the picture, the picture centred within half-inch margins and never enlarged past its size at 96 dpi. JPEGs go in as they are. |
| PDF | JPG, PNG, WEBP, BMP, TIFF | Pages rendered at 200 dpi. One page gives one file; more give a folder, `name_pages`, with `name_page1.png` and so on. |
| PDF | TXT | Each line as on the page, a blank line between paragraphs and pages. |
| PDF | DOCX | Lines joined back into paragraphs, larger text as headings, bold and italic kept, pictures cut from the page. Pages with no text, such as scans, become a picture of the page. |
| DOCX | PDF | Laid out afresh: headings, bold, italic, underline, colours, sizes, bullets and numbering, tables, pictures, page breaks and right-to-left paragraphs. |
| DOCX | TXT | Paragraphs as lines, list markers kept, table cells separated by tabs. |
| TXT | DOCX, PDF | One paragraph per line. UTF-8, UTF-16 and older encodings (such as Windows-1252 and Windows-1256 for Arabic) are recognised. |

PDFs are written with searchable text in Noto Sans and IBM Plex Sans Arabic, with
Windows fonts for other scripts. On PDFs, Compress recompresses the pictures (and below
90% scales large ones down); Crop applies the same rectangle to every page; Redact turns
every page into a picture with the boxes painted in; Metadata removes the document
information, XMP and private application data. Password-protected PDFs are refused with
a message saying so.

Results are written next to the original, or in the folder chosen in Settings, as
`name_converted.png`, `name_compressed.jpg`, `name_cropped.jpg`, `name_edited.jpg`,
`name_annotated.jpg`, `name_redacted.jpg`, `name_clean.jpg` (metadata removed) or
`name_background.png`. Nothing is ever overwritten: a second run gives
`name_compressed (2).jpg`.

### Settings and notifications

Settings opens from the tray menu. Changes apply at once; there is no Save button.

- **Results**: next to the original, or all in one folder. The folder must exist and
  Dropwheel must be able to write in it, or the change is refused with a message.
- **Start with Windows**: adds Dropwheel to the current user's startup programs. No
  admin rights are needed. If the exe moves (a portable copy), the entry follows it the
  next time Dropwheel starts.
- **Notify when done**: when a job ends, Windows shows a notification with the result
  and two buttons. **Open** opens the file in its usual app (or the folder, when there
  are several results); **Show in Explorer** opens its folder with the file selected.
  Clicking the notification itself is the same as Open. The buttons still work from the
  notification centre later on; if Dropwheel has been quit, they start it.
- **Show the shelf**: the same as Show/Hide shelf in the tray menu.

While a job runs, the shelf shows its progress and the dialog offers Cancel. A cancelled
job does not notify.

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

Both `npm run tauri dev` and `npm run tauri build` first run `scripts/fetch-pdfium.mjs`,
which downloads PDFium (about 4 MB, pinned and checked against a SHA-256) into
`vendor/pdfium` once. The build embeds it in the exe. Without it, everything builds and
runs, but PDF actions report that PDF support is missing. To fetch it by hand:
`node scripts/fetch-pdfium.mjs`.

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
| `target\release\bundle\nsis\Dropwheel_0.1.0_x64-setup.exe` | Installer, per user, no admin rights (recommended) |
| `target\release\bundle\msi\Dropwheel_0.1.0_x64_en-US.msi`  | Installer, per machine               |

Paths are relative to `dropwheel/`. The first release build downloads the NSIS and WiX
toolsets.

The portable exe is the whole app: PDFium, the fonts and the pages are inside it. Copy
it anywhere and run it. Both installers add a Start menu entry and an uninstaller; the
NSIS one also removes Dropwheel's registry entries on uninstall
(`src-tauri/windows/hooks.nsh`) and can delete its settings and data if you tick the box.
If WebView2 is missing, the installers download it.

### Checks

```powershell
node scripts/fetch-pdfium.mjs   # once; the PDF tests skip themselves without it
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python scripts/make-notices.py --check
cd ui
npm run typecheck
npm test
```

The notices check needs Python 3. After adding or updating a dependency, run
`python scripts/make-notices.py` to rewrite `THIRD-PARTY-NOTICES.txt`.

CI runs all of these on `windows-latest`, then builds both installers and the portable
exe and uploads them as the `dropwheel-windows-x64` artifact. See
[`.github/workflows/dropwheel.yml`](../.github/workflows/dropwheel.yml).

## Layout

```
dropwheel/
├─ crates/core/        Pure Rust file operations: no UI, no Tauri. Unit tested.
├─ src-tauri/          The Tauri 2 app: tray, windows, settings, IPC commands,
│  │                   notifications and other Windows integration.
│  ├─ capabilities/    What each window may call.
│  ├─ icons/           Generated by scripts/make-icons.py.
│  └─ windows/         NSIS installer hooks.
├─ ui/                 Vanilla TypeScript and Vite. One HTML page per window:
│  │                   shelf, radial (the menu) and settings.
│  ├─ src/lib/         Shared code: IPC types, wedge geometry, icons.
│  └─ src/assets/      Icon sources (SVG).
├─ scripts/            Icon generator; fetch-pdfium.mjs downloads PDFium;
│                      make-notices.py writes THIRD-PARTY-NOTICES.txt.
├─ vendor/pdfium/      PDFium, once downloaded (not in Git).
├─ licenses/           Licences of PDFium and the libraries built into it.
├─ THIRD-PARTY-NOTICES.txt  Every licence of what is built into the app.
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
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\pdfium\` | PDFium, written out of the exe on first start |
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\results.json` | The last 200 results, which a notification's Open may open |
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\notification-icon.png` | The icon Windows shows on notifications |
| `%LOCALAPPDATA%\io.github.mohamed0hussein.dropwheel\THIRD-PARTY-NOTICES.txt` | Written when you open the licences from Settings |

And in the registry, all under `HKEY_CURRENT_USER`:

| Key                                                     | What it is for                 |
| ------------------------------------------------------- | ------------------------------ |
| `Software\Microsoft\Windows\CurrentVersion\Run`, value `Dropwheel` | Start with Windows, when it is on |
| `Software\Classes\AppUserModelId\io.github.mohamed0hussein.dropwheel` | The name and icon on notifications |
| `Software\Classes\dropwheel`                            | The `dropwheel:` links behind the notification buttons |

The `dropwheel:` links can only select a file in Explorer, or open a file Dropwheel
itself wrote (one in `results.json`). Any other request is ignored, so a web page cannot
use them to open files on your computer.

## Known limitations

- Windows only. The code also compiles on Linux so development checks can run there, but
  that is not a supported platform.
- The portable exe needs the WebView2 Runtime and keeps its settings in `%APPDATA%`, the
  same as the installed app.
- Text is set in the bundled Noto Sans and IBM Plex Sans Arabic. Other scripts use fonts
  that come with Windows (Segoe UI, Microsoft YaHei, Malgun Gothic, Nirmala UI and
  others); emoji are drawn as plain glyphs where those fonts have them. Italics are
  slanted Noto Sans, as no italic face is bundled.
- PDF to DOCX rebuilds paragraphs from where the lines are; it does not recover tables,
  columns or the exact layout. Text in two columns comes out column by column. Scanned
  pages become pictures: there is no text recognition (OCR).
- DOCX to PDF leaves out headers, footers, footnotes, comments, text effects and charts,
  and uses the bundled fonts rather than the document's. Pictures in Windows metafile
  formats (EMF, WMF) are skipped.
- Cropping a PDF hides the rest of each page without deleting it, as PDF viewers'
  own crop does. Redact is the way to remove content.
- A redacted PDF's pages are pictures, so its text can no longer be selected or
  searched, and the file is larger.
- PDF compression only touches pictures stored as JPEG, or uncompressed and Flate
  pictures in RGB or grey. Fonts and other content are packed but not rewritten.
- Password-protected PDFs are refused rather than opened.
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
- Notifications follow Windows' own settings: with Do Not Disturb (Focus Assist) on, or
  notifications turned off for Dropwheel in Windows Settings, they go straight to the
  notification centre or nowhere.
- A notification's Open only opens one of the last 200 results; after that it does
  nothing, and Show in Explorer still works.
- Uninstalling the MSI leaves the registry entries above (the NSIS uninstaller removes
  them). They are harmless: the startup entry of a missing exe does nothing. Turn off
  Start with Windows before uninstalling to remove it.
- Some Windows behaviour has been built and reasoned about but not yet checked by hand on
  Windows 10 and 11: Alt+Tab exclusion, position restore across monitors, HEIC and AVIF
  through Windows' codecs, notifications and their buttons, starting with Windows, and
  the installers' uninstall steps. Please report anything that looks off.

## Credits

This software is based in part on the work of the Independent JPEG Group (through the
`jpeg-encoder` crate and PDFium's libjpeg-turbo). WebP is written with Google's libwebp
(BSD licence). PDFs are read with PDFium (BSD licence), in the build by Benoit Blanchon's
pdfium-binaries; its licence and those of the libraries in it, including FreeType
("Portions of this software are copyright © The FreeType Project, www.freetype.org.
All rights reserved."), are in `licenses/pdfium/`. The Noto Sans fonts are © The Noto
Project Authors and IBM Plex Sans Arabic is © IBM Corp., both under the SIL Open Font
License 1.1 (`crates/core/fonts/OFL-*.txt`). Everything else is under permissive
licences (MIT, Apache-2.0, BSD and similar). `THIRD-PARTY-NOTICES.txt` has every licence
text and copyright notice; the app opens it from Settings → Third-party licences. A few
crates that Tauri uses to parse CSS (`cssparser`, `selectors`, `dtoa-short`) and
`option-ext` are under the Mozilla Public License 2.0; their source is on crates.io,
unchanged.

## Renaming

The product name appears in these places:

- `src-tauri/tauri.conf.json`: `productName` (drives window titles, the tray tooltip and the
  `%APPDATA%` folder) and `identifier`.
- `ui/*.html`: page titles.
- `src-tauri/src/integration.rs` (`SCHEME`) and `src-tauri/windows/hooks.nsh`: the
  `dropwheel:` link scheme.
- Crate and package names (`dropwheel`, `dropwheel-core`, `dropwheel-ui`) and the folder
  name. Changing these is optional.

`git grep -i dropwheel` lists every occurrence.

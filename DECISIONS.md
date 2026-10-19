# Decisions

Choices made while building Dropwheel, with the reason for each. Newest phase last.

## Phase 1: scaffold, tray and shelf

### Project

- **Its own repository.** The work was started in `dropwheel/` inside the owner's
  portfolio repository, self-contained (its own Cargo workspace, npm package and
  `.gitignore`). It was then moved here with `git subtree split --prefix dropwheel`,
  which keeps its history. The CI workflow moved from the portfolio's
  `.github/workflows/dropwheel.yml` to `.github/workflows/ci.yml`.
- **"Dropwheel" is a working name** for the placeholder product name. README → Renaming
  lists every place to change.
- **Identifier `io.github.mohamed0hussein.dropwheel`**, after the GitHub account that owns
  the repo. The installer publisher is set to "Mohamed Hussein". Both are in
  `src-tauri/tauri.conf.json` and easy to change.
- **No license is declared.** Picking one is the owner's call.

### Stack

- **Tauri 2.12.** Tauri 3 is still in alpha.
- **Vanilla TypeScript, Vite 8, TypeScript 6.** The UI is a few small windows. A framework
  would add more weight than it saves. TypeScript 7 (the native port) is new enough that I
  held back.
- **Rust edition 2024, minimum Rust 1.92.** Tauri needs 1.90; the PDF writer `krilla`
  (phase 5) needs 1.92.
- **The `ui/` folder has its own `package.json`.** Its `tauri` script runs the CLI from the
  workspace root (`cd .. && tauri`). The Tauri config starts Vite with `cwd: "../ui"`, so
  `cargo tauri dev` from the root and `npm run tauri dev` from `ui/` both work.
  The README avoids `npm --prefix ui install`: some older npm versions read
  `package.json` from the current folder and ignore the prefix. It failed that way on a
  real Windows machine, while npm 10 handles it.
- **Dependencies get `opt-level = 2` in dev builds.** Image codecs are very slow when not
  optimised, and later phases' tests decode real images.
- **Clippy denies `unwrap`, `expect` and `panic!` outside tests.** It warns on them, and CI
  runs with `-D warnings`. This enforces the "no panics on malformed input" rule.
- **Logging** goes through `tauri-plugin-log` to stdout and to
  `%LOCALAPPDATA%\<identifier>\logs`. Release builds have no console, so the log file is
  how problems get diagnosed.

### Windows and tray

- **The shelf is created in Rust, not in `tauri.conf.json`,** so it can be built directly
  at its saved position.
- **Windows are never created hidden.** On Windows, a WebView created hidden and unfocused
  never accepts file drops, even after it is shown. The cursor shows "not allowed" over
  it ([tauri#14643](https://github.com/tauri-apps/tauri/issues/14643), fixed upstream in
  [wry#1638](https://github.com/tauri-apps/wry/pull/1638) but not yet in a wry release).
  Phase 1 created the shelf hidden to avoid a flash before positioning, and drops failed
  on a real Windows machine. So the shelf is now created visible and focusable, at its
  final position: the physical position is converted to logical pixels with that
  display's scale, then set exactly once the window exists. A shelf the user has hidden
  is hidden right after creation, so it appears for an instant at startup. A restored
  position is pulled fully onto its display first. Otherwise a creation point off every
  display makes Windows choose a default spot instead.
- **The shelf is owned by a hidden window** (`src-tauri/src/platform.rs`). tao gives every
  unowned window `WS_EX_APPWINDOW`, which keeps it in Alt+Tab even with `skip_taskbar`.
  Windows leaves owned windows out of both the taskbar and Alt+Tab. Setting
  `WS_EX_TOOLWINDOW` directly does not stick, because tao rewrites the extended style on
  every show and hide.
- **Positions are saved in physical pixels on the virtual desktop**, as Windows reports
  them. On start, a saved position is used only if the shelf's centre lands on a connected
  monitor. Otherwise the shelf goes to the bottom-right of the primary monitor's work area.
- **Dragging the shelf calls `startDragging()` directly** instead of using
  `data-tauri-drag-region`. With the attribute, a double-click asks the window to maximise.
- **Alt+F4 on the shelf hides it.** Closing every window does not quit either, since the
  app lives in the tray. Only Quit in the tray menu exits.
- **Left-clicking the tray icon toggles the shelf.** Right-click opens the menu. The menu's
  first item reads "Hide shelf" or "Show shelf" to match the shelf's state.
- **Single instance.** Launching Dropwheel again shows the existing shelf instead of
  adding a second tray icon.
- **Strict Content Security Policy.** Pages may load only their own scripts and styles,
  inline styles and `data:`/`blob:` images, and may call only Tauri IPC.
- **Release builds turn off the WebView's browser behaviour**: the context menu and the
  reload, print, find and view-source shortcuts. Development builds keep them for
  debugging.
- **NSIS installs per user** (no admin prompt). The MSI uses Tauri's default per-machine
  install. Both use WebView2's download bootstrapper.

### Settings

- **Stored in `%APPDATA%\<productName>\settings.json`** as the brief asks. This uses
  Tauri's roaming config folder plus the product name. Tauri's `app_config_dir()` would use
  the identifier as the folder name instead.
- **Saved in the background 500 ms after the last change**, and again on quit. Dragging the
  shelf fires many move events, and this writes the file once. Each write goes to a
  temporary file that is then renamed over the old one, so a crash cannot leave a
  half-written file.
- **Never blocks startup.** Missing fields take defaults, unknown fields are ignored, and a
  file that cannot be parsed is renamed to `settings.json.bad`. The app then starts with
  defaults.

### Core crate

- **File type comes from content, not the extension.** Binary formats need a matching
  signature, so a damaged `photo.jpg` is reported as unknown instead of failing halfway
  through an action. AVIF and HEIC are told apart by their `ftyp` brands.
- **DOCX means a zip file named `.docx`.** Checking for `word/document.xml` inside the zip
  comes with phase 5, together with the zip reader. Plain text is recognised by the
  extensions `.txt`, `.text`, `.log` and `.md`, and binary content under those names is
  rejected.
- **Output names are `<stem>_<suffix>.<ext>`.** If that name is taken, ` (2)`, ` (3)` and
  so on are added, as Explorer does. Results are written to a temporary file in the
  output folder, then moved into place with a no-overwrite rename. A failed or cancelled
  action leaves nothing behind, and no existing file can be replaced, the original
  included. Long stems are shortened so the name fits in 255 characters.
- **Long paths.** `std::fs` already uses `\\?\` paths internally for long paths.
  `paths::extended` converts paths for code that calls Win32 directly. One example is
  `tempfile`'s rename (`MoveFileExW`), whose arguments `output::write_new` converts.
  `paths::display` strips the prefix again before a path is shown to a person or passed
  to Explorer.
- **Errors carry a message for the user** and a short code for the UI. On Windows,
  sharing and lock violations (a file open in another program) become "locked", which
  gives a clearer message than "access denied".
- **`Options` carries a cancel flag and a progress callback** alongside the action's
  parameters. Long actions can then report progress and stop early without changing the
  trait.

### CI

- **GitHub Actions on `windows-latest`** runs fmt, clippy, the tests and the UI type check,
  then builds the NSIS and MSI installers and the portable exe and uploads them. The
  development machine for this phase ran Linux. It also checked the Windows target with
  `cargo clippy --target x86_64-pc-windows-msvc`, but only CI builds on Windows itself.

## Phase 2: drag and drop, radial menu

### The menu

- **One window, created at startup and reused.** It is hidden between uses, so opening
  the menu costs a move and a show rather than starting a WebView. Background throttling
  is off for it, so a hidden page reacts at once. It is created visible and hidden
  straight away, for the drop bug described under phase 1. Its page is transparent at
  that point.
- **Centred on the cursor at the moment of the drop**, in screen coordinates. The drop
  event's own position is relative to the shelf and is only a fallback. The menu is then
  moved as little as needed to stay inside the work area of that monitor. With the shelf
  in its default bottom-right spot, it would otherwise hang off the screen.
- **Two rings.** Convert wedges go on the inner ring, labelled with the format name. Tools
  go on the outer ring, as icons. When only one group applies it takes the inner ring.
  The centre shows the dropped file: a thumbnail for formats WebView2 can display, or a
  document glyph with the type otherwise. Hovering or focusing a wedge replaces that with
  what it will do ("Convert to PNG", "Crop") and how many files it will touch.
- **Closing.** Esc, a click anywhere that is not a wedge (the centre included), or the
  menu losing focus all close it. tao brings a window forward with `SetForegroundWindow`.
  When Windows refuses, which can happen right after a drag from Explorer, tao falls back
  to the Alt-key trick, so the menu does get focus and the focus-loss rule holds.
- **Keyboard.** The arrow keys move between wedges, inner ring first, and Enter or Space
  runs the focused one.
- **60 fps animation.** Each ring and the centre is a separate layer that animates only
  `transform` and `opacity`, so Chromium runs it on the compositor without repainting.
  Opening uses a 200 ms scale with a slight overshoot plus a fade, with the outer ring
  40 ms behind. Closing is a 120 ms scale-down and fade, and then the window hides. With
  "reduce motion" turned on, only the fade remains.
- **Dropping on the open menu** reopens it for the new files. The menu covers the shelf,
  so that is where a second drop lands.

### Which actions apply

- **Rules live in `crates/core/src/actions/`** with the final ids, labels and groups.
  The README has the table. A Convert target is never the file's own format. A
  multi-file drop offers the actions that apply to every file (PNG plus DOCX gives "PDF"
  and "Metadata").
- **Images do not convert to TXT or DOCX.** Text from an image would need OCR, which is
  outside the brief. **PDF does not convert to AVIF or HEIC.** Rendering pages into those
  formats is slow and an unusual request.
- **Placeholder actions** return a "not available yet" error, so the whole path from drop
  to result works now and later phases only replace `run`.

### Running actions

- **The page never sends file paths.** The menu carries a drop id, and `run_action`
  looks up the files that were actually dropped. A compromised or buggy page cannot point
  an action at other files. Paths also stay out of the serialised menu because serde
  cannot write a path that is not valid Unicode, which Windows allows. The page receives
  a display string for previews instead.
- **Thumbnails use the asset protocol with an empty scope**, and each dropped image is
  added to it individually. Nothing else on disk is readable from the WebView.
- **One background thread per job**, with the files processed in order. Each file runs
  inside `catch_unwind`, so a bug costs that file, not the app. Progress events are
  capped at about 30 a second. The shelf follows the latest job: its ring fills as the
  job progresses, it shows a short result for four seconds, and the full message is in
  its tooltip. Toasts arrive in phase 6.
- **Output goes to the settings folder, or next to each file** when none is set.

### Development and tests

- **`DROPWHEEL_DEV_DROP`** opens the menu for the listed files at startup in debug builds.
  It exists only in development builds. It lets the menu be exercised without dragging,
  on Windows, and in automated runs.
- **UI unit tests use Node's built-in test runner**, which runs TypeScript directly. That
  keeps the UI's dependencies to Vite and TypeScript. The wedge geometry is tested this
  way, including the single-wedge case that needs a full ring.
- **Checked here**, on Linux under a virtual display: the real app opened the menu for a
  dev drop of an image with an Arabic name, showed its thumbnail, and ran the clicked
  action, and the shelf reported the result. Headless Chromium screenshots covered the
  menu and the shelf states in both themes.

## Phase 3: image actions

### Libraries

- **`image` crate for reading** JPEG, PNG, WebP, BMP, TIFF, GIF and ICO. Every decode
  runs under limits: no side over 30,000 pixels and at most 1 GiB allocated. A small file
  claiming enormous dimensions fails at once with a clear message instead of exhausting
  memory (tested with a crafted "60,000 × 60,000" PNG).
- **EXIF orientation is applied on decode**, so every action works on the picture the
  way it is seen, and crop coordinates match the preview.
- **JPEG is written with `jpeg-encoder`** rather than the `image` crate's encoder. It
  supports progressive output and optimised Huffman tables, so files are smaller at the
  same quality, and it embeds colour profiles. Chroma subsampling is 4:2:0 below 90%
  quality and 4:4:4 above. The licence is MIT/Apache plus IJG, whose only condition is the
  credit line in the README.
- **Lossy WebP uses the `webp` crate** (Google's libwebp, BSD), because the `image` crate
  writes lossless WebP only. It is C code compiled with the MSVC tools Rust already
  needs. Conversions from photos (JPG, WebP, AVIF, HEIC) are lossy at 90%. From graphics
  (PNG, BMP, GIF, TIFF, ICO) they are lossless.
- **TIFF is written with the `tiff` crate**, losslessly with Deflate and a horizontal
  predictor. The `image` crate writes TIFF uncompressed.
- **AVIF is written with ravif** (through the `image` crate) at speed 6. It is pure
  Rust, so no assembler is needed to build it.
- **HEIC and AVIF are read, and HEIC written, through Windows Imaging Component**, as
  decided after phase 2. That code is unsafe COM, so it lives in its own small crate,
  `crates/wic`, and core keeps `#![forbid(unsafe_code)]`. A file is read with `std::fs`
  (long paths, clear errors) and handed to WIC from memory. HEIC writing is probed once
  by encoding a 16 × 16 image: the HEIF container is free, but the HEVC encoder inside is
  a paid extension, and its absence only shows when a frame is written. The HEIC wedge
  appears only when that probe succeeds.
- **HEIC is not rotated after decoding.** HEIF stores rotation as container properties
  that the codec applies; applying EXIF orientation as well would turn iPhone photos
  twice. This is one of the things to check on a real Windows machine.
- **AVIF reading uses WIC too**, for the same reason as HEIC. The pure-Rust decoder
  (`avif-decode` with `rav1d`) needs Rust 1.98. The `image` crate's own AVIF decoding
  pulls in `mp4parse` (MPL-2.0) and a C build of dav1d.
- **PNG palette reduction uses NeuQuant** (`color_quant`, MIT), which the `image` crate
  already depends on. libimagequant would give better palettes but is GPL. Fully
  transparent pixels get a palette entry of their own: NeuQuant treats alpha as one more
  channel and otherwise smeared transparency into visible colours.
- **Metadata is read with `kamadak-exif`.** PNG text chunks come from the `png` crate.
  JPEG comments, IPTC and XMP blocks come from `img-parts`. XMP is reported by size, not
  parsed.

### Behaviour

- **Compress, by percentage,** sets the encoder quality. For PNG, which is lossless,
  below 90% reduces the colours to a palette of 64 to 256 entries. TIFF stays lossless,
  so only a target size, which scales the picture down, makes it smaller.
- **Compress, to a target size,** plans on a copy at most 1024 px wide. It searches for
  the highest quality that fits, shrinking the picture when even 10% quality is too big.
  It then encodes at full size and steps down until the real file fits. This keeps AVIF
  usable: a full-size search would encode a large photo a dozen times.
- **The live estimate** encodes the dialog's preview (at most 1600 px) and scales the
  size by the pixel count. It tends to run a little high, and the dialog shows the real
  result afterwards. With several files it is for the first one, and says so.
- **Crop rectangles are in the pixels of the upright full image**, with the size they
  were drawn on. Other files in a batch get the rectangle scaled to their size. The crop
  box can never leave the picture, and aspect chips keep their ratio while resizing from
  any handle.
- **Edit mirrors CSS.** The dialog previews with `filter: brightness() contrast()
  grayscale()` and `transform: rotate() scale()`, and Rust applies the same formulas in
  the same order (sRGB, Rec. 709 luma for grayscale; flips before rotation). The saved
  file matches the preview.
- **Edits are saved in the original format at high quality** (92, or 85 for AVIF),
  because a lossy file's original quality is unknown. Formats this machine cannot write
  are saved as the nearest one it can: ICO as PNG, HEIC as JPEG without the encoder.
- **Remove all metadata does not re-encode JPEG, PNG or WebP.** It removes:
  - JPEG: the APP1, APP3 to APP13 and APP15 segments, comments, the MPF segment, and
    anything after the end marker.
  - PNG: the tEXt, zTXt, iTXt, eXIf and tIME chunks.
  - WebP: the EXIF and XMP chunks.

  The colour profile and the segments the picture needs (JFIF, Adobe) stay. TIFF, AVIF
  and HEIC are rewritten from their pixels.

  `img-parts` leaves the WebP header claiming an XMP chunk after it is removed, and
  decoders then reject the file. Dropwheel clears that flag itself; a test caught it.
- **Structural TIFF tags** (width, strip offsets, compression and so on) are not shown
  as metadata.
- **Output files are ordinary files.** `tempfile` creates private files on Unix, so
  results are given normal read permissions. On Windows they inherit the folder's
  permissions, as before.

### App

- **Each dialog is its own window** (`dialog-1`, `dialog-2`, …) and remembers the files
  it was opened for. Its commands find those files by window label, so a dialog cannot
  act on anything else, and several can be open at once.
- **Previews come from Rust**, not the asset protocol. TIFF, HEIC and AVIF have no
  browser decoder, and Rust's preview is already upright and sized. Each file is decoded
  once per dialog session (six recent files are cached), and the bytes travel as binary
  IPC rather than JSON.
- **Every command that decodes is async** and runs on a blocking thread, so the main
  thread never waits on a large photo.
- **Jobs can be cancelled.** Each has a cancel flag that the dialog's Cancel button sets.
  Actions check it between steps, and a cancelled file leaves no partial output.
- **Job results carry input and output sizes**, which the Compress dialog shows per
  file.
- **File names display left to right**, as in Explorer, even when they are Arabic.
  Metadata values pick their direction from their own text.

### Tests

- **Fixtures are generated by `crates/core/fixtures/make_fixtures.py`** (Pillow). Each
  is a few kilobytes. The photo has four coloured quadrants and EXIF orientation 6, so
  tests can tell which way up a result is. There is also real EXIF, GPS, XMP, a comment,
  Arabic PNG text, and broken files: truncated, garbage, empty, and a decompression
  bomb.
- **One integration test per conversion path** (36), plus a test of every action on
  real files. Every broken file is run through every action, which must fail cleanly
  and leave no temporary files. AVIF and HEIC sources need Windows codecs and are not in
  the automated tests.

## Phase 4: annotate and redact

### Drawing

- **Rust draws the annotations**, from shapes the dialog sends: boxes, arrows and text,
  in the pixels of the upright full image. The alternative, rendering an overlay in the
  browser and sending it as a picture, would tie the result to the WebView's canvas
  limits and send megabytes over IPC. Shapes are small, testable, and scale to the other
  files of a batch.
- **`tiny-skia`** (BSD-3-Clause) strokes and fills the shapes, anti-aliased. It draws
  straight into the image's own buffer. A transparent image is premultiplied first and
  restored after, so its transparency survives.
- **Text is shaped with `rustybuzz`** (MIT, a port of HarfBuzz), ordered with
  `unicode-bidi` and drawn from the fonts' outlines. Arabic letters join and lines read
  right to left. Mixed lines like "Dropwheel يعمل" come out in reading order.
- **Fonts: Noto Sans and Noto Sans Arabic, Regular and Bold, are bundled** (Arabic
  replaced by IBM Plex Sans Arabic in phase 5, see there) (1.6 MB,
  SIL Open Font License 1.1). The OFL lets fonts ship inside software of any licence, as
  long as the font files keep their licence, which sits next to them. Characters these
  fonts lack fall back to fonts that come with Windows (Segoe UI for Hebrew, Microsoft
  YaHei for Chinese, Malgun Gothic for Korean, Nirmala UI for Indian scripts, Leelawadee
  UI for Thai and others). Those are only read and never redistributed.
- **The dialog previews text in the same fonts**, loaded from the same files, with the
  same line height (1.25) and per-line direction. Each line aligns to its own start, as
  `unicode-bidi: plaintext` does. The geometry of arrows is one function written twice,
  in Rust and TypeScript, with tests on both sides pinning the same numbers. A
  side-by-side render of the dialog and of the saved file matched to the pixel row.
- **The same layout code will set the PDFs of phase 5**: it wraps lines at the places
  Unicode allows (`unicode-linebreak`, Apache-2.0) and splits words wider than a line.

### Annotate

- Tools: Select, Box, Arrow and Text, eight colours plus a picker, line thickness, text
  size and bold. Defaults scale with the picture (a 4000 px photo gets 17 px lines and
  168 px text), so they look the same on any image.
- Text is typed in place. Enter finishes, Shift+Enter starts a new line and Esc
  cancels. Double-click edits existing text. Select moves shapes, Delete removes them,
  and Ctrl+Z undoes. A slider drag counts as one step.
- Bold is on by default, since annotations sit on busy photos.

### Redact

- **Boxes are fractions of the page**, not pixels, so the same boxes fit files of any
  size and, in phase 5, pages of any size. Each box has a page number; images have
  page 0.
- **Boxes round outwards to whole pixels**, so a pixel a box only partly covers is
  blacked out too. A soft edge could leave readable traces.
- **The redacted copy is encoded from the pixels**, with no EXIF (which can hold a
  thumbnail of the unredacted picture), no XMP and no comments. Only the colour profile
  is kept. A test checks the camera and GPS fields are gone.
- PDF redaction comes with the PDF stack in phase 5. It rasterises each page, so no text
  survives under a box.

## Phase 5: PDF, DOCX and TXT

### PDF libraries

- **PDFium reads PDFs** (through `pdfium-render`, MIT/Apache): it renders pages, lists
  their characters with positions and fonts, and opens nearly anything. It is the
  engine in Chrome. The alternatives were weaker readers in pure Rust: `pdf-extract`
  for text, and `hayro`, a young renderer.
- **The PDFium binary comes from Benoit Blanchon's pdfium-binaries**, the standard
  prebuilt PDFium, taken from the `pypdfium2` wheels on PyPI (build 8076). PyPI serves
  them pinned, checksummed and from a stable URL. `scripts/fetch-pdfium.mjs` downloads
  the wheel, checks its SHA-256, and extracts the library and its licences into
  `vendor/pdfium` (not in Git). It runs as part of `npm run dev` and `npm run build`,
  so `npm run tauri dev` and `npm run tauri build` need no extra step. CI runs it too,
  and the tests require it there.
- **PDFium is embedded in the exe** (7 MB). At startup the app writes it to
  `%LOCALAPPDATA%\...\pdfium\<hash>\` if it is not already there byte for byte, and
  loads it from there. The portable exe therefore stands alone, and the installers need
  no extra files. A build without the download still works, without PDF actions.
- **All PDFium calls go through one lock.** PDFium is not thread-safe, and dialogs
  render previews while jobs run.
- **New PDFs are written with `krilla`** (MIT/Apache), which embeds subsetted fonts with
  their Unicode text, so the output is searchable and copyable. Dropwheel lays the text
  out itself (see "Text" below) and hands krilla positioned glyphs. `printpdf`, from
  the brief, is built on `lopdf` and has no text shaping, so Arabic would come out
  unjoined.
- **Existing PDFs are changed with `lopdf`** (MIT): crop boxes, picture recompression
  and metadata removal edit the file in place and keep everything else, including
  links, forms and text. Output is packed with object streams. lopdf decompresses
  streams with a size limit, so a crafted file cannot exhaust memory.
- **Password-protected PDFs are refused** for edits: lopdf would save them without the
  protection. PDFium can read some of them, but Dropwheel says plainly why it stops.

### Reading text from PDFs

- **Lines are rebuilt from character positions.** PDFium's own plain text puts the
  words of right-to-left text in different orders depending on the program that made
  the PDF (tested with LibreOffice and krilla). Dropwheel takes PDFium's word breaks,
  which are reliable, and orders words and letters itself: by where each glyph is drawn
  from, left to right, then back into reading order with the Unicode bidirectional
  algorithm. Lines are grouped by height and joined across gaps of up to three ems.
  Mixed lines such as "Hello مرحبا بالعالم world" and "مرحبا Hello big world بالعالم"
  come out right from both producers, and tests pin both.
- **Ligatures and marks stay with their letters.** Characters sharing one glyph (a
  ligature) keep their order, and PDFium's reversed lam-alef is turned back. Vowel
  marks attach to the nearest letter. Presentation forms and Latin ligatures are
  normalised to plain letters.
- **Paragraphs for DOCX** join lines that follow closely in the same size and direction
  when the line before ran to the margin; list markers start new paragraphs, and a
  hyphen at a line end joins the word. Paragraphs at least 1.6, 1.3 or 1.12 times the
  body size become headings 1 to 3, as does a short line that is all bold.
- **Pictures in a PDF** are cut from the page rendered at 150 dpi, not extracted from the
  file: that shows them as the page does (clipped, masked, recoloured), whatever their
  internal format. A page with no text at all becomes a picture of the page.

### Documents

- **One small document model** (`doc.rs`) carries content between formats: paragraphs
  with styled spans, headings, alignment, direction, list markers, tables, pictures and
  page breaks. Each format reads into it and writes from it, so each conversion is one
  reader and one writer.
- **DOCX is read with `zip` and `quick-xml`** into a small XML tree, with limits on
  nesting, element count and part size. Paragraph and character styles are resolved
  through their `basedOn` chains, numbering through `numbering.xml` (bullets from symbol
  fonts become "•"), and right-to-left paragraphs follow `w:bidi`. Text boxes,
  hyperlinks, content controls, field results and tracked insertions are read; deleted
  text, field codes and hidden text are not.
- **DOCX is written by hand** in the element order the schema requires, with heading
  styles, Calibri for Latin and Arial for Arabic. List markers are written as text with
  a hanging indent rather than Word numbering, which keeps the writer small; the
  document looks the same. Checked by opening the output with LibreOffice and
  python-docx.
- **TXT is decoded** by byte order mark, then as UTF-16 when every other byte is zero,
  then UTF-8, then by `chardetng` (MIT/Apache), which picks Windows-1256 for Arabic and
  Windows-1252 for Western text. Files over 32 MB are refused. Text output is UTF-8
  without a byte order mark, with Windows line endings.
- **Pages are A4 with one-inch margins** unless the DOCX says otherwise. Images become
  A4 PDFs, as "Print to PDF" makes them, rather than pages the size of the photo, which
  print badly.

### Text

- **IBM Plex Sans Arabic replaces Noto Sans Arabic** (both SIL OFL 1.1). Noto Sans
  Arabic draws a letter's dots as separate glyphs, and PDF readers then put spaces
  inside words when the text is copied. Plex has one glyph per letter, as do most
  Arabic fonts; it was chosen over Vazirmatn, Cairo and Tajawal for its look beside Noto
  Sans. The Annotate dialog previews with it too.
- **Text layout handles mixed styles**: a paragraph's spans keep their own size and
  weight through shaping and line breaking. Italic is drawn slanted (12°) since no italic
  face is bundled; underline and strike-through are drawn as rules.

### PDF tools

- **Compress** recompresses pictures stored as JPEG or as uncompressed or Flate RGB and
  grey: at the chosen quality, and below 90% with large pictures scaled down (to 1,120 px
  on the long side at 10%, 3,680 px at 90%). A picture is replaced only when the result is
  smaller. Thumbnails are dropped and the file is repacked. To a target size it halves
  the quality range on real output, then scales pictures down further if even 10% is too
  big, and returns the smallest it found. The dialog's estimate encodes small copies of
  the pictures instead.
- **Crop** sets every page's crop box to the same part of the page as displayed, rotated
  pages included. It hides rather than deletes, which the dialog says, pointing to
  Redact.
- **Redact** renders each page at 200 dpi, paints the boxes, and writes the pages as
  JPEG pictures at their original size. Nothing under a box survives, nor any text, so
  the dialog says the copy cannot be searched. Boxes are per page; the dialog pages
  through the PDF.
- **Metadata** lists the document information (dates in readable form; PDF strings in
  UTF-16 or PDFDocEncoding decoded), XMP and private application data, and removes all
  three. For DOCX: core, app and custom properties, the thumbnail, and the names on
  comments and tracked changes, which are blanked in place.

### App

- **PDF previews are rendered pages**, at most 1,600 px, with the page size in points as
  the dialog's units, so crop and redact boxes are in points. Previews are cached per
  page.
- **Multi-page PDFs converted to images** go into a new folder, built under a temporary
  name and renamed into place when complete, never replacing an existing one.

### Tests

- **New fixtures** (`make_documents.py`, using reportlab, pypdf, python-docx and
  LibreOffice): a two-page PDF with a picture and properties, a scan, a rotated page, an
  encrypted PDF, a truncated one, an Arabic PDF laid out by LibreOffice, a Word document
  with headings, styles, lists, a table, a picture, a page break and Arabic, and text
  files in UTF-8, UTF-16 and Windows-1256.
- **One test per document conversion path** (25 in `tests/documents.rs`), the PDF tools,
  and broken and protected files through every action.
- **Debug builds keep line tables only**, without full debug information: the test
  binaries had grown to over 300 MB each and filled the disk.

## Phase 6: settings, notifications, start with Windows, installers

### Settings window

- **Changes apply at once**, with no Save button. Each one goes to the app as a single
  change, so the window never sends back a stale copy of settings changed elsewhere,
  such as the shelf position or the shelf hidden from the tray. The window reloads them
  when it gets focus.
- **A change that cannot be made is refused** and the control springs back, with the
  reason under it. A results folder must exist and accept a test file; turning on
  Start with Windows must manage to write the registry.
- **The folder picker is Tauri's dialog plugin** (MIT/Apache-2.0), allowed for the
  Settings window only.
- **The window is 700 px tall** so everything fits, or less on a short screen, where the
  page scrolls.

### Notifications

- **Windows toast notifications through WinRT** (`windows` crate, already in the build
  through Tauri), not `tauri-plugin-notification`. The plugin cannot add buttons on
  Windows, and the brief asks for Open and Show in Explorer.
- **The app identifier is the notifications' AppUserModelID.** Tauri's NSIS installer
  already stamps it on the Start menu shortcut. For the portable exe, which has no
  shortcut, Dropwheel registers the ID's display name and icon under
  `HKCU\Software\Classes\AppUserModelId` at each start, so its notifications say
  "Dropwheel" with its icon either way.
- **Buttons use protocol activation with Dropwheel's own `dropwheel:` scheme**, registered
  per user at each start (so a moved portable copy keeps working). This works from the
  notification centre after the popup is gone, and even after Dropwheel has quit:
  Windows starts it with the URL, and a running copy receives it through the
  single-instance plugin. The other way, a COM activator, needs a registered COM
  server and much more code for the same result. Plain `file:///` links for Open were
  also ruled out: Windows' launcher refuses local `file:` URIs in some cases.
- **The scheme is deliberately weak**, since any web page can launch it.
  `dropwheel:reveal` only selects an existing file in Explorer. `dropwheel:open` opens
  a file only if Dropwheel wrote it: the last 200 results are remembered in
  `results.json` in the local app data folder. Anything else is logged and ignored.
- **Clicking the notification itself opens the result**, the same as Open.
- **With several results**, Open becomes Open folder (the folder of the first result)
  and Show in Explorer selects the first result.
- **Cancelled jobs do not notify**; failed ones do, with the error.
- **Notifications are on by default** and can be turned off in Settings.

### Start with Windows

- **A value in `HKCU\...\CurrentVersion\Run`**, named after the product, as
  `"path\to\dropwheel.exe" --autostart`. No admin rights are needed, and it is the same
  value name Tauri's NSIS uninstaller already removes. It is rewritten at each start,
  so it follows a portable exe that moved. `tauri-plugin-autostart` would do the same
  with another dependency.

### Installers

- **NSIS is the recommended installer**: per user, no admin rights, and its uninstaller
  removes everything Dropwheel registered. A hook (`src-tauri/windows/hooks.nsh`)
  removes the `dropwheel:` scheme (only when it still points at that install, not at a
  portable copy) and the notification registration. Nothing is removed on updates.
- **The MSI stays as built by Tauri.** Removing per-user registry entries from a
  per-machine MSI needs custom WiX components I could not test here. README lists
  what it leaves behind.
- **The portable exe is the release build itself.** PDFium, the fonts and the pages are
  inside it, so there is nothing else to ship.

### Licences

- **`THIRD-PARTY-NOTICES.txt`** holds every licence text and copyright notice of what is
  built into the app: PDFium and its libraries, the fonts, 364 Rust crates and the two
  JavaScript packages in the pages. `scripts/make-notices.py` writes it from
  `cargo metadata` for the Windows target, with each crate's own licence files; CI
  fails if it is out of date. It is embedded in the exe, and Settings → Third-party
  licences opens a copy.
- **Where a crate offers a choice, MIT is used**, then Apache-2.0, then the others. The
  Apache and Mozilla licences are the same text for every crate, so it appears once at
  the end, with each crate's copyright lines kept.
- **Four crates are under the Mozilla Public License 2.0** (`cssparser`, `selectors` and
  `dtoa-short`, which Tauri uses, and `option-ext`). The MPL is a file-level licence:
  shipping them unchanged in a closed or differently licensed app is allowed, as long
  as the notices say where their source is. They do (crates.io). This needs no decision:
  every Tauri app on Windows carries them.

## Open questions

- **Reading HEIC (phase 3).** Decided: Windows' codec. One option was to decode through Windows Imaging Component when the
  free "HEIF Image Extensions" from the Microsoft Store is installed. That adds nothing to
  bundle or license. The other is `libheif-rs`, which pulls in libheif and libde265. Both
  are **LGPL-3.0**, and shipping them means dynamic linking plus license notices. I
  recommend WIC, with `libheif` as an opt-in Cargo feature only if you accept the LGPL
  terms.
- **AVIF decoding (phase 3).** The `image` crate decodes AVIF through dav1d (BSD-2-Clause,
  a C library that needs extra build tools on Windows). Encoding through `ravif`/`rav1e`
  is pure Rust (BSD-2-Clause). WIC can decode AVIF if Microsoft's AV1 extension is
  installed. All are permissive, so I will pick whichever builds most reliably and note it
  here.
- **PDFium (phase 5).** Decided: bundled, embedded in the exe. Its licences (BSD and
  the permissive licences of the libraries built into it) only ask for notices, which
  are in `licenses/pdfium/` and the README credits.
- **Writing HEIC (phase 3).** Decided: shown only when the encoder is installed. HEIC
  is in the menu as a Convert target, as the brief asks. But no free, permissively licensed HEVC encoder exists. Windows can encode HEIC
  through WIC only with Microsoft's "HEVC Video Extensions", a paid Store app. The
  open-source encoder, x265, is GPL-2.0. The options: drop HEIC as a target, offer it
  only when the paid extension is installed (detected at runtime), or accept x265 under
  the GPL. I recommend the second.

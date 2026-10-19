# Decisions

Choices made while building Dropwheel, with the reason for each. Newest phase last.

## Phase 1: scaffold, tray and shelf

### Project

- **Lives in `dropwheel/` inside the portfolio repository.** The work was started in that
  repo. The folder is self-contained (its own Cargo workspace, npm package, `.gitignore`
  and CI workflow), so `git subtree split --prefix dropwheel` can move it to its own repo
  later. The portfolio's `tsconfig.json` and `eslint.config.mjs` exclude it so the Next.js
  build ignores it.
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
- **Rust edition 2024, minimum Rust 1.90**, the same minimum Tauri needs.
- **The `ui/` folder has its own `package.json`.** Its `tauri` script runs the CLI from the
  workspace root (`cd .. && tauri`). The Tauri config starts Vite with `cwd: "../ui"`, so
  `cargo tauri dev` from `dropwheel/` and `npm run tauri dev` from `ui/` both work.
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

## Open questions

- **Reading HEIC (phase 3).** One option is to decode through Windows Imaging Component when the
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
- **PDFium (phase 5).** `pdfium-render` needs `pdfium.dll` shipped with the app. The
  prebuilt binaries are BSD-3-Clause/Apache-2.0, which is permissive and only requires
  license notices.
- **Writing HEIC (phase 3).** HEIC is in the menu as a Convert target, as the brief
  asks. But no free, permissively licensed HEVC encoder exists. Windows can encode HEIC
  through WIC only with Microsoft's "HEVC Video Extensions", a paid Store app. The
  open-source encoder, x265, is GPL-2.0. The options: drop HEIC as a target, offer it
  only when the paid extension is installed (detected at runtime), or accept x265 under
  the GPL. I recommend the second.

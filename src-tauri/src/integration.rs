//! Windows integration: notifications when jobs finish, starting with Windows, and
//! showing results in Explorer.
//!
//! Notification buttons use protocol activation, so they work from the notification
//! centre too, long after the notification popped up. They launch `dropwheel:` URLs,
//! a scheme registered for the current user: Windows starts Dropwheel with the URL, and
//! the single-instance plugin passes it to the running copy.
//!
//! Any web page can launch such a URL too, so the scheme does very little. `reveal`
//! only selects a file in Explorer. `open` opens a file only if Dropwheel wrote it:
//! results are remembered (the last [`REMEMBERED`]) in `results.json` next to the log.
//!
//! Elsewhere these are no-ops, so the app still builds and runs for development.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tauri::{AppHandle, Manager, Runtime};

/// The URL scheme of notification buttons.
pub const SCHEME: &str = "dropwheel";
/// How many results `dropwheel:open` can still open.
const REMEMBERED: usize = 200;
/// Serialises updates of the remembered results.
static RESULTS: Mutex<()> = Mutex::new(());

fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = text.get(i + 1..i + 3)
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            out.push(b);
            i += 3;
            continue;
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// What a `dropwheel:` URL asks for.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Request {
    /// Open a result in its usual app.
    Open,
    /// Show a file or folder selected in Explorer.
    Reveal,
}

impl Request {
    fn verb(self) -> &'static str {
        match self {
            Request::Open => "open",
            Request::Reveal => "reveal",
        }
    }
}

/// The URL that asks Dropwheel to do `request` with `path`.
pub fn url(request: Request, path: &Path) -> String {
    format!(
        "{SCHEME}:{}?path={}",
        request.verb(),
        percent_encode(&path.to_string_lossy())
    )
}

/// What a `dropwheel:` URL asks for, and the absolute path it names.
pub fn parse(arg: &str) -> Option<(Request, PathBuf)> {
    let rest = arg.strip_prefix(SCHEME)?.strip_prefix(':')?;
    // Some launchers add slashes after the scheme, or one at the end.
    let rest = rest.trim_start_matches('/');
    let (verb, query) = rest.split_once('?')?;
    let request = [Request::Open, Request::Reveal]
        .into_iter()
        .find(|r| r.verb() == verb)?;
    let value = query.strip_prefix("path=")?.trim_end_matches('/');
    let path = PathBuf::from(percent_decode(value));
    path.is_absolute().then_some((request, path))
}

fn results_file<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path()
        .app_local_data_dir()
        .ok()
        .map(|d| d.join("results.json"))
}

fn remembered<R: Runtime>(app: &AppHandle<R>) -> Vec<String> {
    results_file(app)
        .and_then(|f| std::fs::read(f).ok())
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Remembers `paths` as Dropwheel's own results, which `dropwheel:open` may open.
pub fn remember<R: Runtime>(app: &AppHandle<R>, paths: &[PathBuf]) {
    let Some(file) = results_file(app) else {
        return;
    };
    let _guard = RESULTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut list = remembered(app);
    for path in paths {
        let path = path.to_string_lossy().into_owned();
        list.retain(|p| *p != path);
        list.push(path);
    }
    let excess = list.len().saturating_sub(REMEMBERED);
    list.drain(..excess);
    let saved = (|| -> anyhow::Result<()> {
        let dir = file.parent().ok_or_else(|| anyhow::anyhow!("no folder"))?;
        std::fs::create_dir_all(dir)?;
        let mut temp = tempfile::NamedTempFile::new_in(dir)?;
        serde_json::to_writer(&mut temp, &list)?;
        temp.persist(&file)?;
        Ok(())
    })();
    if let Err(err) = saved {
        log::warn!("could not remember results: {err:#}");
    }
}

/// Handles command-line arguments that are `dropwheel:` URLs, from a launch by
/// Windows or forwarded from a second instance.
pub fn handle_args<R: Runtime>(app: &AppHandle<R>, args: &[String]) {
    for arg in args {
        let Some((request, path)) = parse(arg) else {
            continue;
        };
        if !path.exists() {
            log::warn!("{} is gone", path.display());
            continue;
        }
        match request {
            Request::Reveal => reveal(&path),
            Request::Open => {
                let ours = remembered(app).iter().any(|p| Path::new(p) == path);
                if ours {
                    open(&path);
                } else {
                    log::warn!("not opening {}: Dropwheel did not make it", path.display());
                }
            }
        }
    }
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// What a notification says and offers.
pub struct Notice {
    pub title: String,
    pub body: String,
    /// The file or folder its buttons act on.
    pub target: Option<PathBuf>,
    /// Several results: "Open" opens the folder rather than one file.
    pub several: bool,
}

impl Notice {
    /// What "Open" opens.
    fn opens(&self) -> Option<&Path> {
        let target = self.target.as_deref()?;
        Some(if self.several {
            target.parent().unwrap_or(target)
        } else {
            target
        })
    }
}

/// The notification's XML, in the Windows toast schema.
pub fn toast_xml(notice: &Notice) -> String {
    let mut launch = String::new();
    let mut actions = String::new();
    if let (Some(target), Some(opens)) = (&notice.target, notice.opens()) {
        let open = xml_escape(&url(Request::Open, opens));
        let reveal = xml_escape(&url(Request::Reveal, target));
        launch = format!(r#" launch="{open}" activationType="protocol""#);
        let open_label = if notice.several {
            "Open folder"
        } else {
            "Open"
        };
        actions = format!(
            concat!(
                "<actions>",
                r#"<action content="{open_label}" activationType="protocol" arguments="{open}"/>"#,
                r#"<action content="Show in Explorer" activationType="protocol" arguments="{reveal}"/>"#,
                "</actions>"
            ),
            open_label = open_label,
            open = open,
            reveal = reveal
        );
    }
    format!(
        concat!(
            "<toast{launch}><visual><binding template=\"ToastGeneric\">",
            "<text>{title}</text><text>{body}</text>",
            "</binding></visual>{actions}</toast>"
        ),
        launch = launch,
        title = xml_escape(&notice.title),
        body = xml_escape(&notice.body),
        actions = actions
    )
}

#[cfg(windows)]
mod platform {
    use std::path::Path;

    use windows::Data::Xml::Dom::XmlDocument;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
    use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, WIN32_ERROR};
    use windows::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ, RegCloseKey,
        RegCreateKeyExW, RegDeleteKeyValueW, RegSetValueExW,
    };
    use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
    use windows::core::{HSTRING, PCWSTR};

    pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    fn check(code: WIN32_ERROR, what: &str) -> anyhow::Result<()> {
        if code.is_ok() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("{what}: {:?}", code.to_hresult().message()))
        }
    }

    /// Sets a string value under HKEY_CURRENT_USER, creating the key if needed. `name`
    /// `None` sets the key's default value.
    pub fn set_value(subkey: &str, name: Option<&str>, value: &str) -> anyhow::Result<()> {
        let mut key = HKEY::default();
        // SAFETY: plain Win32 calls with valid, NUL-terminated strings; the key is closed
        // below whatever happens.
        unsafe {
            check(
                RegCreateKeyExW(
                    HKEY_CURRENT_USER,
                    &HSTRING::from(subkey),
                    None,
                    PCWSTR::null(),
                    REG_OPTION_NON_VOLATILE,
                    KEY_WRITE,
                    None,
                    &mut key,
                    None,
                ),
                subkey,
            )?;
            let data: Vec<u8> = value
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes)
                .collect();
            let name = name.map(HSTRING::from);
            let result = match &name {
                Some(n) => RegSetValueExW(key, n, None, REG_SZ, Some(&data)),
                None => RegSetValueExW(key, PCWSTR::null(), None, REG_SZ, Some(&data)),
            };
            let _ = RegCloseKey(key);
            check(result, subkey)
        }
    }

    pub fn delete_value(subkey: &str, name: &str) -> anyhow::Result<()> {
        // SAFETY: as above.
        let code = unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                &HSTRING::from(subkey),
                &HSTRING::from(name),
            )
        };
        if code == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        check(code, subkey)
    }

    pub fn show_toast(aumid: &str, xml: &str) -> anyhow::Result<()> {
        // SAFETY: initialises WinRT on this thread, which the caller owns.
        unsafe {
            let _ = RoInitialize(RO_INIT_MULTITHREADED);
        }
        let doc = XmlDocument::new()?;
        doc.LoadXml(&HSTRING::from(xml))?;
        let toast = ToastNotification::CreateToastNotification(&doc)?;
        ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(aumid))?.Show(&toast)?;
        Ok(())
    }

    pub fn reveal(path: &Path) {
        use std::os::windows::process::CommandExt;
        // Explorer wants `/select,"path"` as one argument, quoted just so.
        let arg = format!("/select,\"{}\"", path.display());
        if let Err(err) = std::process::Command::new("explorer.exe")
            .raw_arg(arg)
            .spawn()
        {
            log::warn!("could not open Explorer: {err}");
        }
    }
}

/// Registers what notifications and the `dropwheel:` scheme need, for the current user:
/// the app's name and icon for its notifications, and the scheme pointing at this exe.
/// Repeated at each start, so a portable copy that moved keeps working.
pub fn register<R: Runtime>(app: &AppHandle<R>) {
    #[cfg(windows)]
    {
        if let Err(err) = register_windows(app) {
            log::warn!("could not register with Windows: {err:#}");
        }
    }
    #[cfg(not(windows))]
    let _ = app;
}

#[cfg(windows)]
fn register_windows<R: Runtime>(app: &AppHandle<R>) -> anyhow::Result<()> {
    let exe = std::env::current_exe()?;
    let exe = dropwheel_core::paths::strip_verbatim(&exe);
    let name = &app.package_info().name;
    let aumid = &app.config().identifier;

    let icon = app
        .path()
        .app_local_data_dir()?
        .join("notification-icon.png");
    if !icon.is_file() {
        if let Some(dir) = icon.parent() {
            std::fs::create_dir_all(dir)?;
        }
        std::fs::write(&icon, include_bytes!("../icons/128x128.png"))?;
    }
    let key = format!(r"Software\Classes\AppUserModelId\{aumid}");
    platform::set_value(&key, Some("DisplayName"), name)?;
    platform::set_value(&key, Some("IconUri"), &icon.to_string_lossy())?;

    let scheme = format!(r"Software\Classes\{SCHEME}");
    platform::set_value(&scheme, None, &format!("URL:{name}"))?;
    platform::set_value(&scheme, Some("URL Protocol"), "")?;
    platform::set_value(
        &format!(r"{scheme}\shell\open\command"),
        None,
        &format!("\"{}\" \"%1\"", exe.display()),
    )?;
    Ok(())
}

/// Turns starting with Windows on or off for the current user.
pub fn set_start_with_windows<R: Runtime>(app: &AppHandle<R>, on: bool) -> anyhow::Result<()> {
    #[cfg(windows)]
    {
        let name = &app.package_info().name;
        if on {
            let exe = std::env::current_exe()?;
            let exe = dropwheel_core::paths::strip_verbatim(&exe);
            platform::set_value(
                platform::RUN_KEY,
                Some(name),
                &format!("\"{}\" --autostart", exe.display()),
            )
        } else {
            platform::delete_value(platform::RUN_KEY, name)
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (app, on);
        log::info!("starting with the system is only supported on Windows");
        Ok(())
    }
}

/// Shows a notification, off the calling thread.
pub fn notify<R: Runtime>(app: &AppHandle<R>, notice: Notice) {
    if let Some(opens) = notice.opens() {
        remember(app, &[opens.to_path_buf()]);
    }
    let xml = toast_xml(&notice);
    #[cfg(windows)]
    {
        let aumid = app.config().identifier.clone();
        let spawned = std::thread::Builder::new()
            .name("notification".into())
            .spawn(move || {
                if let Err(err) = platform::show_toast(&aumid, &xml) {
                    log::warn!("could not show a notification: {err:#}");
                }
            });
        if let Err(err) = spawned {
            log::warn!("could not show a notification: {err}");
        }
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        log::info!("notification: {xml}");
    }
}

/// Opens `path` in the app Windows associates with it.
pub fn open(path: &Path) {
    #[cfg(windows)]
    let spawned = {
        use std::os::windows::process::CommandExt;
        std::process::Command::new("explorer.exe")
            .raw_arg(format!("\"{}\"", path.display()))
            .spawn()
    };
    #[cfg(not(windows))]
    let spawned = std::process::Command::new("xdg-open").arg(path).spawn();
    if let Err(err) = spawned {
        log::warn!("could not open {}: {err}", path.display());
    }
}

/// Shows `path` selected in Explorer.
pub fn reveal(path: &Path) {
    if !path.exists() {
        log::warn!("not revealing {}: it does not exist", path.display());
        return;
    }
    #[cfg(windows)]
    platform::reveal(path);
    #[cfg(not(windows))]
    log::info!("would reveal {}", path.display());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_round_trip() {
        let path = if cfg!(windows) {
            PathBuf::from(r"C:\Users\me\تقرير & co.pdf")
        } else {
            PathBuf::from("/home/me/تقرير & co.pdf")
        };
        for request in [Request::Open, Request::Reveal] {
            let url = url(request, &path);
            assert!(url.starts_with("dropwheel:"));
            assert!(!url.contains(' ') && !url.contains('&') && !url.contains('"'));
            assert_eq!(parse(&url), Some((request, path.clone())));
            // As some launchers pass it back.
            assert_eq!(parse(&format!("{url}/")), Some((request, path.clone())));
        }
        assert_eq!(
            parse("dropwheel://reveal?path=%2Fx").map(|(r, _)| r),
            cfg!(not(windows)).then_some(Request::Reveal)
        );
    }

    #[test]
    fn other_urls_are_ignored() {
        assert!(parse("dropwheel:reveal?path=relative").is_none());
        assert!(parse("https://example.com").is_none());
        assert!(parse("dropwheel:run?path=%2Fetc").is_none());
        assert!(parse("dropwheel:open").is_none());
        assert!(parse("--autostart").is_none());
    }

    #[test]
    fn toast_xml_is_escaped() {
        let xml = toast_xml(&Notice {
            title: "Converted <1> file".into(),
            body: "a & b.pdf".into(),
            target: Some(PathBuf::from("/tmp/a & b.pdf")),
            several: false,
        });
        assert!(xml.contains("Converted &lt;1&gt; file"));
        assert!(xml.contains("a &amp; b.pdf"));
        assert!(xml.contains(r#"content="Open""#));
        assert!(xml.contains(r#"content="Show in Explorer""#));
        assert!(xml.contains("dropwheel:open?path=%2Ftmp%2Fa%20%26%20b.pdf"));
        assert!(!xml.contains("a & b"));
        let plain = toast_xml(&Notice {
            title: "Failed".into(),
            body: "x".into(),
            target: None,
            several: false,
        });
        assert!(!plain.contains("<actions>"));
    }

    #[test]
    fn several_results_open_their_folder() {
        let notice = Notice {
            title: "Converted 2 files".into(),
            body: "x".into(),
            target: Some(PathBuf::from("/tmp/out/a.png")),
            several: true,
        };
        assert_eq!(notice.opens(), Some(Path::new("/tmp/out")));
        assert!(toast_xml(&notice).contains(r#"content="Open folder""#));
    }
}

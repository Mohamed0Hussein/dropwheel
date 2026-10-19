import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";

import { byId } from "./lib/dom";
import { openNotices, settingsInfo, updateSettings, type SettingsChange, type SettingsInfo } from "./lib/ipc";
import { lockDownWebview } from "./lib/webview";

lockDownWebview();

const nextRadio = byId<HTMLInputElement>("output-next");
const folderRadio = byId<HTMLInputElement>("output-folder");
const outputPath = byId("output-path");
const chooseButton = byId<HTMLButtonElement>("choose-folder");
const startSwitch = byId<HTMLInputElement>("start-with-windows");
const notifySwitch = byId<HTMLInputElement>("notifications");
const shelfSwitch = byId<HTMLInputElement>("shelf-visible");
const status = byId("status");

/** The folder last chosen, kept while "next to the original" is selected. */
let folder: string | null = null;

function show(info: SettingsInfo): void {
  const s = info.settings;
  folder = s.outputDir ?? folder;
  nextRadio.checked = s.outputDir === null;
  folderRadio.checked = s.outputDir !== null;
  outputPath.textContent = folder ?? "No folder chosen";
  startSwitch.checked = s.startWithWindows;
  notifySwitch.checked = s.notifications;
  shelfSwitch.checked = s.shelf.visible;
  byId("version").textContent = `Dropwheel ${info.version}`;
  byId("settings-path").textContent = info.path;
  byId("pdf-note").hidden = info.pdf;
}

async function apply(change: SettingsChange): Promise<void> {
  status.textContent = "";
  try {
    show(await updateSettings(change));
  } catch (err) {
    status.textContent = String(err);
    show(await settingsInfo());
  }
}

async function chooseFolder(): Promise<void> {
  const picked = await open({ directory: true, multiple: false, defaultPath: folder ?? undefined, title: "Save results in" });
  if (typeof picked === "string") {
    folder = picked;
    await apply({ output: { mode: "folder", path: picked } });
  } else if (folder === null) {
    // Nothing chosen and nothing before: stay next to the original.
    nextRadio.checked = true;
  }
}

nextRadio.addEventListener("change", () => void apply({ output: { mode: "nextToOriginal" } }));
folderRadio.addEventListener("change", () => {
  if (folder) void apply({ output: { mode: "folder", path: folder } });
  else void chooseFolder();
});
chooseButton.addEventListener("click", () => void chooseFolder());
startSwitch.addEventListener("change", () => void apply({ startWithWindows: startSwitch.checked }));
notifySwitch.addEventListener("change", () => void apply({ notifications: notifySwitch.checked }));
shelfSwitch.addEventListener("change", () => void apply({ shelfVisible: shelfSwitch.checked }));
byId("notices").addEventListener("click", () => void openNotices().catch((err: unknown) => (status.textContent = String(err))));
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") void getCurrentWindow().close();
});
// The shelf can be shown or hidden from the tray while this window is open.
window.addEventListener("focus", () => void settingsInfo().then(show));

settingsInfo()
  .then(show)
  .catch((err: unknown) => {
    status.textContent = `Could not load settings: ${String(err)}`;
  });

import { byId } from "./lib/dom";
import { settingsInfo } from "./lib/ipc";
import { lockDownWebview } from "./lib/webview";

lockDownWebview();

async function render(): Promise<void> {
  const { path, settings } = await settingsInfo();
  byId("output-dir").textContent = settings.outputDir ?? "Next to the original file";
  byId("start-with-windows").textContent = settings.startWithWindows ? "On" : "Off";
  byId("settings-path").textContent = path;
}

render().catch((err: unknown) => {
  console.error("could not load settings", err);
  byId("settings-path").textContent = "(unavailable)";
});

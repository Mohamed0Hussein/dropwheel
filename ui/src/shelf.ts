import { getCurrentWindow } from "@tauri-apps/api/window";

import { byId } from "./lib/dom";
import { lockDownWebview } from "./lib/webview";

lockDownWebview();

const shelf = byId("shelf");
const appWindow = getCurrentWindow();

// Move the shelf by dragging any part of it. This is done by hand rather than with
// data-tauri-drag-region so that a double-click never tries to maximise the window.
shelf.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  e.preventDefault();
  appWindow.startDragging().catch((err: unknown) => console.error("startDragging failed", err));
});

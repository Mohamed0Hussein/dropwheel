import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { byId } from "./lib/dom";
import { EVENTS, type JobFinished, type JobProgress, type JobStarted } from "./lib/ipc";
import { lockDownWebview } from "./lib/webview";

lockDownWebview();

const IDLE_TEXT = "Drop files here";
/** How long a result stays on the shelf before it goes back to idle. */
const RESULT_MS = 4000;

const shelf = byId("shelf");
const label = byId("shelf-label");
const appWindow = getCurrentWindow();

// Move the shelf by dragging any part of it. This is done by hand rather than with
// data-tauri-drag-region so that a double-click never tries to maximise the window.
shelf.addEventListener("mousedown", (e) => {
  if (e.button !== 0) return;
  e.preventDefault();
  appWindow.startDragging().catch((err: unknown) => console.error("startDragging failed", err));
});

// Files from Explorer. The drop itself is handled in Rust, which opens the radial menu.
await getCurrentWebview().onDragDropEvent(({ payload }) => {
  const over = payload.type === "enter" || payload.type === "over";
  shelf.classList.toggle("is-over", over);
  if (!shelf.classList.contains("is-busy")) setText(over ? "Release to open" : IDLE_TEXT);
});

/** The shelf follows the most recently started job. */
let currentJob: number | null = null;
let resetTimer: number | undefined;

function setText(text: string, detail = ""): void {
  label.textContent = text;
  shelf.title = detail;
}

function setProgress(fraction: number | null): void {
  shelf.classList.toggle("is-busy", fraction !== null);
  shelf.style.setProperty("--progress", String(Math.round((fraction ?? 0) * 100)));
}

await listen<JobStarted>(EVENTS.jobStarted, ({ payload }) => {
  currentJob = payload.jobId;
  window.clearTimeout(resetTimer);
  shelf.classList.remove("is-error");
  setProgress(0);
  setText("Working…");
});

await listen<JobProgress>(EVENTS.jobProgress, ({ payload }) => {
  if (payload.jobId !== currentJob) return;
  setProgress(payload.fraction);
  if (payload.total > 1) setText(`${payload.done} of ${payload.total}`);
});

await listen<JobFinished>(EVENTS.jobFinished, ({ payload }) => {
  if (payload.jobId !== currentJob) return;
  setProgress(null);
  shelf.classList.toggle("is-error", payload.outputs.length === 0 && payload.failures.length > 0);
  setText(payload.summary, payload.detail);
  resetTimer = window.setTimeout(() => {
    shelf.classList.remove("is-error");
    setText(IDLE_TEXT);
  }, RESULT_MS);
});

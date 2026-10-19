import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { byId } from "../lib/dom";
import { fileName } from "../lib/format";
import {
  EVENTS,
  cancelJob,
  dialogInfo,
  dialogRun,
  imageInfo,
  imagePreview,
  type ImageInfo,
  type JobFinished,
  type JobProgress,
} from "../lib/ipc";
import { lockDownWebview } from "../lib/webview";
import { backgroundPanel } from "./background";
import { compressPanel } from "./compress";
import { cropPanel } from "./crop";
import { editPanel } from "./edit";
import { metadataPanel } from "./metadata";
import type { Panel, PanelFactory } from "./types";

lockDownWebview();

const PANELS: Record<string, PanelFactory> = {
  compress: compressPanel,
  crop: cropPanel,
  edit: editPanel,
  metadata: metadataPanel,
  add_background: backgroundPanel,
};

const title = byId("title");
const subtitle = byId("subtitle");
const stage = byId("stage");
const panelHost = byId("panel");
const status = byId("status");
const progress = byId("progress");
const progressBar = byId("progress-bar");
const applyButton = byId<HTMLButtonElement>("apply");
const closeButton = byId<HTMLButtonElement>("close");
const appWindow = getCurrentWindow();

let panel: Panel | null = null;
let runningJob: number | null = null;

function setStatus(text: string, tone: "info" | "ok" | "error" = "info"): void {
  // File names read left to right overall, as in Explorer, even when they are Arabic.
  status.dir = "ltr";
  status.textContent = text;
  status.dataset.tone = tone;
}

function refresh(): void {
  applyButton.disabled = runningJob !== null || !panel || panel.params() === null;
  closeButton.textContent = runningJob !== null ? "Cancel" : "Close";
}

function showError(message: string): void {
  stage.replaceChildren();
  const box = document.createElement("div");
  box.className = "stage__message";
  box.textContent = message;
  stage.append(box);
}

async function loadPreview(): Promise<{ image: ImageInfo; url: string } | null> {
  try {
    const image = await imageInfo(0);
    const bytes = await imagePreview(0);
    const url = URL.createObjectURL(new Blob([bytes], { type: image.previewMime }));
    return { image, url };
  } catch (err) {
    showError(String(err));
    return null;
  }
}

async function apply(): Promise<void> {
  const params = panel?.params();
  if (!panel || !params) return;
  setStatus("Working…");
  progress.hidden = false;
  progressBar.style.width = "0%";
  try {
    runningJob = await dialogRun(params);
  } catch (err) {
    runningJob = null;
    progress.hidden = true;
    setStatus(String(err), "error");
  }
  refresh();
}

function onFinished(result: JobFinished): void {
  if (result.jobId !== runningJob) return;
  runningJob = null;
  progress.hidden = true;
  const failed = result.failures.length > 0 && result.outputs.length === 0;
  const [only] = result.outputs;
  const text =
    result.outputs.length === 1 && only && result.failures.length === 0
      ? `Saved ${fileName(only.path)}`
      : failed && result.failures[0]
        ? result.failures[0].message
        : result.summary;
  setStatus(text, failed ? "error" : "ok");
  status.title = result.detail;
  panel?.finished?.(result);
  refresh();
}

async function start(): Promise<void> {
  const info = await dialogInfo();
  const factory = PANELS[info.action.id];
  title.textContent = info.title;
  const [first, ...rest] = info.files;
  subtitle.textContent = first ? (rest.length ? `${first.name} and ${rest.length} more` : first.name) : "";
  document.title = info.title;
  if (!factory) {
    showError(`${info.title} has no settings.`);
    return;
  }

  const loaded = await loadPreview();
  panel = await factory({
    info,
    image: loaded?.image ?? null,
    previewUrl: loaded?.url ?? null,
    stage,
    panel: panelHost,
    changed: refresh,
  });
  applyButton.textContent = panel.applyLabel;
  if (rest.length) setStatus(`Applies to all ${info.files.length} files.`);
  refresh();
}

applyButton.addEventListener("click", () => void apply());
closeButton.addEventListener("click", () => {
  if (runningJob !== null) {
    void cancelJob(runningJob);
    setStatus("Cancelling…");
  } else {
    void appWindow.close();
  }
});
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && runningJob === null) void appWindow.close();
});

await listen<JobProgress>(EVENTS.jobProgress, ({ payload }) => {
  if (payload.jobId !== runningJob) return;
  progressBar.style.width = `${Math.round(payload.fraction * 100)}%`;
  if (payload.total > 1) setStatus(`Working… ${payload.done} of ${payload.total}`);
});
await listen<JobFinished>(EVENTS.jobFinished, ({ payload }) => onFinished(payload));

start().catch((err: unknown) => showError(String(err)));

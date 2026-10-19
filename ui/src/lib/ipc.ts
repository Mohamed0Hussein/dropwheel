// Typed wrappers for the commands in src-tauri/src/commands.rs and the events the app
// emits. Field names follow the Rust structs (serde camelCase).
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
  shelf: {
    visible: boolean;
    position: { x: number; y: number } | null;
  };
  /** `null` writes results next to each source file. */
  outputDir: string | null;
  startWithWindows: boolean;
  notifications: boolean;
}

export interface SettingsInfo {
  path: string;
  settings: Settings;
  version: string;
  /** Whether this build can read PDFs. */
  pdf: boolean;
}

/** Mirrors `SettingsChange` in src-tauri/src/commands.rs; leave out what stays. */
export interface SettingsChange {
  output?: { mode: "nextToOriginal" } | { mode: "folder"; path: string };
  startWithWindows?: boolean;
  notifications?: boolean;
  shelfVisible?: boolean;
}

export type FileKind =
  | "jpeg"
  | "png"
  | "webp"
  | "bmp"
  | "tiff"
  | "gif"
  | "ico"
  | "avif"
  | "heic"
  | "pdf"
  | "docx"
  | "txt"
  | "unknown";

export interface DroppedFile {
  name: string;
  kind: FileKind;
  kindLabel: string;
  /** Set when the WebView can show the file itself. */
  previewPath: string | null;
}

export interface MenuAction {
  id: string;
  label: string;
  group: "convert" | "tools";
  /** Opens a settings dialog instead of running straight away. */
  dialog: boolean;
}

export interface Menu {
  dropId: number;
  files: DroppedFile[];
  actions: MenuAction[];
}

export interface JobStarted {
  jobId: number;
  action: string;
  total: number;
}

export interface JobProgress {
  jobId: number;
  done: number;
  total: number;
  /** Overall, from 0 to 1. */
  fraction: number;
}

export interface JobOutput {
  path: string;
  bytes: number;
  inputBytes: number;
}

export interface JobFinished {
  jobId: number;
  outputs: JobOutput[];
  failures: { file: string; code: string; message: string }[];
  cancelled: boolean;
  summary: string;
  detail: string;
}

export type Compress = { mode: "quality"; percent: number } | { mode: "size"; bytes: number };

export interface Edit {
  rotate: 0 | 90 | 180 | 270;
  flipHorizontal: boolean;
  flipVertical: boolean;
  grayscale: boolean;
  brightness: number;
  contrast: number;
}

export interface Crop {
  x: number;
  y: number;
  width: number;
  height: number;
  refWidth: number;
  refHeight: number;
}

export type Rgb = [number, number, number];

/** Mirrors `Shape` in crates/core/src/action.rs. Lengths are in image pixels. */
export type Shape =
  | { shape: "rect"; x: number; y: number; width: number; height: number; color: Rgb; stroke: number }
  | { shape: "arrow"; from: [number, number]; to: [number, number]; color: Rgb; stroke: number }
  | { shape: "text"; x: number; y: number; text: string; color: Rgb; size: number; bold: boolean };

/** A rectangle as fractions of the upright page or image. */
export interface Region {
  page: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

/** Mirrors `Params` in crates/core/src/action.rs. */
export type Params =
  | { type: "none" }
  | ({ type: "compress" } & Compress)
  | ({ type: "crop" } & Crop)
  | ({ type: "edit" } & Edit)
  | { type: "annotate"; shapes: Shape[]; refWidth: number; refHeight: number }
  | { type: "redact"; regions: Region[] }
  | { type: "background"; color: Rgb };

export interface DialogInfo {
  action: MenuAction;
  title: string;
  files: DroppedFile[];
}

export interface ImageInfo {
  kind: FileKind;
  kindLabel: string;
  /** The full image, upright; for a PDF, the page in points. */
  width: number;
  height: number;
  /** Pages in the file; 1 for images. */
  pages: number;
  unit: "px" | "pt";
  bytes: number;
  previewWidth: number;
  previewHeight: number;
  previewMime: string;
}

export interface Estimate {
  original: number;
  estimate: { bytes: number; percent: number; scale: number };
}

export interface MetadataField {
  group: string;
  name: string;
  value: string;
}

export const EVENTS = {
  radialOpen: "radial:open",
  jobStarted: "job:started",
  jobProgress: "job:progress",
  jobFinished: "job:finished",
} as const;

export const settingsInfo = () => invoke<SettingsInfo>("settings_info");
export const updateSettings = (change: SettingsChange) => invoke<SettingsInfo>("update_settings", { change });
export const openNotices = () => invoke<void>("open_notices");
export const radialMenu = () => invoke<Menu | null>("radial_menu");
export const radialHide = () => invoke<void>("radial_hide");
export const runAction = (dropId: number, actionId: string) =>
  invoke<number>("run_action", { dropId, actionId });
export const openDialog = (dropId: number, actionId: string) =>
  invoke<void>("open_dialog", { dropId, actionId });

export const dialogInfo = () => invoke<DialogInfo>("dialog_info");
export const imageInfo = (index: number, page = 0) => invoke<ImageInfo>("image_info", { index, page });
export const imagePreview = (index: number, page = 0) => invoke<ArrayBuffer>("image_preview", { index, page });
export const estimateCompress = (target: Compress) => invoke<Estimate>("estimate_compress", { target });
export const readMetadata = (index: number) => invoke<MetadataField[]>("read_metadata", { index });
export const dialogRun = (params: Params) => invoke<number>("dialog_run", { params });
export const cancelJob = (jobId: number) => invoke<boolean>("cancel_job", { jobId });

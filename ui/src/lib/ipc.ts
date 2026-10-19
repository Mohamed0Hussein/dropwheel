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
}

export interface SettingsInfo {
  path: string;
  settings: Settings;
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

export interface JobFinished {
  jobId: number;
  outputs: string[];
  failures: { file: string; code: string; message: string }[];
  summary: string;
  detail: string;
}

export const EVENTS = {
  radialOpen: "radial:open",
  jobStarted: "job:started",
  jobProgress: "job:progress",
  jobFinished: "job:finished",
} as const;

export const settingsInfo = () => invoke<SettingsInfo>("settings_info");
export const radialMenu = () => invoke<Menu | null>("radial_menu");
export const radialHide = () => invoke<void>("radial_hide");
export const runAction = (dropId: number, actionId: string) =>
  invoke<number>("run_action", { dropId, actionId });

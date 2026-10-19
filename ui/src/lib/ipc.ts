// Typed wrappers for the commands in src-tauri/src/commands.rs.
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

export const settingsInfo = () => invoke<SettingsInfo>("settings_info");

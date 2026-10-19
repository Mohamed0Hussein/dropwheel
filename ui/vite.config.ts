import { fileURLToPath } from "node:url";
import { defineConfig, searchForWorkspaceRoot } from "vite";

const page = (name: string) => fileURLToPath(new URL(`${name}.html`, import.meta.url));

// Tauri loads the dev server from a fixed port (see src-tauri/tauri.conf.json).
export default defineConfig({
  clearScreen: false,
  envPrefix: ["VITE_", "TAURI_ENV_"],
  server: {
    port: 1420,
    strictPort: true,
    // The Annotate dialog previews text in the fonts the Rust side draws with.
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd()), fileURLToPath(new URL("../crates/core/fonts", import.meta.url))],
    },
  },
  build: {
    // WebView2 is evergreen Chromium.
    target: "chrome120",
    sourcemap: Boolean(process.env.TAURI_ENV_DEBUG),
    rolldownOptions: {
      input: {
        shelf: page("shelf"),
        radial: page("radial"),
        dialog: page("dialog"),
        settings: page("settings"),
      },
    },
  },
});

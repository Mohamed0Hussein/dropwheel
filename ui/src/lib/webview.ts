const BLOCKED_KEYS = new Set(["F3", "F5", "F7"]);
const BLOCKED_WITH_CTRL = new Set(["f", "g", "j", "p", "r", "s", "u"]);

/**
 * Makes the WebView behave like a native window in release builds: no browser context
 * menu, and no reload, print, find or view-source shortcuts. Development builds keep them
 * for debugging.
 */
export function lockDownWebview(): void {
  if (!import.meta.env.PROD) return;

  window.addEventListener("contextmenu", (e) => {
    const editable = e.target instanceof HTMLElement && e.target.closest("input, textarea");
    if (!editable) e.preventDefault();
  });

  window.addEventListener(
    "keydown",
    (e) => {
      const ctrl = e.ctrlKey || e.metaKey;
      const key = e.key.toLowerCase();
      if (BLOCKED_KEYS.has(e.key) || (ctrl && BLOCKED_WITH_CTRL.has(key)) || (ctrl && e.shiftKey && key === "i")) {
        e.preventDefault();
      }
    },
    { capture: true },
  );
}

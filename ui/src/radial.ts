import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

import { byId, svg } from "./lib/dom";
import { TOOL_ICONS } from "./lib/icons";
import { EVENTS, radialHide, radialMenu, runAction, type Menu, type MenuAction } from "./lib/ipc";
import { lockDownWebview } from "./lib/webview";
import { wedgeCentre, wedgePath, type Ring } from "./lib/wedges";

lockDownWebview();

/** The first group present gets the inner ring, the second the outer. */
const RINGS: Ring[] = [
  { inner: 60, outer: 112 },
  { inner: 117, outer: 170 },
];
const GAP = 3;
const ICON_SIZE = 22;
/** Matches the closing transition in radial.css. */
const CLOSE_MS = 120;
/** Focus can bounce while the window comes forward; ignore losing it this soon. */
const BLUR_GRACE_MS = 250;

type State = "closed" | "open" | "closing";

const root = byId("radial");
const layers = [byId<SVGSVGElement>("ring-inner"), byId<SVGSVGElement>("ring-outer")];
const hub = byId("hub");
const hubThumb = byId("hub-thumb");
const hubTitle = byId("hub-title");
const hubSubtitle = byId("hub-subtitle");

let state: State = "closed";
let menu: Menu | null = null;
let openedAt = 0;
/** In keyboard order: inner ring first, each ring clockwise from the top. */
let wedges: SVGGElement[] = [];

function title(action: MenuAction): string {
  if (action.group === "convert") return `Convert to ${action.label}`;
  const words = action.label.toLowerCase();
  return words.charAt(0).toUpperCase() + words.slice(1);
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? "" : "s"}`;
}

function render(m: Menu): void {
  const groups = [
    m.actions.filter((a) => a.group === "convert"),
    m.actions.filter((a) => a.group === "tools"),
  ].filter((g) => g.length > 0);

  wedges = [];
  layers.forEach((layer, i) => {
    layer.replaceChildren();
    const actions = groups[i];
    const ring = RINGS[i];
    if (!actions || !ring) return;
    layer.append(gradient(`hot-${i}`));
    actions.forEach((action, index) => {
      const w = wedge(action, ring, index, actions.length, `hot-${i}`);
      layer.append(w);
      wedges.push(w);
    });
  });
  root.classList.toggle("is-empty", wedges.length === 0);
  renderThumb(m);
  showFiles();
}

function gradient(id: string): SVGDefsElement {
  const defs = svg("defs");
  const grad = svg("linearGradient", { id, x1: 0, y1: 0, x2: 1, y2: 1 });
  grad.append(
    // Presentation attributes do not resolve var(); inline style does.
    svg("stop", { offset: 0, style: "stop-color: var(--ember-300)" }),
    svg("stop", { offset: 1, style: "stop-color: var(--ember-500)" }),
  );
  defs.append(grad);
  return defs;
}

function wedge(action: MenuAction, ring: Ring, index: number, count: number, hot: string): SVGGElement {
  const g = svg("g", {
    class: `wedge wedge--${action.group}`,
    role: "button",
    tabindex: 0,
    "aria-label": title(action),
    "data-id": action.id,
    style: `--hot: url(#${hot})`,
  });
  g.append(svg("path", { class: "wedge__shape", d: wedgePath(ring, index, count, GAP), "fill-rule": "evenodd" }));

  const c = wedgeCentre(ring, index, count);
  const icon = TOOL_ICONS[action.id];
  if (action.group === "tools" && icon) {
    const at = svg("g", {
      class: "wedge__icon",
      transform: `translate(${c.x - ICON_SIZE / 2} ${c.y - ICON_SIZE / 2}) scale(${ICON_SIZE / 24})`,
    });
    for (const part of icon) at.append(svg("path", { d: part.d, class: part.fill ? "is-solid" : "" }));
    g.append(at);
  } else {
    const text = svg("text", { class: "wedge__label", x: c.x, y: c.y });
    text.textContent = action.label;
    g.append(text);
  }

  g.addEventListener("click", (e) => {
    e.stopPropagation();
    void choose(action, g);
  });
  g.addEventListener("pointerenter", () => showAction(action));
  g.addEventListener("pointerleave", showFiles);
  g.addEventListener("focus", () => showAction(action));
  g.addEventListener("blur", showFiles);
  return g;
}

function renderThumb(m: Menu): void {
  const first = m.files[0];
  hubThumb.replaceChildren();
  if (!first) return;

  const glyph = () => {
    const doc = document.createElement("div");
    doc.className = "doc";
    doc.textContent = first.kind === "unknown" ? "?" : first.kindLabel;
    return doc;
  };
  if (first.previewPath) {
    const img = document.createElement("img");
    img.alt = "";
    img.decoding = "async";
    img.src = convertFileSrc(first.previewPath);
    img.addEventListener("error", () => img.replaceWith(glyph()), { once: true });
    hubThumb.append(img);
  } else {
    hubThumb.append(glyph());
  }
  if (m.files.length > 1) {
    const count = document.createElement("span");
    count.className = "hub__count";
    count.textContent = String(m.files.length);
    hubThumb.append(count);
  }
}

function showFiles(): void {
  if (!menu) return;
  const [first, ...rest] = menu.files;
  hubTitle.textContent = first?.name ?? "";
  hubTitle.title = first?.name ?? "";
  if (menu.actions.length === 0) {
    hubSubtitle.textContent = rest.length > 0 ? "No action fits all of them" : "No actions for this file";
  } else {
    hubSubtitle.textContent = rest.length > 0 ? `and ${plural(rest.length, "more file")}` : (first?.kindLabel ?? "");
  }
  hub.classList.remove("is-action");
}

function showAction(action: MenuAction): void {
  if (!menu) return;
  hubTitle.textContent = title(action);
  hubTitle.title = "";
  hubSubtitle.textContent = plural(menu.files.length, "file");
  hub.classList.add("is-action");
}

function open(m: Menu): void {
  menu = m;
  render(m);
  // Start from the closed pose, then let the transition run to the open one.
  state = "closed";
  root.dataset.state = "closed";
  void root.offsetWidth;
  requestAnimationFrame(() => {
    state = "open";
    openedAt = performance.now();
    root.dataset.state = "open";
    root.focus({ preventScroll: true });
  });
}

function close(): void {
  if (state !== "open") return;
  state = "closing";
  root.dataset.state = "closing";
  window.setTimeout(() => {
    if (state !== "closing") return;
    state = "closed";
    root.dataset.state = "closed";
    radialHide().catch((err: unknown) => console.error("radial_hide failed", err));
  }, CLOSE_MS);
}

async function choose(action: MenuAction, from: SVGGElement): Promise<void> {
  if (state !== "open" || !menu) return;
  from.classList.add("is-chosen");
  const dropId = menu.dropId;
  close();
  try {
    await runAction(dropId, action.id);
  } catch (err) {
    console.error("run_action failed", err);
  }
}

function step(delta: number): void {
  if (wedges.length === 0) return;
  const current = wedges.indexOf(document.activeElement as SVGGElement);
  const next =
    current < 0 ? (delta > 0 ? 0 : wedges.length - 1) : (current + delta + wedges.length) % wedges.length;
  wedges[next]?.focus();
}

window.addEventListener("keydown", (e) => {
  if (state !== "open") return;
  switch (e.key) {
    case "Escape":
      close();
      break;
    case "ArrowRight":
    case "ArrowDown":
      step(1);
      break;
    case "ArrowLeft":
    case "ArrowUp":
      step(-1);
      break;
    case "Enter":
    case " ": {
      const active = document.activeElement;
      if (active instanceof SVGGElement && wedges.includes(active)) active.dispatchEvent(new MouseEvent("click"));
      break;
    }
    default:
      return;
  }
  e.preventDefault();
});

// Anything that is not a wedge closes the menu: the hub, the gaps, the corners.
root.addEventListener("click", close);

await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
  if (!focused && state === "open" && performance.now() - openedAt > BLUR_GRACE_MS) close();
});

await listen<Menu>(EVENTS.radialOpen, (e) => open(e.payload));

// A drop can arrive before this page has finished loading.
const pending = await radialMenu();
if (pending && state === "closed" && (await getCurrentWindow().isVisible())) open(pending);

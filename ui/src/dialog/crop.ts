import { h } from "../lib/dom";
import type { PanelFactory } from "./types";
import { section, unavailable } from "./widgets";

interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

const ASPECTS: { label: string; ratio: number | null }[] = [
  { label: "Free", ratio: null },
  { label: "1:1", ratio: 1 },
  { label: "4:3", ratio: 4 / 3 },
  { label: "3:4", ratio: 3 / 4 },
  { label: "16:9", ratio: 16 / 9 },
  { label: "9:16", ratio: 9 / 16 },
];
const HANDLES = ["nw", "n", "ne", "e", "se", "s", "sw", "w"] as const;
type Handle = (typeof HANDLES)[number];

const clamp = (v: number, lo: number, hi: number) => Math.min(Math.max(v, lo), hi);

/**
 * Crop rectangle editor. The rectangle lives in the pixels of the full, upright image;
 * the screen shows it scaled.
 */
export const cropPanel: PanelFactory = (ctx) => {
  if (!ctx.image || !ctx.previewUrl) return unavailable(ctx, "This file cannot be previewed yet.");
  const W = ctx.image.width;
  const H = ctx.image.height;
  const MIN = Math.max(1, Math.round(Math.min(W, H) / 100));
  let rect: Rect = { x: 0, y: 0, w: W, h: H };
  let ratio: number | null = null;
  let scale = 1;

  // Stage: the picture with a movable box over it.
  const img = h("img", { class: "crop__image", src: ctx.previewUrl, alt: "", draggable: "false" });
  const box = h("div", { class: "crop__box", tabindex: 0, "aria-label": "Crop area" });
  for (const handle of HANDLES) box.append(h("span", { class: `crop__handle crop__handle--${handle}`, "data-handle": handle }));
  const frame = h("div", { class: "crop__frame" }, img, box);
  ctx.stage.replaceChildren(h("div", { class: "crop" }, frame));

  // Panel.
  const widthInput = h("input", { type: "number", min: 1, max: W, value: W, "aria-label": "Width in pixels" });
  const heightInput = h("input", { type: "number", min: 1, max: H, value: H, "aria-label": "Height in pixels" });
  const position = h("p", { class: "hint" });
  const chips = h("div", { class: "chips", role: "radiogroup", "aria-label": "Aspect ratio" });
  for (const a of ASPECTS) {
    const chip = h("button", { type: "button", class: "chip", role: "radio", "aria-checked": a.ratio === null }, a.label);
    chip.addEventListener("click", () => {
      for (const c of chips.children) c.setAttribute("aria-checked", String(c === chip));
      ratio = a.ratio;
      if (ratio) rect = fitAspect(rect, ratio);
      update();
    });
    chips.append(chip);
  }
  const reset = h("button", { type: "button" }, "Reset");
  reset.addEventListener("click", () => {
    rect = ratio ? fitAspect({ x: 0, y: 0, w: W, h: H }, ratio) : { x: 0, y: 0, w: W, h: H };
    update();
  });
  ctx.panel.replaceChildren(
    section("Aspect ratio", chips),
    section(
      "Size in pixels",
      h("div", { class: "pair" }, h("label", {}, "W", widthInput), h("span", { "aria-hidden": "true" }, "×"), h("label", {}, "H", heightInput)),
      position,
    ),
    section("", reset),
  );

  /** The largest rectangle of `r` aspect that fits the image, centred on `around`. */
  function fitAspect(around: Rect, r: number): Rect {
    let w = W;
    let hh = w / r;
    if (hh > H) {
      hh = H;
      w = hh * r;
    }
    const cx = around.x + around.w / 2;
    const cy = around.y + around.h / 2;
    return round({ x: clamp(cx - w / 2, 0, W - w), y: clamp(cy - hh / 2, 0, H - hh), w, h: hh });
  }

  function round(r: Rect): Rect {
    const x = clamp(Math.round(r.x), 0, W - 1);
    const y = clamp(Math.round(r.y), 0, H - 1);
    return { x, y, w: clamp(Math.round(r.w), 1, W - x), h: clamp(Math.round(r.h), 1, H - y) };
  }

  function layout(): void {
    const style = getComputedStyle(ctx.stage);
    const availW = ctx.stage.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    const availH = ctx.stage.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom);
    scale = Math.min(availW / W, availH / H);
    frame.style.width = `${W * scale}px`;
    frame.style.height = `${H * scale}px`;
    update();
  }

  function update(): void {
    box.style.left = `${rect.x * scale}px`;
    box.style.top = `${rect.y * scale}px`;
    box.style.width = `${rect.w * scale}px`;
    box.style.height = `${rect.h * scale}px`;
    if (document.activeElement !== widthInput) widthInput.value = String(rect.w);
    if (document.activeElement !== heightInput) heightInput.value = String(rect.h);
    position.textContent = `From ${rect.x}, ${rect.y} of ${W} × ${H}`;
    ctx.changed();
  }

  /** Pointer position in image pixels. */
  function toImage(e: PointerEvent): { px: number; py: number } {
    const b = frame.getBoundingClientRect();
    return { px: (e.clientX - b.left) / scale, py: (e.clientY - b.top) / scale };
  }

  function resize(start: Rect, handle: Handle, px: number, py: number): Rect {
    const sx = handle.includes("e") ? 1 : handle.includes("w") ? -1 : 0;
    const sy = handle.includes("s") ? 1 : handle.includes("n") ? -1 : 0;
    // The opposite corner or edge stays put.
    const ax = sx > 0 ? start.x : sx < 0 ? start.x + start.w : start.x + start.w / 2;
    const ay = sy > 0 ? start.y : sy < 0 ? start.y + start.h : start.y + start.h / 2;
    // Room between the anchor and the image edges, in each direction.
    const roomX = sx > 0 ? W - ax : sx < 0 ? ax : 2 * Math.min(ax, W - ax);
    const roomY = sy > 0 ? H - ay : sy < 0 ? ay : 2 * Math.min(ay, H - ay);

    let w = sx === 0 ? start.w : Math.max(MIN, sx * (px - ax));
    let hh = sy === 0 ? start.h : Math.max(MIN, sy * (py - ay));
    if (ratio) {
      if (sx === 0) w = hh * ratio;
      else if (sy === 0) hh = w / ratio;
      else if (w / hh > ratio) w = hh * ratio;
      else hh = w / ratio;
      const k = Math.min(1, roomX / w, roomY / hh);
      w *= k;
      hh *= k;
    } else {
      w = Math.min(w, roomX);
      hh = Math.min(hh, roomY);
    }
    const x = sx > 0 ? ax : sx < 0 ? ax - w : ax - w / 2;
    const y = sy > 0 ? ay : sy < 0 ? ay - hh : ay - hh / 2;
    return round({ x, y, w, h: hh });
  }

  box.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    const handle = (e.target as HTMLElement).dataset.handle as Handle | undefined;
    const start = { ...rect };
    const origin = toImage(e);
    box.setPointerCapture(e.pointerId);
    const move = (ev: PointerEvent) => {
      const { px, py } = toImage(ev);
      rect = handle
        ? resize(start, handle, px, py)
        : round({
            x: clamp(start.x + px - origin.px, 0, W - start.w),
            y: clamp(start.y + py - origin.py, 0, H - start.h),
            w: start.w,
            h: start.h,
          });
      update();
    };
    const end = () => {
      box.removeEventListener("pointermove", move);
      box.removeEventListener("pointerup", end);
      box.removeEventListener("pointercancel", end);
    };
    box.addEventListener("pointermove", move);
    box.addEventListener("pointerup", end);
    box.addEventListener("pointercancel", end);
  });

  // Arrow keys nudge the box; with Shift, by ten pixels.
  box.addEventListener("keydown", (e) => {
    const step = e.shiftKey ? 10 : 1;
    const delta = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] }[e.key];
    if (!delta) return;
    e.preventDefault();
    rect = round({ ...rect, x: clamp(rect.x + (delta[0] ?? 0), 0, W - rect.w), y: clamp(rect.y + (delta[1] ?? 0), 0, H - rect.h) });
    update();
  });

  const fromInputs = (changed: "w" | "h") => {
    let w = clamp(Number(widthInput.value) || rect.w, MIN, W);
    let hh = clamp(Number(heightInput.value) || rect.h, MIN, H);
    if (ratio) {
      if (changed === "w") hh = w / ratio;
      else w = hh * ratio;
      const k = Math.min(1, W / w, H / hh);
      w *= k;
      hh *= k;
    }
    rect = round({ x: clamp(rect.x, 0, W - w), y: clamp(rect.y, 0, H - hh), w, h: hh });
    update();
  };
  widthInput.addEventListener("change", () => fromInputs("w"));
  heightInput.addEventListener("change", () => fromInputs("h"));

  new ResizeObserver(layout).observe(ctx.stage);
  if (img.complete) layout();
  else img.addEventListener("load", layout, { once: true });

  return {
    applyLabel: "Crop",
    params: () =>
      rect.x === 0 && rect.y === 0 && rect.w === W && rect.h === H
        ? null
        : { type: "crop", x: rect.x, y: rect.y, width: rect.w, height: rect.h, refWidth: W, refHeight: H },
  };
};

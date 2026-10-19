import { h } from "../lib/dom";
import type { Edit } from "../lib/ipc";
import type { PanelFactory } from "./types";
import { section, slider, toggle, unavailable } from "./widgets";

const NONE: Edit = { rotate: 0, flipHorizontal: false, flipVertical: false, grayscale: false, brightness: 0, contrast: 0 };

/**
 * The preview uses the same CSS filters and transforms that the Rust side reproduces:
 * brightness, then contrast, then grayscale; flips, then rotation.
 */
export const editPanel: PanelFactory = (ctx) => {
  if (!ctx.image || !ctx.previewUrl) return unavailable(ctx, "This file cannot be previewed yet.");
  const { width: W, height: H } = ctx.image;
  let edit: Edit = { ...NONE };

  const img = h("img", { class: "edit__image", src: ctx.previewUrl, alt: "" });
  ctx.stage.replaceChildren(h("div", { class: "edit" }, img));

  function render(): void {
    const style = getComputedStyle(ctx.stage);
    const availW = ctx.stage.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    const availH = ctx.stage.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom);
    const sideways = edit.rotate === 90 || edit.rotate === 270;
    const [effW, effH] = sideways ? [H, W] : [W, H];
    const scale = Math.min(availW / effW, availH / effH);
    img.style.width = `${W * scale}px`;
    img.style.height = `${H * scale}px`;
    img.style.transform = `translate(-50%, -50%) rotate(${edit.rotate}deg) scale(${edit.flipHorizontal ? -1 : 1}, ${edit.flipVertical ? -1 : 1})`;
    img.style.filter = `brightness(${1 + edit.brightness / 100}) contrast(${1 + edit.contrast / 100}) grayscale(${edit.grayscale ? 1 : 0})`;
    brightness.setLabel(signed(edit.brightness));
    contrast.setLabel(signed(edit.contrast));
    ctx.changed();
  }

  const signed = (n: number) => (n > 0 ? `+${n}` : String(n));
  const rotateLeft = h("button", { type: "button" }, "↺ Rotate left");
  const rotateRight = h("button", { type: "button" }, "↻ Rotate right");
  rotateLeft.addEventListener("click", () => {
    edit.rotate = ((edit.rotate + 270) % 360) as Edit["rotate"];
    render();
  });
  rotateRight.addEventListener("click", () => {
    edit.rotate = ((edit.rotate + 90) % 360) as Edit["rotate"];
    render();
  });
  const flipH = toggle("Flip horizontal", (on) => {
    edit.flipHorizontal = on;
    render();
  });
  const flipV = toggle("Flip vertical", (on) => {
    edit.flipVertical = on;
    render();
  });
  const gray = toggle("Grayscale", (on) => {
    edit.grayscale = on;
    render();
  });
  const brightness = slider("Brightness", -100, 100, 0);
  const contrast = slider("Contrast", -100, 100, 0);
  brightness.input.addEventListener("input", () => {
    edit.brightness = Number(brightness.input.value);
    render();
  });
  contrast.input.addEventListener("input", () => {
    edit.contrast = Number(contrast.input.value);
    render();
  });
  const reset = h("button", { type: "button" }, "Reset");
  reset.addEventListener("click", () => {
    edit = { ...NONE };
    brightness.input.value = "0";
    contrast.input.value = "0";
    for (const t of [flipH, flipV, gray]) t.set(false);
    render();
  });

  ctx.panel.replaceChildren(
    section("Rotate and flip", h("div", { class: "button-grid" }, rotateLeft, rotateRight, flipH.element, flipV.element)),
    section("Colour", gray.element, brightness.element, contrast.element),
    section("", reset),
  );

  new ResizeObserver(render).observe(ctx.stage);
  render();

  return {
    applyLabel: "Save edits",
    params: () => (JSON.stringify(edit) === JSON.stringify(NONE) ? null : { type: "edit", ...edit }),
  };
};

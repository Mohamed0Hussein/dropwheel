import { h } from "../lib/dom";
import type { PanelFactory } from "./types";
import { hint, section, showPreview, unavailable } from "./widgets";

const SWATCHES = ["#ffffff", "#000000", "#f2f2f2", "#fff8e7", "#2b2456", "#ff8a5c"];

function parse(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export const backgroundPanel: PanelFactory = (ctx) => {
  if (!ctx.image || !ctx.previewUrl) return unavailable(ctx, "This file cannot be previewed yet.");
  let colour = "#ffffff";
  showPreview(ctx, "stage__image--backdrop");
  ctx.stage.classList.add("stage--checker");
  const img = ctx.stage.querySelector("img");

  const picker = h("input", { type: "color", value: colour, "aria-label": "Background colour" });
  const code = h("code", {}, colour);
  const swatches = h("div", { class: "swatches" });
  const set = (value: string) => {
    colour = value;
    picker.value = value;
    code.textContent = value;
    if (img) img.style.backgroundColor = value;
    for (const s of swatches.children) s.setAttribute("aria-pressed", String((s as HTMLElement).dataset.colour === value));
  };
  for (const s of SWATCHES) {
    const b = h("button", { type: "button", class: "swatch", "data-colour": s, "aria-label": s, style: `background:${s}` });
    b.addEventListener("click", () => set(s));
    swatches.append(b);
  }
  picker.addEventListener("input", () => set(picker.value));

  ctx.panel.replaceChildren(
    section("Background colour", swatches, h("label", { class: "colour" }, picker, code)),
    section("", hint("Transparent areas are filled with this colour. The file keeps its format.")),
  );
  set(colour);

  return {
    applyLabel: "Add background",
    params: () => ({ type: "background", color: parse(colour) }),
  };
};

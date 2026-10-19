// Small building blocks shared by the dialog panels.
import { h } from "../lib/dom";
import type { PanelContext } from "./types";

/** Shows the preview, fitted inside the stage. */
export function showPreview(ctx: PanelContext, extraClass = ""): HTMLImageElement | null {
  ctx.stage.replaceChildren();
  if (!ctx.previewUrl) return null;
  const img = h("img", { class: `stage__image ${extraClass}`.trim(), src: ctx.previewUrl, alt: "" });
  ctx.stage.append(img);
  return img;
}

export function section(title: string, ...children: (Node | null)[]): HTMLElement {
  return h("section", { class: "group" }, h("h2", {}, title), ...children);
}

export function hint(text: string): HTMLParagraphElement {
  return h("p", { class: "hint" }, text);
}

export interface Slider {
  element: HTMLElement;
  input: HTMLInputElement;
  setLabel(text: string): void;
}

export function slider(label: string, min: number, max: number, value: number, step = 1): Slider {
  const input = h("input", { type: "range", min, max, step, value });
  const output = h("output", {});
  const element = h("label", { class: "slider" }, h("span", { class: "slider__top" }, h("span", {}, label), output), input);
  return { element, input, setLabel: (text) => (output.textContent = text) };
}

/** A row of mutually exclusive buttons. */
export function segmented<T extends string>(
  label: string,
  options: { value: T; text: string }[],
  initial: T,
  onChange: (value: T) => void,
): HTMLElement {
  const group = h("div", { class: "segmented", role: "radiogroup", "aria-label": label });
  const buttons = options.map((o) => {
    const b = h("button", { type: "button", role: "radio", "aria-checked": o.value === initial }, o.text);
    b.addEventListener("click", () => {
      for (const other of buttons) other.setAttribute("aria-checked", String(other === b));
      onChange(o.value);
    });
    return b;
  });
  group.append(...buttons);
  return group;
}

/** A button that stays pressed until clicked again. */
export function toggle(text: string, onChange: (on: boolean) => void): { element: HTMLButtonElement; set(on: boolean): void } {
  const element = h("button", { type: "button", class: "toggle", "aria-pressed": "false" }, text);
  element.addEventListener("click", () => {
    const on = element.getAttribute("aria-pressed") !== "true";
    element.setAttribute("aria-pressed", String(on));
    onChange(on);
  });
  return { element, set: (on) => element.setAttribute("aria-pressed", String(on)) };
}

export function unavailable(ctx: PanelContext, message: string) {
  ctx.stage.replaceChildren(h("div", { class: "stage__message" }, message));
  return { params: () => null, applyLabel: "Apply" };
}

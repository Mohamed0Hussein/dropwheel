import { h, svg } from "../lib/dom";
import type { Point } from "../lib/geometry";
import type { PanelContext } from "./types";

/**
 * The picture with an SVG layer over it whose coordinates are the full image's pixels,
 * so shapes are kept, drawn and saved in the same units. An HTML layer on top holds
 * inline editors.
 */
export interface Surface {
  readonly frame: HTMLElement;
  readonly layer: SVGSVGElement;
  readonly overlay: HTMLElement;
  /** Size of the full, upright image. */
  width: number;
  height: number;
  /** Screen pixels per image pixel. */
  scale(): number;
  toImage(e: { clientX: number; clientY: number }): Point;
  /** Swaps the picture, as when moving to another page. */
  setImage(url: string, width: number, height: number): void;
  onLayout(callback: () => void): void;
}

export function surface(ctx: PanelContext, url: string, width: number, height: number): Surface {
  const img = h("img", { class: "surface__image", src: url, alt: "", draggable: "false" });
  const layer = svg("svg", { class: "surface__layer", preserveAspectRatio: "none" });
  const overlay = h("div", { class: "surface__overlay" });
  const frame = h("div", { class: "surface" }, img, layer, overlay);
  ctx.stage.replaceChildren(h("div", { class: "surface__host" }, frame));

  let scale = 1;
  const listeners: (() => void)[] = [];
  const s: Surface = {
    frame,
    layer,
    overlay,
    width,
    height,
    scale: () => scale,
    toImage(e) {
      const b = frame.getBoundingClientRect();
      return [(e.clientX - b.left) / scale, (e.clientY - b.top) / scale];
    },
    setImage(next, w, hh) {
      s.width = w;
      s.height = hh;
      img.src = next;
      layout();
    },
    onLayout: (callback) => listeners.push(callback),
  };

  function layout(): void {
    const style = getComputedStyle(ctx.stage);
    const availW = ctx.stage.clientWidth - parseFloat(style.paddingLeft) - parseFloat(style.paddingRight);
    const availH = ctx.stage.clientHeight - parseFloat(style.paddingTop) - parseFloat(style.paddingBottom);
    scale = Math.max(0.01, Math.min(availW / s.width, availH / s.height));
    frame.style.width = `${s.width * scale}px`;
    frame.style.height = `${s.height * scale}px`;
    layer.setAttribute("viewBox", `0 0 ${s.width} ${s.height}`);
    for (const l of listeners) l();
  }

  new ResizeObserver(layout).observe(ctx.stage);
  layout();
  return s;
}

export function rgb(color: [number, number, number]): string {
  return `rgb(${color[0]} ${color[1]} ${color[2]})`;
}

export function parseHex(hex: string): [number, number, number] {
  const n = Number.parseInt(hex.slice(1), 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255];
}

export function toHex(color: [number, number, number]): string {
  return `#${color.map((c) => c.toString(16).padStart(2, "0")).join("")}`;
}

/** Keeps a list of earlier states for Ctrl+Z. */
export class History<T> {
  private past: T[] = [];
  private lastKey: string | null = null;

  constructor(private readonly limit = 100) {}

  /**
   * Remembers `state` before a change. Consecutive changes with the same `key`, such as
   * the steps of one slider drag, are undone together.
   */
  push(state: T, key: string | null = null): void {
    if (key !== null && key === this.lastKey) return;
    this.lastKey = key;
    this.past.push(structuredClone(state));
    if (this.past.length > this.limit) this.past.shift();
  }

  undo(): T | undefined {
    this.lastKey = null;
    return this.past.pop();
  }

  get canUndo(): boolean {
    return this.past.length > 0;
  }
}

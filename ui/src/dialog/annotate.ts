import arabicBold from "../../../crates/core/fonts/NotoSansArabic-Bold.ttf?url";
import arabicRegular from "../../../crates/core/fonts/NotoSansArabic-Regular.ttf?url";
import latinBold from "../../../crates/core/fonts/NotoSans-Bold.ttf?url";
import latinRegular from "../../../crates/core/fonts/NotoSans-Regular.ttf?url";
import { h, svg } from "../lib/dom";
import { arrowGeometry, boxBetween, contains, distanceToSegment, type Box, type Point } from "../lib/geometry";
import type { Shape } from "../lib/ipc";
import { History, parseHex, rgb, surface, toHex } from "./surface";
import type { PanelFactory } from "./types";
import { hint, section, segmented, slider, toggle, unavailable } from "./widgets";

type Tool = "select" | "rect" | "arrow" | "text";
type Rgb = [number, number, number];

const COLOURS = ["#e53935", "#fb8c00", "#fdd835", "#43a047", "#1e88e5", "#8e24aa", "#000000", "#ffffff"];
/** The same fonts, in the same order, as the Rust side draws with. */
export const ANNOTATION_FONTS = '"Dropwheel Sans", "Dropwheel Arabic", "Segoe UI", sans-serif';
/** Line height of annotation text; LINE_HEIGHT in crates/core/src/actions/annotate.rs. */
const LINE_HEIGHT = 1.25;

let fontsLoaded: Promise<void> | null = null;

export function loadAnnotationFonts(): Promise<void> {
  fontsLoaded ??= (async () => {
    const faces = [
      new FontFace("Dropwheel Sans", `url("${latinRegular}")`, { weight: "400" }),
      new FontFace("Dropwheel Sans", `url("${latinBold}")`, { weight: "700" }),
      new FontFace("Dropwheel Arabic", `url("${arabicRegular}")`, { weight: "400" }),
      new FontFace("Dropwheel Arabic", `url("${arabicBold}")`, { weight: "700" }),
    ];
    for (const f of faces) document.fonts.add(f);
    await Promise.all(faces.map((f) => f.load().catch(() => undefined)));
  })();
  return fontsLoaded;
}

/** Text laid out like the Rust side: one block per line, each aligned to its own start. */
export function textBlock(text: string, size: number, color: Rgb, bold: boolean): HTMLElement {
  const block = h("div", { class: "annot-text" });
  block.style.fontSize = `${size}px`;
  block.style.fontWeight = bold ? "700" : "400";
  block.style.color = rgb(color);
  block.style.fontFamily = ANNOTATION_FONTS;
  block.style.lineHeight = String(LINE_HEIGHT);
  for (const line of text.replace(/\t/g, "    ").split("\n")) {
    block.append(h("div", { class: "annot-text__line", dir: "auto" }, line));
  }
  return block;
}

export const annotatePanel: PanelFactory = async (ctx) => {
  if (!ctx.image || !ctx.previewUrl) return unavailable(ctx, "This file cannot be previewed yet.");
  await loadAnnotationFonts();
  const W = ctx.image.width;
  const H = ctx.image.height;
  const s = surface(ctx, ctx.previewUrl, W, H);
  const shapesLayer = svg("g");
  const selectionLayer = svg("g", { class: "annot-selection" });
  s.layer.append(shapesLayer, selectionLayer);
  s.frame.classList.add("surface--draw");

  let shapes: Shape[] = [];
  let draft: Shape | null = null;
  let selected: number | null = null;
  let editing: { index: number | null; at: Point; element: HTMLElement } | null = null;
  let tool: Tool = "rect";
  let colour: Rgb = parseHex(COLOURS[0] ?? "#e53935");
  const short = Math.min(W, H);
  let stroke = Math.max(2, Math.round(short / 160));
  let size = Math.max(12, Math.round(short / 16));
  let bold = true;
  const history = new History<Shape[]>();

  // Panel.
  const tools = segmented<Tool>(
    "Tool",
    [
      { value: "select", text: "Select" },
      { value: "rect", text: "Box" },
      { value: "arrow", text: "Arrow" },
      { value: "text", text: "Text" },
    ],
    tool,
    (t) => {
      commitEditor();
      tool = t;
      s.frame.dataset.tool = t;
      if (t !== "select") selected = null;
      render();
    },
  );
  s.frame.dataset.tool = tool;

  const swatches = h("div", { class: "swatches" });
  const picker = h("input", { type: "color", value: toHex(colour), "aria-label": "Colour" });
  const setColour = (hex: string) => {
    colour = parseHex(hex);
    picker.value = hex;
    for (const b of swatches.children) b.setAttribute("aria-pressed", String((b as HTMLElement).dataset.colour === hex));
    restyle({ color: colour }, "colour");
  };
  for (const c of COLOURS) {
    const b = h("button", { type: "button", class: "swatch", "data-colour": c, "aria-label": c, style: `background:${c}` });
    b.addEventListener("click", () => setColour(c));
    swatches.append(b);
  }
  picker.addEventListener("input", () => setColour(picker.value));

  const thickness = slider("Line thickness", 1, Math.max(20, Math.round(short / 20)), stroke);
  thickness.input.addEventListener("input", () => {
    stroke = Number(thickness.input.value);
    thickness.setLabel(`${stroke} px`);
    restyle({ stroke }, "stroke");
  });
  thickness.setLabel(`${stroke} px`);

  const textSize = slider("Text size", 8, Math.max(64, Math.round(short / 4)), size);
  textSize.input.addEventListener("input", () => {
    size = Number(textSize.input.value);
    textSize.setLabel(`${size} px`);
    restyle({ size }, "size");
  });
  textSize.setLabel(`${size} px`);
  const boldToggle = toggle("Bold", (on) => {
    bold = on;
    restyle({ bold }, "bold");
  });
  boldToggle.set(bold);

  const undoButton = h("button", { type: "button" }, "Undo");
  const deleteButton = h("button", { type: "button" }, "Delete");
  const clearButton = h("button", { type: "button" }, "Clear all");
  undoButton.addEventListener("click", undo);
  deleteButton.addEventListener("click", removeSelected);
  clearButton.addEventListener("click", () => {
    if (!shapes.length) return;
    history.push(shapes);
    shapes = [];
    selected = null;
    render();
  });

  ctx.panel.replaceChildren(
    section("Tool", tools),
    section("Colour", swatches, h("label", { class: "colour" }, picker, h("span", {}, "Custom"))),
    section("Lines", thickness.element),
    section("Text", textSize.element, boldToggle.element),
    section("", h("div", { class: "button-grid" }, undoButton, deleteButton, clearButton)),
    section(
      "",
      hint("Drag to draw a box or an arrow. With Text, click where it goes; Enter finishes and Shift+Enter starts a new line."),
      hint("Select moves a shape; Delete removes it. Ctrl+Z undoes."),
    ),
  );
  setColour(toHex(colour));

  /** Applies a style change to the selected shape too. */
  function restyle(change: { color?: Rgb; stroke?: number; size?: number; bold?: boolean }, key: string): void {
    if (editing) {
      styleEditor();
      return;
    }
    const i = selected;
    const shape = i === null ? undefined : shapes[i];
    if (i === null || !shape) return;
    const next: Shape = { ...shape };
    if (change.color) next.color = change.color;
    if (change.stroke !== undefined && next.shape !== "text") next.stroke = change.stroke;
    if (next.shape === "text") {
      if (change.size !== undefined) next.size = change.size;
      if (change.bold !== undefined) next.bold = change.bold;
    }
    if (JSON.stringify(next) === JSON.stringify(shape)) return;
    history.push(shapes, `${key}:${i}`);
    shapes[i] = next;
    render();
  }

  function undo(): void {
    commitEditor();
    const previous = history.undo();
    if (!previous) return;
    shapes = previous;
    selected = null;
    render();
  }

  function removeSelected(): void {
    if (selected === null) return;
    history.push(shapes);
    shapes.splice(selected, 1);
    selected = null;
    render();
  }

  function drawShape(shape: Shape): SVGElement {
    const g = svg("g");
    if (shape.shape === "rect") {
      const b = boxBetween([shape.x, shape.y], [shape.x + shape.width, shape.y + shape.height]);
      g.append(
        svg("rect", {
          x: b.x,
          y: b.y,
          width: b.w,
          height: b.h,
          fill: "none",
          stroke: rgb(shape.color),
          "stroke-width": shape.stroke,
          "stroke-linejoin": "miter",
        }),
      );
    } else if (shape.shape === "arrow") {
      const geo = arrowGeometry(shape.from, shape.to, shape.stroke);
      if (geo) {
        g.append(
          svg("line", {
            x1: shape.from[0],
            y1: shape.from[1],
            x2: geo.shaftEnd[0],
            y2: geo.shaftEnd[1],
            stroke: rgb(shape.color),
            "stroke-width": shape.stroke,
            "stroke-linecap": "round",
          }),
          svg("polygon", { points: geo.head.map((p) => p.join(",")).join(" "), fill: rgb(shape.color) }),
        );
      }
    } else {
      const fo = svg("foreignObject", { x: shape.x, y: shape.y, width: W * 4, height: H * 4, class: "annot-fo" });
      fo.append(textBlock(shape.text, shape.size, shape.color, shape.bold));
      g.append(fo);
    }
    return g;
  }

  /** The shape's extent in image pixels. */
  function bounds(i: number): Box | null {
    const shape = shapes[i];
    if (!shape) return null;
    if (shape.shape === "rect") {
      const b = boxBetween([shape.x, shape.y], [shape.x + shape.width, shape.y + shape.height]);
      const m = shape.stroke / 2;
      return { x: b.x - m, y: b.y - m, w: b.w + 2 * m, h: b.h + 2 * m };
    }
    if (shape.shape === "arrow") {
      const b = boxBetween(shape.from, shape.to);
      const m = shape.stroke * 2 + 3;
      return { x: b.x - m, y: b.y - m, w: b.w + 2 * m, h: b.h + 2 * m };
    }
    const block = shapesLayer.children[i]?.querySelector(".annot-text");
    if (!block) return null;
    const r = block.getBoundingClientRect();
    const [x, y] = s.toImage({ clientX: r.left, clientY: r.top });
    return { x, y, w: r.width / s.scale(), h: r.height / s.scale() };
  }

  function hit(p: Point): number | null {
    const tolerance = 6 / s.scale();
    for (let i = shapes.length - 1; i >= 0; i--) {
      const shape = shapes[i];
      if (!shape) continue;
      if (shape.shape === "arrow") {
        if (distanceToSegment(p, shape.from, shape.to) <= shape.stroke * 2 + tolerance) return i;
      } else {
        const b = bounds(i);
        if (b && contains(b, p, tolerance)) return i;
      }
    }
    return null;
  }

  function render(): void {
    shapesLayer.replaceChildren(
      ...shapes.map((shape, i) => {
        const g = drawShape(shape);
        if (editing?.index === i) g.setAttribute("visibility", "hidden");
        return g;
      }),
    );
    if (draft) shapesLayer.append(drawShape(draft));
    selectionLayer.replaceChildren();
    const b = selected === null ? null : bounds(selected);
    if (b) {
      const pad = 4 / s.scale();
      selectionLayer.append(
        svg("rect", {
          x: b.x - pad,
          y: b.y - pad,
          width: b.w + 2 * pad,
          height: b.h + 2 * pad,
          "vector-effect": "non-scaling-stroke",
        }),
      );
    }
    undoButton.disabled = !history.canUndo;
    deleteButton.disabled = selected === null;
    clearButton.disabled = shapes.length === 0;
    ctx.changed();
  }
  s.onLayout(() => {
    render();
    if (editing) styleEditor();
  });

  // Text editing, in an HTML layer over the picture.
  function openEditor(at: Point, index: number | null): void {
    commitEditor();
    const existing = index === null ? undefined : shapes[index];
    if (existing?.shape === "text") {
      colour = existing.color;
      size = existing.size;
      bold = existing.bold;
      textSize.input.value = String(size);
      textSize.setLabel(`${size} px`);
      boldToggle.set(bold);
      picker.value = toHex(colour);
    }
    const element = h("div", { class: "annot-editor", contenteditable: "plaintext-only", dir: "auto", spellcheck: "false" });
    element.textContent = existing?.shape === "text" ? existing.text : "";
    editing = { index, at, element };
    styleEditor();
    s.overlay.append(element);
    element.focus();
    const range = document.createRange();
    range.selectNodeContents(element);
    range.collapse(false);
    getSelection()?.removeAllRanges();
    getSelection()?.addRange(range);
    element.addEventListener("keydown", (e) => {
      e.stopPropagation();
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        commitEditor();
      } else if (e.key === "Escape") {
        e.preventDefault();
        cancelEditor();
      }
    });
    element.addEventListener("blur", () => commitEditor());
    render();
  }

  function styleEditor(): void {
    if (!editing) return;
    const e = editing.element;
    const k = s.scale();
    e.style.left = `${editing.at[0] * k}px`;
    e.style.top = `${editing.at[1] * k}px`;
    e.style.fontSize = `${size * k}px`;
    e.style.fontWeight = bold ? "700" : "400";
    e.style.color = rgb(colour);
    e.style.fontFamily = ANNOTATION_FONTS;
    e.style.lineHeight = String(LINE_HEIGHT);
  }

  function commitEditor(): void {
    if (!editing) return;
    const { index, at, element } = editing;
    editing = null;
    element.remove();
    const text = element.innerText.replace(/\r/g, "").replace(/\n+$/, "");
    const shape: Shape = { shape: "text", x: at[0], y: at[1], text, color: colour, size, bold };
    if (index === null) {
      if (text.trim()) {
        history.push(shapes);
        shapes.push(shape);
        selected = tool === "select" ? shapes.length - 1 : null;
      }
    } else if (!text.trim()) {
      history.push(shapes);
      shapes.splice(index, 1);
      selected = null;
    } else if (JSON.stringify(shapes[index]) !== JSON.stringify(shape)) {
      history.push(shapes);
      shapes[index] = shape;
    }
    render();
  }

  function cancelEditor(): void {
    if (!editing) return;
    editing.element.remove();
    editing = null;
    render();
  }

  // Drawing and moving.
  s.layer.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    commitEditor();
    const start = s.toImage(e);
    const k = s.scale();
    let moved = false;

    if (tool === "text") {
      const up = (ev: PointerEvent) => {
        s.layer.removeEventListener("pointerup", up);
        const i = hit(s.toImage(ev));
        const shape = i === null ? undefined : shapes[i];
        if (i !== null && shape?.shape === "text") openEditor([shape.x, shape.y], i);
        else openEditor(start, null);
      };
      s.layer.addEventListener("pointerup", up);
      return;
    }

    s.layer.setPointerCapture(e.pointerId);
    let onMove: (p: Point) => void;
    let onEnd: () => void = () => undefined;

    if (tool === "select") {
      selected = hit(start);
      render();
      const index = selected;
      const original = index === null ? undefined : structuredClone(shapes[index]);
      if (index === null || !original) return;
      onMove = (p) => {
        const dx = p[0] - start[0];
        const dy = p[1] - start[1];
        if (!moved) history.push(shapes);
        moved = true;
        shapes[index] = translate(original, dx, dy);
        render();
      };
    } else {
      const base = { color: colour, stroke };
      onMove = (p) => {
        moved = moved || Math.hypot(p[0] - start[0], p[1] - start[1]) * k > 3;
        if (!moved) return;
        draft =
          tool === "rect"
            ? { shape: "rect", x: start[0], y: start[1], width: p[0] - start[0], height: p[1] - start[1], ...base }
            : { shape: "arrow", from: start, to: p, ...base };
        render();
      };
      onEnd = () => {
        if (draft && moved) {
          if (draft.shape === "rect") {
            const b = boxBetween([draft.x, draft.y], [draft.x + draft.width, draft.y + draft.height]);
            draft = { ...draft, x: b.x, y: b.y, width: b.w, height: b.h };
          }
          history.push(shapes);
          shapes.push(draft);
        }
        draft = null;
        render();
      };
    }

    const move = (ev: PointerEvent) => onMove(s.toImage(ev));
    const end = () => {
      s.layer.removeEventListener("pointermove", move);
      s.layer.removeEventListener("pointerup", end);
      s.layer.removeEventListener("pointercancel", end);
      onEnd();
    };
    s.layer.addEventListener("pointermove", move);
    s.layer.addEventListener("pointerup", end);
    s.layer.addEventListener("pointercancel", end);
  });

  s.layer.addEventListener("dblclick", (e) => {
    const i = hit(s.toImage(e));
    const shape = i === null ? undefined : shapes[i];
    if (i !== null && shape?.shape === "text") openEditor([shape.x, shape.y], i);
  });

  window.addEventListener("keydown", (e) => {
    if (editing || e.target instanceof HTMLInputElement) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      undo();
    } else if ((e.key === "Delete" || e.key === "Backspace") && selected !== null) {
      e.preventDefault();
      removeSelected();
    }
  });

  render();

  return {
    applyLabel: "Save annotated copy",
    params: () => (shapes.length ? { type: "annotate", shapes, refWidth: W, refHeight: H } : null),
  };
};

function translate(shape: Shape, dx: number, dy: number): Shape {
  switch (shape.shape) {
    case "rect":
    case "text":
      return { ...shape, x: shape.x + dx, y: shape.y + dy };
    case "arrow":
      return { ...shape, from: [shape.from[0] + dx, shape.from[1] + dy], to: [shape.to[0] + dx, shape.to[1] + dy] };
  }
}

import { h, svg } from "../lib/dom";
import { boxBetween, contains, moveWithin, type Box, type Point } from "../lib/geometry";
import type { Region } from "../lib/ipc";
import { History, surface } from "./surface";
import type { PanelFactory } from "./types";
import { hint, section, unavailable } from "./widgets";

interface Mark extends Box {
  page: number;
}

/** Black boxes drawn over the picture, kept in image pixels and sent as fractions. */
export const redactPanel: PanelFactory = (ctx) => {
  if (!ctx.image || !ctx.previewUrl) return unavailable(ctx, "This file cannot be previewed yet.");
  const W = ctx.image.width;
  const H = ctx.image.height;
  const s = surface(ctx, ctx.previewUrl, W, H);
  const layer = svg("g");
  s.layer.append(layer);
  s.frame.classList.add("surface--draw");
  s.frame.dataset.tool = "rect";

  let marks: Mark[] = [];
  let draft: Box | null = null;
  let selected: number | null = null;
  const page = 0;
  const history = new History<Mark[]>();

  const count = h("p", { class: "hint" });
  const undoButton = h("button", { type: "button" }, "Undo");
  const deleteButton = h("button", { type: "button" }, "Delete");
  const clearButton = h("button", { type: "button" }, "Clear all");
  undoButton.addEventListener("click", undo);
  deleteButton.addEventListener("click", removeSelected);
  clearButton.addEventListener("click", () => {
    if (!marks.length) return;
    history.push(marks);
    marks = [];
    selected = null;
    render();
  });

  const several = ctx.info.files.length > 1;
  ctx.panel.replaceChildren(
    section("Boxes", count, h("div", { class: "button-grid" }, undoButton, deleteButton, clearButton)),
    section(
      "",
      hint("Drag over anything that should be hidden. Drag a box to move it; Delete removes it."),
      hint("The copy has solid black boxes in place of what was under them, and none of the original file's metadata."),
      several ? hint("The same areas are blacked out in every file, scaled to its size.") : null,
    ),
  );

  function undo(): void {
    const previous = history.undo();
    if (!previous) return;
    marks = previous;
    selected = null;
    render();
  }

  function removeSelected(): void {
    if (selected === null) return;
    history.push(marks);
    marks.splice(selected, 1);
    selected = null;
    render();
  }

  function render(): void {
    const onPage = marks.map((m, i) => ({ m, i })).filter(({ m }) => m.page === page);
    layer.replaceChildren(
      ...onPage.map(({ m, i }) =>
        svg("rect", {
          x: m.x,
          y: m.y,
          width: m.w,
          height: m.h,
          class: i === selected ? "redact-box redact-box--selected" : "redact-box",
          "vector-effect": "non-scaling-stroke",
        }),
      ),
    );
    if (draft) layer.append(svg("rect", { x: draft.x, y: draft.y, width: draft.w, height: draft.h, class: "redact-box" }));
    count.textContent = marks.length === 1 ? "1 box" : `${marks.length} boxes`;
    undoButton.disabled = !history.canUndo;
    deleteButton.disabled = selected === null;
    clearButton.disabled = marks.length === 0;
    ctx.changed();
  }
  s.onLayout(render);

  function hit(p: Point): number | null {
    for (let i = marks.length - 1; i >= 0; i--) {
      const m = marks[i];
      if (m && m.page === page && contains(m, p, 4 / s.scale())) return i;
    }
    return null;
  }

  const clampPoint = (p: Point): Point => [Math.min(Math.max(p[0], 0), W), Math.min(Math.max(p[1], 0), H)];

  s.layer.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    e.preventDefault();
    s.layer.setPointerCapture(e.pointerId);
    const start = clampPoint(s.toImage(e));
    const index = hit(start);
    selected = index;
    let moved = false;
    let onMove: (p: Point) => void;
    let onEnd: () => void = () => undefined;
    const found = index === null ? undefined : marks[index];

    if (index !== null && found) {
      const from = { ...found };
      onMove = (p) => {
        if (!moved) history.push(marks);
        moved = true;
        marks[index] = { ...moveWithin(from, p[0] - start[0], p[1] - start[1], W, H), page: from.page };
        render();
      };
    } else {
      onMove = (p) => {
        moved = moved || Math.hypot(p[0] - start[0], p[1] - start[1]) * s.scale() > 3;
        if (moved) draft = boxBetween(start, clampPoint(p));
        render();
      };
      onEnd = () => {
        if (draft && draft.w > 0 && draft.h > 0) {
          history.push(marks);
          marks.push({ ...draft, page });
          selected = marks.length - 1;
        }
        draft = null;
        render();
      };
    }
    render();

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

  window.addEventListener("keydown", (e) => {
    if (e.target instanceof HTMLInputElement) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      undo();
    } else if ((e.key === "Delete" || e.key === "Backspace") && selected !== null) {
      e.preventDefault();
      removeSelected();
    }
  });

  render();

  const toRegion = (m: Mark): Region => ({ page: m.page, x: m.x / W, y: m.y / H, width: m.w / W, height: m.h / H });
  return {
    applyLabel: "Save redacted copy",
    params: () => (marks.length ? { type: "redact", regions: marks.map(toRegion) } : null),
  };
};

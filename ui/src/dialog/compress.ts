import { h } from "../lib/dom";
import { fileName, formatBytes, formatChange } from "../lib/format";
import { estimateCompress, type Compress, type JobFinished } from "../lib/ipc";
import type { PanelFactory } from "./types";
import { hint, section, segmented, showPreview, slider, unavailable } from "./widgets";

const HINTS: Partial<Record<string, string>> = {
  png: "PNG is lossless. Below 90%, colours are reduced to a palette, which may show banding in photos.",
  tiff: "TIFF stays lossless. Choose a target size to make it smaller: the picture is scaled down to fit.",
  avif: "AVIF takes a while to encode, especially for large pictures.",
  pdf: "Pictures in the PDF are recompressed, and below 90% large ones are scaled down. Text and drawings stay sharp.",
};

/** Wait this long after the slider stops before asking for a new estimate. */
const ESTIMATE_DELAY_MS = 180;

export const compressPanel: PanelFactory = (ctx) => {
  if (!ctx.image) return unavailable(ctx, "This file cannot be previewed yet.");
  showPreview(ctx);
  const original = ctx.image.bytes;
  let mode: Compress["mode"] = "quality";
  let percent = 70;
  /** Target as a share of the original size, in percent. */
  let share = 50;

  const target = (): Compress =>
    mode === "quality"
      ? { mode, percent }
      : { mode, bytes: Math.max(1024, Math.round((original * share) / 100)) };

  const quality = slider("Quality", 1, 100, percent);
  const size = slider("Target size", 5, 100, share);
  const estimateLine = h("dd", {}, "…");
  const results = h("ul", { class: "results" });

  const updateLabels = () => {
    quality.setLabel(`${percent}%`);
    const t = target();
    size.setLabel(t.mode === "size" ? `≤ ${formatBytes(t.bytes)}` : "");
    quality.element.hidden = mode !== "quality";
    size.element.hidden = mode !== "size";
  };

  let timer: number | undefined;
  let asked = 0;
  const refreshEstimate = () => {
    window.clearTimeout(timer);
    estimateLine.textContent = "…";
    timer = window.setTimeout(async () => {
      const ticket = ++asked;
      try {
        const { estimate } = await estimateCompress(target());
        if (ticket !== asked) return;
        let text = `≈ ${formatBytes(estimate.bytes)} (${formatChange(original, estimate.bytes)})`;
        if (estimate.scale < 0.999) text += `, scaled to ${Math.round(estimate.scale * 100)}%`;
        if (mode === "size" && estimate.percent > 0) text += ` at ${estimate.percent}% quality`;
        estimateLine.textContent = text;
      } catch (err) {
        if (ticket === asked) estimateLine.textContent = String(err);
      }
    }, ESTIMATE_DELAY_MS);
  };

  const onChange = () => {
    updateLabels();
    refreshEstimate();
    ctx.changed();
  };
  quality.input.addEventListener("input", () => {
    percent = Number(quality.input.value);
    onChange();
  });
  size.input.addEventListener("input", () => {
    share = Number(size.input.value);
    onChange();
  });

  const modes = segmented(
    "Compress by",
    [
      { value: "quality", text: "Percentage" },
      { value: "size", text: "Target size" },
    ],
    mode,
    (value) => {
      mode = value;
      onChange();
    },
  );

  const formatHint = HINTS[ctx.image.kind];
  ctx.panel.replaceChildren(
    section("Compress by", modes, quality.element, size.element, formatHint ? hint(formatHint) : null),
    section(
      "Size",
      h("dl", { class: "sizes" }, h("dt", {}, "Original"), h("dd", {}, formatBytes(original)), h("dt", {}, "Estimated"), estimateLine),
      ctx.info.files.length > 1 ? hint("The estimate is for the first file.") : null,
    ),
    results,
  );
  onChange();

  return {
    applyLabel: "Compress",
    params: () => ({ type: "compress", ...target() }),
    finished(result: JobFinished) {
      results.replaceChildren(
        ...result.outputs.map((o) =>
          h(
            "li",
            {},
            h("span", { class: "results__name", dir: "ltr" }, fileName(o.path)),
            h("span", {}, `${formatBytes(o.inputBytes)} → ${formatBytes(o.bytes)} (${formatChange(o.inputBytes, o.bytes)})`),
          ),
        ),
      );
    },
  };
};

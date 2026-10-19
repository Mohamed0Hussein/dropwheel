import { h } from "../lib/dom";
import { readMetadata, type MetadataField } from "../lib/ipc";
import type { PanelFactory } from "./types";
import { hint, section } from "./widgets";

export const metadataPanel: PanelFactory = async (ctx) => {
  const files = ctx.info.files;
  const table = h("div", { class: "meta" });
  const summary = h("p", { class: "meta__summary" });
  const warning = h("p", { class: "hint hint--warn" });
  let found = 0;

  async function show(index: number): Promise<void> {
    table.replaceChildren(h("p", { class: "stage__message" }, "Reading…"));
    let fields: MetadataField[];
    try {
      fields = await readMetadata(index);
    } catch (err) {
      table.replaceChildren(h("p", { class: "stage__message" }, String(err)));
      found = 0;
      ctx.changed();
      return;
    }
    found = fields.length;
    summary.textContent = found === 0 ? "No metadata found." : `${found} ${found === 1 ? "entry" : "entries"} found.`;
    warning.textContent = fields.some((f) => f.group === "Location") ? "This file records where it was taken." : "";
    warning.hidden = !warning.textContent;
    if (found === 0) {
      table.replaceChildren(h("p", { class: "stage__message" }, "This file carries no metadata."));
    } else {
      const rows: HTMLElement[] = [];
      let group = "";
      for (const f of fields) {
        if (f.group !== group) {
          group = f.group;
          rows.push(h("tr", { class: "meta__group" }, h("th", { colspan: 2, scope: "colgroup" }, group)));
        }
        rows.push(h("tr", {}, h("th", { scope: "row" }, f.name), h("td", { dir: "auto" }, f.value)));
      }
      table.replaceChildren(h("table", {}, h("tbody", {}, ...rows)));
    }
    ctx.changed();
  }

  const picker =
    files.length > 1
      ? h("select", { "aria-label": "File" }, ...files.map((f, i) => h("option", { value: i }, f.name)))
      : null;
  picker?.addEventListener("change", () => void show(Number(picker.value)));

  ctx.stage.replaceChildren(h("div", { class: "meta-stage" }, picker, table));
  ctx.panel.replaceChildren(
    section(
      "Metadata",
      summary,
      warning,
      hint("Removing keeps the picture and its colour profile, and takes out camera details, location, dates, comments and editing history."),
    ),
  );
  await show(0);

  return {
    applyLabel: "Remove all metadata",
    // With several files, some may have metadata even when the first has none.
    params: () => (found > 0 || files.length > 1 ? { type: "none" } : null),
  };
};

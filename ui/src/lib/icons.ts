// Line icons for the Tools wedges, drawn on a 24-unit grid. Parts marked `fill` are
// solid; the rest are strokes.

export interface IconPart {
  d: string;
  fill?: boolean;
}

export const TOOL_ICONS: Record<string, IconPart[]> = {
  compress: [{ d: "M12 2v6m-3-3 3 3 3-3M12 22v-6m-3 3 3-3 3 3M4 12h16" }],
  crop: [{ d: "M6 2v14a2 2 0 0 0 2 2h14M2 6h14a2 2 0 0 1 2 2v14" }],
  edit: [
    { d: "M4 6h8m4 0h4M4 12h2m4 0h10M4 18h10m4 0h2" },
    { d: "M12 6a2 2 0 1 0 4 0a2 2 0 1 0-4 0M6 12a2 2 0 1 0 4 0a2 2 0 1 0-4 0M14 18a2 2 0 1 0 4 0a2 2 0 1 0-4 0" },
  ],
  annotate: [{ d: "M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16v4zM13.5 6.5l4 4" }],
  redact: [{ d: "M3 5h7m4 0h7M3 19h18" }, { d: "M3 9.5h18v5H3z", fill: true }],
  metadata: [{ d: "M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18M12 11v6M12 7.5v.01" }],
  add_background: [{ d: "M12 3 3 7.5 12 12l9-4.5L12 3zM3 12l9 4.5 9-4.5M3 16.5 12 21l9-4.5" }],
};

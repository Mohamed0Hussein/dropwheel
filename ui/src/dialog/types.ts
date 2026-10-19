import type { DialogInfo, ImageInfo, JobFinished, Params } from "../lib/ipc";

/** What the dialog shell hands to a panel. */
export interface PanelContext {
  info: DialogInfo;
  /** The first file's image; null for files that have no preview yet (documents). */
  image: ImageInfo | null;
  /** Blob URL of the upright preview of the first file. */
  previewUrl: string | null;
  stage: HTMLElement;
  panel: HTMLElement;
  /** A panel calls this when its settings change, so the shell can refresh Apply. */
  changed(): void;
}

export interface Panel {
  /** Settings to run with, or null while there is nothing to apply. */
  params(): Params | null;
  applyLabel: string;
  /** For panels that report on the result, such as Compress. */
  finished?(result: JobFinished): void;
}

export type PanelFactory = (ctx: PanelContext) => Panel | Promise<Panel>;

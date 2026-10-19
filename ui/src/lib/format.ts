/** 1536 → "1.5 KB". Decimal-free below 10 of a unit is the usual file-size style. */
export function formatBytes(bytes: number): string {
  const units = ["bytes", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  if (unit === 0) return `${Math.round(value)} ${value === 1 ? "byte" : "bytes"}`;
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${units[unit]}`;
}

/** Change from `before` to `after` as a signed percentage, e.g. "−72%". */
export function formatChange(before: number, after: number): string {
  if (before <= 0) return "";
  const pct = Math.round(((after - before) / before) * 100);
  if (pct === 0) return "±0%";
  return pct < 0 ? `−${Math.abs(pct)}%` : `+${pct}%`;
}

export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

// Geometry shared with the Rust side, so a preview matches what gets saved.

export type Point = [number, number];

/**
 * Where an arrow's line ends, and its head: tip, then the two barbs. Mirrors
 * `arrow_geometry` in crates/core/src/actions/annotate.rs. Null for an arrow too short to
 * draw.
 */
export function arrowGeometry(from: Point, to: Point, stroke: number): { shaftEnd: Point; head: [Point, Point, Point] } | null {
  const dx = to[0] - from[0];
  const dy = to[1] - from[1];
  const len = Math.hypot(dx, dy);
  if (len < 0.5) return null;
  const ux = dx / len;
  const uy = dy / len;
  const head = Math.min(stroke * 4 + 6, len);
  const half = head * 0.5;
  const base: Point = [to[0] - ux * head, to[1] - uy * head];
  return {
    shaftEnd: [to[0] - ux * head * 0.8, to[1] - uy * head * 0.8],
    head: [to, [base[0] - uy * half, base[1] + ux * half], [base[0] + uy * half, base[1] - ux * half]],
  };
}

/** Distance from `p` to the segment `a`–`b`. */
export function distanceToSegment(p: Point, a: Point, b: Point): number {
  const dx = b[0] - a[0];
  const dy = b[1] - a[1];
  const len2 = dx * dx + dy * dy;
  const t = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / len2));
  return Math.hypot(p[0] - (a[0] + t * dx), p[1] - (a[1] + t * dy));
}

export interface Box {
  x: number;
  y: number;
  w: number;
  h: number;
}

/** The box between two corners, whichever way it was dragged. */
export function boxBetween(a: Point, b: Point): Box {
  return { x: Math.min(a[0], b[0]), y: Math.min(a[1], b[1]), w: Math.abs(b[0] - a[0]), h: Math.abs(b[1] - a[1]) };
}

export function contains(box: Box, p: Point, margin = 0): boolean {
  return p[0] >= box.x - margin && p[0] <= box.x + box.w + margin && p[1] >= box.y - margin && p[1] <= box.y + box.h + margin;
}

/** Moves `box` by `dx`, `dy` without leaving a `width` × `height` area. */
export function moveWithin(box: Box, dx: number, dy: number, width: number, height: number): Box {
  return {
    ...box,
    x: Math.min(Math.max(box.x + dx, 0), Math.max(0, width - box.w)),
    y: Math.min(Math.max(box.y + dy, 0), Math.max(0, height - box.h)),
  };
}

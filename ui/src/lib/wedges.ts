// Geometry for the radial menu, in SVG user units around (0, 0). Angles are in radians,
// clockwise from twelve o'clock, so the first wedge sits at the top.

export interface Ring {
  inner: number;
  outer: number;
}

export interface Point {
  x: number;
  y: number;
}

const round = (n: number) => Math.round(n * 100) / 100 + 0; // + 0 turns -0 into 0

function polar(r: number, angle: number): Point {
  return { x: round(r * Math.sin(angle)), y: round(-r * Math.cos(angle)) };
}

const pt = (r: number, angle: number) => {
  const p = polar(r, angle);
  return `${p.x} ${p.y}`;
};

/**
 * The outline of wedge `index` out of `count` on `ring`. `gap` is the space between
 * neighbouring wedges; it is measured along each edge, so it has the same width at the
 * inside and outside of the ring.
 */
export function wedgePath(ring: Ring, index: number, count: number, gap: number): string {
  if (count <= 1) return annulus(ring);
  const step = (2 * Math.PI) / count;
  const start = index * step - step / 2;
  const end = start + step;
  const inset = (r: number) => gap / 2 / r;
  const [o0, o1] = [start + inset(ring.outer), end - inset(ring.outer)];
  const [i0, i1] = [start + inset(ring.inner), end - inset(ring.inner)];
  const large = (from: number, to: number) => (to - from > Math.PI ? 1 : 0);
  return [
    `M${pt(ring.outer, o0)}`,
    `A${ring.outer} ${ring.outer} 0 ${large(o0, o1)} 1 ${pt(ring.outer, o1)}`,
    `L${pt(ring.inner, i1)}`,
    `A${ring.inner} ${ring.inner} 0 ${large(i0, i1)} 0 ${pt(ring.inner, i0)}`,
    "Z",
  ].join(" ");
}

/**
 * A whole ring, for a menu with one action. An arc cannot start and end on the same
 * point, so each circle is drawn as two halves. Fill it with fill-rule="evenodd".
 */
function annulus({ inner, outer }: Ring): string {
  const circle = (r: number) => `M${r} 0 A${r} ${r} 0 1 1 ${-r} 0 A${r} ${r} 0 1 1 ${r} 0 Z`;
  return `${circle(outer)} ${circle(inner)}`;
}

/** Where a wedge's label or icon goes: halfway across the ring, mid-wedge. */
export function wedgeCentre(ring: Ring, index: number, count: number): Point {
  const r = (ring.inner + ring.outer) / 2;
  return polar(r, count <= 1 ? 0 : (index * 2 * Math.PI) / count);
}

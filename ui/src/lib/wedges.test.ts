// Run with `npm test` (Node's built-in test runner; Node strips the types).
import assert from "node:assert/strict";
import { test } from "node:test";

import { wedgeCentre, wedgePath } from "./wedges.ts";

const ring = { inner: 60, outer: 120 };

test("four wedges are centred on the compass points", () => {
  assert.deepEqual(wedgeCentre(ring, 0, 4), { x: 0, y: -90 });
  assert.deepEqual(wedgeCentre(ring, 1, 4), { x: 90, y: 0 });
  assert.deepEqual(wedgeCentre(ring, 2, 4), { x: 0, y: 90 });
  assert.deepEqual(wedgeCentre(ring, 3, 4), { x: -90, y: 0 });
});

test("a wedge spans its share of the ring, less the gap", () => {
  // No gap: the top wedge of four runs from -45° to 45° on the outer edge.
  const d = wedgePath(ring, 0, 4, 0);
  assert.ok(d.startsWith("M-84.85 -84.85 A120 120 0 0 1 84.85 -84.85"), d);
  assert.ok(d.includes("L42.43 -42.43 A60 60 0 0 0 -42.43 -42.43"), d);

  // A gap pulls both ends in; the outer edge loses less angle than the inner.
  const gapped = wedgePath(ring, 0, 4, 10);
  const firstX = Number(gapped.slice(1).split(" ")[0]);
  assert.ok(firstX > -84.85 && firstX < -80, gapped);
});

test("two wedges never need the large-arc flag", () => {
  for (const i of [0, 1]) assert.ok(!/ 0 1 1 /.test(wedgePath(ring, i, 2, 4)));
});

test("a single action fills the whole ring", () => {
  const d = wedgePath(ring, 0, 1, 4);
  assert.equal((d.match(/M/g) ?? []).length, 2, d);
  assert.ok(d.includes("M120 0") && d.includes("M60 0"), d);
  assert.deepEqual(wedgeCentre(ring, 0, 1), { x: 0, y: -90 });
});

test("no negative zero in path data", () => {
  assert.ok(!wedgePath(ring, 2, 4, 0).includes("-0 "));
});

import assert from "node:assert/strict";
import { test } from "node:test";

import { arrowGeometry, boxBetween, contains, distanceToSegment, moveWithin } from "./geometry.ts";

test("arrow heads match the Rust geometry", () => {
  const geo = arrowGeometry([10, 50], [90, 50], 4);
  assert.ok(geo);
  // head = 4 * 4 + 6 = 22, half = 11, base at x = 68.
  assert.deepEqual(geo.head, [
    [90, 50],
    [68, 61],
    [68, 39],
  ]);
  assert.deepEqual(geo.shaftEnd, [90 - 22 * 0.8, 50]);
  assert.equal(arrowGeometry([1, 1], [1, 1], 4), null);
});

test("short arrows are all head", () => {
  const geo = arrowGeometry([0, 0], [10, 0], 4);
  assert.ok(geo);
  assert.deepEqual(geo.head[1], [0, 5]);
});

test("segment distance", () => {
  assert.equal(distanceToSegment([5, 3], [0, 0], [10, 0]), 3);
  assert.equal(distanceToSegment([-4, 3], [0, 0], [10, 0]), 5);
  assert.equal(distanceToSegment([2, 2], [2, 2], [2, 2]), 0);
});

test("boxes", () => {
  assert.deepEqual(boxBetween([10, 10], [4, 2]), { x: 4, y: 2, w: 6, h: 8 });
  assert.ok(contains({ x: 0, y: 0, w: 10, h: 10 }, [11, 5], 2));
  assert.ok(!contains({ x: 0, y: 0, w: 10, h: 10 }, [13, 5], 2));
  assert.deepEqual(moveWithin({ x: 5, y: 5, w: 10, h: 10 }, 100, -100, 20, 20), { x: 10, y: 0, w: 10, h: 10 });
});

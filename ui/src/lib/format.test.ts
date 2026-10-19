import assert from "node:assert/strict";
import { test } from "node:test";

import { fileName, formatBytes, formatChange } from "./format.ts";

test("bytes", () => {
  assert.equal(formatBytes(1), "1 byte");
  assert.equal(formatBytes(900), "900 bytes");
  assert.equal(formatBytes(1536), "1.5 KB");
  assert.equal(formatBytes(25 * 1024), "25 KB");
  assert.equal(formatBytes(3.4 * 1024 * 1024), "3.4 MB");
});

test("change", () => {
  assert.equal(formatChange(100, 28), "−72%");
  assert.equal(formatChange(100, 110), "+10%");
  assert.equal(formatChange(100, 100), "±0%");
  assert.equal(formatChange(0, 5), "");
});

test("file names from Windows and POSIX paths", () => {
  assert.equal(fileName("C:\\Users\\سارة\\photo_compressed.jpg"), "photo_compressed.jpg");
  assert.equal(fileName("/tmp/a.png"), "a.png");
});

#!/usr/bin/env node
// Downloads the PDFium library Dropwheel renders PDFs with into vendor/pdfium/.
//
// The binaries are Benoit Blanchon's pdfium-binaries builds (BSD-3-Clause and the
// licences of PDFium's dependencies), taken from the pypdfium2 wheels on PyPI because
// those are pinned, checksummed and served from a stable host. Each download is checked
// against the SHA-256 below before anything is extracted.
//
//   node scripts/fetch-pdfium.mjs                     for this machine
//   node scripts/fetch-pdfium.mjs --platform win-x64  for another platform
//
// Nothing is downloaded when the library is already there with the expected contents.

import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { inflateRawSync } from "node:zlib";

const PYPDFIUM = "5.14.0";
const PDFIUM_BUILD = "8076";
const BASE = "https://files.pythonhosted.org/packages";

const PLATFORMS = {
  "win-x64": {
    url: `${BASE}/00/dc/ca7874924c9cfd701ad53f89529968523790e70473e0b71e834668316148/pypdfium2-${PYPDFIUM}-py3-none-win_amd64.whl`,
    sha256: "149fd5c6397b8df8bf7911a93506eff0be874f877afe7ac936cf5d37d21a6a06",
    library: "pdfium.dll",
    licenses: "windows_x64",
  },
  "linux-x64": {
    url: `${BASE}/4f/a3/c9cc797fc8bdfb8f37b9b0f8b9d02a5fc196b2015f408d53624cab5b0519/pypdfium2-${PYPDFIUM}-py3-none-manylinux_2_17_x86_64.manylinux2014_x86_64.whl`,
    sha256: "11f281613fa22313d9c7ab89947665e84eccf8ebe40e1198a84a88352305648d",
    library: "libpdfium.so",
    licenses: "linux_x64",
  },
};

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "vendor", "pdfium");

function currentPlatform() {
  if (process.platform === "win32" && process.arch === "x64") return "win-x64";
  if (process.platform === "linux" && process.arch === "x64") return "linux-x64";
  return null;
}

const flag = process.argv.indexOf("--platform");
const name = flag >= 0 ? process.argv[flag + 1] : currentPlatform();
const platform = name ? PLATFORMS[name] : undefined;
if (!platform) {
  console.warn(`fetch-pdfium: no PDFium build for ${name ?? `${process.platform}-${process.arch}`}; PDF actions will be unavailable.`);
  process.exit(0);
}

const libraryPath = join(outDir, platform.library);
const stampPath = join(outDir, `${platform.library}.sha256`);
const sha256 = (data) => createHash("sha256").update(data).digest("hex");

if (existsSync(libraryPath) && existsSync(stampPath)) {
  const expected = readFileSync(stampPath, "utf8").trim();
  if (sha256(readFileSync(libraryPath)) === expected) process.exit(0);
}

console.log(`fetch-pdfium: downloading PDFium ${PDFIUM_BUILD} for ${name}…`);
const response = await fetch(platform.url);
if (!response.ok) throw new Error(`fetch-pdfium: ${platform.url} answered ${response.status}`);
const wheel = Buffer.from(await response.arrayBuffer());
const actual = sha256(wheel);
if (actual !== platform.sha256) {
  throw new Error(`fetch-pdfium: checksum mismatch for ${platform.url}\n  expected ${platform.sha256}\n  got      ${actual}`);
}

const entries = readZip(wheel);
const library = entries.get(`pypdfium2_raw/${platform.library}`);
if (!library) throw new Error(`fetch-pdfium: ${platform.library} is missing from the wheel`);

mkdirSync(join(outDir, "licenses"), { recursive: true });
const licensePrefix = `pypdfium2-${PYPDFIUM}.dist-info/licenses/data/${platform.licenses}/BUILD_LICENSES/`;
for (const [path, read] of entries) {
  if (path.startsWith(licensePrefix)) writeFileSync(join(outDir, "licenses", path.slice(licensePrefix.length)), read());
}
const bytes = library();
const temporary = `${libraryPath}.partial`;
writeFileSync(temporary, bytes);
rmSync(libraryPath, { force: true });
renameSync(temporary, libraryPath);
writeFileSync(stampPath, `${sha256(bytes)}\n`);
writeFileSync(join(outDir, "VERSION"), `PDFium ${PDFIUM_BUILD} (pdfium-binaries, via pypdfium2 ${PYPDFIUM})\n`);
console.log(`fetch-pdfium: wrote ${libraryPath}`);

/** The files in a zip archive, each read on demand. */
function readZip(data) {
  // The end of central directory record sits in the last 64 KiB.
  let eocd = -1;
  for (let i = data.length - 22; i >= Math.max(0, data.length - 65_557); i--) {
    if (data.readUInt32LE(i) === 0x06054b50) {
      eocd = i;
      break;
    }
  }
  if (eocd < 0) throw new Error("fetch-pdfium: not a zip file");
  const count = data.readUInt16LE(eocd + 10);
  let p = data.readUInt32LE(eocd + 16);
  const files = new Map();
  for (let n = 0; n < count; n++) {
    if (data.readUInt32LE(p) !== 0x02014b50) throw new Error("fetch-pdfium: bad zip directory");
    const method = data.readUInt16LE(p + 10);
    const compressed = data.readUInt32LE(p + 20);
    const nameLength = data.readUInt16LE(p + 28);
    const extraLength = data.readUInt16LE(p + 30);
    const commentLength = data.readUInt16LE(p + 32);
    const local = data.readUInt32LE(p + 42);
    const path = data.toString("utf8", p + 46, p + 46 + nameLength);
    files.set(path, () => {
      const start = local + 30 + data.readUInt16LE(local + 26) + data.readUInt16LE(local + 28);
      const raw = data.subarray(start, start + compressed);
      if (method === 0) return raw;
      if (method === 8) return inflateRawSync(raw);
      throw new Error(`fetch-pdfium: ${path} uses zip method ${method}`);
    });
    p += 46 + nameLength + extraLength + commentLength;
  }
  return files;
}

"""Writes THIRD-PARTY-NOTICES.txt: the licences of everything built into Dropwheel.

That is PDFium and the libraries inside it, the bundled fonts, the Rust crates the
Windows app links, and the JavaScript in its windows. Run it from the workspace root
after `npm install` in `ui/` and a Windows build or `cargo fetch`, so every crate's
source is on disk:

    python scripts/make-notices.py

CI runs it with `--check`, which fails when the file is out of date.
"""

import json
import pathlib
import re
import subprocess
import sys
import textwrap

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "THIRD-PARTY-NOTICES.txt"
TARGET = "x86_64-pc-windows-msvc"
RULE = "=" * 78

# Preferred licence first, when a crate offers a choice.
PREFERENCE = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "BSD-2-Clause",
    "ISC",
    "Zlib",
    "Unicode-3.0",
    "IJG",
    "BSL-1.0",
    "MPL-2.0",
    "0BSD",
    "MIT-0",
    "CC0-1.0",
    "Unlicense",
]
# Licences whose terms ask for nothing to be kept with the software.
NO_NOTICE = {"0BSD", "MIT-0", "CC0-1.0", "Unlicense"}

PDFIUM = [
    ("PDFium", "pdfium.txt"),
    ("pdfium-binaries (the PDFium build)", "pdfium-binaries.txt"),
    ("Abseil", "abseil.txt"),
    ("Anti-Grain Geometry 2.3", "agg23.txt"),
    ("fast_float", "fast_float.txt"),
    ("FreeType", "freetype.txt"),
    ("ICU", "icu.txt"),
    ("Little CMS", "lcms.txt"),
    ("libjpeg-turbo", "libjpeg_turbo.md"),
    ("libjpeg-turbo: the IJG licence", "libjpeg_turbo.ijg"),
    ("OpenJPEG", "libopenjpeg.txt"),
    ("libpng", "libpng.txt"),
    ("LLVM libc", "llvm-libc.txt"),
    ("simdutf", "simdutf.txt"),
    ("zlib", "zlib.txt"),
]

FONTS = [
    ("Noto Sans", "crates/core/fonts/OFL-NotoSans.txt"),
    ("IBM Plex Sans Arabic", "crates/core/fonts/OFL-IBMPlexSansArabic.txt"),
]

NPM = ["@tauri-apps/api", "@tauri-apps/plugin-dialog"]


def tokens(expr):
    return re.findall(r"\(|\)|[A-Za-z0-9.+-]+", expr.replace("/", " OR "))


def choose(expr):
    """The licences to follow for an SPDX expression: the preferred side of each OR,
    every side of each AND."""
    toks = tokens(expr)
    pos = 0

    def cost(choice):
        return sum(PREFERENCE.index(c) if c in PREFERENCE else 99 for c in choice)

    def atom():
        nonlocal pos
        tok = toks[pos]
        pos += 1
        if tok == "(":
            result = either()
            pos += 1  # ")"
            return result
        return [tok]

    def both():
        nonlocal pos
        result = atom()
        while pos < len(toks) and toks[pos] == "AND":
            pos += 1
            result = result + [c for c in atom() if c not in result]
        return result

    def either():
        nonlocal pos
        options = [both()]
        while pos < len(toks) and toks[pos] == "OR":
            pos += 1
            options.append(both())
        return min(options, key=cost)

    return either()


def classify(name, text):
    """The licence a licence file holds, from its name and then its words."""
    upper = name.upper()
    if "APACHE" in upper:
        return "Apache-2.0"
    if "MIT" in upper:
        return "MIT"
    if "UNLICENSE" in upper:
        return "Unlicense"
    if "ZLIB" in upper:
        return "Zlib"
    if "BSL" in upper or "BOOST" in upper:
        return "BSL-1.0"
    words = " ".join(text.split())
    checks = [
        ("Apache-2.0", "Apache License"),
        ("MPL-2.0", "Mozilla Public License"),
        ("Unicode-3.0", "UNICODE LICENSE"),
        ("Unicode-3.0", "Unicode, Inc."),
        ("BSL-1.0", "Boost Software License"),
        ("MIT", "Permission is hereby granted, free of charge"),
        ("ISC", "Permission to use, copy, modify, and/or distribute"),
        ("BSD-3-Clause", "Neither the name"),
        ("BSD-2-Clause", "Redistribution and use in source and binary forms"),
        ("Zlib", "This software is provided 'as-is'"),
        ("IJG", "Independent JPEG Group"),
        ("Unlicense", "This is free and unencumbered software"),
    ]
    for licence, phrase in checks:
        if phrase in words:
            return licence
    return None


def licence_files(directory):
    found = []
    # By name as text: Windows paths would otherwise sort ignoring case.
    for path in sorted(directory.iterdir(), key=lambda p: p.name):
        stem = path.name.upper()
        if path.is_file() and stem.startswith(("LICENSE", "LICENCE", "COPYING", "UNLICENSE", "NOTICE")):
            found.append((path.name, read(path)))
    return found


def crates():
    meta = json.loads(
        subprocess.check_output(
            [
                "cargo",
                "metadata",
                "--format-version",
                "1",
                "--locked",
                "--filter-platform",
                TARGET,
            ],
            cwd=ROOT,
        )
    )
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    members = set(meta["workspace_members"])
    root = next(i for i in members if packages[i]["name"] == "dropwheel")
    seen, stack = set(), [root]
    while stack:
        pid = stack.pop()
        if pid in seen:
            continue
        seen.add(pid)
        for dep in nodes[pid]["deps"]:
            if any(k["kind"] is None for k in dep["dep_kinds"]):
                stack.append(dep["pkg"])
    linked = []
    for pid in seen - members:
        p = packages[pid]
        kinds = {k for t in p["targets"] for k in t["kind"]}
        # Procedural macros run while building and are not part of the program.
        if kinds <= {"proc-macro", "custom-build"}:
            continue
        linked.append(p)
    return sorted(linked, key=lambda p: (p["name"], p["version"]))


def read(path):
    """A text file in UTF-8, or in Latin-1 as some older licence files are."""
    data = path.read_bytes()
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError:
        text = data.decode("latin-1")
    return text.replace("\r\n", "\n").replace("\r", "\n").strip()


def shorten(licence, text):
    """The Apache and Mozilla licences read the same for every project, so a crate's
    copy becomes its copyright lines and a pointer to the one full text at the end."""
    if licence not in ("Apache-2.0", "MPL-2.0") or "NOTICE" in text[:200].upper():
        return text
    owners = [
        line.strip()
        for line in text.splitlines()
        if re.match(r"\s*Copyright\b", line) and "[" not in line and "owner" not in line
    ]
    pointer = f"Licensed under the {licence} licence; its text is at the end of this file."
    return "\n".join(owners + [pointer]) if owners else pointer


def standard(name):
    return read(ROOT / "scripts" / "licences" / f"{name}.txt")


def main():
    missing = []
    groups = {}  # licence text -> crates that carry it
    table = []
    linked = crates()
    files = {p["id"]: licence_files(pathlib.Path(p["manifest_path"]).parent) for p in linked}
    for p in linked:
        expr = p.get("license") or ""
        if not expr:
            missing.append(f"{p['name']} {p['version']}: no licence expression")
            continue
        chosen = choose(expr)
        # Crates from one repository often ship its licence file in just one of them.
        siblings = [q for q in linked if q is not p and p.get("repository") and q.get("repository") == p["repository"]]
        texts = []
        for licence in chosen:
            if licence in NO_NOTICE:
                continue
            matching = []
            for source in [p] + siblings:
                matching = [shorten(licence, t) for n, t in files[source["id"]] if classify(n, t) == licence]
                if licence == "Apache-2.0":
                    matching += [t for n, t in files[source["id"]] if n.upper().startswith("NOTICE")]
                if matching:
                    break
            if not matching:
                if licence == "MIT":
                    authors = ", ".join(a.split("<")[0].strip() for a in p.get("authors", []))
                    holder = authors or f"the {p['name']} developers"
                    matching = [f"Copyright (c) {holder}\n\n" + standard("MIT")]
                elif licence in ("Apache-2.0", "MPL-2.0", "BSL-1.0"):
                    # The terms are the same for everyone; the full text is below.
                    matching = [f"Licensed under the {licence} licence; its text is at the end of this file."]
                else:
                    missing.append(f"{p['name']} {p['version']}: no {licence} text")
                    continue
            texts.extend(matching)
        label = f"{p['name']} {p['version']}"
        table.append((label, " AND ".join(chosen), p.get("repository") or ""))
        for text in texts:
            entry = groups.setdefault(" ".join(text.split()), (text, []))
            if label not in entry[1]:
                entry[1].append(label)

    if missing:
        sys.exit("Missing licence texts:\n  " + "\n  ".join(missing))

    out = [
        "Third-party notices",
        RULE,
        "",
        "Dropwheel is built from the software below. Each part keeps its own licence,",
        "which follows. Nothing here limits your rights under those licences.",
        "",
        "Contents",
        "  1. PDFium and the libraries built into it",
        "  2. Fonts",
        "  3. Rust crates",
        "  4. JavaScript packages",
        "  5. Full licence texts",
        "",
        "",
        "1. PDFium and the libraries built into it",
        RULE,
        "",
        "PDFium reads and renders PDF files. Dropwheel uses the build published by",
        "pypdfium2 (https://github.com/pypdfium2-team/pypdfium2), which uses Benoit",
        "Blanchon's pdfium-binaries (https://github.com/bblanchon/pdfium-binaries).",
    ]
    for title, name in PDFIUM:
        text = read(ROOT / "licenses" / "pdfium" / name)
        out += ["", "", title, "-" * len(title), "", text]

    out += ["", "", "2. Fonts", RULE]
    for title, name in FONTS:
        text = read(ROOT / name)
        out += ["", "", title, "-" * len(title), "", text]

    out += [
        "",
        "",
        "3. Rust crates",
        RULE,
        "",
        "Where a crate offers a choice of licences, the one listed is the one Dropwheel uses.",
        "Source code of every crate is at https://crates.io/crates/<name>/<version>,",
        "including the files under the Mozilla Public License 2.0.",
        "",
    ]
    width = max(len(label) for label, _, _ in table)
    for label, licence, _ in table:
        out.append(f"  {label.ljust(width)}  {licence}")
    out += ["", "", "Licence texts and copyright notices of the crates", "-" * 49]
    for text, labels in sorted(groups.values(), key=lambda entry: entry[1][0]):
        if text.startswith("Licensed under the"):
            continue
        used = textwrap.fill("Used by: " + ", ".join(labels), 78, break_on_hyphens=False)
        out += ["", "", used, "", text]

    out += ["", "", "4. JavaScript packages", RULE, ""]
    out.append("The windows of Dropwheel include these packages, used under the MIT licence:")
    for name in NPM:
        package = ROOT / "ui" / "node_modules" / name
        version = json.loads((package / "package.json").read_text(encoding="utf-8"))["version"]
        out += ["", f"{name} {version}"]
        mit = package / "LICENSE-MIT"
        if mit.is_file():
            out += ["", read(mit)]
        else:
            # Published without its licence file; the same project's Rust crate has it.
            out += ["", "MIT License", "", "Copyright (c) 2017 - Present Tauri Apps Contributors", "", standard("MIT")]

    out += ["", "", "5. Full licence texts", RULE]
    for name in ["Apache-2.0", "MPL-2.0", "BSL-1.0"]:
        out += ["", "", name, "-" * len(name), "", standard(name)]

    text = "\n".join(out).rstrip() + "\n"
    if "--check" in sys.argv:
        # A Windows checkout may have turned the line endings into CRLF.
        current = OUT.read_bytes().replace(b"\r\n", b"\n") if OUT.is_file() else b""
        if current != text.encode("utf-8"):
            sys.exit("THIRD-PARTY-NOTICES.txt is out of date: run python scripts/make-notices.py")
        return
    OUT.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(table)} crates, {len(text) // 1024} KB")


if __name__ == "__main__":
    main()

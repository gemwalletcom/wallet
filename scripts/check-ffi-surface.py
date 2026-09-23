#!/usr/bin/env python3
"""Report exported Core functions that no app calls.

Every `#[uniffi::export] pub fn` becomes a symbol in both generated bindings,
so one that no app names is Swift and Kotlin nobody reads and a contract
nobody honours. The surface only grows back silently, which is why this is a
check rather than a sweep somebody remembers to re-run.

A function that Core itself uses keeps its body and loses only the export.
"""

import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / "core/gemstone/src"
APPS = ["ios/Features", "ios/Packages", "ios/Gem", "ios/GemPriceWidget", "android"]
SKIP = [
    "--exclude-dir=build",
    "--exclude-dir=.build",
    "--exclude-dir=generated",
    "--exclude-dir=Submodules",
    "--exclude-dir=.gradle",
    "--exclude=Gemstone.swift",
    "--exclude=gemstone.kt",
]
EXPORTED = re.compile(r"#\[uniffi::export[^\]]*\]\s*\npub (?:async )?fn (\w+)")


def camel(name):
    head, *rest = name.split("_")
    return head + "".join(part.capitalize() for part in rest)


def exported_functions():
    for path in sorted(CORE.rglob("*.rs")):
        for match in EXPORTED.finditer(path.read_text()):
            yield match.group(1), path.relative_to(ROOT)


def is_called(name):
    found = subprocess.run(
        ["grep", "-rlF", *SKIP, camel(name), *APPS],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    return bool(found.stdout.strip())


def main():
    functions = sorted(set(exported_functions()))
    unused = [(name, path) for name, path in functions if not is_called(name)]

    for name, path in unused:
        print(f"  {path} -> {name} ({camel(name)}) is exported but no app calls it")
    print(f"checked {len(functions)} exported functions")

    return 1 if unused else 0


if __name__ == "__main__":
    sys.exit(main())

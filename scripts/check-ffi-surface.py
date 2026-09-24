#!/usr/bin/env python3
"""Report exported Core functions and methods that no app calls.

Every `#[uniffi::export]` function, and every method of an exported `impl`
block, becomes a symbol in both generated bindings, so one that no app calls
is Swift and Kotlin nobody reads and a contract nobody honours. A call from a
test, a mock or a preview does not count: those keep a symbol alive for
themselves. The surface only grows back silently, which is why this is a check
rather than a sweep somebody remembers to re-run.

A function that Core itself uses keeps its body and loses only the export.
An entry in ALLOWED names the open TODO item that removes it, or the test
double that needs it to answer as Core does.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / "core/gemstone/src"
APPS = ["ios/Features", "ios/Packages", "ios/Gem", "ios/GemPriceWidget", "android"]
SKIPPED_DIRS = {"build", ".build", "generated", "Submodules", ".gradle"}
SKIPPED_FILES = {"Gemstone.swift", "gemstone.kt"}
TEST_MARKERS = ["/Tests/", "/TestKit/", "/src/test/", "/src/androidTest/", "/testkit/", "/testFixtures/", "Mock"]
EXPORT = re.compile(r"#\[uniffi::export\]\s*\n\s*((?:pub(?:\([^)]*\))? )?(?:async )?fn (\w+)|impl (\w+) \{)")
METHOD = re.compile(r"^    (?:pub(?:\([^)]*\))? )?(?:async )?fn (\w+)", re.MULTILINE)
CALL = re.compile(r"(\bfunc\s+|\bfun\s+)?\b([a-z][A-Za-z0-9]*)\(")
DOUBLE = "a test double answers it as Core does"
ALLOWED = {
    "GemKeystore.create_store": "VM181",
    "GemKeystore.export_private_key": "VM181",
    "GemKeystore.export_recovery_phrase": "VM181",
    "GemConfirmation.fee_rate_rows": DOUBLE,
    "GemConfirmation.row_contents": DOUBLE,
    "GemConfirmScreen.button": DOUBLE,
    "GemConfirmScreen.fee_row": DOUBLE,
    "confirm_error_info": DOUBLE,
}


def camel(name):
    head, *rest = name.split("_")
    return head + "".join(part.capitalize() for part in rest)


def impl_body(source, start):
    depth = 0
    for index in range(start, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[start:index]
    return source[start:]


def exported():
    for path in sorted(CORE.rglob("*.rs")):
        source = path.read_text()
        relative = path.relative_to(ROOT)
        for match in EXPORT.finditer(source):
            if match.group(2):
                yield match.group(2), None, relative
                continue
            body = impl_body(source, match.end() - 1)
            constructors = set(re.findall(r"#\[uniffi::constructor[^\]]*\]\s*\n\s*(?:pub )?(?:async )?fn (\w+)", body))
            for method in METHOD.finditer(body):
                if method.group(1) not in constructors:
                    yield method.group(1), match.group(3), relative


def app_files():
    for app in APPS:
        for path in (ROOT / app).rglob("*"):
            if path.suffix not in (".swift", ".kt") or path.name in SKIPPED_FILES:
                continue
            if SKIPPED_DIRS.intersection(path.relative_to(ROOT).parts):
                continue
            yield path


def is_preview(lines, index):
    for line in reversed(lines[: index + 1]):
        if line and not line[0].isspace() and line[0] not in "})]":
            start = lines.index(line)
            return any(re.search(r"#Preview\b|@Preview\b|: PreviewProvider\b", candidate) for candidate in lines[max(0, start - 3) : start + 1])
    return False


def callers():
    found = {}
    for path in app_files():
        relative = "/" + str(path.relative_to(ROOT))
        in_tests = any(marker in relative for marker in TEST_MARKERS)
        lines = path.read_text(errors="ignore").splitlines()
        for index, line in enumerate(lines):
            for match in CALL.finditer(line):
                if match.group(1):
                    continue
                kind = "test" if in_tests else "preview" if is_preview(lines, index) else "app"
                found.setdefault(match.group(2), set()).add(kind)
    return found


def main():
    functions = sorted(set(exported()), key=lambda item: (str(item[2]), item[1] or "", item[0]))
    calls = callers()
    reported = 0
    for name, owner, path in functions:
        kinds = calls.get(camel(name), set())
        if "app" in kinds:
            continue
        label = f"{owner}.{name}" if owner else name
        if label in ALLOWED:
            continue
        reason = f"only {' and '.join(sorted(kinds))} code calls it" if kinds else "no app calls it"
        print(f"  {path} -> {label} ({camel(name)}): {reason}")
        reported += 1
    print(f"checked {len(functions)} exported functions and methods")
    return 1 if reported else 0


if __name__ == "__main__":
    sys.exit(main())

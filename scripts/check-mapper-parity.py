#!/usr/bin/env python3
"""Compare the English string each app maps a Core enum variant to.

Core names a row or a state; each app maps that name to its own localized
string in one mapper file. When the two mappers reach for different keys the
apps show different words for the same Core decision, which is invisible in
either app on its own.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SKIP = ("/Submodules/", "/build/", "/.build/", "/target/", "DerivedData")
MINIMUM_COMPARED = 512
SWIFT_MAPPER = ROOT / "ios/Packages/PrimitivesComponents/Sources/Extensions/Gemstone+Localized.swift"
KOTLIN_MAPPER = ROOT / "android/ui/src/main/kotlin/com/gemwallet/android/ui/localization/GemstoneText.kt"
MAPPER_NAMES = {
    "ios": ("Gemstone+Localized.swift", "Gemstone+Style.swift"),
    "android": ("GemstoneText.kt", "GemstoneStyle.kt"),
}
EXPECTED = {
    ROOT / "ios/Packages/PrimitivesComponents/Sources/Extensions/Gemstone+Localized.swift",
    ROOT / "ios/Packages/PrimitivesComponents/Sources/Extensions/Gemstone+Style.swift",
    ROOT / "android/ui/src/main/kotlin/com/gemwallet/android/ui/localization/GemstoneText.kt",
    ROOT / "android/ui/src/main/kotlin/com/gemwallet/android/ui/style/GemstoneStyle.kt",
}


def stray_mappers():
    found = set()
    for app, names in MAPPER_NAMES.items():
        for name in names:
            for path in ROOT.joinpath(app).rglob(name):
                if not any(part in str(path) for part in SKIP):
                    found.add(path)
    return sorted(found - EXPECTED)


def english_by_key():
    text = (ROOT / "localization/app/en.ftl").read_text()
    return dict(re.findall(r"^([a-z0-9_]+) = (.*)$", text, re.M))


def key_by_swift_path():
    paths = {}
    stack, depths, brace, pending = [], [], 0, None
    for line in (ROOT / "ios/Packages/Localization/Sources/Localized.swift").read_text().split("\n"):
        opened = re.search(r"public enum (\w+) \{", line)
        if opened:
            stack.append(opened.group(1))
            depths.append(brace)
        declared = re.search(r"public static (?:let|var|func) (`?\w+`?)", line)
        if declared:
            pending = declared.group(1).strip("`")
        used = re.search(r'Localized\.tr\("Localizable", "([^"]+)"', line)
        if used and pending:
            paths[".".join(stack + [pending])] = used.group(1).replace(".", "_")
        brace += line.count("{") - line.count("}")
        while depths and brace <= depths[-1]:
            stack.pop()
            depths.pop()
    return paths


def variant(name):
    return name.replace("_", "").upper()


def swift_mappings(paths):
    found = {}
    for path in [SWIFT_MAPPER]:
        current = None
        for line in path.read_text().split("\n"):
            opened = re.search(r"^(?:public |internal |private |fileprivate )?extension (?:Gemstone\.)?(\w+)", line.strip())
            if opened:
                current = opened.group(1)
            arm = re.match(r"case (?:let )?([^:]+):\s*(Localized\.[\w.]+)", line.strip())
            if current and arm:
                key = paths.get(arm.group(2))
                if not key:
                    continue
                for case in arm.group(1).split(","):
                    name = re.match(r"\s*\.(\w+)", case)
                    if name:
                        found[(current, variant(name.group(1)))] = key
    return found


def kotlin_mappings():
    found = {}
    for path in [KOTLIN_MAPPER]:
        current = None
        for line in path.read_text().split("\n"):
            opened = re.search(r"fun (\w+)\.\w+\(", line)
            if opened:
                current = opened.group(1)
            arm = re.match(r"(?:is )?(\w+)\.(\w+)\s*->.*?R\.string\.(\w+)", line.strip())
            if current and arm and arm.group(1) == current:
                found[(current, variant(arm.group(2)))] = arm.group(3)
    return found


def main():
    strays = stray_mappers()
    for path in strays:
        print(f"  {path.relative_to(ROOT)} is a second mapper; its mappings belong in the one mapper file of that app")
    english = english_by_key()
    swift = swift_mappings(key_by_swift_path())
    kotlin = kotlin_mappings()
    shared = sorted(set(swift) & set(kotlin))
    divergent = [
        (core_type, case, swift[(core_type, case)], kotlin[(core_type, case)])
        for core_type, case in shared
        if english.get(swift[(core_type, case)]) != english.get(kotlin[(core_type, case)])
    ]
    print(f"compared {len(shared)} variants mapped by both apps")
    for core_type, case, ios_key, android_key in divergent:
        print(f"  {core_type}.{case}")
        print(f"    iOS     {ios_key} = {english.get(ios_key)!r}")
        print(f"    Android {android_key} = {english.get(android_key)!r}")
    if len(shared) < MINIMUM_COMPARED:
        print(f"  the comparison covers {len(shared)} variants, below the {MINIMUM_COMPARED} it reached before")
        print("  a mapper the checker used to read stopped matching, or a variant was removed on purpose")
        print(f"  read the mappers before lowering MINIMUM_COMPARED in {pathlib.Path(__file__).name}")
        return 1
    return 1 if divergent or strays else 0


if __name__ == "__main__":
    sys.exit(main())

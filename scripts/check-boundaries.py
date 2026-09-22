#!/usr/bin/env python3
"""Check the boundary rules that a regex can decide exactly.

Every rule here comes from a section of [ARCHITECTURE.md](../docs/ARCHITECTURE.md)
and holds only where the allowed paths can be named without guessing. A rule
that would need to read intent belongs in review, not here, so this file grows
one exact rule at a time rather than one heuristic at a time.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CORE = ROOT / "core/gemstone/src"

SOURCES = [("ios/Features", "ios/Packages", "ios/Gem", "ios/GemPriceWidget"), ("android",)]
SUFFIXES = {".swift", ".kt"}
SKIP_DIRS = {"build", ".build", "generated", "Submodules", "DerivedData", "test", "androidTest", "testFixtures", "Tests", "TestKit"}
SKIP_FILES = {"Gemstone.swift", "gemstone.kt"}

SERVICE_STRUCT = re.compile(r"pub struct (Gem\w*Service)\s*(\{[^}]*\})", re.S)
SERVICE_FIELD = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?\w+\s*:", re.M)
SERVICE_CALL = re.compile(r"\b(Gem\w*Service)\s*\(")
# The composition roots § 8 names: the factory that owns the graph, the
# per-screen factory it hands to views, the gateway both build from, and
# Android's dependency injection modules.
COMPOSITION = re.compile(r"(ServicesFactory\.swift|ViewModelFactory[^/]*\.swift|Gateway/GatewayService\.swift|/di/)")

LOCALIZED_MAPPER = re.compile(r"(?:extension GemLocalizedText\b(?!: Sendable)|fun GemLocalizedText\.)")
LOCALIZED_HOMES = {"Gemstone+Localized.swift", "GemstoneText.kt"}


def app_files():
    for roots in SOURCES:
        for root in roots:
            for path in sorted((ROOT / root).rglob("*")):
                if path.suffix not in SUFFIXES or path.name in SKIP_FILES:
                    continue
                if set(path.relative_to(ROOT).parts) & SKIP_DIRS:
                    continue
                yield path


def stateless_services():
    """A service Core declares with no fields carries no state to substitute."""
    names = set()
    for path in CORE.rglob("*.rs"):
        for name, body in SERVICE_STRUCT.findall(path.read_text()):
            if not SERVICE_FIELD.search(body):
                names.add(name)
    return names


def services_are_injected():
    """§ 8: a service comes from the composition root, never from a call site."""
    stateless = stateless_services()
    for path in app_files():
        relative = str(path.relative_to(ROOT))
        if COMPOSITION.search(relative):
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            for name in SERVICE_CALL.findall(line):
                if name not in stateless:
                    yield f"{relative}:{number} builds {name} outside the composition root"


def one_localization_mapper():
    """One mapper per app names every Core key it renders, in one place."""
    for path in app_files():
        if path.name in LOCALIZED_HOMES:
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            if LOCALIZED_MAPPER.search(line):
                yield f"{path.relative_to(ROOT)}:{number} renders GemLocalizedText outside its module mapper"


RULES = [
    ("services are injected, never constructed at a call site", services_are_injected),
    ("one localization mapper names every Core key it renders", one_localization_mapper),
]


def main():
    failures = 0
    for rule, check in RULES:
        found = sorted(check())
        for line in found:
            print(f"  {line}")
        failures += len(found)
    print(f"checked {len(RULES)} boundary rules")

    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())

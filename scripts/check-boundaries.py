#!/usr/bin/env python3
"""Check the boundary rules declared in check-boundaries.toml.

The config holds the rules; this file only knows how to check each kind of rule.
A rule holds only where the allowed paths can be named without guessing: a rule
that would need to read intent belongs in review, not here.

Every rule has an `id`, the `doc` section that states it, `why` it exists, its `kind`
and the `files` it reads: the name of a `[files.*]` set or a list of globs. `exclude`
and `within` narrow the files, `allowed` names the files the rule does not apply to,
and `known` lists files that still break a new rule (a file that stops breaking it
is reported, so the list only shrinks). `message` may use `{name}` for the name a
rule found.

Kinds:
- forbid: no line matches `pattern`. `names` collects the first group of a pattern over
  other files into `{names}`, `skip_names` lets those names through, `between` limits the
  check to the text between two patterns, and `examples` lists lines that must and must
  not match.
- implements: no Swift or Kotlin type declares conformance to one of `names`.
- parameters: no function has a parameter whose type matches `type`.
- trait_methods: every method of a trait whose header matches `trait` starts with one of `verbs`.
- dependencies: only `dependents` depend on `crates`; `"*"` allows every crate.
- room_migration: the Room `database` version has its exported schema and a migration in `registry`.

Usage: check-boundaries.py [--rule ID ...] [--list]
"""

import argparse
import functools
import pathlib
import re
import subprocess
import sys

try:
    import tomllib
except ModuleNotFoundError:
    sys.exit("check-boundaries needs Python 3.11 or newer for tomllib, for example `brew install python`")

ROOT = pathlib.Path(__file__).resolve().parent.parent
CONFIG = pathlib.Path(__file__).with_suffix(".toml")


# Files


@functools.cache
def repository_files():
    listed = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"], cwd=ROOT, capture_output=True, check=True).stdout
    return sorted(path for path in listed.decode().split("\0") if path and (ROOT / path).is_file())


@functools.cache
def glob(pattern):
    parts = re.split(r"(\*\*/|\*\*|\*|\?)", pattern)
    wildcards = {"**/": "(?:.*/)?", "**": ".*", "*": "[^/]*", "?": "[^/]"}
    return re.compile("".join(wildcards.get(part, re.escape(part)) for part in parts))


def matches(path, globs):
    return any(glob(pattern).fullmatch(path) for pattern in globs)


@functools.cache
def read(path):
    return (ROOT / path).read_text()


def line_of(text, offset):
    return text.count("\n", 0, offset) + 1


def select(spec, sets, exclude=(), within=None):
    include, set_exclude = (sets[spec]["include"], sets[spec].get("exclude", [])) if isinstance(spec, str) else (spec, [])
    return [
        path
        for path in repository_files()
        if matches(path, include) and not matches(path, [*set_exclude, *exclude]) and (within is None or matches(path, within))
    ]


def collect_names(spec, sets):
    pattern = re.compile(spec["pattern"], re.M)
    return {name for path in select(spec["files"], sets) for name in pattern.findall(read(path))}


# Kinds


def forbid(rule, files, sets):
    names = "|".join(sorted(collect_names(rule["names"], sets))) if "names" in rule else ""
    pattern = re.compile(rule["pattern"].replace("{names}", names or "(?!)"))
    skipped = collect_names(rule["skip_names"], sets) if "skip_names" in rule else set()
    for path in files:
        text = read(path)
        region = section(text, rule.get("between"))
        if region is None:
            yield path, None, f"no longer has the {rule['between']['start']} section this rule reads"
            continue
        first, last = line_of(text, region[0]), line_of(text, region[1])
        for number, line in enumerate(text.splitlines()[first - 1 : last], start=first):
            for match in pattern.finditer(line):
                name = match.group(1) if pattern.groups else None
                if name not in skipped:
                    yield path, number, rule["message"].format(name=name)


def section(text, between):
    if between is None:
        return 0, len(text)
    match = re.search(f"{between['start']}.*?{between['end']}", text, re.S)
    return (match.start(), match.end()) if match else None


SWIFT_CONFORMANCE = re.compile(r"\b(?:class|struct|actor|enum|extension)\s+[\w.]+(?:<[^>]*>)?\s*:\s*([^{]*)\{")
KOTLIN_SUPERTYPES = re.compile(r"\b(?:class|object|interface)\s+\w+(?:\s*\((?:[^()]|\([^()]*\))*\))?\s*:\s*([^{=]*)")


def implements(rule, files, sets):
    traits = collect_names(rule["names"], sets)
    for path in files:
        text = read(path)
        declaration = SWIFT_CONFORMANCE if path.endswith(".swift") else KOTLIN_SUPERTYPES
        for match in declaration.finditer(text):
            for name in sorted(traits & set(re.findall(r"\b\w+\b", match.group(1)))):
                yield path, line_of(text, match.start()), rule["message"].format(name=name)


FUNCTION = re.compile(r"\bfn\s+\w+\s*(?:<[^{;()]*>)?\s*\(")


def parameters(rule, files, sets):
    parameter_type = re.compile(rule["type"])
    for path in files:
        text = read(path)
        for match in FUNCTION.finditer(text):
            depth, index = 1, match.end()
            while depth and index < len(text):
                depth += {"(": 1, ")": -1}.get(text[index], 0)
                index += 1
            if parameter_type.search(text[match.end() : index - 1]):
                yield path, line_of(text, match.start()), rule["message"]


TRAIT_METHOD = re.compile(r"^\s+(?:async\s+)?fn\s+([a-z_0-9]+)\s*\(", re.M)


def trait_methods(rule, files, sets):
    header = re.compile(rule["trait"])
    verbs = set(rule["verbs"])
    for path in files:
        text = read(path)
        for trait in header.finditer(text):
            for method in TRAIT_METHOD.finditer(text, trait.end(), text.index("\n}\n", trait.end())):
                if method.group(1).split("_")[0] not in verbs:
                    yield path, line_of(text, method.start()), rule["message"].format(name=method.group(1))


DEPENDENCY_SECTIONS = {"dependencies", "dev-dependencies", "build-dependencies"}


def dependencies(rule, files, sets):
    crates = set(rule["crates"])
    for path in files:
        manifest = tomllib.loads(read(path))
        name = manifest.get("package", {}).get("name")
        if name in crates:
            continue
        tables = [manifest, *manifest.get("target", {}).values()]
        used = {crate for table in tables for section_name in DEPENDENCY_SECTIONS for crate in table.get(section_name, {})}
        allowed = crates if rule["dependents"].get(name) == "*" else set(rule["dependents"].get(name, []))
        for crate in sorted(used & crates - allowed):
            yield path, None, rule["message"].format(name=crate)
        if rule["dependents"].get(name) != "*":
            for crate in sorted(allowed - used):
                yield path, None, f"no longer depends on {crate}; remove it from {rule['id']} dependents"


def room_migration(rule, files, sets):
    version = int(re.search(r"^\s*version\s*=\s*(\d+)", read(rule["database"]), re.M).group(1))
    if not (ROOT / rule["schemas"] / f"{version}.json").exists():
        yield f"{rule['schemas']}/{version}.json", None, f"is missing for version {version}"
    registry = f"{rule['migrations']}/{rule['registry']}"
    registered = set(re.findall(r"\b(Migration_\w+)\b", read(registry)))
    migration = re.compile(r"\b(?:object|class)\s+(\w+)[^:{]*:\s*Migration\((\d+),\s*(\d+)\)")
    reaching = {name for path in (ROOT / rule["migrations"]).glob("Migration_*.kt") for name, _, end in migration.findall(path.read_text()) if int(end) == version}
    if not reaching & registered:
        yield registry, None, f"registers no migration to version {version}"


KINDS = {kind.__name__: kind for kind in (forbid, implements, parameters, trait_methods, dependencies, room_migration)}


# Rules


def examples_hold(rule):
    if "examples" not in rule:
        return
    pattern = re.compile(rule["pattern"])
    examples = rule["examples"]
    yield from (f"example no longer matches: {line}" for line in examples.get("violates", []) if not pattern.search(line))
    yield from (f"example now matches: {line}" for line in examples.get("passes", []) if pattern.search(line))


def check(rule, sets):
    selected = select(rule["files"], sets, rule.get("exclude", ()), rule.get("within")) if "files" in rule else []
    files = [path for path in selected if not matches(path, rule.get("allowed", []))]
    found = set(KINDS[rule["kind"]](rule, files, sets))
    known = set(rule.get("known", []))
    violations = sorted(f"{path}:{number} {message}" if number else f"{path} {message}" for path, number, message in found if path not in known)
    stale = sorted(f"{path} no longer breaks this rule; remove it from known" for path in known - {path for path, _, _ in found})
    return [*examples_hold(rule), *violations, *stale]


def main():
    parser = argparse.ArgumentParser(description="Check the boundary rules declared in check-boundaries.toml.")
    parser.add_argument("--rule", action="append", help="check only this rule id; repeatable")
    parser.add_argument("--list", action="store_true", help="list the rules and exit")
    arguments = parser.parse_args()

    config = tomllib.loads(CONFIG.read_text())
    rules = [rule for rule in config["rule"] if not arguments.rule or rule["id"] in arguments.rule]
    unknown = set(arguments.rule or []) - {rule["id"] for rule in config["rule"]}
    if unknown:
        sys.exit(f"unknown rule: {', '.join(sorted(unknown))}")
    if arguments.list:
        for rule in rules:
            print(f"{rule['id']}: {rule['why']} ({rule['doc']})")
        return 0

    failures = 0
    for rule in rules:
        lines = check(rule, config["files"])
        if lines:
            print(f"{rule['id']}: {rule['why']} ({rule['doc']})")
        for line in lines:
            print(f"  {line}")
        failures += len(lines)
    print(f"checked {len(rules)} boundary rules")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())

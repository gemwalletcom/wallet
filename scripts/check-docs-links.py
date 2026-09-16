#!/usr/bin/env python3
"""Check that every link in the guidance still points at something.

A design doc earns its keep by being accurate. A link to a file that moved,
a heading that was renamed, or a symbol that no longer exists turns the doc
into a map of a repo that is gone.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SOURCE_SUFFIXES = (".swift", ".kt", ".rs")


def documents():
    yield from sorted(ROOT.glob("docs/*.md"))
    yield from sorted(ROOT.glob("skills/*.md"))
    yield from sorted(ROOT.glob("*/AGENTS.md"))
    yield ROOT / "AGENTS.md"


def headings(text):
    found = set()
    for line in text.split("\n"):
        heading = re.match(r"^#+\s+(.*)$", line)
        if heading:
            slug = re.sub(r"[^\w\s-]", "", heading.group(1).strip().lower()).replace(" ", "-")
            found.add(slug)
    return found


def names(path, symbol):
    if path.is_dir():
        return any(
            re.search(r"\b" + re.escape(symbol) + r"\b", f.read_text(errors="ignore"))
            for f in path.rglob("*")
            if f.is_file() and f.suffix in SOURCE_SUFFIXES
        )
    if path.suffix not in SOURCE_SUFFIXES:
        return True
    return re.search(r"\b" + re.escape(symbol) + r"\b", path.read_text(errors="ignore")) is not None


def main():
    failures = []
    for doc in documents():
        if not doc.exists():
            continue
        text = doc.read_text()
        own = headings(text)
        for link in re.finditer(r"\[([^\]]*)\]\(([^)]+)\)", text):
            label, target = link.group(1), link.group(2)
            if target.startswith(("http://", "https://", "mailto:", "gem:")):
                continue
            path_part, _, fragment = target.partition("#")
            if not path_part:
                if fragment not in own:
                    failures.append(f"{doc.relative_to(ROOT)} -> #{fragment} is not a heading here")
                continue
            path = (doc.parent / path_part).resolve()
            if not path.exists():
                failures.append(f"{doc.relative_to(ROOT)} -> {path_part} does not exist")
                continue
            if fragment and path.suffix == ".md" and fragment not in headings(path.read_text()):
                failures.append(f"{doc.relative_to(ROOT)} -> {target} is not a heading in that file")
            if not (label.startswith("`") and label.endswith("`")):
                continue
            symbol = label.strip("`")
            if re.fullmatch(r"[A-Za-z_]\w*", symbol) and not names(path, symbol):
                failures.append(f"{doc.relative_to(ROOT)} -> {path_part} no longer names {symbol}")
    print(f"checked the guidance links in {len(list(documents()))} documents")
    for failure in failures:
        print(f"  {failure}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())

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
# per-screen factory it hands to views, the gateway both build from, the
# keystore layer that builds what it will not hand out, and Android's
# dependency injection modules.
COMPOSITION = re.compile(r"(ServicesFactory\.swift|ViewModelFactory[^/]*\.swift|Gateway/GatewayService\.swift|LocalKeystore\+Services\.swift|/di/)")

LOCALIZED_MAPPER = re.compile(r"(?:extension GemLocalizedText\b(?!: Sendable)|fun GemLocalizedText\.)")
LOCALIZED_HOMES = {"Gemstone+Localized.swift", "GemstoneText.kt"}

KEYSTORE = re.compile(r"\bGemKeystore\b")
KEYSTORE_LAYERS = re.compile(r"(ios/Packages/GemstoneServices/|android/data/services/gemstone/)")

NATIVE_STORES = ("ios/Packages/Store/", "android/data/services/store/")
GEMSTONE = re.compile(r"\bGemstone\w*|\buniffi\.gemstone\b")

STORE_TRAIT = re.compile(r"pub trait (Gem\w+Store)\b")
STORE_ADAPTERS = ("ios/Packages/GemstoneServices/", "android/data/services/gemstone/")
SWIFT_CONFORMANCE = re.compile(r"\b(?:class|struct|actor|enum|extension)\s+[\w.]+(?:<[^>]*>)?\s*:\s*([^{]*)\{")
KOTLIN_SUPERTYPES = re.compile(r"\b(?:class|object|interface)\s+\w+(?:\s*\((?:[^()]|\([^()]*\))*\))?\s*:\s*([^{=]*)")

IOS_MIGRATIONS = ROOT / "ios/Packages/Store/Sources/Migrations.swift"
IOS_START_MIGRATIONS = re.compile(r"mutating func run\(.*?mutating func runChanges\(", re.S)
ALTERATION = re.compile(r"\balter\(table:|\baddColumnIfMissing\(|\bdrop\(column:")

ROOM_DATABASE = ROOT / "android/data/services/store/src/main/kotlin/com/gemwallet/android/data/services/store/database/GemDatabase.kt"
ROOM_MIGRATIONS = ROOM_DATABASE.parent / "di"
ROOM_SCHEMAS = ROOT / "android/data/services/store/schemas/com.gemwallet.android.data.services.store.database.GemDatabase"
ROOM_VERSION = re.compile(r"^\s*version\s*=\s*(\d+)", re.M)
ROOM_MIGRATION = re.compile(r"\b(?:object|class)\s+(\w+)[^:{]*:\s*Migration\((\d+),\s*(\d+)\)")
DESTRUCTIVE_FALLBACK = re.compile(r"\bfallbackToDestructiveMigration\w*\(")


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


def the_keystore_stays_in_its_layer():
    """§ 8: an app takes the services that sign, never the keystore they sign with."""
    for path in app_files():
        relative = str(path.relative_to(ROOT))
        if KEYSTORE_LAYERS.search(relative):
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            if KEYSTORE.search(line):
                yield f"{relative}:{number} reaches for the keystore outside its layer"



def native_stores_speak_primitives():
    """§ 4: the native store never references Gemstone; its adapter maps at the boundary."""
    for path in app_files():
        relative = str(path.relative_to(ROOT))
        if not relative.startswith(NATIVE_STORES):
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            if GEMSTONE.search(line):
                yield f"{relative}:{number} references Gemstone from the native store"


def store_traits():
    return {name for path in CORE.rglob("*.rs") for name in STORE_TRAIT.findall(path.read_text())}


def store_traits_live_in_their_adapters():
    """§ 4: a Core store trait is implemented only in the Gemstone adapter layer, never by a DAO, a native store or a feature."""
    traits = store_traits()
    for path in app_files():
        relative = str(path.relative_to(ROOT))
        if relative.startswith(STORE_ADAPTERS):
            continue
        text = path.read_text()
        declaration = SWIFT_CONFORMANCE if path.suffix == ".swift" else KOTLIN_SUPERTYPES
        for match in declaration.finditer(text):
            for name in sorted(traits & set(re.findall(r"\b\w+\b", match.group(1)))):
                number = text.count("\n", 0, match.start()) + 1
                yield f"{relative}:{number} implements {name} outside the store adapters"


def ios_start_migrations_only_create():
    """§ 4: run() only creates or recreates tables; a column change belongs in runChanges(), after the tables it alters exist."""
    text = IOS_MIGRATIONS.read_text()
    start = IOS_START_MIGRATIONS.search(text)
    if start is None:
        yield f"{IOS_MIGRATIONS.relative_to(ROOT)} no longer has run() before runChanges()"
        return
    for match in ALTERATION.finditer(start.group(0)):
        number = text.count("\n", 0, start.start() + match.start()) + 1
        yield f"{IOS_MIGRATIONS.relative_to(ROOT)}:{number} alters a table in run()"


def ios_migrations_fail_loudly():
    """§ 4: a migration checks what exists instead of swallowing the error with try?."""
    for number, line in enumerate(IOS_MIGRATIONS.read_text().splitlines(), start=1):
        if "try?" in line:
            yield f"{IOS_MIGRATIONS.relative_to(ROOT)}:{number} swallows a migration error with try?"


def room_version_ships_with_its_migration():
    """§ 4: the Room version has its exported schema and a registered migration that reaches it."""
    version = int(ROOM_VERSION.search(ROOM_DATABASE.read_text()).group(1))
    if not (ROOM_SCHEMAS / f"{version}.json").exists():
        yield f"{ROOM_SCHEMAS.relative_to(ROOT)}/{version}.json is missing for version {version}"
    registered = set(re.findall(r"\b(Migration_\w+)\b", (ROOM_MIGRATIONS / "GemDatabaseMigrations.kt").read_text()))
    reaching = [name for path in ROOM_MIGRATIONS.glob("Migration_*.kt") for name, _, end in ROOM_MIGRATION.findall(path.read_text()) if int(end) == version]
    if not set(reaching) & registered:
        yield f"{ROOM_MIGRATIONS.relative_to(ROOT)}/GemDatabaseMigrations.kt registers no migration to version {version}"


def room_never_drops_user_data():
    """§ 4: a missing migration is a bug to fix, never a reason to wipe the database."""
    for path in app_files():
        if path.suffix != ".kt":
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            if DESTRUCTIVE_FALLBACK.search(line):
                yield f"{path.relative_to(ROOT)}:{number} falls back to a destructive migration"

BACKEND = ROOT / "core"
INFRA_CRATES = {"storage", "cacher", "streamer", "search_index", "pusher"}
INFRA_DEPENDENTS = {
    "services": INFRA_CRATES,
    "daemon": {"streamer"},
}
CARGO_SECTION = re.compile(r"^\[(.+)\]\s*$")
CARGO_KEY = re.compile(r"^([A-Za-z0-9_-]+)\s*=\s*(.*)$")
DEPENDENCY_SECTIONS = {"dependencies", "dev-dependencies", "build-dependencies"}


def cargo_packages():
    for path in sorted(BACKEND.rglob("Cargo.toml")):
        if "target" in path.relative_to(BACKEND).parts or path.parent == BACKEND:
            continue
        name, section, dependencies = None, None, set()
        for line in path.read_text().splitlines():
            header = CARGO_SECTION.match(line)
            if header:
                section = header.group(1)
                continue
            key = CARGO_KEY.match(line)
            if not key or section is None:
                continue
            if section == "package" and key.group(1) == "name":
                name = key.group(2).strip().strip('"')
            elif section.split(".")[-1] in DEPENDENCY_SECTIONS:
                dependencies.add(key.group(1))
        yield path.relative_to(ROOT), name, dependencies


def only_services_reach_infra():
    """core/skills/architecture.md § Backend Layers: only services depends on infra crates."""
    for path, name, dependencies in cargo_packages():
        if name in INFRA_CRATES:
            continue
        used = dependencies & INFRA_CRATES
        allowed = INFRA_DEPENDENTS.get(name, set())
        for crate in sorted(used - allowed):
            yield f"{path} depends on infra crate {crate}"
        if name != "services":
            for crate in sorted(allowed - used):
                yield f"{path} no longer depends on {crate}; remove it from INFRA_DEPENDENTS"


ANDROID_FEATURES = ROOT / "android/features"
DATA_INTERNALS = re.compile(r'project\(":data:(?:services:gemstone|coordinators)"\)')
DATA_INTERNAL_DEPENDENTS = {
    "android/features/asset_select/viewmodels/build.gradle.kts",
    "android/features/assets/viewmodels/build.gradle.kts",
    "android/features/settings/price_alerts/viewmodels/build.gradle.kts",
    "android/features/swap/viewmodels/build.gradle.kts",
}


def android_features_stay_off_data_internals():
    """§ 5: an Android feature observes through requests and calls Core services, never the data layer behind them."""
    found = set()
    for path in sorted(ANDROID_FEATURES.rglob("build.gradle.kts")):
        if "build" in path.relative_to(ANDROID_FEATURES).parts[:-1]:
            continue
        relative = str(path.relative_to(ROOT))
        if DATA_INTERNALS.search(path.read_text()):
            found.add(relative)
            if relative not in DATA_INTERNAL_DEPENDENTS:
                yield f"{relative} depends on :data:services:gemstone or :data:coordinators"
    for relative in sorted(DATA_INTERNAL_DEPENDENTS - found):
        yield f"{relative} no longer depends on the data internals; remove it from DATA_INTERNAL_DEPENDENTS"


IOS_STORES = ROOT / "ios/Packages/Store/Sources/Stores"
IOS_STORE_TYPE = re.compile(r"^public (?:final )?(?:class|struct|actor) (\w+Store)\b", re.M)
ANDROID_STORE_TYPE = re.compile(r"\b(?:Gemstone\w*Store|\w+Dao)\b")
STORE_HOLDING_FEATURES = {
    "android/features/assets/viewmodels/src/main/kotlin/com/gemwallet/android/features/assets/viewmodels/NetworkAssetsViewModel.kt",
}


def features_never_hold_a_store():
    """§ 7: a feature reads through queries and calls services; a store is the database side of a Core service."""
    ios_stores = {name for path in IOS_STORES.rglob("*.swift") for name in IOS_STORE_TYPE.findall(path.read_text())}
    ios_store_type = re.compile(r"\b(?:" + "|".join(sorted(ios_stores)) + r")\b")
    found = set()
    for path in app_files():
        relative = str(path.relative_to(ROOT))
        if relative.startswith("ios/Features/"):
            pattern = ios_store_type
        elif relative.startswith("android/features/"):
            pattern = ANDROID_STORE_TYPE
        else:
            continue
        for number, line in enumerate(path.read_text().splitlines(), start=1):
            if pattern.search(line):
                found.add(relative)
                if relative not in STORE_HOLDING_FEATURES:
                    yield f"{relative}:{number} holds a store; read through a query and call the service"
    for relative in sorted(STORE_HOLDING_FEATURES - found):
        yield f"{relative} no longer holds a store; remove it from STORE_HOLDING_FEATURES"


RULES = [
    ("services are injected, never constructed at a call site", services_are_injected),
    ("one localization mapper names every Core key it renders", one_localization_mapper),
    ("the keystore stays in its layer", the_keystore_stays_in_its_layer),
    ("the native store speaks primitives only", native_stores_speak_primitives),
    ("store traits live in their adapters", store_traits_live_in_their_adapters),
    ("iOS start migrations only create tables", ios_start_migrations_only_create),
    ("iOS migrations fail loudly", ios_migrations_fail_loudly),
    ("the Room version ships with its migration", room_version_ships_with_its_migration),
    ("Room never drops user data", room_never_drops_user_data),
    ("only services depends on infra crates", only_services_reach_infra),
    ("Android features stay off the data internals", android_features_stay_off_data_internals),
    ("features never hold a store", features_never_hold_a_store),
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

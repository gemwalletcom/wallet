# Decision Records

Non-obvious architectural choices. Do not "improve" these without understanding the rationale.

## Gemstone is bundled locally

Gemstone (the Rust-to-mobile bridge) is built and bundled from source rather than fetched as a prebuilt package. This ensures the mobile apps always link against the exact Core revision in the repo and avoids version drift between Core logic and mobile bindings.

## Docker for Android release builds

Android release artifacts are built inside Docker to guarantee reproducible builds. This lets anyone verify that a published APK matches the source. Do not bypass Docker for release builds even if local Gradle works.

## TypeShare + UniFFI for code generation

TypeShare generates shared model types; UniFFI generates FFI bindings. Both run from `just generate`. Two tools are used because TypeShare handles pure data models efficiently while UniFFI handles the full FFI bridge (functions, callbacks, async). Do not consolidate them.

## CLAUDE.md symlinks to AGENTS.md

Each directory has `CLAUDE.md -> AGENTS.md`. This is intentional — different AI agents look for different filenames. The symlink avoids content duplication. Do not replace symlinks with copies.

## Diesel inline `use` for DSL imports

Diesel query functions import DSL names (`use crate::schema::*::dsl::*`) inside the function body. This is the one exception to the no-inline-imports rule — it prevents DSL name collisions at module scope and is idiomatic Diesel.

## Five Android build flavors

Google, Huawei, Samsung, Solana, and Universal flavors exist to satisfy different app store requirements and partnership constraints. Do not remove flavors or assume Google-only distribution.

## Core submodule pinning

`core/` is a Git submodule pinned to a specific commit. Use `just core-upgrade` to advance it. Do not manually update the submodule ref — the justfile handles validation.

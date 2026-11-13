# Setup

Use when preparing a machine for Core work: toolchain, backend prerequisites, and the Rust build cache.

## Prerequisites

1. Rust stable through `rustup` (`just install-rust`)
2. `just`
3. Homebrew for local tool installation
4. PostgreSQL client libraries and Diesel CLI for `storage` and the backend apps

## Initial Setup

```sh
just install-rust
just install-postgres
just install-diesel
```

`just install` installs PostgreSQL, Diesel, and the legacy `install-sccache` dependency. Set up Rust and mbx explicitly as shown here; sccache is not needed for local builds.

Mobile targets (`just gemstone install-ios-targets`, `just gemstone install-android-targets`) are listed in [Development Commands](development-commands.md) § Mobile.

## Build Cache

Local builds use [mbx](https://mr-boxington.jdx.dev/installation) through its Cargo shim. Compiled artifacts are shared across checkouts, while mbx manages per-workspace target directories and automatically prunes build storage. CI continues to use sccache.

```sh
cargo +stable install mbx --locked
mbx setup
mbx doctor
mbx stats
```

After setup, open a new shell and verify that `command -v cargo` resolves to the mbx shim before the rustup Cargo executable. Check the shell used by the agent as well as the terminal: zsh, bash, and fish can have different startup configuration. Codex and Claude should keep using plain Cargo commands through that shim; `mbx check` is an explicit alternative when shell activation is unavailable.

- Remove stale local kache or sccache `build.rustc-wrapper` settings and `RUSTC_WRAPPER` overrides when migrating to mbx. Preserve CI's explicit sccache configuration.
- A worktree's `target` symlink into the mbx cache is expected. The link can remain after mbx prunes its destination; do not replace it with a shared manually configured `CARGO_TARGET_DIR`.
- Inspect configuration with `mbx settings ls` and storage with `mbx stats`. Set a combined budget with `mbx settings set gc.max_total_size 40GiB`; inspect proposed cleanup with `mbx gc --dry-run` before requesting manual collection.
- After upgrading Cargo, verify a cold build, a warm build, and cache reuse in a second checkout. Build scripts and tooling must consume Cargo's reported output paths rather than assume the layout of intermediate artifacts.

The local/CI split is documented in [Architecture](../../docs/ARCHITECTURE.md#rust-build-cache-mbx-locally-sccache-in-ci).

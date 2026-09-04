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
just install-typeshare
just install-postgres
just install-diesel
```

`just install` runs the same steps plus `install-sccache`, a legacy step from before the local cache moved to kache (below); sccache is not needed on a developer machine.

Mobile targets (`just gemstone install-ios-targets`, `just gemstone install-android-targets`) are listed in [Development Commands](development-commands.md) § Mobile.

## Build Cache

Local builds use [kache](https://github.com/kunobi-ninja/kache) as the `rustc` wrapper. It is content-addressed, so it pays off on external dependencies and on clean or cross-worktree rebuilds; a workspace crate you are editing misses on every edit by design, so do not raise the cache cap to chase that miss rate.

```sh
brew install kunobi-ninja/kunobi/kache
kache init -y        # writes build.rustc-wrapper = "kache" to ~/.cargo/config.toml and installs the daemon as a login service
kache doctor         # verify wrapper, daemon, and cache integrity
```

- After `brew upgrade kache`, run `kache daemon install` again; the launch agent points at the previous Cellar path otherwise.
- The store cap lives in `~/.config/kache/config.toml` under `[cache] local_max_size`; edit the file and `kache daemon restart` (the `kache config` editor is a TUI). 20 GiB is plenty.
- `kache why-miss <crate>` explains a miss; `kache stats` shows hit rates.
- `RUSTC_WRAPPER` in the environment overrides the config wrapper. CI sets it to `sccache` deliberately; see [docs/DECISIONS.md](../../docs/DECISIONS.md) § Rust build cache.

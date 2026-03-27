# Setup

Use this skill for iOS environment setup, bootstrap work, and local tooling prerequisites.

## Prerequisites

1. macOS
2. Xcode
3. `just`
4. Homebrew for local tool installation

Apple Silicon is the default supported environment for Gemstone builds. Intel Macs may require rebuilding additional `x86_64` artifacts.

## Initial Setup

```bash
just setup-git
cd ios
just bootstrap
just spm-resolve
```

`just bootstrap` installs the Rust toolchain pieces, typeshare, UniFFI targets, SwiftGen, and SwiftFormat needed by the iOS app.

## Useful Setup Commands

```bash
just spm-resolve-all
just generate
just generate-stone
```

## Notes

- Use the repo root `just setup-git` if submodules are missing or out of date
- If you are on an Intel Mac, update `core` and run `just generate-stone` to build the extra Gemstone artifacts

# Setup

Use this skill for iOS environment setup, bootstrap work, and local tooling prerequisites.

## Prerequisites

1. macOS
2. Xcode
3. `just`
4. Homebrew for local tool installation

Apple Silicon is the supported environment for Gemstone builds. Intel Macs are not supported.

## Initial Setup

```bash
just setup-git
just core install-rust
just core install-typeshare
cd ios
just install
just spm-resolve
```

`just install` installs iOS targets, SwiftFormat, SwiftLint, and xcbeautify, using the shared Rust and TypeShare tools installed above. It also creates the local Gemstone UniFFI Swift/header sources and iOS Rust static libraries that SwiftPM and Xcode need.

From the repo root, `just install` installs shared tools once and then both platforms' dependencies. `just ios install` installs only iOS-specific dependencies and assumes shared tools are already installed.

After checkout, the default repo-root workflow is `just generate-stone`, then `just run-ios`. The optional `GemStone` Xcode scheme runs the same generation before the normal app build for people who prefer staying in Xcode.

## Useful Setup Commands

```bash
just spm-resolve-all
just generate
just generate-stone
```

## Notes

- Use the repo root `just setup-git` if submodules are missing or out of date
- Run `just install` before opening a fresh checkout in Xcode so the local Gemstone UniFFI sources exist
- Xcode latest stable is expected; Apple framework documentation ships inside Xcode at `/Applications/Xcode.app/Contents/PlugIns/IDEIntelligenceChat.framework/Versions/A/Resources/AdditionalDocumentation`

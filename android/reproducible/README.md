# Reproducible Builds

Gem Wallet Android uses [Nix](https://nixos.org/) to provide a hermetic, cross-platform build environment. Anyone can rebuild a tagged release and verify the output matches the published APK.

## How it works

`shell.nix` at the repo root pins every build dependency to exact versions via a locked nixpkgs commit:

- **JDK**: Temurin 17
- **Android SDK**: platform 36, build-tools 36.0.0, NDK 28.1.13356709
- **Rust**: via rustup, with `cargo-ndk` for Android cross-compilation
- **Gradle**: pinned via the wrapper (`gradle-wrapper.properties`)
- **R8 map-id**: overridden with `-renamesourcefileattribute SourceFile` in ProGuard rules to eliminate non-deterministic DEX output

## Current status

- [x] `shell.nix` with pinned nixpkgs, JDK, Android SDK, NDK, Rust, cargo-ndk, just
- [x] `-renamesourcefileattribute SourceFile` in ProGuard rules
- [x] Nix CI job (`.github/workflows/android-nix.yml`) builds on `ubuntu-latest`
- [x] Verified: clean build of tagged release succeeds inside `nix-shell`
- [x] End-to-end APK comparison — `verify.sh` script added
- [x] Byte-identical builds confirmed (two independent builds from same tag produce identical APKs)
- [ ] Cross-platform verification tested (Linux x86_64, Linux aarch64, macOS arm64)
- [ ] Release builds via Nix in CI (release repo prepared, pending first tagged release)

## Quick start

```bash
# Install Nix (one-time)
curl -L https://nixos.org/nix/install | sh -s -- --daemon

# Clone and build
git clone --recurse-submodules --branch <tag> https://github.com/gemwalletcom/wallet.git
cd wallet
touch android/local.properties
nix-shell shell.nix --run "cd android && ./gradlew assembleGoogleDebug"
```

## Updating the pinned toolchain

1. Pick a commit from [nixpkgs nixos-unstable](https://github.com/NixOS/nixpkgs/commits/nixos-unstable)
2. Run: `nix-prefetch-url --unpack https://github.com/NixOS/nixpkgs/archive/<commit>.tar.gz`
3. Update `url` and `sha256` in `shell.nix`

## Previous approach (Docker)

Docker was previously used but removed due to:
- R8's non-deterministic map-id required fragile post-build patching
- Base image publishing added friction to the release process
- Slow builds compared to native

Nix replaces Docker by providing a hermetic environment without container overhead.

## Next steps

1. Add a `verify.sh` script that rebuilds a tagged release in `nix-shell` and compares against the published APK (strip signatures with `apksigcopier`, then diff)
2. Test cross-platform: confirm identical output on Linux x86_64 and macOS arm64
3. Integrate verification into CI as an optional workflow dispatch

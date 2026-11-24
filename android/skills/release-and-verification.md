# Release

Use when producing release builds, choosing a distribution channel, or working on the reproducible builds.

## Release Builds

```bash
reproducible/docker.sh google       # Google Play AAB
reproducible/docker.sh universal    # universal APK; any other channel name builds that channel's APK
```

Every release file is built this way, from the checked-out commit and unsigned; the release pipeline signs it afterwards. The Gradle build has no release signing config, so `:app:assemble<Channel>Release` and `:app:bundleGoogleRelease` return unsigned files wherever they run. To try a release build on a device, sign it with `apksigner` and the debug keystore.

`reproducible/build.sh` sets `BUILD_MODE=release`, which makes the `:gemstone` module build the Rust library with `--release` for the ABIs the channel lists in `gradle/channels.gradle.kts`, and passes `-PuseLegacyPackaging=true` for an APK but not for the AAB. The container starts from the commit alone, so a release file never reuses a debug Rust build.

The Android Gradle plugin, not cargo, strips the native library that ships: it runs `llvm-strip --strip-unneeded` on the packaged copy and writes the symbol table to `BUNDLE-METADATA/com.android.tools.build.debugsymbols/` in the AAB, which Play uses to symbolicate native crashes and never delivers to devices. That is why `buildCargoNdk` overrides the shared cargo profile's `strip = true` with `CARGO_PROFILE_RELEASE_STRIP=debuginfo`: cargo keeps the symbol table for the plugin to extract, the download size does not change, and iOS, which calls cargo directly, keeps the shared setting.

## Channels

Store variants are Gradle product flavors driven by `gradle/channels.gradle.kts`. Pass `-Pchannel=<name>` explicitly for release builds; the build fails if the property disagrees with the requested flavor task. The `fdroid` channel must be built alone.

## Reproducible Builds

`reproducible/` is the public build recipe; [its README](../reproducible/README.md) has the commands and the layout.

- `reproducible/docker.sh <channel>...` builds the checked-out commit, not the working tree, in a pinned Ubuntu container and returns unsigned files. Every channel builds as `linux/amd64`, because F-Droid's servers are x86-64 and an ARM64 host compiles a different `libgemstone.so`. A rebuild matches only on the same machine type, so the script sets it, never the caller. It builds any channel: an APK for `fdroid`, `universal`, `solana` and `emerald`, the AAB for `google`.
- Nothing under `reproducible/` reads a keystore, a password or any other secret, and the container gets no host environment beyond `UPDATE_URL`. Signing and publishing belong to the release pipeline, which signs the unsigned file afterwards; do not add a release signing config to the Gradle build. Keep it that way: a build input that is not in the commit cannot be reproduced by anyone else.
- `reproducible/compare.py` compares two APKs outside the signing block, which is the check F-Droid makes.

## F-Droid

- F-Droid's recipe runs `reproducible/fdroid/init.sh` and `reproducible/fdroid/build.sh` on its own build server; keep those two paths. They call `reproducible/init.sh` and `reproducible/build.sh fdroid`, which builds `:app:assembleFdroidRelease` with `-Pchannel=fdroid`.
- F-Droid publishes the developer-signed APK attached to the GitHub release only when its own rebuild matches byte for byte, so every published version is reproducibility-verified at publish time.
- Keep toolchain pins aligned: the NDK version in `gradle/libs.versions.toml`, the Rust version in the root `rust-toolchain.toml`, the `cargo-ndk` version in `reproducible/versions.sh`, the image digests, download checksums and SDK build tools and platform versions in `reproducible/Dockerfile`, dependency locking, and Rust build flags all feed the reproducible build. Changing one without the others breaks the byte match and blocks the F-Droid release.
- Before a change to anything under `reproducible/` reaches a tag, build the previous tag's commit with the changed scripts and run `compare.py` against that tag's published F-Droid APK.

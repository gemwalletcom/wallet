# Release

Use when producing release builds, choosing a distribution channel, or working on the F-Droid reproducible build.

## Release Builds

```bash
just release        # Google Play AAB: BUILD_MODE=release ./gradlew clean :app:bundleGoogleRelease
just release-apk    # universal APK:   BUILD_MODE=release ./gradlew clean assembleUniversalRelease -PuseLegacyPackaging=true
```

Other channel APKs follow the same shape: `BUILD_MODE=release ./gradlew clean -Pchannel=<name> :app:assemble<Channel>Release`. `BUILD_MODE=release` makes the `:gemstone` module build the Rust library with `--release`; both recipes clean first and disable the configuration cache so the release artifact never reuses a debug Rust build.

## Channels

Store variants are Gradle product flavors driven by `gradle/channels.gradle.kts`. Pass `-Pchannel=<name>` explicitly for release builds; the build fails if the property disagrees with the requested flavor task. The `fdroid` channel must be built alone.

## F-Droid

- `reproducible/fdroid/` holds the F-Droid build-server helpers (`init.sh`, `build.sh`); the recipe runs `build.sh`, which builds `:app:assembleFdroidRelease` with `-Pchannel=fdroid`.
- F-Droid publishes the developer-signed APK attached to the GitHub release only when its own rebuild matches byte for byte, so every published version is reproducibility-verified at publish time.
- Keep toolchain pins aligned: the NDK version in `gradle/libs.versions.toml`, the `cargo-ndk` version in `reproducible/fdroid/init.sh`, dependency locking, and Rust build flags all feed the reproducible build. Changing one without the others breaks the byte match and blocks the F-Droid release.

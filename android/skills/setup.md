# Setup

Use this skill for Android environment setup and bootstrap work.

## Prerequisites

1. JDK 17
2. `just`
3. Homebrew on macOS, for the JDK and the Android SDK command-line tools
4. Android Studio, optional for building; its SDK at `~/Library/Android/sdk` is the default `ANDROID_HOME`

## Initial Setup

```bash
just core install-rust
cd android
just install
```

`just install` installs Temurin JDK 17, Android Rust targets, cargo-ndk, and the NDK, using the shared Rust toolchain installed above. When the SDK at `ANDROID_HOME` has no `sdkmanager`, it installs the `android-commandlinetools` cask, accepts the SDK licenses, and installs the command-line tools into that SDK first, so a machine needs no Android Studio step. Setup verifies Java by running `./gradlew --version`. The JDK installer can require a password prompt; `just install-java` remains available separately.

From the repo root, `just install` installs shared tools once and then both platforms' dependencies. `just android install` installs only Android-specific dependencies and assumes shared tools are already installed. Run setup before `just generate`.

Optional shared codegen after setup:

```bash
just generate
```

For local Android app iteration from the repo root:

```bash
just start-emulator
just run-android
```

## Notes

- `local.properties` is local machine configuration and must not be committed

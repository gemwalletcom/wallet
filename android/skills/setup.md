# Setup

Use this skill for Android environment setup and bootstrap work.

## Prerequisites

1. Android Studio
2. JDK 17
3. `just`
4. Homebrew for local JDK installation on macOS
5. Android SDK command-line tools, with `ANDROID_HOME` pointing to the SDK

## Initial Setup

```bash
just core install-rust
just core install-typeshare
cd android
just install
```

`just install` installs Temurin JDK 17, Android Rust targets, cargo-ndk, and the NDK, using the shared Rust and TypeShare tools installed above. Setup verifies Java by running `./gradlew --version`. The JDK installer can require a password prompt; `just install-java` remains available separately.

From the repo root, `just install` installs shared tools once and then both platforms' dependencies. `just android install` installs only Android-specific dependencies and assumes shared tools are already installed. Run setup before `just generate`, which uses the installed TypeShare CLI.

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

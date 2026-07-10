# Setup

Use this skill for Android environment setup and bootstrap work.

## Prerequisites

1. Android Studio
2. JDK 17
3. `just`

## Initial Setup

```bash
just install-java
just bootstrap
```

`just install-java` installs Temurin JDK 17 with Homebrew. It is separate from `just bootstrap` because the macOS package installer can require a password prompt.

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

# Setup

Use this skill for Android environment setup, bootstrap work, and local credential prerequisites.

## Prerequisites

1. Android Studio
2. JDK 17
3. `just`
4. A GitHub Personal Access Token with `read:packages` for GitHub Packages access

## Initial Setup

```bash
just install-java
just bootstrap
```

`just install-java` installs Temurin JDK 17 with Homebrew. It is separate from `just bootstrap` because the macOS package installer can require a password prompt.

Add GitHub Packages credentials to `local.properties` when dependency resolution or verification workflows need them:

```bash
echo "gpr.username=<your-github-username>" >> local.properties
echo "gpr.token=<your-github-personal-token>" >> local.properties
```

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
- Release verification and dependency metadata workflows read `gpr.username` and `gpr.token` from `local.properties`

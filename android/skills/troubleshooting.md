# Troubleshooting

## Common Recovery Commands

### Build Failures

```bash
./gradlew clean
just bootstrap
./gradlew build
```

### NDK Issues

```bash
just install-ndk
just install-toolchains
```

### Submodule Issues

```bash
cd ..
just setup-git
```

Use the repo root `just setup-git` command so submodule sync and recursive setup stay aligned with the monorepo workflow.

### Docker and Verification Issues

- Ensure Docker has sufficient memory
- Check that the base image is available or rebuild it locally
- Verify `gpr.username` and `gpr.token` are present in `local.properties` when dependency or verification jobs fail

## Useful Paths

- `app/src/main/kotlin/com/gemwallet/android/MainActivity.kt`
- `app/build.gradle.kts`
- `gradle/libs.versions.toml`
- `ui/src/main/res/values/`
- `app/src/main/assets/`

## Build Notes

- Rust handles the heavy cryptographic operations
- Gradle build cache is enabled
- Docker tooling lives under `reproducible/` for reproducible builds and verification

## Getting Help

- Discord: `https://discord.gg/aWkq5sj7SY`
- Telegram: `https://t.me/gemwallet_developers`
- Issues: `https://github.com/gemwalletcom/gem-android/issues`

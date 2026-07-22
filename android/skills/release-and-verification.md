# Release

## Release Builds

```bash
just release
```

This builds the Google Play AAB. Store APK variants are built by the workflows in the release repository.

Use `just release-apk` to build only the universal APK locally.

## Practical Rules

- Release builds are handled natively on the self-hosted runner for speed

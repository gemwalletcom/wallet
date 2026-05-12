# Release

## Release Builds

```bash
just release
```

This builds the main release variants defined in the Android `justfile`.

## Practical Rules

- Release and CI workflows may require GitHub Packages credentials in `local.properties`
- Release builds are handled natively on the self-hosted runner for speed

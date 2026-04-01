# Release

## Release Builds

```bash
just release
```

This builds the main release variants defined in the Android `justfile`.

## Reproducible Builds

Reproducible builds use Nix (`shell.nix` at the repo root) to pin the full Android toolchain. See `android/reproducible/README.md` for verification instructions.

## Practical Rules

- Release and CI workflows may require GitHub Packages credentials in `local.properties`
- Release builds are handled natively on the self-hosted runner for speed
- Reproducible build verification uses `nix-shell` for cross-machine consistency

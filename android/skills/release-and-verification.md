# Release and Verification

Use this skill for release packaging, Docker-based builds, reproducible-build work, and artifact verification.

## Release Builds

```bash
just release
```

This builds the main release variants defined in the Android `justfile`.

## Docker Workflows

```bash
just build-base-image
just build-app-image
TAG=feature-branch just build-app-image
just build-app-in-docker
```

## Verification and Metadata

```bash
just generate-verification-metadata
just add-verification-dependency com.example:library:1.2.3
just verify <TAG> <APK>
```

## CI/CD Context

- Base image builds start from `android/Dockerfile`
- Reproducible app builds use `android/reproducible/Dockerfile`
- Verification tooling lives under `android/reproducible/`
- Release and Docker workflows may require GitHub Packages credentials in `local.properties`

## Practical Rules

- Use these workflows for release, supply-chain, or reproducibility tasks, not day-to-day feature work
- Check Docker memory and base image availability when Docker builds fail
- Keep artifact verification steps aligned with the existing reproducible build scripts

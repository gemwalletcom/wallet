# Setup

Use this skill for Android environment setup, bootstrap work, and local credential prerequisites.

## Prerequisites

1. Android Studio
2. JDK 17
3. `just`
4. A GitHub Personal Access Token with `read:packages` for GitHub Packages access

## Initial Setup

```bash
just bootstrap
```

Add GitHub Packages credentials to `local.properties` when dependency resolution or verification workflows need them:

```bash
echo "gpr.username=<your-github-username>" >> local.properties
echo "gpr.token=<your-github-personal-token>" >> local.properties
```

Optional shared codegen after setup:

```bash
just generate
```

## Notes

- `local.properties` is local machine configuration and must not be committed
- Release verification and dependency metadata workflows read `gpr.username` and `gpr.token` from `local.properties`

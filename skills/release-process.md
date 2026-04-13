# Release Process

## Branching

The repo follows a GitFlow-like release model:

- `main` tracks the latest production release
- `develop` is the primary integration branch
- `feature/...` branches start from `develop`
- `release/...` branches prepare production releases
- `hotfix/...` branches handle production fixes

## Versioning

- App marketing versions use `Major.Release` on iOS and Android, for example `2.27`
- Internal build numbers remain separate and must keep increasing on every release
- Bump versions with:
  ```bash
  just bump patch
  just bump major
  just bump 2.30
  ```

## Commits

- Run the relevant tests, linters, and formatters before committing
- Write concise commit messages that explain the reason for the change, not just the file edits

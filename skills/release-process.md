# Release Process

## Branching

The repo follows a GitFlow-like release model:

- `main` tracks the latest production release
- `develop` is the primary integration branch
- `feature/...` branches start from `develop`
- `release/...` branches prepare production releases
- `hotfix/...` branches handle production fixes

## Versioning

- Use semantic versioning: Major.Minor.Patch
- Bump versions with:
  ```bash
  just bump patch
  just bump minor
  just bump major
  ```

## Commits

- Run the relevant tests, linters, and formatters before committing
- Write concise commit messages that explain the reason for the change, not just the file edits

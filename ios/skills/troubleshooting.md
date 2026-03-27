# Troubleshooting

## Common Recovery Commands

### Build Failures

```bash
just clean
just bootstrap
just build
```

Build logs live under `build/DerivedData`.

### Swift Package Resolution Issues

```bash
just spm-resolve
just spm-resolve-all
```

### Failing Tests

- Re-run the narrowest relevant target with `just test <TARGET>`
- Use Xcode directly when you need interactive debugging for test failures

### Core and Generated Code Issues

```bash
just generate
just generate-stone
```

Read `core/AGENTS.md` when the failure is caused by shared Rust or generated-model changes.

### Submodule Issues

```bash
cd ..
just setup-git
```

Use the repo root `just setup-git` command so submodule sync and recursive setup stay aligned with the monorepo workflow.

## Useful Paths

- `Gem.xcodeproj`
- `build/DerivedData`
- `Packages/Localization/`
- `scripts/generate-stone.sh`

## Build Notes

- `just build` already uses `xcodebuild` internally
- If command-line iteration is blocked, use Xcode directly for build and debugger context

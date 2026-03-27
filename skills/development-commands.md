# Development Commands

Use the root `justfile` to drive common repo workflows.

## Common Commands

```bash
just                    # list all commands
just build              # build both iOS and Android
just generate           # regenerate models + bindings for both platforms
just localize           # update localization for both platforms
just bump patch         # bump the repo version
just core-upgrade       # update core submodule to latest
```

## Platform Entry Points

```bash
just ios build          # build iOS only
just ios test-all       # run all iOS tests
just android build      # build Android only
just android test       # run Android tests
```

Load the platform guide before dropping into platform-specific commands or tooling details.

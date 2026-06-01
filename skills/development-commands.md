# Development Commands

Use the root `justfile` to drive common repo workflows.

## Common Commands

```bash
just                    # list all commands
just build              # build both iOS and Android
just generate           # regenerate models + bindings for both platforms
just localize           # update localization for both platforms
just run-ios            # build, install, and run iOS
just start-emulator     # start the Android emulator
just run-android        # build, install, and run Android
just bump patch         # bump the repo version
```

## Platform Entry Points

```bash
just ios build          # build iOS only
just ios test           # run iOS unit tests
just ios test-ui        # run iOS integration/UI tests
just android build      # build Android only
just android start-emulator # start the Android emulator
just android test       # run Android tests
```

Load the platform guide before dropping into platform-specific commands or tooling details.

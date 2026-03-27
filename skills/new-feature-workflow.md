# New Feature Workflow

End-to-end sequence for adding a feature that spans Core, iOS, and Android.

## 1. Core (Rust)

Start in `core/`. Read `core/AGENTS.md` first.

1. Add or modify crate logic under `crates/`
2. If the feature exposes new types to mobile, add TypeShare annotations and/or UniFFI exports in `gemstone/`
3. Run tests and clippy:
   ```bash
   cd core && just test <CRATE>
   cd core && cargo clippy -p <crate> -- -D warnings
   cd core && just format
   ```

## 2. Regenerate Bindings

From the repo root:

```bash
just generate        # regenerates models + bindings for both platforms
```

This runs `generate-models` (TypeShare) and `generate-stone` (UniFFI). Do not edit the generated output.

## 3. iOS

Read `ios/AGENTS.md`. Work in `ios/`.

1. Add Swift UI, ViewModel, or service code that consumes the new types
2. If extending generated models, add Swift extensions in separate files — never edit generated files
3. Build and test:
   ```bash
   cd ios && just build
   cd ios && just test-all
   cd ios && just lint
   ```

## 4. Android

Read `android/AGENTS.md`. Work in `android/`.

1. Add Kotlin UI (Compose), ViewModel, or repository code that consumes the new types
2. Wire dependencies through Hilt modules
3. Build and test:
   ```bash
   cd android && ./gradlew assembleGoogleDebug
   cd android && ./gradlew test
   cd android && ./gradlew lint
   ```

## 5. Final Verification

```bash
just build           # builds both platforms end-to-end
```

## Rules

- Always start from Core and work outward — do not stub mobile code before the Rust types exist
- Regenerate after every Core change — do not manually sync types
- Feature parity: if the feature is user-facing, implement both platforms or explicitly call out the gap
- Security: if the feature touches keys, signing, or transactions, read `skills/security.md` before starting

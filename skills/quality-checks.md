# Quality Checks

Run the checks that match the area you touched.

## Verification Matrix

| Change Type | Minimum Closing Checks |
|-------------|------------------------|
| iOS-only Swift or SwiftUI change | `cd ios && just build`<br>`cd ios && just test <TARGET>` or `cd ios && just test-all`<br>`cd ios && just lint`<br>`cd ios && swiftformat .` |
| Android-only Kotlin, Compose, or resource change | `cd android && ./gradlew assembleGoogleDebug` or build the affected module/variant<br>`cd android && ./gradlew test`<br>`cd android && ./gradlew lint`<br>`cd android && ./gradlew detekt`<br>`cd android && ./gradlew ktlintFormat` |
| Core-only Rust change with no mobile API impact | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> -- -D warnings`<br>`cd core && just format` |
| Core change that affects mobile bindings or shared models | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> -- -D warnings`<br>`cd core && just format`<br>`just generate`<br>`just ios build`<br>`just android build` |
| Shared localization input change | `just localize`<br>Rebuild the affected app(s) if the generated strings are consumed by the change |

Use the narrowest relevant test target while iterating, then finish with the broader validation required by the change.

## iOS

```bash
cd ios && just build
cd ios && just test-all
cd ios && just lint
cd ios && swiftformat .
```

## Android

```bash
cd android && ./gradlew test
cd android && ./gradlew lint
cd android && ./gradlew detekt
cd android && ./gradlew ktlintFormat
```

## Core

```bash
cd core && just test <CRATE>
cd core && cargo clippy -p <crate> -- -D warnings
cd core && just format
```

If you change shared models or bindings, also run the generation steps and validate both mobile apps.

If a user-facing shared flow changes on only one platform, call out the parity gap explicitly before finishing.

For detailed platform-specific commands, flags, and workflows see:
- [iOS Development Commands](../ios/skills/development-commands.md)
- [Android Development Commands](../android/skills/development-commands.md)
- [Core Development Commands](../core/skills/development-commands.md)

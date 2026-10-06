# Quality Checks

Use narrow checks while iterating, then the closing matrix for the changed area's risk.

For SwiftUI and Compose work, pure presentation changes should not spend most of the loop in full app builds.

Before running checks, confirm the checkout and command directory. Use features that compile the changed path and confirm filters selected the intended tests; zero selected tests is not a pass. If concurrent checks contend or return unclear status, rerun individually. Tool-specific caveats live in platform development-command guides.

## Iteration Matrix

| Change Type | Inner Loop Checks |
|-------------|-------------------|
| iOS presentation-only SwiftUI | `cd ios && just build-package <PACKAGE>` |
| iOS ViewModel, formatter, validation, or display-model behavior | `cd ios && just test-package <PACKAGE>` when package tests exist; otherwise `just build-package <PACKAGE>`<br>Use `just test <TARGET>` for app-hosted tests or new-target registration |
| Android presentation-only Compose or resource change | `cd android && ./gradlew :<module>:assembleDebug` |
| Android ViewModel, formatter, validation, or display-model behavior | `cd android && ./gradlew :<module>:assembleDebug`<br>`cd android && ./gradlew :<module>:testDebugUnitTest` when a targeted test exists or is added |
| Core-only Rust change with no mobile API impact | `cd core && just test <CRATE>` |
| Core change that affects mobile bindings or shared models | `cd core && just test <CRATE>`<br>`just generate`<br>then targeted mobile compile until app integration is ready |
| Shared localization input change | `just localize`<br>then targeted app/package/module compile where generated strings are consumed |

## Closing Matrix

| Change Type | Minimum Closing Checks |
|-------------|------------------------|
| iOS presentation-only SwiftUI | `cd ios && just build-package <PACKAGE>`<br>Simulator/device smoke when the changed flow is reachable |
| iOS ViewModel, navigation, app wiring, or behavioral UI change | `cd ios && just build`<br>`cd ios && just test <TARGET>` or `cd ios && just test` |
| Android presentation-only Compose or resource change | `cd android && ./gradlew :<module>:assembleDebug`<br>Emulator/device smoke when the changed flow is reachable |
| Android ViewModel, navigation, app wiring, or behavioral UI change | `cd android && ./gradlew assembleGoogleDebug` or build the affected app/module variant<br>`cd android && ./gradlew :<module>:testDebugUnitTest` or `cd android && ./gradlew test` |
| Cross-platform UI flow or navigation change | Optional: a Maestro flow on a booted simulator/emulator when requested — see [Maestro UI Testing](testing-maestro.md) |
| Core-only Rust change with no mobile API impact | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> --all-features -- -D warnings`<br>`cd core && just lint` (same command and extra lint flags as CI)<br>`cd core && just format` |
| Core change that affects mobile bindings or shared models | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> --all-features -- -D warnings`<br>`cd core && just lint` (same command and extra lint flags as CI)<br>`cd core && just format`<br>`just generate`<br>`just ios build`<br>`just android build` |
| Shared localization input change | `just localize`<br>Rebuild the affected app(s) if the generated strings are consumed by the change |
| Core enum mapper change (`Gemstone+Localized.swift`, `Gemstone+Style.swift`, `GemstoneText.kt`, `GemstoneStyle.kt`) | `just check-mappers` |
| New or removed `#[uniffi::export]` function | `just check-ffi` |
| Service construction or a Core key rendered outside its mapper, or a backend crate depending on an infra crate | `just check-boundaries` |
| Documentation-only change | `git diff --check`<br>`just check-docs`<br>Inspect changed links, paths, commands, and instructions |

Repo Checks runs `check-mappers`, `check-docs`, `check-ffi`, and `check-boundaries` on every PR. When removing an unused FFI export that Core still calls, keep its body and remove only `#[uniffi::export]`.

Navigation, app wiring, wallet-critical UI, security-sensitive code, Room migrations, signing, transaction construction, wallet import/export, seed phrases, private keys, and auth flows are never presentation-only. Use the stricter platform/security checks for those tasks.

For Core feature coverage and CI lint details, see [Core Development Commands](../core/skills/development-commands.md#code-quality).

## Format every platform you touched

Run the formatter for each platform your change touched, before the closing checks:

| Platform | Command |
|---|---|
| Core | `cd core && just format` |
| iOS | `cd ios && just format` |
| Android | `cd android && just format` (`just android format-all` sweeps every file) |

`just generate-models` formats the Rust it writes.

Code changes require a real build or test for the changed area; inspection and reasoning cannot substitute. Report exact commands and results, including blocking failures.

Report deterministic tests, compiled-only checks, gated live tests, skipped tests, and expected failures separately. Compilation does not prove runtime behavior; correctness, performance, compatibility, and exact output are separate claims.

If a broad suite fails outside the changed path, rerun the narrow affected check to separate a regression from an environmental or pre-existing failure. Report both results; a targeted pass does not turn the failed broad suite into a pass.

## Ready-to-Commit Batch

Run the closing matrix once the change is stable: format → required generation → targeted tests → builds → required UI smoke. If checks change source or require fixes, return to the narrow loop and rerun affected final checks. [Cross-Platform Awareness](cross-platform-awareness.md) owns generation and parity requirements.

Run `just generate-stone` before iOS checks to rebuild the linked Core library and bindings. Android app builds generate their bindings and native libraries. Gradle builds and fingerprints the host library before unit tests, including single-module runs.

For detailed platform-specific commands, flags, and workflows see:

- [iOS Development Commands](../ios/skills/development-commands.md)
- [Android Development Commands](../android/skills/development-commands.md)
- [Core Development Commands](../core/skills/development-commands.md)

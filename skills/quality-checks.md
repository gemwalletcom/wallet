# Quality Checks

Run the checks that match the area you touched. Use the narrowest meaningful command while iterating, then finish with the broader validation required by the risk of the change.

For SwiftUI and Compose work, pure presentation changes should not spend most of the loop in full app builds.

Before running checks, confirm the active checkout/worktree and command directory. Use the feature flags that compile the changed path, and verify that filtered commands selected the intended tests. Tool-specific caveats (Cargo test filters, lock contention) live in the platform development-commands skills.

## Iteration Matrix

| Change Type | Inner Loop Checks |
|-------------|-------------------|
| iOS presentation-only SwiftUI | `cd ios && just build-package <PACKAGE>` |
| iOS ViewModel, formatter, validation, or display-model behavior | `cd ios && just build-package <PACKAGE>`<br>`cd ios && just test <TARGET>` when a targeted test exists or is added |
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
| Core-only Rust change with no mobile API impact | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> -- -D warnings`<br>`cd core && just format` |
| Core change that affects mobile bindings or shared models | `cd core && just test <CRATE>`<br>`cd core && cargo clippy -p <crate> -- -D warnings`<br>`cd core && just format`<br>`just generate`<br>`just ios build`<br>`just android build` |
| Shared localization input change | `just localize`<br>Rebuild the affected app(s) if the generated strings are consumed by the change |
| Documentation-only change | `git diff --check`<br>Inspect changed links, paths, commands, and instructions |

Navigation, app wiring, wallet-critical UI, security-sensitive code, Room migrations, signing, transaction construction, wallet import/export, seed phrases, private keys, and auth flows are never presentation-only. Use the stricter platform/security checks for those tasks.

For Core crates with `default = []`, per-crate `cargo clippy -p <crate>` and `cargo test -p <crate>` skip feature-gated modules and pass in seconds. Add `--all-features` or the gating feature (`just test <CRATE>` already does); see [Core Development Commands](../core/skills/development-commands.md).

Except for documentation-only changes, closing a task requires at least one real build or test command for the changed area. Do not substitute `git diff`, static inspection, or reasoning for execution. If execution is blocked by unrelated repo state, include the exact command and the blocking failure in the handoff.

Compiling a gated integration test with `--no-run` proves build compatibility, not live provider behavior. Report deterministic tests, gated live tests, and checks that were compiled but not executed as separate results.

If a broad suite fails outside the changed path, rerun the narrow affected check to separate a regression from an environmental or pre-existing failure. Report both results; a targeted pass does not turn the failed broad suite into a pass.

## Ready-to-Commit Batch

Do not run the closing matrix after every edit. Once the implementation is stable and no more code edits are expected, run the applicable closing checks as one batch:

1. Regenerate models/bindings or localization if the changed inputs require it.
2. Run the targeted tests that cover the changed behavior.
3. Build the affected package/module/app according to the closing matrix.
4. Exercise the changed UI flow when the platform guide requires a simulator, emulator, or device smoke check.

If any step modifies source files or forces a compile fix, return to the narrow iteration loop, then run the affected final checks again.

If you change shared models or bindings, also run the generation steps and validate both mobile apps.

If a user-facing shared flow changes on only one platform, call out the parity gap explicitly before finishing.

For detailed platform-specific commands, flags, and workflows see:
- [iOS Development Commands](../ios/skills/development-commands.md)
- [Android Development Commands](../android/skills/development-commands.md)
- [Core Development Commands](../core/skills/development-commands.md)

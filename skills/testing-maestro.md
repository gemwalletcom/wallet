# Maestro UI Testing

Maestro (`docs.maestro.dev`) drives a real UI flow against a booted simulator or emulator. Use it for user journeys that must behave the same on iOS and Android. One YAML flow runs on both platforms; the same flow logic switches apps by `appId`.

Maestro complements, does not replace, the native UI layers: `ios/GemUITestsAppTests` (XCUITest, run in CI by `.github/workflows/ios-ui-tests.yml`) and Android `androidTest` Compose tests stay platform-native. Do not port those into Maestro.

## When to Use What

Decide in one glance:

| Situation | Verify with |
|-----------|-------------|
| Cross-platform user journey (import, receive, send, swap), navigation between screens, or E2E regression across both apps | **Maestro flow** |
| Behavior that must stay identical on iOS and Android — parity-sensitive UI, shared state transitions | **Maestro flow** (one flow, both `appId`s) |
| Pure logic: mappers, formatters, validators, display models, ViewModel state | **Unit test** — see [Quality Checks](quality-checks.md) |
| Platform-specific mechanics: permission dialogs, deep links, XCUITest interruption monitors, Compose semantics | **Native UI test** — `ios/GemUITestsAppTests` (XCUITest), Android `androidTest` |
| Static screen with no interaction, visual polish/spacing, single-platform micro-interaction, copy/localization-only | **Not worth a flow** |

A Maestro flow earns its cost only when the journey is reachable, cross-platform, and regression-prone. If a unit test can assert the same rule, write the unit test. Reach for a flow when the change needs it or when asked — it is not an automatic step on every UI change.

## Setup

- **Install** (official installer): `curl -fsSL "https://get.maestro.mobile.dev" | bash`. Verify: `maestro --version` (validated against `2.6.1`).
- **iOS simulator** needs Xcode Command Line Tools (`xcode-select --install`); Maestro installs its own xctest driver on the first simulator run — no `idb` required on current versions. Physical devices need extra signing setup (out of scope here).
- **Build and install the app first.** Maestro drives the *installed* build, not your source tree. Run `just ios run` / `just android run` before a flow, and reinstall after any structural, rename, or DI change (Android hot-swap silently skips those).

Device targeting — both a simulator and an emulator are usually connected, so always pass the device explicitly:

| | iOS | Android |
|---|-----|---------|
| appId | `com.gemwallet.ios` | `com.gemwallet.android` |
| list devices | `xcrun simctl list devices booted` | `adb devices` |
| both platforms | `maestro list-devices` | `maestro list-devices` |

```bash
maestro --device <id> test -e APP_ID=com.gemwallet.android flow.yaml
```

`--device` accepts a simulator UDID or an emulator id (e.g. `emulator-5554`); `--udid` is the same flag.

## Authoring Rules

These are the core of this doc. A flow that breaks them is worse than no flow.

### Selectors — ids, never text

The app ships ~30 languages. A `text:` selector is a per-locale time bomb; it also collides across elements (Maestro text match is regex, case-insensitive). Select by `id`.

`id` maps to `accessibilityIdentifier` on iOS and the Android resource-id on Compose. A Compose `Modifier.testTag("…")` is exposed as a resource-id because the nav host sets `testTagsAsResourceId = true` once (`android/.../navigation/WalletNavGraph.kt`).

If the element you need has no id, **adding the id to the app is part of writing the flow**:

| Platform | Add the id | Existing example |
|----------|-----------|------------------|
| SwiftUI | `.accessibilityIdentifier("assetsManageAction")` | `AssetScene.swift` (`price`, `stake`, `earn`) |
| Compose | `Modifier.testTag("assetsManageAction")` | `AssetsTopBar.kt`, `MainScreen.kt` (`mainTab`, `settingsTab`) |

Use the **same id string on both platforms** so one flow logic runs on both. `id` is regex — keep ids specific so they do not substring-match a neighbor.

### One flow per behavior; one flow, two appIds

Parameterize the app, not the flow:

```yaml
appId: ${APP_ID}
```

Run it per platform with `-e APP_ID=com.gemwallet.ios` / `-e APP_ID=com.gemwallet.android`. Do not fork a flow per platform for shared behavior — that is the parity gap the flow exists to catch.

### Deterministic state

- `launchApp: { clearState: true }` for regression flows. On Android this is `pm clear` (wipes app data → onboarding); on iOS it is a fresh reinstall. Expect the onboarding or lock screen after it.
- **Never mainnet funds or real keys.** Use a dedicated test seed only (there is already one at `ios/GemUITestsAppTests/Types/UITestKitConstants.swift`).
- Assert with `assertVisible` / `assertNotVisible` / `takeScreenshot`. **Never `sleep`.** For async work use `extendedWaitUntil` or `waitForAnimationToEnd`, keyed to a visible id — not a fixed delay.

### Biometrics

- **Android emulator:** satisfy the prompt out-of-band with `adb -e emu finger touch 1` (issue it while the prompt is showing).
- **iOS simulator:** Maestro has no built-in Face ID pass, and the Gem app re-locks on every `launchApp`. Run flows against a **debug build that bypasses the biometric gate**; otherwise every step after launch lands on the lock screen.

### Text input

Use Maestro `inputText`, never `adb shell input text`. Raw adb into a Compose `BasicTextField` is flaky — focus frequently fails to stick, so typed characters are dropped.

## Authoring Aids

| Command | Use |
|---------|-----|
| `maestro studio` | Interactive selector/flow explorer (local web UI) |
| `maestro hierarchy` (`--compact` for CSV) | Dump the current screen's element tree to find ids |
| `maestro check-syntax flow.yaml` | Validate a flow before running it |

## Example Flow

Verified green on `emulator-5554` (`com.gemwallet.android`). Non-destructive (`clearState:false`) and id-only; a regression flow flips `clearState:true` and imports the test seed first.

```yaml
# Home bottom-nav smoke — id selectors only, no text, no sleeps.
#   maestro --device <id> test -e APP_ID=com.gemwallet.android flow.yaml
appId: ${APP_ID}
---
- launchApp:
    clearState: false          # a regression flow uses clearState:true + a dedicated test seed
- assertVisible:
    id: "assetsManageAction"   # Assets (home) top bar rendered
- tapOn:
    id: "settingsTab"
- assertNotVisible:
    id: "assetsManageAction"   # navigated to Settings — Assets top bar gone
- tapOn:
    id: "mainTab"
- assertVisible:
    id: "assetsManageAction"   # back on home
- takeScreenshot: home-nav
```

## Anti-Patterns

| Do not | Instead |
|--------|---------|
| Select by visible text | Select by `id` (add `accessibilityIdentifier` / `testTag` if missing) |
| `sleep` / fixed waits | `extendedWaitUntil` / `waitForAnimationToEnd` on a visible id |
| Fork a flow per platform for shared behavior | One flow, `appId: ${APP_ID}`, two `-e APP_ID=` runs |
| Assert live prices or market data | Assert structure and ids; keep the flow deterministic |
| `adb shell input text` into Compose fields | Maestro `inputText` |
| Commit flows that need real keys or mainnet funds | Dedicated test seed on testnet/debug build |

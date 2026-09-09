# Code Style

Use for any Swift or SwiftUI change.
## Language and Framework

- SwiftUI with MVVM. View models are `@Observable` and `@MainActor` with injected dependencies and computed state, never `ObservableObject`
- async/await over Combine for new work
- Services are protocol-based. Depend on the protocol; construct in `ServicesFactory` / `ViewModelFactory`
- Dependencies enter through the app resolver and environment injection (`.inject(resolver:)`), not singletons

References: `ios/Gem/App.swift` (app composition), `ios/Gem/ViewModels/MainTabViewModel.swift` (view model shape).

## Organization

- One type per file
- Protocol conformances in extensions
- Group action methods in a view-model extension; existing section markers do not require adding new comments
- Shared functionality lives in `Packages/`; features do not depend on each other directly

## Style

- `Spacing` constants from the `Style` package, never hardcoded spacing (`ios/Packages/Style/Sources/Spacing.swift`)
- Follow the shared [comment and API-surface rules](../../skills/engineering-principles.md#clean-code-principles)

## TestKit Mocks

Reusable mocks are `static func mock(...)` extensions on the type in the package's `TestKit` target, with defaulted parameters so a test overrides only what it asserts. Reference: `ios/Packages/Primitives/TestKit/Perpetual+PrimitivesTestKit.swift`.

Shared clean-code principles live in `../../skills/engineering-principles.md`.

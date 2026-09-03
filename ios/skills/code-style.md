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
- Action methods grouped in a `// MARK: - Actions` extension of the view model (`ios/Features/Perpetuals/Sources/ViewModels/PerpetualSceneViewModel.swift`)
- Shared functionality lives in `Packages/`; features do not depend on each other directly

## Style

- `Spacing` constants from the `Style` package, never hardcoded spacing (`ios/Packages/Style/Sources/Spacing.swift`)
- No comments for obvious code or test behavior; minimal API surface; remove dead code and unnecessary abstractions

## TestKit Mocks

Reusable mocks are `static func mock(...)` extensions on the type in the package's `TestKit` target, with defaulted parameters so a test overrides only what it asserts. Reference: `ios/Packages/Primitives/TestKit/Perpetual+PrimitivesTestKit.swift`.

Shared clean-code principles live in `../../skills/engineering-principles.md`.

# Code Style

## Language and Framework Patterns

- Follow existing SwiftUI and Swift concurrency patterns
- Use `@Observable` instead of `ObservableObject` for view models
- Prefer async/await over Combine for new work
- Keep services protocol-based
- Prefer the concrete patterns below over inventing new local conventions

## Organization

- One type per file
- Use explicit protocol conformances in extensions
- Use `// MARK: - Actions` to separate action methods in view models
- Keep shared functionality in `Packages/`, not direct feature-to-feature dependencies

## Style Rules

- Use `Spacing` constants from the `Style` package instead of hardcoded spacing values
- Avoid comments for obvious code or test behavior
- Minimize API surface and keep code self-documenting
- Remove dead code and avoid unnecessary abstractions
- Prefer explicit protocol conformances in extensions:
  ```swift
  extension MyService: ServiceProtocol {
      func performAction() { }
  }
  ```

Shared clean-code and code-review principles live in `../../skills/engineering-principles.md`.

## Concrete Patterns

Use current code as the source of truth for style and composition.

### App Composition and Dependency Injection

`GemApp` builds the root scene from `AppResolver` services and injects them through the environment:

```swift
@main
struct GemApp: App {
    @UIApplicationDelegateAdaptor(AppDelegate.self) var appDelegate

    private let resolver: AppResolver = .main

    var body: some Scene {
        WindowGroup {
            RootScene(
                model: RootSceneViewModel(
                    observablePreferences: resolver.storages.observablePreferences,
                    walletConnectorPresenter: resolver.services.walletConnectorManager.presenter,
                    walletService: resolver.services.walletService
                )
            )
            .inject(resolver: resolver)
        }
    }
}
```

Reference: `ios/Gem/App.swift`

### View Models

Prefer `@Observable` + `@MainActor` view models with injected dependencies and computed state:

```swift
@Observable
@MainActor
final class MainTabViewModel {
    var wallet: Wallet
    let transactionsQuery: ObservableQuery<TransactionsCountRequest>

    var transactions: Int { transactionsQuery.value }

    init(wallet: Wallet) {
        self.wallet = wallet
        self.transactionsQuery = ObservableQuery(
            TransactionsCountRequest(walletId: wallet.walletId, state: .pending),
            initialValue: 0
        )
    }
}
```

Reference: `ios/Gem/ViewModels/MainTabViewModel.swift`

### Action Organization

Keep action methods grouped in an extension marked with `// MARK: - Actions`:

```swift
// MARK: - Actions

public extension PerpetualSceneViewModel {
    func fetch() {
        Task { await observerService.update(for: wallet) }
        Task { try await perpetualService.updateMarket(symbol: perpetual.coin) }
    }
}
```

Reference: `ios/Features/Perpetuals/Sources/ViewModels/PerpetualSceneViewModel.swift`

### TestKit Mocks

Put reusable mocks in TestKit extensions on the type:

```swift
public extension Perpetual {
    static func mock(
        id: String = "hypercore_ETH-USD",
        name: String = "ETH-USD"
    ) -> Perpetual {
        Perpetual(
            id: id,
            name: name,
            provider: .hypercore,
            assetId: AssetId(chain: .ethereum, tokenId: nil),
            identifier: "0",
            price: 1000.0,
            pricePercentChange24h: 0,
            openInterest: 1_000_000,
            volume24h: 10_000_000,
            funding: 0.0001,
            maxLeverage: 50,
            isIsolatedOnly: false
        )
    }
}
```

Reference: `ios/Packages/Primitives/TestKit/Perpetual+PrimitivesTestKit.swift`

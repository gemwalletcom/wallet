import Formatters
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transactions

struct TransactionRateViewModelTests {
    @Test
    func itemModel() {
        let ethereum = Asset.mockEthereum()
        let usdt = Asset.mockEthereumUSDT()
        let assets = [ethereum, usdt]
        let swap = AnyCodableValue.encode(TransactionSwapMetadata.mock(
            fromAsset: ethereum.id,
            fromValue: "1000000000000000000",
            toAsset: usdt.id,
            toValue: "3000000000",
        ))

        guard
            case let .rate(_, direct) = makeItemModel(metadata: swap, assets: assets, direction: .direct),
            case let .rate(_, inverse) = makeItemModel(metadata: swap, assets: assets, direction: .inverse)
        else {
            Issue.record("Expected rate item")
            return
        }
        #expect(direct.hasPrefix("1 ETH"))
        #expect(inverse.hasPrefix("1 USDT"))

        if case .empty = makeItemModel(metadata: nil, assets: []) {
        } else {
            Issue.record("Expected empty for non-swap transaction")
        }
    }

    private func makeItemModel(
        metadata: AnyCodableValue?,
        assets: [Asset],
        direction: AssetRateFormatter.Direction = .direct,
    ) -> TransactionItemModel {
        TransactionRateViewModel(
            transaction: .mock(
                transaction: .mock(type: .swap, metadata: metadata),
                assets: assets,
            ),
            direction: direction,
        ).itemModel
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
@testable import Primitives
import Testing

final class AssetRateFormatterTests {
    let formatter = AssetRateFormatter()

    @Test func testRate() {
        #expect(throws: Never.self) {
            let amountDirect = try self.rate(fromValue: 100, toValue: 100, direction: .direct)
            let amountInverse = try self.rate(fromValue: 100, toValue: 100, direction: .inverse)
            #expect(amountDirect == "1 BTC ≈ 100.00 USDT")
            #expect(amountInverse == "1 USDT ≈ 0.01 BTC")
        }

        #expect(throws: Never.self) {
            let amountDirect = try self.rate(fromValue: 131_231_200, toValue: 1100, direction: .direct)
            let amountInverse = try self.rate(fromValue: 131_231_200, toValue: 1100, direction: .inverse)
            #expect(amountDirect == "1 BTC ≈ 0.0008382 USDT")
            #expect(amountInverse == "1 USDT ≈ 1,193.01 BTC")
        }

        #expect(throws: Never.self) {
            let amountDirect = try self.rate(
                fromAsset: .mockEthereum(),
                toAsset: .mockBNB(),
                fromValue: 1_000_000_000_000_000_000,
                toValue: 997_365_015_742_592_220,
                direction: .direct,
            )
            let amountInverse = try self.rate(
                fromAsset: .mockEthereum(),
                toAsset: .mockBNB(),
                fromValue: 1_000_000_000_000_000_000,
                toValue: 997_365_015_742_592_220,
                direction: .inverse,
            )
            #expect(amountDirect == "1 ETH ≈ 1.00 BNB")
            #expect(amountInverse == "1 BNB ≈ 1.00 ETH")
        }
    }

    private func rate(
        fromAsset: Asset = .mock(),
        toAsset: Asset = .mockEthereumUSDT(),
        fromValue: Int,
        toValue: Int,
        direction: AssetRateFormatter.Direction,
    ) throws -> String {
        try formatter.rate(
            fromAsset: fromAsset,
            toAsset: toAsset,
            fromValue: BigInt(fromValue),
            toValue: BigInt(toValue),
            direction: direction,
        )
    }
}

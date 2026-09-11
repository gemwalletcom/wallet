// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemAssetRate
import struct Gemstone.GemSwapRate
import Testing
@testable import Transactions

struct TransactionRateViewModelTests {
    @Test
    func itemModel() {
        let rate = GemSwapRate(
            direct: GemAssetRate(baseSymbol: "ETH", quoteSymbol: "USDT", value: 3000),
            inverse: GemAssetRate(baseSymbol: "USDT", quoteSymbol: "ETH", value: 1 / 3000),
        )

        guard
            case let .rate(_, direct) = TransactionRateViewModel(rate: rate, isInverse: false).itemModel,
            case let .rate(_, inverse) = TransactionRateViewModel(rate: rate, isInverse: true).itemModel
        else {
            Issue.record("Expected rate item")
            return
        }
        #expect(direct.hasPrefix("1 ETH"))
        #expect(inverse.hasPrefix("1 USDT"))

        if case .empty = TransactionRateViewModel(rate: nil, isInverse: false).itemModel {
        } else {
            Issue.record("Expected empty without a rate")
        }
    }
}

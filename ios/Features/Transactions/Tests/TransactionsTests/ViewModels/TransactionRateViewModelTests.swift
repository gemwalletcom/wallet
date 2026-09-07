// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Formatters
import struct Gemstone.GemSwapRate
import struct Gemstone.GemTransactionAmount
import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transactions

struct TransactionRateViewModelTests {
    @Test
    func itemModel() {
        let rate = GemSwapRate(
            from: GemTransactionAmount(asset: Asset.mockEthereum().map(), value: BigUInt(1_000_000_000_000_000_000), sign: .outgoing, price: nil),
            to: GemTransactionAmount(asset: Asset.mockEthereumUSDT().map(), value: BigUInt(3_000_000_000), sign: .incoming, price: nil),
        )

        guard
            case let .rate(_, direct) = TransactionRateViewModel(rate: rate, direction: .direct).itemModel,
            case let .rate(_, inverse) = TransactionRateViewModel(rate: rate, direction: .inverse).itemModel
        else {
            Issue.record("Expected rate item")
            return
        }
        #expect(direct.hasPrefix("1 ETH"))
        #expect(inverse.hasPrefix("1 USDT"))

        if case .empty = TransactionRateViewModel(rate: nil, direction: .direct).itemModel {
        } else {
            Issue.record("Expected empty without a rate")
        }
    }
}

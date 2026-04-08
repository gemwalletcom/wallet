// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Primitives
import Testing
@testable import Transfer

struct TransferTransactionProviderTests {
    @Test
    func selectFeeRateUsesRequestedPriority() throws {
        let rates = [
            FeeRate(priority: .slow, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        #expect(try selectFeeRate(from: rates, requestedPriority: .fast).priority == .fast)
    }

    @Test
    func selectFeeRateFallsBackToFirstAvailableRate() throws {
        let rates = [
            FeeRate(priority: .slow, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        #expect(try selectFeeRate(from: rates, requestedPriority: .normal).priority == .slow)
    }

    @Test
    func selectFeeRateThrowsWhenRatesMissing() {
        #expect(throws: ChainCoreError.feeRateMissed) {
            _ = try selectFeeRate(from: [], requestedPriority: .normal)
        }
    }
}

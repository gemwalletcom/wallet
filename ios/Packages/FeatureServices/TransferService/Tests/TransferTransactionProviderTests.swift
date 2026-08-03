// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Primitives
import Testing
import TransferServiceTestKit
@testable import TransferService

struct TransferTransactionProviderTests {
    @Test
    func selectFeeRateUsesRequestedPriority() throws {
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        #expect(try selectFeeRate(from: rates, selection: .preset(.fast)).priority == .fast)
    }

    @Test
    func selectFeeRateFallsBackToFirstAvailableRate() throws {
        let rates = [
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        #expect(try selectFeeRate(from: rates, selection: .preset(.normal)).priority == .fast)
    }

    @Test
    func selectFeeRateThrowsWhenRatesMissing() {
        #expect(throws: ChainCoreError.feeRateMissed) {
            _ = try selectFeeRate(from: [], selection: .preset(.normal))
        }
    }

    @Test
    func selectFeeRateUsesCustomGasPrice() throws {
        let rates = [
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 2)),
        ]

        let selected = try selectFeeRate(from: rates, selection: .custom(7))

        #expect(selected.gasPriceType == .regular(gasPrice: 7))
    }
}

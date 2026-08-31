// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemSwapQuoteService
import Localization
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmDetailsViewModelTests {
    @Test
    func swap() {
        let model = ConfirmDetailsViewModel(type: .swap(.mock(), .mock(), .mock()), metadata: nil, currency: Currency.usd.rawValue, perpetualService: GemPerpetualServiceMock(), swapQuoteService: GemSwapQuoteService())

        guard case .swapDetails = model.itemModel else {
            Issue.record("Expected .swapDetails")
            return
        }
    }

    @Test
    func transfer() {
        let model = ConfirmDetailsViewModel(type: .transfer(.mock()), metadata: nil, currency: Currency.usd.rawValue, perpetualService: GemPerpetualServiceMock(), swapQuoteService: GemSwapQuoteService())

        guard case .empty = model.itemModel else {
            Issue.record("Expected .empty")
            return
        }
    }

    @Test
    func perpetual() {
        let model = ConfirmDetailsViewModel(type: .perpetual(.mock(), .open(.mock())), metadata: nil, currency: Currency.usd.rawValue, perpetualService: GemPerpetualServiceMock(), swapQuoteService: GemSwapQuoteService())

        guard case .perpetualDetails = model.itemModel else {
            Issue.record("Expected .perpetualDetails")
            return
        }
    }
}

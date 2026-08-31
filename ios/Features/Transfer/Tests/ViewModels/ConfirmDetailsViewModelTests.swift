// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmDetailsViewModelTests {
    @Test
    func swap() {
        let model = ConfirmDetailsViewModel(type: .swap(.mock(), .mock(), .mock()), metadata: nil, currency: Currency.usd.rawValue, service: GemConfirmSceneServiceMock())

        guard case .swapDetails = model.itemModel else {
            Issue.record("Expected .swapDetails")
            return
        }
    }

    @Test
    func transfer() {
        let model = ConfirmDetailsViewModel(type: .transfer(.mock()), metadata: nil, currency: Currency.usd.rawValue, service: GemConfirmSceneServiceMock())

        guard case .empty = model.itemModel else {
            Issue.record("Expected .empty")
            return
        }
    }

    @Test
    func perpetual() {
        let model = ConfirmDetailsViewModel(type: .perpetual(.mock(), .open(.mock())), metadata: nil, currency: Currency.usd.rawValue, service: GemConfirmSceneServiceMock())

        guard case .perpetualDetails = model.itemModel else {
            Issue.record("Expected .perpetualDetails")
            return
        }
    }
}

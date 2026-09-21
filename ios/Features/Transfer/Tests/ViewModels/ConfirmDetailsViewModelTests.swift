// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmDetailsViewModelTests {
    @Test
    func swap() {
        let model = ConfirmDetailsViewModel(type: .swap(.mock(), .mock(), .mock()), metadata: nil, confirmation: GemConfirmationMock())

        guard case .swapDetails = model.itemModel else {
            Issue.record("Expected .swapDetails")
            return
        }
    }

    @Test
    func transfer() {
        let model = ConfirmDetailsViewModel(type: .transfer(.mock()), metadata: nil, confirmation: GemConfirmationMock())

        guard case .empty = model.itemModel else {
            Issue.record("Expected .empty")
            return
        }
    }

    @Test
    func perpetual() {
        let model = ConfirmDetailsViewModel(type: .perpetual(.mock(), .open(data: .mock())), metadata: nil, confirmation: GemConfirmationMock())

        guard case .perpetualDetails = model.itemModel else {
            Issue.record("Expected .perpetualDetails")
            return
        }
    }
}

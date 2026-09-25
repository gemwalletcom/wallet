// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Swap
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmDetailsViewModelTests {
    @Test
    func swap() {
        let model = ConfirmDetailsViewModel(type: .swap(fromAsset: Primitives.Asset.mock().toGem(), toAsset: Primitives.Asset.mock().toGem(), swapData: .mock()), metadata: nil, confirmation: GemConfirmationMock())

        guard case let .swapDetails(details) = model.itemModel else {
            Issue.record("Expected .swapDetails")
            return
        }
        #expect(details.allowSelectProvider == false)
    }

    @Test
    func transfer() {
        let model = ConfirmDetailsViewModel(type: .transfer(asset: Primitives.Asset.mock().toGem()), metadata: nil, confirmation: GemConfirmationMock())

        guard case .empty = model.itemModel else {
            Issue.record("Expected .empty")
            return
        }
    }

    @Test
    func perpetual() {
        let model = ConfirmDetailsViewModel(type: .perpetual(asset: Primitives.Asset.mock().toGem(), perpetualType: .open(data: .mock())), metadata: nil, confirmation: GemConfirmationMock())

        guard case .perpetualDetails = model.itemModel else {
            Issue.record("Expected .perpetualDetails")
            return
        }
    }
}

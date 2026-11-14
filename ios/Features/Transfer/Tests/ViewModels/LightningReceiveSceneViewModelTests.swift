// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemServiceError
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Primitives
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct LightningReceiveSceneViewModelTests {
    @Test
    func theScreenOpensOnAnAnyAmountInvoiceAndAnAmountReplacesIt() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)

        await model.onLoad()
        model.amount = "0.001"
        await model.onLoad()

        #expect(model.state == .ready(service.receive))
        #expect(service.receiveAmounts == ["", "0.001"])
        #expect(model.renderedImage != nil)
    }

    @Test
    func aLockedWalletAsksToUnlockAndAnyOtherFailureIsUnavailable() async {
        let locked = GemLightningServiceMock()
        locked.receiveError = GemServiceError.Cancelled
        let unavailable = GemLightningServiceMock()
        unavailable.receiveError = GemServiceError.Core(msg: "storage")
        let lockedModel = LightningReceiveSceneViewModel.mock(service: locked)
        let unavailableModel = LightningReceiveSceneViewModel.mock(service: unavailable)

        await lockedModel.onLoad()
        await unavailableModel.onLoad()

        #expect(lockedModel.state == .locked)
        #expect(unavailableModel.state == .unavailable)
    }

    @Test
    func anExpiredInvoiceIsReplacedAndAReceivedPaymentCompletes() async {
        let service = GemLightningServiceMock()
        var completed: ToastMessage?
        let model = LightningReceiveSceneViewModel.mock(service: service, onComplete: { completed = $0 })
        await model.onLoad()

        service.receiveState = .expired
        await model.onRefresh()
        #expect(service.receiveAmounts == ["", ""])
        #expect(completed == nil)

        service.receiveState = .received
        await model.onRefresh()
        #expect(completed != nil)
    }
}

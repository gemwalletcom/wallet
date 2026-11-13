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
    func theScreenOpensOnAnAnyAmountInvoice() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)

        await model.onChangeAmount()

        #expect(model.state == .ready(service.receive))
        #expect(service.receiveAmounts == [""])
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

        await lockedModel.onChangeAmount()
        await unavailableModel.onChangeAmount()

        #expect(lockedModel.state == .locked)
        #expect(unavailableModel.state == .unavailable)
    }

    @Test
    func anEnteredAmountAsksForANewInvoice() async {
        let service = GemLightningServiceMock()
        let model = LightningReceiveSceneViewModel.mock(service: service)

        model.amount = "0.001"
        await model.onChangeAmount()

        #expect(service.receiveAmounts == ["0.001"])
    }

    @Test
    func anExpiredInvoiceIsReplacedAndAReceivedPaymentCompletes() async {
        let service = GemLightningServiceMock()
        var completed: ToastMessage?
        let model = LightningReceiveSceneViewModel.mock(service: service, onComplete: { completed = $0 })
        await model.onChangeAmount()

        service.receiveState = .expired
        await model.onRefresh()
        #expect(service.receiveAmounts == ["", ""])
        #expect(completed == nil)

        service.receiveState = .received
        await model.onRefresh()
        #expect(completed != nil)
    }

    @Test
    func unlockingCreatesTheInvoice() async {
        let service = GemLightningServiceMock()
        service.receiveError = GemServiceError.Cancelled
        let model = LightningReceiveSceneViewModel.mock(service: service)
        await model.onChangeAmount()

        service.receiveError = nil
        await model.onUnlock()

        #expect(model.state == .ready(service.receive))
    }
}

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

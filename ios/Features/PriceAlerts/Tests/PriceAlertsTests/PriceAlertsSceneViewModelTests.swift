// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import PriceAlerts
import PriceAlertsTestKit
import Primitives
import Testing

@MainActor
struct PriceAlertsSceneViewModelTests {
    @Test
    func aFailedToggleShowsTheErrorAndKeepsTheStoredState() async {
        let model = PriceAlertsSceneViewModel.mock(service: GemPriceAlertServiceMock(setEnabledError: AnyError("offline")))
        model.isPriceAlertsEnabled = true

        await model.setAlertsEnabled(true)

        #expect(model.isPresentingAlertMessage?.message == "offline")
        #expect(model.isPriceAlertsEnabled == false)
    }
}

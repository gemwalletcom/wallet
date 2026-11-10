// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
@testable import PriceAlerts
import PriceAlertsTestKit
import Primitives
import PrimitivesTestKit
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

    @Test
    func addingAnAssetShowsTheAlertsTurnedOn() async {
        let model = PriceAlertsSceneViewModel.mock(service: GemPriceAlertServiceMock(enabled: false))

        _ = await model.includeAsset(.mock())

        #expect(model.isPriceAlertsEnabled == true)
    }

    @Test
    func addingAnAssetWithNotificationsRefusedKeepsTheSwitchOff() async {
        let model = PriceAlertsSceneViewModel.mock(service: GemPriceAlertServiceMock(enabled: false, allowsNotifications: false))

        _ = await model.includeAsset(.mock())

        #expect(model.isPriceAlertsEnabled == false)
    }
}

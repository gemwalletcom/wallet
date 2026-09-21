// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import PriceAlerts

public extension PriceAlertsSceneViewModel {
    @MainActor
    static func mock(service: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock()) -> PriceAlertsSceneViewModel {
        PriceAlertsSceneViewModel(service: service)
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import PriceAlerts
import Primitives
import PrimitivesTestKit

public extension AssetPriceAlertsSceneViewModel {
    @MainActor
    static func mock(service: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock()) -> AssetPriceAlertsSceneViewModel {
        AssetPriceAlertsSceneViewModel(
            service: service,
            walletId: .mock(),
            asset: .mock(),
        )
    }
}

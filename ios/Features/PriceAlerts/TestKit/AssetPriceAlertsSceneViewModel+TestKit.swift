// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import PriceAlerts
import Primitives
import PrimitivesTestKit

public extension AssetPriceAlertsSceneViewModel {
    @MainActor
    static func mock() -> AssetPriceAlertsSceneViewModel {
        AssetPriceAlertsSceneViewModel(
            service: GemPriceAlertServiceMock(),
            walletId: .mock(),
            asset: .mock(),
        )
    }
}

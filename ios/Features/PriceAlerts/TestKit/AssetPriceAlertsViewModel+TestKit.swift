// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import PriceAlerts
import Primitives
import PrimitivesTestKit

public extension AssetPriceAlertsViewModel {
    @MainActor
    static func mock() -> AssetPriceAlertsViewModel {
        AssetPriceAlertsViewModel(
            service: GemPriceAlertServiceMock(),
            walletId: .mock(),
            asset: .mock(),
        )
    }
}

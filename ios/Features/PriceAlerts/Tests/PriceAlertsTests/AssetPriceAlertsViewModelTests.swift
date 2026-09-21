// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import PriceAlerts
import PriceAlertsTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct AssetPriceAlertsViewModelTests {
    @Test
    func alertsModelSorting() {
        let alert1 = PriceAlertData.mock(priceAlert: .mock(price: 100, priceDirection: .up))
        let alert2 = PriceAlertData.mock(priceAlert: .mock(price: 200, priceDirection: .down))
        let alert3 = PriceAlertData.mock(priceAlert: .mock(price: 200, priceDirection: .up))
        let autoAlert = PriceAlertData.mock(priceAlert: .mock(priceDirection: nil))

        let model = AssetPriceAlertsViewModel.mock()
        model.query.value = [alert1, alert2, alert3, autoAlert]

        #expect(model.alerts == [alert3, alert2, alert1])
        #expect(model.isAutoAlertEnabledBinding.wrappedValue == true)
    }
}

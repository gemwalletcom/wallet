// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.emptyState
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import PriceAlerts
import PriceAlertsTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct AssetPriceAlertsSceneViewModelTests {
    @Test
    func alertsModelSorting() {
        let alert1 = PriceAlertData.mock(priceAlert: .mock(price: 100, priceDirection: .up))
        let alert2 = PriceAlertData.mock(priceAlert: .mock(price: 200, priceDirection: .down))
        let alert3 = PriceAlertData.mock(priceAlert: .mock(price: 200, priceDirection: .up))
        let autoAlert = PriceAlertData.mock(priceAlert: .mock(priceDirection: nil))

        let model = AssetPriceAlertsSceneViewModel.mock()
        model.query.value = [alert1, alert2, alert3, autoAlert]

        #expect(model.alerts(model.assetAlerts).map { $0.data.priceAlert.toPrimitives() } == [alert3, alert2, alert1].map(\.priceAlert))
        #expect(model.isAutoAlertEnabledBinding(model.assetAlerts).wrappedValue == true)
        #expect(model.assetAlerts.phase == .rows)
    }

    @Test
    func aFailedRefreshWithNoAlertsShowsTheErrorInsteadOfTheEmptyState() async {
        let service = GemPriceAlertServiceMock()
        let model = AssetPriceAlertsSceneViewModel.mock(service: service)
        #expect(model.assetAlerts.phase == .empty(state: emptyState(kind: .priceAlerts)))

        service.refreshState = .error(error: .Gateway(msg: "offline"))
        await model.load()

        #expect(model.assetAlerts.phase == .error(error: .Gateway(msg: "offline")))
    }
}

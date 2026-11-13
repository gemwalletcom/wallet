// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.emptyState
import struct Gemstone.GemPriceAlertItem
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

        let alerts = model.assetAlerts.list.groups.flatMap(\.items).compactMap { item -> GemPriceAlertItem? in
            guard case let .priceAlert(alert) = item else { return nil }
            return alert
        }
        #expect(alerts.map { $0.data.priceAlert.toPrimitives() } == [alert3, alert2, alert1].map(\.priceAlert))
        #expect(model.isAutoAlertEnabledBinding(model.assetAlerts).wrappedValue == true)
        #expect(model.assetAlerts.list.phase == .rows)
    }

    @Test
    func aFailedRefreshWithNoAlertsShowsTheErrorInsteadOfTheEmptyState() async {
        let service = GemPriceAlertServiceMock()
        let model = AssetPriceAlertsSceneViewModel.mock(service: service)
        #expect(model.assetAlerts.list.phase == .empty(state: emptyState(kind: .priceAlerts)))

        service.refreshState = .error(error: .Gateway(msg: "offline"))
        await model.load()

        #expect(model.assetAlerts.list.phase == .error(error: .Gateway(msg: "offline")))
    }
}

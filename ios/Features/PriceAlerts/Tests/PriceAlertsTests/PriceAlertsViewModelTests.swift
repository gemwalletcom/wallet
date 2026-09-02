// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import PriceAlerts
import Primitives
import PrimitivesTestKit
import Testing

struct PriceAlertsViewModelTests {
    @Test @MainActor
    func testSections() {
        let model = PriceAlertsSceneViewModel(priceAlertService: GemPriceAlertServiceMock())

        let autoAlert = PriceAlertData.mock()
        let manualAlert = PriceAlertData.mock(priceAlert: .mock(price: 5, priceDirection: .down))
        let manualSolAlert = PriceAlertData.mock(asset: .mockSolana(), priceAlert: .mock(assetId: .mockSolana(), price: 5, priceDirection: .down))
        let notifiedAlert = PriceAlertData.mock(priceAlert: .mock(price: 7, priceDirection: .down, lastNotifiedAt: Date()))

        let sections = model.sections(for: [autoAlert, manualAlert, manualSolAlert, notifiedAlert])

        #expect(sections.autoAlerts == [autoAlert])
        #expect(sections.manualAlerts[manualAlert.asset] == [manualAlert])
        #expect(sections.manualAlerts[manualSolAlert.asset] == [manualSolAlert])

        #expect(sections.manualAlerts.values.flatMap(\.self).contains(where: { $0 == notifiedAlert }) == false)
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import DeviceServiceTestKit
import Foundation
import GemstonePrimitivesTestKit
import NotificationServiceTestKit
@testable import PriceAlertService
import PriceAlertServiceTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import Testing

struct PriceAlertServiceTests {
    @Test
    func enableAlertRequestsPermissionsAndEnablesAlerts() async throws {
        let store = try createStore()
        let apiService = GemPriceAlertServiceMock()
        let pushNotificationService = PushNotificationEnablerMock()
        let deviceService = DeviceServiceMock()
        let service = PriceAlertService.mock(
            store: store,
            apiService: apiService,
            deviceService: deviceService,
            pushNotificationService: pushNotificationService,
        )

        try await service.enable(priceAlert: .mock(assetId: .mock(.bitcoin)))

        #expect(pushNotificationService.didRequestPermissions)
        #expect(try apiService.isEnabled())
        #expect(try store.getPriceAlerts().count == 1)
        #expect(await deviceService.updateCalls == 1)
    }

    @Test
    func enableAlertSyncsDeviceWhenPriceAlertsAlreadyEnabled() async throws {
        let store = try createStore()
        let deviceService = DeviceServiceMock()
        let service = PriceAlertService.mock(
            store: store,
            apiService: GemPriceAlertServiceMock(enabled: true),
            deviceService: deviceService,
        )

        try await service.enable(priceAlert: .mock(assetId: .mock(.bitcoin)))

        #expect(await deviceService.updateCalls == 1)
    }

    // MARK: - Private methods

    private func createStore(with alerts: [PriceAlert] = []) throws -> PriceAlertStore {
        let db = DB.mockAssets()
        let store = PriceAlertStore.mock(db: db)
        try store.addPriceAlerts(alerts)
        return store
    }

}

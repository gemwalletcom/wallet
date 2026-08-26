// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import DeviceServiceTestKit
import Foundation
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import NotificationService
import NotificationServiceTestKit
import PriceAlertService
import PriceService
import PriceServiceTestKit
import Store
import StoreTestKit

public extension PriceAlertService {
    static func mock(
        store: PriceAlertStore = .mock(),
        apiService: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock(),
        deviceService: any DeviceServiceable = DeviceServiceMock(),
        priceUpdater: any PriceUpdater = .mock(),
        pushNotificationService: any PushNotificationEnabler = PushNotificationEnablerMock(),
    ) -> PriceAlertService {
        PriceAlertService(
            store: store,
            apiService: apiService,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            pushNotificationService: pushNotificationService,
        )
    }
}

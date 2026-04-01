// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import DeviceServiceTestKit
import Foundation
import GemAPI
import GemAPITestKit
import Preferences
import PreferencesTestKit
import PriceAlertService
import PriceService
import PriceServiceTestKit
import Store
import StoreTestKit

public extension PriceAlertService {
    static func mock(
        store: PriceAlertStore = .mock(),
        apiService: any GemAPIPriceAlertService = GemAPIPriceAlertServiceMock(),
        deviceService: any DeviceServiceable = DeviceServiceMock(),
        priceUpdater: any PriceUpdater = .mock(),
        preferences: Preferences = .mock(),
    ) -> PriceAlertService {
        PriceAlertService(
            store: store,
            apiService: apiService,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            preferences: preferences,
        )
    }
}

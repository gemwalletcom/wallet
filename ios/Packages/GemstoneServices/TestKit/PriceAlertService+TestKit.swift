// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import PrimitivesTestKit
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit

public extension PriceAlertService {
    static func mock(
        service: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock(),
        deviceService: any DeviceServiceable = DeviceServiceMock(),
        priceUpdater: any PriceUpdater = .mock(),
        pushNotificationService: any PushNotificationEnabler = PushNotificationEnablerMock(),
    ) -> PriceAlertService {
        PriceAlertService(
            service: service,
            deviceService: deviceService,
            priceUpdater: priceUpdater,
            pushNotificationService: pushNotificationService,
        )
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import protocol Gemstone.GemDeviceServiceProtocol
import PrimitivesTestKit
import GemstoneServices
import Store
import StoreTestKit

public extension DeviceObserverService {
    static func mock(
        deviceService: any GemDeviceServiceProtocol = GemDeviceServiceMock(),
        subscriptionsObserver: SubscriptionsObserver = .mock(),
    ) -> DeviceObserverService {
        DeviceObserverService(
            deviceService: deviceService,
            subscriptionsObserver: subscriptionsObserver,
        )
    }
}

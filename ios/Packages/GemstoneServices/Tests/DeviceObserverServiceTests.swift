// Copyright (c). Gem Wallet. All rights reserved.

@testable import GemstoneServices
import PrimitivesTestKit
import GemstoneServicesTestKit
import StoreTestKit
import Testing

struct DeviceObserverServiceTests {
    @Test
    func handleSubscriptionsChangeUpdatesDevice() async throws {
        let deviceService = DeviceServiceMock()
        let observerService = DeviceObserverService(
            deviceService: deviceService,
            subscriptionsObserver: .mock(),
        )

        try await observerService.handleSubscriptionsChange()

        #expect(await deviceService.updateCalls == 1)
    }
}

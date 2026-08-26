// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public actor DeviceObserverService {
    private let deviceService: any DeviceServiceable
    private let subscriptionsObserver: SubscriptionsObserver


    public init(
        deviceService: any DeviceServiceable,
        subscriptionsObserver: SubscriptionsObserver,
    ) {
        self.deviceService = deviceService
        self.subscriptionsObserver = subscriptionsObserver
    }

    public func startSubscriptionsObserver() async throws {
        for try await _ in subscriptionsObserver.observe().dropFirst() {
            try await handleSubscriptionsChange()
        }
    }

    public func synchronizeIfNeeded() async throws {
        try await deviceService.synchronizeIfNeeded()
    }

    func handleSubscriptionsChange() async throws {
        try await deviceService.update()
    }
}

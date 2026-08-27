// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import Store

public actor DeviceObserverService {
    private let deviceService: any GemDeviceServiceProtocol
    private let subscriptionsObserver: SubscriptionsObserver


    public init(
        deviceService: any GemDeviceServiceProtocol,
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

    public func isRegistered() async throws -> Bool {
        try await deviceService.isRegistered()
    }

    public func synchronizeIfNeeded() async throws {
        try await deviceService.synchronizeIfNeeded()
    }

    func handleSubscriptionsChange() async throws {
        _ = try await deviceService.synchronize()
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public struct PriceAlertService: Sendable {
    private let store: PriceAlertStore
    private let service: any GemPriceAlertServiceProtocol
    private let deviceService: any DeviceServiceable
    private let priceUpdater: any PriceUpdater
    private let pushNotificationService: any PushNotificationEnabler

    public init(
        store: PriceAlertStore,
        service: any GemPriceAlertServiceProtocol,
        deviceService: any DeviceServiceable,
        priceUpdater: any PriceUpdater,
        pushNotificationService: any PushNotificationEnabler,
    ) {
        self.store = store
        self.service = service
        self.deviceService = deviceService
        self.priceUpdater = priceUpdater
        self.pushNotificationService = pushNotificationService
    }

    @discardableResult
    public func requestPermissions() async throws -> Bool {
        try await pushNotificationService.requestPermissions()
    }

    public func deviceUpdate() async throws {
        try await deviceService.update()
    }

    public func update() async throws {
        try await service.sync(assetId: .none)
    }

    public func update(assetId: String) async throws {
        try await service.sync(assetId: assetId)
    }

    public func enable(priceAlert: PriceAlert) async throws {
        try await add(priceAlert: priceAlert)
        try await requestPermissions()
        try await enablePriceAlerts()
    }

    public func add(priceAlert: PriceAlert) async throws {
        try store.addPriceAlerts([priceAlert])
        try await add(priceAlerts: [priceAlert])
        try await priceUpdater.addPrices(assetIds: [priceAlert.assetId])
    }

    public func add(priceAlerts: [PriceAlert]) async throws {
        try await service.addPriceAlerts(alerts: priceAlerts.map { try $0.json() })
    }

    public func isEnabled() throws -> Bool {
        try service.isEnabled()
    }

    public func setEnabled(_ enabled: Bool) throws {
        try service.setEnabled(enabled: enabled)
    }

    public func enablePriceAlerts() async throws {
        try setEnabled(true)
        try await deviceService.update()
    }

    public func delete(priceAlerts: [PriceAlert]) async throws {
        try store.deletePriceAlerts(priceAlerts.ids)
        try await service.deletePriceAlerts(alerts: priceAlerts.map { try $0.json() })
    }
}

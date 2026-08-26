// Copyright (c). Gem Wallet. All rights reserved.

import DeviceService
import Foundation
import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitives
import NotificationService
import PriceService
import Primitives
import Store

public struct PriceAlertService: Sendable {
    private let store: PriceAlertStore
    private let apiService: any GemPriceAlertServiceProtocol
    private let deviceService: any DeviceServiceable
    private let priceUpdater: any PriceUpdater
    private let pushNotificationService: any PushNotificationEnabler

    public init(
        store: PriceAlertStore,
        apiService: any GemPriceAlertServiceProtocol,
        deviceService: any DeviceServiceable,
        priceUpdater: any PriceUpdater,
        pushNotificationService: any PushNotificationEnabler,
    ) {
        self.store = store
        self.apiService = apiService
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
        try await apiService.sync(assetId: .none)
    }

    public func update(assetId: String) async throws {
        try await apiService.sync(assetId: assetId)
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
        try await apiService.addPriceAlerts(alerts: priceAlerts.map { try $0.json() })
    }

    public func isEnabled() throws -> Bool {
        try apiService.isEnabled()
    }

    public func setEnabled(_ enabled: Bool) throws {
        try apiService.setEnabled(enabled: enabled)
    }

    public func enablePriceAlerts() async throws {
        try setEnabled(true)
        try await deviceService.update()
    }

    public func delete(priceAlerts: [PriceAlert]) async throws {
        try store.deletePriceAlerts(priceAlerts.ids)
        try await apiService.deletePriceAlerts(alerts: priceAlerts.map { try $0.json() })
    }
}

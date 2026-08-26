// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import Preferences
import Primitives
import Store
import UIKit

public struct DeviceService: DeviceServiceable {
    private let deviceProvider: any GemDeviceServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let preferences: Preferences
    private let securePreferences: SecurePreferences
    private let syncCoordinator: DeviceSyncCoordinator

    public init(
        deviceProvider: any GemDeviceServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        preferences: Preferences = .standard,
        securePreferences: SecurePreferences = SecurePreferences(),
    ) {
        self.deviceProvider = deviceProvider
        self.preferencesService = preferencesService
        self.preferences = preferences
        self.securePreferences = securePreferences
        syncCoordinator = DeviceSyncCoordinator()
    }

    @discardableResult
    private static func getOrCreateDeviceId(securePreferences: SecurePreferences) throws -> String {
        try securePreferences.getDeviceId()
    }

    @discardableResult
    public static func getOrCreateKeyPair(securePreferences: SecurePreferences) throws -> (privateKey: Data, publicKey: Data) {
        try securePreferences.getOrCreateDeviceKeyPair()
    }

    public func update() async throws {
        try await synchronizeDevice()
    }

    public func synchronizeIfNeeded() async throws {
        try await syncCoordinator.waitForSyncIfNeeded()
        let deviceId = try getOrCreateDeviceId()
        guard try await deviceProvider.needsSync(device: currentDevice(deviceId: deviceId).json()) else { return }
        try await synchronizeDevice()
    }

    private func getOrCreateDeviceId() throws -> String {
        let storedDeviceId = try securePreferences.get(key: .deviceId)
        let deviceId = try Self.getOrCreateDeviceId(securePreferences: securePreferences)
        if storedDeviceId != deviceId {
            preferences.isDeviceRegistered = false
        }
        return deviceId
    }

    private func synchronizeDevice() async throws {
        try await syncCoordinator.coordinate {
            let deviceId = try getOrCreateDeviceId()
            _ = try await deviceProvider.sync(device: currentDevice(deviceId: deviceId).json())
        }
    }

    @MainActor
    private func currentDevice(deviceId: String) throws -> Primitives.Device {
        let deviceToken = try securePreferences.get(key: .deviceToken) ?? .empty
        return Primitives.Device(
            id: deviceId,
            platform: .ios,
            platformStore: .current,
            os: UIDevice.current.osName,
            model: UIDevice.current.modelName,
            token: deviceToken,
            locale: Locale.current.deviceLocale(),
            version: Bundle.main.releaseVersionNumber,
            currency: try Currency(id: preferences.currency),
            isPushEnabled: preferences.isPushNotificationsEnabled,
            isPriceAlertsEnabled: try preferencesService.isPriceAlertsEnabled(),
            subscriptionsVersion: 0,
        )
    }
}

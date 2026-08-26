// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Gemstone
import Preferences
import Primitives
import Store
import UIKit

public struct DeviceService: DeviceServiceable {
    private static let nodeAuthConfiguration = nodeAuthConfig()
    public static let nodeAuthTokenUpdateInterval: Duration = .seconds(nodeAuthConfiguration.checkIntervalSeconds)
    private static let nodeAuthTokenRefreshThreshold = UInt64(nodeAuthConfiguration.refreshThresholdSeconds)

    private let deviceProvider: any GemDeviceServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let preferences: Preferences
    private let securePreferences: SecurePreferences
    private let syncCoordinator: DeviceSyncCoordinator
    private static let nodeAuthTokenUpdateExecutor = SerialExecutor()

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
        try? await updateNodeAuthTokenIfNeeded()
    }

    public func synchronizeIfNeeded() async throws {
        try await syncCoordinator.waitForSyncIfNeeded()
        let deviceId = try getOrCreateDeviceId()
        guard try await deviceProvider.needsSync(device: currentDevice(deviceId: deviceId).json()) else { return }
        try await synchronizeDevice()
    }

    public func updateNodeAuthTokenIfNeeded() async throws {
        try await Self.nodeAuthTokenUpdateExecutor.execute {
            guard preferences.isDeviceRegistered, shouldUpdateNodeAuthToken() else { return }
            let nodeAuthToken = try await DeviceToken(deviceProvider.getToken())
            try securePreferences.setNodeAuthToken(nodeAuthToken)
        }
    }

    private func shouldUpdateNodeAuthToken() -> Bool {
        guard let token = try? securePreferences.nodeAuthToken() else { return true }
        let now = UInt64(Date.now.timeIntervalSince1970)
        let remainingTime = token.expiresAt > now ? token.expiresAt - now : 0
        return remainingTime < Self.nodeAuthTokenRefreshThreshold
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

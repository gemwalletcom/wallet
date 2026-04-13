// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Preferences
import Primitives
import Store
import UIKit

public struct DeviceService: DeviceServiceable {
    public static let nodeAuthTokenUpdateInterval: Duration = .seconds(300)

    private let deviceProvider: any GemAPIDeviceService
    private let subscriptionsService: SubscriptionService
    private let preferences: Preferences
    private let securePreferences: SecurePreferences
    private let syncCoordinator: DeviceSyncCoordinator
    private static let nodeAuthTokenUpdateExecutor = SerialExecutor()

    public init(
        deviceProvider: any GemAPIDeviceService,
        subscriptionsService: SubscriptionService,
        preferences: Preferences = .standard,
        securePreferences: SecurePreferences = SecurePreferences(),
    ) {
        self.deviceProvider = deviceProvider
        self.subscriptionsService = subscriptionsService
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

    public func prepareForWalletRequest() async throws {
        try await syncCoordinator.waitForSyncIfNeeded()
        _ = try getOrCreateDeviceId()
        guard !isReadyForWalletRequest else { return }
        try await synchronizeDevice()
    }

    private func updateDevice() async throws {
        let deviceId = try getOrCreateDeviceId()
        var device = try await getOrCreateDevice(deviceId)
        let localDevice = try await currentDevice(deviceId: deviceId)

        let needsSubscriptionUpdate = device.subscriptionsVersion != localDevice.subscriptionsVersion || preferences.subscriptionsVersionHasChange
        let needsDeviceUpdate = device != localDevice

        if needsSubscriptionUpdate {
            try await subscriptionsService.update()
        }

        if needsSubscriptionUpdate || needsDeviceUpdate {
            device = try await updateDevice(localDevice)
        }
    }

    public func updateNodeAuthTokenIfNeeded() async throws {
        try await Self.nodeAuthTokenUpdateExecutor.execute {
            guard preferences.isDeviceRegistered, shouldUpdateNodeAuthToken() else { return }
            let nodeAuthToken = try await deviceProvider.getNodeAuthToken()
            try securePreferences.setNodeAuthToken(nodeAuthToken)
        }
    }

    private func shouldUpdateNodeAuthToken() -> Bool {
        guard let token = try? securePreferences.nodeAuthToken() else { return true }
        let now = UInt64(Date.now.timeIntervalSince1970)
        let remainingTime = token.expiresAt > now ? token.expiresAt - now : 0
        return remainingTime < UInt64(Self.nodeAuthTokenUpdateInterval.components.seconds)
    }

    private func getOrCreateDevice(_ deviceId: String) async throws -> Device {
        var shouldFetchDevice = preferences.isDeviceRegistered
        if !shouldFetchDevice {
            shouldFetchDevice = try await deviceProvider.isDeviceRegistered()
        }

        if shouldFetchDevice {
            if let device = try await getDevice() {
                preferences.isDeviceRegistered = true
                return device
            }
            preferences.isDeviceRegistered = false
        }

        let device = try await currentDevice(deviceId: deviceId, ignoreSubscriptionsVersion: true)
        let result = try await addDevice(device)
        preferences.isDeviceRegistered = true
        return result
    }

    private func getOrCreateDeviceId() throws -> String {
        let storedDeviceId = try securePreferences.get(key: .deviceId)
        let deviceId = try Self.getOrCreateDeviceId(securePreferences: securePreferences)
        if storedDeviceId != deviceId {
            preferences.isDeviceRegistered = false
        }
        return deviceId
    }

    private var isReadyForWalletRequest: Bool {
        preferences.isDeviceRegistered
            && !preferences.subscriptionsVersionHasChange
    }

    private func synchronizeDevice() async throws {
        try await syncCoordinator.coordinate {
            try await updateDevice()
        }
    }

    @MainActor
    private func currentDevice(
        deviceId: String,
        ignoreSubscriptionsVersion: Bool = false,
    ) throws -> Device {
        let deviceToken = try securePreferences.get(key: .deviceToken) ?? .empty
        let locale = Locale.current.usageLanguageIdentifier()
        #if targetEnvironment(simulator)
            let platformStore = PlatformStore.local
        #else
            let platformStore = PlatformStore.appStore
        #endif

        return Device(
            id: deviceId,
            platform: .ios,
            platformStore: platformStore,
            os: UIDevice.current.osName,
            model: UIDevice.current.modelName,
            token: deviceToken,
            locale: locale,
            version: Bundle.main.releaseVersionNumber,
            currency: preferences.currency,
            isPushEnabled: preferences.isPushNotificationsEnabled,
            isPriceAlertsEnabled: preferences.isPriceAlertsEnabled,
            subscriptionsVersion: ignoreSubscriptionsVersion ? 0 : preferences.subscriptionsVersion.asInt32,
        )
    }

    private func getDevice() async throws -> Device? {
        try await deviceProvider.getDevice()
    }

    @discardableResult
    private func addDevice(_ device: Device) async throws -> Device {
        try await deviceProvider.addDevice(device: device)
    }

    @discardableResult
    private func updateDevice(_ device: Device) async throws -> Device {
        try await deviceProvider.updateDevice(device: device)
    }
}

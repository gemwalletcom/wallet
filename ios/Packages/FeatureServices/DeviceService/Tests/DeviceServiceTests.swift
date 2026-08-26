// Copyright (c). Gem Wallet. All rights reserved.

@testable import DeviceService
import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import protocol Gemstone.GemSubscriptionServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import StoreTestKit
import Testing

struct DeviceServiceTests {
    @Test
    func synchronizeIfNeededSkipsCleanState() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = false

        let securePreferences = SecurePreferences.mock()
        let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        try securePreferences.set(value: keyPair.publicKey.hex, key: .deviceId)

        let deviceProvider = GemDeviceServiceMock(
            isRegistered: false,
            getDeviceResult: nil,
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemSubscriptionServiceMock(),
            securePreferences: securePreferences,
        )

        try await service.synchronizeIfNeeded()

        #expect(await deviceProvider.isRegisteredCalls == 0)
        #expect(await deviceProvider.getDeviceCalls == 0)
        #expect(await deviceProvider.addDeviceCalls == 0)
        #expect(await deviceProvider.updateDeviceCalls == 0)
    }

    @Test
    func synchronizeIfNeededSharesInFlightSync() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = false
        preferences.subscriptionsVersionHasChange = true

        let deviceProvider = GemDeviceServiceMock(
            delay: .milliseconds(50),
            isRegistered: false,
            getDeviceResult: nil,
        )
        let subscriptionProvider = GemSubscriptionServiceMock(delay: .milliseconds(50))
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: subscriptionProvider,
        )

        async let first: Void = service.synchronizeIfNeeded()
        async let second: Void = service.synchronizeIfNeeded()
        _ = try await (first, second)

        #expect(await deviceProvider.isRegisteredCalls == 1)
        #expect(await deviceProvider.addDeviceCalls == 1)
        #expect(await deviceProvider.updateDeviceCalls == 1)
        #expect(await subscriptionProvider.getSubscriptionsCalls == 1)
        #expect(!preferences.subscriptionsVersionHasChange)
    }

    @Test
    func synchronizeIfNeededReplacesLegacyDeviceIdAndRegistersDevice() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = false

        let securePreferences = SecurePreferences.mock()
        try securePreferences.set(value: "legacy-device-id", key: .deviceId)

        let deviceProvider = GemDeviceServiceMock(
            isRegistered: false,
            getDeviceResult: nil,
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemSubscriptionServiceMock(),
            securePreferences: securePreferences,
        )

        try await service.synchronizeIfNeeded()

        #expect(await deviceProvider.addDeviceCalls == 1)
        let publicKey = try securePreferences.getData(key: .devicePublicKey)
        #expect(try securePreferences.get(key: .deviceId) == publicKey?.hex)
    }

    @Test
    func synchronizeIfNeededRegistersWhenMirroredDeviceIdIsMissing() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = false

        let securePreferences = SecurePreferences.mock()
        _ = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)

        let deviceProvider = GemDeviceServiceMock(
            isRegistered: false,
            getDeviceResult: nil,
        )
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: GemSubscriptionServiceMock(),
            securePreferences: securePreferences,
        )

        try await service.synchronizeIfNeeded()

        #expect(await deviceProvider.addDeviceCalls == 1)
        let publicKey = try securePreferences.getData(key: .devicePublicKey)
        #expect(try securePreferences.get(key: .deviceId) == publicKey?.hex)
    }

    @Test
    func synchronizeIfNeededWaitsForInFlightSyncBeforeFastPath() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        preferences.subscriptionsVersionHasChange = true

        let securePreferences = SecurePreferences.mock()
        let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        try securePreferences.set(value: keyPair.publicKey.hex, key: .deviceId)

        let deviceProvider = GemDeviceServiceMock(
            delay: .milliseconds(50),
            isRegistered: true,
            getDeviceResult: Device.mock(),
        )
        let subscriptionProvider = GemSubscriptionServiceMock(delay: .milliseconds(50))
        let service = makeService(
            preferences: preferences,
            deviceProvider: deviceProvider,
            subscriptionProvider: subscriptionProvider,
            securePreferences: securePreferences,
        )

        async let update: Void = service.update()
        async let ready: Void = service.synchronizeIfNeeded()
        _ = try await (update, ready)

        #expect(await deviceProvider.isRegisteredCalls == 0)
        #expect(await deviceProvider.getDeviceCalls == 1)
        #expect(await deviceProvider.addDeviceCalls == 0)
        #expect(await deviceProvider.updateDeviceCalls == 1)
        #expect(await deviceProvider.getTokenCalls == 1)
        #expect(await subscriptionProvider.getSubscriptionsCalls == 1)
        #expect(!preferences.subscriptionsVersionHasChange)
    }

    @Test
    func synchronizeIfNeededPropagatesSyncErrors() async {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = false
        preferences.subscriptionsVersionHasChange = true

        let service = makeService(
            preferences: preferences,
            deviceProvider: GemDeviceServiceMock(
                isRegistered: false,
                getDeviceResult: nil,
            ),
            subscriptionProvider: GemSubscriptionServiceMock(getSubscriptionsError: TestError.failed),
        )

        await #expect(throws: TestError.self) {
            try await service.synchronizeIfNeeded()
        }

        #expect(preferences.subscriptionsVersionHasChange)
    }
}

private extension DeviceServiceTests {
    func makeService(
        preferences: Preferences,
        deviceProvider: any GemDeviceServiceProtocol,
        subscriptionProvider: any GemSubscriptionServiceProtocol,
        securePreferences: SecurePreferences = .mock(),
    ) -> DeviceService {
        DeviceService(
            deviceProvider: deviceProvider,
            subscriptionsService: SubscriptionService(
                subscriptionProvider: subscriptionProvider,
                walletStore: .mock(),
                preferences: preferences,
            ),
            preferences: preferences,
            securePreferences: securePreferences,
        )
    }
}

private enum TestError: Error {
    case failed
}

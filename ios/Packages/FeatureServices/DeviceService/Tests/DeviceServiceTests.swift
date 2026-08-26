// Copyright (c). Gem Wallet. All rights reserved.

@testable import DeviceService
import Foundation
import protocol Gemstone.GemDeviceServiceProtocol
import GemstonePrimitivesTestKit
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import StoreTestKit
import Testing

struct DeviceServiceTests {
    @Test
    func synchronizeIfNeededSkipsWhenCoreReportsNothingToSync() async throws {
        let securePreferences = SecurePreferences.mock()
        let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        try securePreferences.set(value: keyPair.publicKey.hex, key: .deviceId)
        let deviceProvider = GemDeviceServiceMock(needsSync: false)
        let service = makeService(deviceProvider: deviceProvider, securePreferences: securePreferences)

        try await service.synchronizeIfNeeded()

        #expect(await deviceProvider.needsSyncCalls == 1)
        #expect(await deviceProvider.syncCalls == 0)
    }

    @Test
    func synchronizeIfNeededSharesInFlightSync() async throws {
        let deviceProvider = GemDeviceServiceMock(delay: .milliseconds(50), needsSync: true)
        let service = makeService(deviceProvider: deviceProvider)

        async let first: Void = service.synchronizeIfNeeded()
        async let second: Void = service.synchronizeIfNeeded()
        _ = try await (first, second)

        #expect(await deviceProvider.syncCalls == 1)
    }

    @Test
    func synchronizeIfNeededReplacesLegacyDeviceIdBeforeSyncing() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        let securePreferences = SecurePreferences.mock()
        try securePreferences.set(value: "legacy-device-id", key: .deviceId)
        let deviceProvider = GemDeviceServiceMock(needsSync: true)
        let service = makeService(preferences: preferences, deviceProvider: deviceProvider, securePreferences: securePreferences)

        try await service.synchronizeIfNeeded()

        let publicKey = try securePreferences.getData(key: .devicePublicKey)
        #expect(try securePreferences.get(key: .deviceId) == publicKey?.hex)
        #expect(!preferences.isDeviceRegistered)
        #expect(await deviceProvider.syncCalls == 1)
        #expect(await deviceProvider.syncedDeviceIds == [publicKey?.hex])
    }

    @Test
    func synchronizeIfNeededResetsRegistrationWhenMirroredDeviceIdIsMissing() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        let securePreferences = SecurePreferences.mock()
        _ = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        let deviceProvider = GemDeviceServiceMock(needsSync: true)
        let service = makeService(preferences: preferences, deviceProvider: deviceProvider, securePreferences: securePreferences)

        try await service.synchronizeIfNeeded()

        let publicKey = try securePreferences.getData(key: .devicePublicKey)
        #expect(try securePreferences.get(key: .deviceId) == publicKey?.hex)
        #expect(!preferences.isDeviceRegistered)
        #expect(await deviceProvider.syncCalls == 1)
    }

    @Test
    func synchronizeIfNeededWaitsForInFlightUpdateBeforeFastPath() async throws {
        let preferences = Preferences.mock()
        preferences.isDeviceRegistered = true
        let securePreferences = SecurePreferences.mock()
        let keyPair = try DeviceService.getOrCreateKeyPair(securePreferences: securePreferences)
        try securePreferences.set(value: keyPair.publicKey.hex, key: .deviceId)
        let deviceProvider = GemDeviceServiceMock(delay: .milliseconds(50), needsSync: false)
        let service = makeService(preferences: preferences, deviceProvider: deviceProvider, securePreferences: securePreferences)

        async let update: Void = service.update()
        async let ready: Void = service.synchronizeIfNeeded()
        _ = try await (update, ready)

        #expect(await deviceProvider.syncCalls == 1)
        #expect(await deviceProvider.getTokenCalls == 1)
    }

    @Test
    func synchronizeIfNeededPropagatesSyncErrors() async {
        let service = makeService(deviceProvider: GemDeviceServiceMock(needsSync: true, syncError: TestError.failed))

        await #expect(throws: TestError.self) {
            try await service.synchronizeIfNeeded()
        }
    }
}

private extension DeviceServiceTests {
    func makeService(
        preferences: Preferences = .mock(),
        deviceProvider: any GemDeviceServiceProtocol,
        securePreferences: SecurePreferences = .mock(),
    ) -> DeviceService {
        DeviceService(
            deviceProvider: deviceProvider,
            preferencesService: GemPreferencesServiceMock(),
            preferences: preferences,
            securePreferences: securePreferences,
        )
    }
}

private enum TestError: Error {
    case failed
}

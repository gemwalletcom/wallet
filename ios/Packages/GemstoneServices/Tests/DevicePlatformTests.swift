// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemDeviceKeyService
import GemstonePrimitivesTestKit
@testable import GemstoneServices
import Keychain
import Primitives
import Testing

@MainActor
struct DevicePlatformTests {
    @Test
    func pushTokenRoundTripsThroughTheKeychain() async throws {
        let keychain = RecordingKeychain()
        let platform = makePlatform(keychain: keychain)

        #expect(try await platform.pushToken() == "")

        try platform.setPushToken("token-1")

        #expect(try await platform.pushToken() == "token-1")
        #expect(try keychain.get("deviceToken") == "token-1")
        #expect(keychain.storage.accessibility(for: "deviceToken") == .whenUnlockedThisDeviceOnly)
    }

    @Test
    func clearDeviceEntriesRemovesTheLegacyDeviceEntriesAndTheToken() async throws {
        let keychain = RecordingKeychain()
        let platform = makePlatform(keychain: keychain)
        for key in ["deviceId", "deviceToken", "devicePrivateKey", "devicePublicKey", "gatewaydevice_private_key"] {
            try keychain.set("value", key: key)
        }

        try platform.clearDeviceEntries()

        #expect(try await platform.pushToken() == "")
        #expect(try keychain.get("deviceId") == nil)
        #expect(try keychain.get("devicePrivateKey") == nil)
        #expect(try keychain.get("devicePublicKey") == nil)
        #expect(try keychain.get("gatewaydevice_private_key") == "value")
    }

    private func makePlatform(keychain: RecordingKeychain) -> GemstoneDevicePlatform {
        GemstoneDevicePlatform(
            preferencesService: GemPreferencesServiceMock(),
            deviceKeyService: GemDeviceKeyService(store: GemSecureStoreMock()),
            keychain: keychain,
        )
    }
}

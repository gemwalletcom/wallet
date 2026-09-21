// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import Keychain
import Primitives
import Testing

struct SecurePreferencesStoreTests {
    @Test
    func newSecureValueUsesDeviceOnlyStorage() throws {
        let keychain = RecordingKeychain()
        let store = GemstoneSecurePreferencesStore(namespace: "gateway", keychain: keychain)

        try store.set(key: "credential", value: "test-value")

        #expect(keychain.storage.accessibility(for: "gatewaycredential") == .whenUnlockedThisDeviceOnly)
    }

    @Test
    func deviceKeyMigratesFromTheLegacyKeychainEntry() throws {
        let keychain = RecordingKeychain()
        let privateKey = Data(repeating: 0x04, count: 32)
        try keychain.set(privateKey, key: "devicePrivateKey")
        let store = GemstoneSecurePreferencesStore(namespace: "gateway", keychain: keychain)

        #expect(try store.get(key: "device_private_key") == privateKey.hex)
        #expect(try keychain.get("gatewaydevice_private_key") == privateKey.hex)
        #expect(keychain.storage.accessibility(for: "gatewaydevice_private_key") == .whenUnlockedThisDeviceOnly)
    }

    @Test
    func missingDeviceKeyStaysMissing() throws {
        let store = GemstoneSecurePreferencesStore(namespace: "gateway", keychain: RecordingKeychain())

        #expect(try store.get(key: "device_private_key") == nil)
        #expect(try store.get(key: "unrelated") == nil)
    }

    @Test
    func storedValueWinsOverTheLegacyEntry() throws {
        let keychain = RecordingKeychain()
        try keychain.set(Data(repeating: 0x04, count: 32), key: "devicePrivateKey")
        try keychain.set("current", key: "gatewaydevice_private_key")
        let store = GemstoneSecurePreferencesStore(namespace: "gateway", keychain: keychain)

        #expect(try store.get(key: "device_private_key") == "current")
    }
}

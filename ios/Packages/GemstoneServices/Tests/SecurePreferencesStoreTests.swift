// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstoneServices
import Keychain
import LocalAuthentication
import Primitives
import Testing

struct SecurePreferencesStoreTests {
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

private final class KeychainStorage: @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: Data] = [:]
    private var accessibilities: [String: Accessibility] = [:]

    func value(for key: String) -> Data? {
        lock.withLock { values[key] }
    }

    func accessibility(for key: String) -> Accessibility? {
        lock.withLock { accessibilities[key] }
    }

    func set(_ value: Data, key: String, accessibility: Accessibility) {
        lock.withLock {
            values[key] = value
            accessibilities[key] = accessibility
        }
    }

    func remove(key: String) {
        lock.withLock {
            values[key] = nil
            accessibilities[key] = nil
        }
    }
}

private struct RecordingKeychain: Keychain {
    let storage: KeychainStorage
    private let itemAccessibility: Accessibility

    init(storage: KeychainStorage = KeychainStorage(), accessibility: Accessibility = .afterFirstUnlock) {
        self.storage = storage
        itemAccessibility = accessibility
    }

    func accessibility(_ accessibility: Accessibility, authenticationPolicy _: AuthenticationPolicy) -> Keychain {
        RecordingKeychain(storage: storage, accessibility: accessibility)
    }

    func authenticationContext(_: LAContext) -> Keychain { self }

    func get(_ key: String, ignoringAttributeSynchronizable _: Bool) throws -> String? {
        try storage.value(for: key).map { try $0.encodeString() }
    }

    func getString(_ key: String, ignoringAttributeSynchronizable: Bool) throws -> String? {
        try get(key, ignoringAttributeSynchronizable: ignoringAttributeSynchronizable)
    }

    func getData(_ key: String, ignoringAttributeSynchronizable _: Bool) throws -> Data? {
        storage.value(for: key)
    }

    func set(_ value: String, key: String, ignoringAttributeSynchronizable: Bool) throws {
        try set(Data(value.utf8), key: key, ignoringAttributeSynchronizable: ignoringAttributeSynchronizable)
    }

    func set(_ value: Data, key: String, ignoringAttributeSynchronizable _: Bool) throws {
        storage.set(value, key: key, accessibility: itemAccessibility)
    }

    func remove(_ key: String, ignoringAttributeSynchronizable _: Bool) throws {
        storage.remove(key: key)
    }
}

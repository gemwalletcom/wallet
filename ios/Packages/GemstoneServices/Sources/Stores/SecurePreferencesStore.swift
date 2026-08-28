// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Keychain
import Primitives

public final class GemstoneSecurePreferencesStore: GemSecureStore, @unchecked Sendable {
    private static let deviceKeys = [
        "device_private_key": "devicePrivateKey",
        "device_public_key": "devicePublicKey",
    ]

    private let keychain: Keychain
    private let namespace: String

    public init(
        namespace: String,
        keychain: Keychain = KeychainDefault(),
    ) {
        self.namespace = namespace
        self.keychain = keychain
    }

    public func get(key: String) throws -> String? {
        if let value = try keychain.get(namespace + key) {
            return value
        }
        return try deviceKey(key)
    }

    private func storage(for key: String) -> Keychain {
        guard Self.deviceKeys[key] != nil else {
            return keychain
        }
        return keychain.accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: [])
    }

    private func deviceKey(_ key: String) throws -> String? {
        guard let legacyKey = Self.deviceKeys[key], let value = try keychain.getData(legacyKey) else {
            return .none
        }
        let hex = value.hex
        try set(key: key, value: hex)
        return hex
    }

    public func set(key: String, value: String) throws {
        try storage(for: key).set(value, key: namespace + key)
    }

    public func remove(key: String) throws {
        try keychain.remove(namespace + key)
    }
}

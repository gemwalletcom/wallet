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
        self.keychain = keychain.accessibility(.whenUnlockedThisDeviceOnly, authenticationPolicy: [])
    }

    public func get(key: String) throws -> String? {
        if let value = try keychain.get(namespace + key) {
            return value
        }
        return try deviceKey(key)
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
        try keychain.set(value, key: namespace + key)
    }

    public func remove(key: String) throws {
        try keychain.remove(namespace + key)
    }
}

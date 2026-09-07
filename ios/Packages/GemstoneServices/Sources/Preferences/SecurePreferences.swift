// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Keychain
import Primitives

public final class SecurePreferences: Sendable {
    private static let accessibility: Accessibility = .whenUnlockedThisDeviceOnly

    public enum Keys: String, CaseIterable {
        /// Deprecated. Use devicePublicKey
        case deviceId

        case deviceToken
        case devicePrivateKey
        case devicePublicKey
    }

    public static let standard = SecurePreferences()

    private let keychain: any KeychainPreferenceStorable

    public init(keychain: any KeychainPreferenceStorable = KeychainDefault()) {
        self.keychain = keychain
    }

    @discardableResult
    public func set(value: String, key: SecurePreferences.Keys) throws -> String {
        try keychain.set(value: value, key: key.rawValue, accessibility: Self.accessibility)
        return value
    }

    public func get(key: SecurePreferences.Keys) throws -> String? {
        try keychain.get(key: key.rawValue)
    }

    @discardableResult
    public func set(value: Data, key: SecurePreferences.Keys) throws -> Data {
        try keychain.set(value: value, key: key.rawValue, accessibility: Self.accessibility)
        return value
    }

    public func getData(key: SecurePreferences.Keys) throws -> Data? {
        try keychain.getData(key: key.rawValue)
    }

    public func delete(key: SecurePreferences.Keys) throws {
        try keychain.remove(key: key.rawValue)
    }

    public func clear() throws {
        for key in Keys.allCases {
            try delete(key: key)
        }
    }
}

extension KeychainDefault: KeychainPreferenceStorable {
    public func set(value: String, key: String, accessibility: Accessibility) throws {
        try self.accessibility(accessibility, authenticationPolicy: []).set(value, key: key)
    }

    public func get(key: String) throws -> String? {
        try get(key, ignoringAttributeSynchronizable: true)
    }

    public func set(value: Data, key: String, accessibility: Accessibility) throws {
        try self.accessibility(accessibility, authenticationPolicy: []).set(value, key: key)
    }

    public func getData(key: String) throws -> Data? {
        try getData(key, ignoringAttributeSynchronizable: true)
    }

    public func remove(key: String) throws {
        try remove(key, ignoringAttributeSynchronizable: true)
    }
}

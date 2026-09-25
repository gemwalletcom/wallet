// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keychain
import LocalAuthentication

final class KeychainStorage: @unchecked Sendable {
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

struct RecordingKeychain: Keychain {
    let storage: KeychainStorage
    private let itemAccessibility: Accessibility

    init(storage: KeychainStorage = KeychainStorage(), accessibility: Accessibility = .afterFirstUnlock) {
        self.storage = storage
        itemAccessibility = accessibility
    }

    func accessibility(_ accessibility: Accessibility, authenticationPolicy _: AuthenticationPolicy) -> Keychain {
        RecordingKeychain(storage: storage, accessibility: accessibility)
    }

    func authenticationContext(_: LAContext) -> Keychain {
        self
    }

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

    func add(_ value: String, key: String, ignoringAttributeSynchronizable _: Bool) throws -> Bool {
        guard storage.value(for: key) == nil else {
            return false
        }
        storage.set(Data(value.utf8), key: key, accessibility: itemAccessibility)
        return true
    }

    func remove(_ key: String, ignoringAttributeSynchronizable _: Bool) throws {
        storage.remove(key: key)
    }
}

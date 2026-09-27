// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keychain
import LocalAuthentication
import Primitives

final class KeychainStorage: @unchecked Sendable {
    private let lock = NSLock()
    private var values: [String: Data] = [:]
    private var accessibilities: [String: Accessibility] = [:]
    private var authenticationPolicies: [String: AuthenticationPolicy] = [:]
    private var writes: [String: Int] = [:]
    private var readErrors: [String: AnyError] = [:]
    private var writeErrors: [String: AnyError] = [:]

    func value(for key: String) -> Data? {
        lock.withLock { values[key] }
    }

    func accessibility(for key: String) -> Accessibility? {
        lock.withLock { accessibilities[key] }
    }

    func authenticationPolicy(for key: String) -> AuthenticationPolicy? {
        lock.withLock { authenticationPolicies[key] }
    }

    func writeCount(for key: String) -> Int {
        lock.withLock { writes[key, default: 0] }
    }

    func setReadError(_ error: AnyError, key: String) {
        lock.withLock { readErrors[key] = error }
    }

    func setWriteError(_ error: AnyError, key: String) {
        lock.withLock { writeErrors[key] = error }
    }

    func readError(for key: String) -> AnyError? {
        lock.withLock { readErrors[key] }
    }

    func writeError(for key: String) -> AnyError? {
        lock.withLock { writeErrors[key] }
    }

    func set(_ value: Data, key: String, accessibility: Accessibility, authenticationPolicy: AuthenticationPolicy = []) {
        lock.withLock {
            values[key] = value
            accessibilities[key] = accessibility
            authenticationPolicies[key] = authenticationPolicy
            writes[key, default: 0] += 1
        }
    }

    func remove(key: String) {
        lock.withLock {
            values[key] = nil
            accessibilities[key] = nil
            authenticationPolicies[key] = nil
        }
    }
}

struct RecordingKeychain: Keychain {
    let storage: KeychainStorage
    private let itemAccessibility: Accessibility
    private let itemAuthenticationPolicy: AuthenticationPolicy

    init(storage: KeychainStorage = KeychainStorage(), accessibility: Accessibility = .afterFirstUnlock, authenticationPolicy: AuthenticationPolicy = []) {
        self.storage = storage
        itemAccessibility = accessibility
        itemAuthenticationPolicy = authenticationPolicy
    }

    func accessibility(_ accessibility: Accessibility, authenticationPolicy: AuthenticationPolicy) -> Keychain {
        RecordingKeychain(storage: storage, accessibility: accessibility, authenticationPolicy: authenticationPolicy)
    }

    func authenticationContext(_: LAContext) -> Keychain {
        self
    }

    func get(_ key: String) throws -> String? {
        try getData(key).map { try $0.encodeString() }
    }

    func getData(_ key: String) throws -> Data? {
        if let error = storage.readError(for: key) {
            throw error
        }
        return storage.value(for: key)
    }

    func set(_ value: String, key: String) throws {
        try set(Data(value.utf8), key: key)
    }

    func set(_ value: Data, key: String) throws {
        if let error = storage.writeError(for: key) {
            throw error
        }
        storage.set(value, key: key, accessibility: itemAccessibility, authenticationPolicy: itemAuthenticationPolicy)
    }

    func remove(_ key: String) throws {
        storage.remove(key: key)
    }
}

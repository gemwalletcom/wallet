// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Keychain

final class GemstoneSecurePreferences: GemPreferences, @unchecked Sendable {
    private let keychain: Keychain
    private let namespace: String

    init(
        namespace: String,
        keychain: Keychain = KeychainDefault(),
    ) {
        self.namespace = namespace
        self.keychain = keychain
    }

    func get(key: String) throws -> String? {
        try keychain.get(namespace + key)
    }

    func set(key: String, value: String) throws {
        try keychain.set(value, key: namespace + key)
    }

    func remove(key: String) throws {
        try keychain.remove(namespace + key)
    }
}

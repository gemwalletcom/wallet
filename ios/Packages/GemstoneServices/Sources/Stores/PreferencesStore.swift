// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public final class GemstonePreferencesStore: GemPreferencesStore, @unchecked Sendable {
    private let userDefaults: UserDefaults
    private let namespace: String
    private let sharedKeys: Set<String>
    private let keyAliases: [String: String]

    public init(
        namespace: String,
        sharedKeys: Set<String> = [],
        keyAliases: [String: String] = [:],
        userDefaults: UserDefaults = .standard,
    ) {
        self.namespace = namespace
        self.sharedKeys = sharedKeys
        self.keyAliases = keyAliases
        self.userDefaults = userDefaults
    }

    public func get(key: String) -> String? {
        userDefaults.string(forKey: storageKey(key))
    }

    public func set(key: String, value: String) throws {
        userDefaults.set(value, forKey: storageKey(key))
    }

    public func remove(key: String) throws {
        userDefaults.removeObject(forKey: storageKey(key))
    }

    private func storageKey(_ key: String) -> String {
        keyAliases[key] ?? (sharedKeys.contains(key) ? key : namespace + key)
    }
}

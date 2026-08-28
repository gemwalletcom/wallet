// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public final class GemstonePreferencesStore: GemPreferencesStore, @unchecked Sendable {
    private let namespace: String
    private let unprefixedKeys: Set<String>
    private let keyAliases: [String: String]
    private let appGroupKeys: Set<String>
    private let userDefaults: UserDefaults
    private let appGroupDefaults: UserDefaults?

    public init(
        namespace: String,
        unprefixedKeys: Set<String> = [],
        keyAliases: [String: String] = [:],
        appGroupKeys: Set<String> = [],
        userDefaults: UserDefaults = .standard,
        appGroupDefaults: UserDefaults? = .none,
    ) {
        self.namespace = namespace
        self.unprefixedKeys = unprefixedKeys
        self.keyAliases = keyAliases
        self.appGroupKeys = appGroupKeys
        self.userDefaults = userDefaults
        self.appGroupDefaults = appGroupDefaults
    }

    public func get(key: String) -> String? {
        userDefaults.string(forKey: storageKey(key))
    }

    public func set(key: String, value: String) throws {
        userDefaults.set(value, forKey: storageKey(key))
        if appGroupKeys.contains(key) {
            appGroupDefaults?.set(value, forKey: storageKey(key))
        }
    }

    public func remove(key: String) throws {
        userDefaults.removeObject(forKey: storageKey(key))
        if appGroupKeys.contains(key) {
            appGroupDefaults?.removeObject(forKey: storageKey(key))
        }
    }

    public func clear() throws {
        for key in userDefaults.dictionaryRepresentation().keys where isManaged(key) {
            userDefaults.removeObject(forKey: key)
        }
        for key in appGroupKeys {
            appGroupDefaults?.removeObject(forKey: storageKey(key))
        }
    }

    private func storageKey(_ key: String) -> String {
        keyAliases[key] ?? (unprefixedKeys.contains(key) ? key : namespace + key)
    }

    private func isManaged(_ key: String) -> Bool {
        key.hasPrefix(namespace) || unprefixedKeys.contains(key) || keyAliases.values.contains(key)
    }
}

public extension GemstonePreferencesStore {
    private static let namespace = "gemstone_"

    private static let unprefixedKeys: Set<String> = [
        "currency",
        "appearance",
        "is_perpetual_enabled",
    ]

    private static let keyAliases: [String: String] = [
        "current_wallet_id": "currentWallet",
        "is_hide_balance_enabled": "is_balance_privacy_enabled",
        "is_accept_terms_completed": "is_accepted_terms",
    ]

    private static let appGroupKeys: Set<String> = ["currency"]

    static func application(userDefaults: UserDefaults = .standard) -> GemstonePreferencesStore {
        GemstonePreferencesStore(
            namespace: namespace,
            unprefixedKeys: unprefixedKeys,
            keyAliases: keyAliases,
            appGroupKeys: appGroupKeys,
            userDefaults: userDefaults,
            appGroupDefaults: UserDefaults(suiteName: Constants.appGroupIdentifier),
        )
    }
}

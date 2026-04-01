// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

final class GemstonePreferences: GemPreferences, @unchecked Sendable {
    private let userDefaults: UserDefaults
    private let namespace: String

    init(
        namespace: String,
        userDefaults: UserDefaults = .standard,
    ) {
        self.namespace = namespace
        self.userDefaults = userDefaults
    }

    func get(key: String) throws -> String? {
        userDefaults.string(forKey: namespace + key)
    }

    func set(key: String, value: String) throws {
        userDefaults.set(value, forKey: namespace + key)
    }

    func remove(key: String) throws {
        userDefaults.removeObject(forKey: namespace + key)
    }
}

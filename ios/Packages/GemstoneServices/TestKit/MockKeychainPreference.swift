// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Keychain

public final class MockKeychainPreference: KeychainPreferenceStorable, @unchecked Sendable {
    private let storage: UserDefaults

    public init(storage: UserDefaults) {
        self.storage = storage
    }

    public func set(value: String, key: String, accessibility: Accessibility) throws {
        storage.set(value, forKey: key)
        storage.set(accessibility.rawValue, forKey: accessibilityKey(key))
    }

    public func get(key: String) throws -> String? {
        storage.string(forKey: key)
    }

    public func set(value: Data, key: String, accessibility: Accessibility) throws {
        storage.set(value, forKey: key)
        storage.set(accessibility.rawValue, forKey: accessibilityKey(key))
    }

    public func getData(key: String) throws -> Data? {
        storage.data(forKey: key)
    }

    public func remove(key: String) throws {
        storage.removeObject(forKey: key)
        storage.removeObject(forKey: accessibilityKey(key))
    }

    public func accessibility(for key: String) -> Accessibility? {
        storage.string(forKey: accessibilityKey(key)).flatMap(Accessibility.init(rawValue:))
    }

    private func accessibilityKey(_ key: String) -> String {
        "\(key).accessibility"
    }
}

public extension MockKeychainPreference {
    static func mock(storage: UserDefaults = .mock()) -> any KeychainPreferenceStorable {
        MockKeychainPreference(storage: storage)
    }
}

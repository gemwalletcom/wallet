// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletPreferencesStore
import typealias Gemstone.WalletId

public final class GemstoneWalletPreferencesStore: GemWalletPreferencesStore, Sendable {
    public init() {}

    public func get(walletId: Gemstone.WalletId, key: String) -> String? {
        defaults(walletId: walletId).preferenceValue(forKey: key)
    }

    public func set(walletId: Gemstone.WalletId, key: String, value: String) throws {
        defaults(walletId: walletId).set(value, forKey: key)
    }

    public func deletePreferences(walletId: Gemstone.WalletId) throws {
        UserDefaults.standard.removePersistentDomain(forName: suiteName(walletId: walletId))
    }

    private func defaults(walletId: Gemstone.WalletId) -> UserDefaults {
        UserDefaults(suiteName: suiteName(walletId: walletId)) ?? .standard
    }

    private func suiteName(walletId: Gemstone.WalletId) -> String {
        "wallet_preferences_\(walletId)_v2"
    }
}

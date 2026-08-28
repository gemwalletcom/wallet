// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstoneServices
import Testing

struct GemstonePreferencesStoreTests {
    @Test
    func readsValuesStoredByTheLegacyAppPreferences() throws {
        let defaults = makeDefaults(#function)
        let store = GemstonePreferencesStore.application(userDefaults: defaults, appGroupDefaults: .none)

        defaults.set(true, forKey: "is_price_alerts_enabled")
        defaults.set(false, forKey: "is_balance_privacy_enabled")
        defaults.set("multicoin_0x1", forKey: "currentWallet")
        defaults.set(20, forKey: "perpetual_leverage")
        defaults.set("Etherscan", forKey: "explorer_name_ethereum")

        #expect(store.get(key: "price_alerts_enabled") == "true")
        #expect(store.get(key: "is_hide_balance_enabled") == "false")
        #expect(store.get(key: "current_wallet_id") == "multicoin_0x1")
        #expect(store.get(key: "perpetual_leverage") == "20")
        #expect(store.get(key: "explorer_name_ethereum") == "Etherscan")
    }

    @Test
    func mirrorsCurrencyIntoTheAppGroup() throws {
        let defaults = makeDefaults(#function)
        let appGroupDefaults = makeDefaults("\(#function).group")
        let store = GemstonePreferencesStore.application(userDefaults: defaults, appGroupDefaults: appGroupDefaults)

        try store.set(key: "currency", value: "EUR")

        #expect(appGroupDefaults.string(forKey: "currency") == "EUR")
    }

    @Test
    func clearRemovesOwnedKeysOnly() throws {
        let defaults = makeDefaults(#function)
        let store = GemstonePreferencesStore.application(userDefaults: defaults, appGroupDefaults: .none)

        try store.set(key: "currency", value: "EUR")
        try store.set(key: "is_developer_enabled", value: "true")
        try store.set(key: "current_wallet_id", value: "multicoin_0x1")
        defaults.set("keep", forKey: "unrelated_key")

        try store.clear()

        #expect(store.get(key: "currency") == nil)
        #expect(store.get(key: "is_developer_enabled") == nil)
        #expect(store.get(key: "current_wallet_id") == nil)
        #expect(defaults.string(forKey: "unrelated_key") == "keep")
    }

    private func makeDefaults(_ suiteName: String) -> UserDefaults {
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }
}

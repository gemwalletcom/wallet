// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import Keystore
import KeystoreTestKit
import Preferences
import Primitives
import PreferencesTestKit
import Store
import StoreTestKit

public extension WalletService {
    func mockWallets() throws -> [Wallet] {
        try mockWalletStore.getWallets()
    }

    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletService {
        let gemWalletStore = GemstoneWalletStore(store: walletStore)
        let session = GemWalletSessionService(store: GemstoneWalletSessionStore(preferences: preferences), wallets: gemWalletStore)
        return WalletService(
            service: GemWalletService(
                keystore: keystore.gemKeystore,
                password: GemstoneKeystorePassword(keystore: keystore),
                store: gemWalletStore,
                session: session,
                deviceStore: GemstoneDeviceStore(preferences: preferences.preferences),
            ),
            keystore: keystore,
            walletStore: walletStore,
            preferences: preferences,
            avatarService: AvatarService(store: walletStore),
        )
    }

    static func mock(isAcceptedTerms: Bool) -> Self {
        .mock(
            preferences: .mock(
                preferences: .mock(
                    defaults: .mockWithValues(values: ["is_accepted_terms": isAcceptedTerms]),
                ),
            ),
        )
    }
}

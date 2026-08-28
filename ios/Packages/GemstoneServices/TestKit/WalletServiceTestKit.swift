// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import class Gemstone.GemPreferencesService
import class Gemstone.GemWalletPreferencesService
import GemstoneServices
import Foundation
import class Gemstone.GemWalletService
import class Gemstone.GemWalletSessionService
import Preferences
import Primitives
import PreferencesTestKit
import Store
import StoreTestKit

public extension WalletService {
    static func mock(
        keystore: LocalKeystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
        preferencesStore: GemPreferencesStoreMock = GemPreferencesStoreMock(),
    ) -> WalletService {
        let gemWalletStore = GemstoneWalletStore(store: walletStore)
        let session = GemWalletSessionService(store: GemstoneWalletSessionStore(preferences: preferences), wallets: gemWalletStore)
        let gemWalletService = GemWalletService(
            keystore: keystore.gemKeystore,
            password: GemstoneKeystorePassword(keystore: keystore),
            store: gemWalletStore,
            session: session,
            appPreferences: GemPreferencesService(store: preferencesStore),
            files: GemstoneFileStore(),
            preferences: GemWalletPreferencesService.mock(),
        )
        return WalletService(
            service: gemWalletService,
            keystore: keystore,
            walletSessionService: WalletSessionService(service: session),
            preferences: preferences,
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

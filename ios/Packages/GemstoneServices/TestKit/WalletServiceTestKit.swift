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
        sessionStore: GemstoneWalletSessionStore = .mock(),
        preferences: ObservablePreferences = .mock(),
        preferencesStore: GemPreferencesStoreMock = GemPreferencesStoreMock(),
    ) -> WalletService {
        let gemWalletStore = GemstoneWalletStore(store: walletStore)
        let session = GemWalletSessionService(store: sessionStore, wallets: gemWalletStore)
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
        let preferencesStore = GemPreferencesStoreMock()
        if isAcceptedTerms {
            try? preferencesStore.set(key: "is_accept_terms_completed", value: "true")
        }
        return .mock(preferences: .mock(preferencesService: GemPreferencesService(store: preferencesStore)))
    }
}

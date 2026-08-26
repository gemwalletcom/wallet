// Copyright (c). Gem Wallet. All rights reserved.

import AvatarService
import Foundation
import Keystore
import KeystoreTestKit
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit
import WalletService
import GemstoneServices
import GemstoneServicesTestKit

public extension WalletService {
    static func mock(
        keystore: any Keystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletService {
        mock(
            keystore: keystore,
            walletStore: walletStore,
            preferences: preferences,
            walletSessionService: WalletSessionService.mock(store: walletStore, preferences: preferences),
        )
    }

    static func mock(
        keystore: any Keystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
        walletSessionService: any WalletSessionManageable,
    ) -> WalletService {
        WalletService(
            keystore: keystore,
            walletStore: walletStore,
            preferences: preferences,
            avatarService: AvatarService(store: walletStore),
            walletSessionService: walletSessionService,
        )
    }

    static func mock(isAcceptedTerms: Bool) -> Self {
        .mock(
            keystore: KeystoreMock(),
            preferences: .mock(
                preferences: .mock(
                    defaults: .mockWithValues(values: ["is_accepted_terms": isAcceptedTerms]),
                ),
            ),
        )
    }
}

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

public extension WalletService {
    static func mock(
        keystore: any Keystore = LocalKeystore.mock(),
        walletStore: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> WalletService {
        WalletService(
            keystore: keystore,
            walletStore: walletStore,
            preferences: preferences,
            avatarService: AvatarService(store: walletStore),
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

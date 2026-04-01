// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit
import WalletSessionService

public extension WalletSessionService {
    static func mock(
        store: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> any WalletSessionManageable {
        WalletSessionService(
            walletStore: store,
            preferences: preferences,
        )
    }

    static func mock(wallet: Wallet) throws -> WalletSessionService {
        let db = DB.mock()
        let store = WalletStore.mock(db: db)
        try store.addWallet(wallet)
        return WalletSessionService(
            walletStore: store,
            preferences: .mock(),
        )
    }
}

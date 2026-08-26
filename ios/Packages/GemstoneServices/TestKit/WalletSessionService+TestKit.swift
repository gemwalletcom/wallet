// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemWalletSessionService
import GemstoneServices
import Preferences
import PreferencesTestKit
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension WalletSessionService {
    static func mock(
        store: WalletStore = .mock(),
        preferences: ObservablePreferences = .mock(),
    ) -> any WalletSessionManageable {
        WalletSessionService(
            service: GemWalletSessionService(store: GemstoneWalletSessionStore(preferences: preferences), wallets: GemstoneWalletStore(store: store)),
            walletStore: store,
        )
    }

    static func mock(wallet: Wallet) throws -> WalletSessionService {
        let db = DB.mock()
        let store = WalletStore.mock(db: db)
        try store.addWallet(wallet)
        return WalletSessionService(
            service: GemWalletSessionService(store: GemstoneWalletSessionStore(preferences: .mock()), wallets: GemstoneWalletStore(store: store)),
            walletStore: store,
        )
    }
}

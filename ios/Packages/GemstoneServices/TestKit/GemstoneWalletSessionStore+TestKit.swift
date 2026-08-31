// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import GemstoneServices

public extension GemstoneWalletSessionStore {
    static func mock() -> GemstoneWalletSessionStore {
        GemstoneWalletSessionStore(store: GemPreferencesStoreMock())
    }
}

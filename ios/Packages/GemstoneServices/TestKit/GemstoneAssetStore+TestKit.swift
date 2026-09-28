// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Store
import StoreTestKit

public extension GemstoneAssetStore {
    static func mock(db: DB = .mock()) -> GemstoneAssetStore {
        GemstoneAssetStore(assetStore: .mock(db: db), balanceStore: .mock(db: db))
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import Preferences
import PreferencesTestKit
import Store
import StoreTestKit

public extension WalletSearchService {
    static func mock(
        assetsService: AssetsService = .mock(),
        searchStore: SearchStore = .mock(),
        perpetualStore: PerpetualStore = .mock(),
        assetListStore: AssetListStore = .mock(),
        priceStore: PriceStore = .mock(),
        preferences: Preferences = .mock(),
    ) -> WalletSearchService {
        WalletSearchService(
            assetsService: assetsService,
            searchStore: searchStore,
            perpetualStore: perpetualStore,
            assetListStore: assetListStore,
            priceStore: priceStore,
            preferences: preferences,
        )
    }
}

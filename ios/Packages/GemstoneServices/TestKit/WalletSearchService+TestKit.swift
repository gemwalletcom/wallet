// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import PrimitivesTestKit
import class Gemstone.GemAssetsService
import protocol Gemstone.GemAssetsServiceProtocol
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
        priceService: PriceService = .mock(),
        preferences: Preferences = .mock(),
        searchProvider: any GemAssetsServiceProtocol = GemAssetsService.mock(),
    ) -> WalletSearchService {
        WalletSearchService(
            assetsService: assetsService,
            searchStore: searchStore,
            perpetualStore: perpetualStore,
            assetListStore: assetListStore,
            priceService: priceService,
            preferences: preferences,
            searchProvider: searchProvider,
        )
    }
}

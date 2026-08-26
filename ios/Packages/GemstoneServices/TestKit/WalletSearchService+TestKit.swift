// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemPriceService
import protocol Gemstone.GemPriceServiceProtocol
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
        priceService: any GemPriceServiceProtocol = GemPriceService.mock(),
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

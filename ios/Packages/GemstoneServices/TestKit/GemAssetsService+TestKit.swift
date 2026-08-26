// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemAssetsService
import class Gemstone.GemPreferencesService
import class Gemstone.GemPriceService
import GemstonePrimitivesTestKit
import GemstoneStore
import NativeProviderService
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension GemAssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
        priceService: GemPriceService = .mock(),
    ) -> GemAssetsService {
        GemAssetsService(
            api: GemApiClient(
                provider: NativeProvider(url: Constants.apiURL),
                baseUrl: Constants.apiURL.absoluteString,
            ),
            store: GemstoneAssetStore(assetStore: assetStore, balanceStore: balanceStore),
            price: priceService,
            preferences: GemPreferencesService(store: GemPreferencesStoreMock()),
        )
    }
}

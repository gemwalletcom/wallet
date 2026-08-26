// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import class Gemstone.GemApiClient
import class Gemstone.GemAssetsService
import NativeProviderService
import Primitives
import Store
import StoreTestKit

public extension GemAssetsService {
    static func mock(
        assetStore: AssetStore = .mock(),
        balanceStore: BalanceStore = .mock(),
    ) -> GemAssetsService {
        GemAssetsService(
            api: GemApiClient(
                provider: NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor()),
                baseUrl: Constants.apiURL.absoluteString,
            ),
            store: GemstoneAssetStore(assetStore: assetStore, balanceStore: balanceStore),
        )
    }
}

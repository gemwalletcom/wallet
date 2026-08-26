// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAssetsServiceProtocol
import GemstoneServices
import Primitives
import PrimitivesTestKit
import Store
import StoreTestKit

public extension AssetSearchService {
    static func mock(
        assetsService: any GemAssetsServiceProtocol = GemAssetsServiceMock(),
        assetStore: AssetStore = .mock(),
        searchStore: SearchStore = .mock(),
    ) -> AssetSearchService {
        AssetSearchService(
            assetsService: assetsService,
            assetStore: assetStore,
            searchStore: searchStore,
        )
    }
}

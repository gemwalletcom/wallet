// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import Primitives
import Store
import StoreTestKit

public extension AssetSearchService {
    static func mock(
        assetsService: AssetsService = .mock(),
        searchStore: SearchStore = .mock(),
    ) -> AssetSearchService {
        AssetSearchService(
            assetsService: assetsService,
            searchStore: searchStore,
        )
    }
}

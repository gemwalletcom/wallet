// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public extension WalletSearchResult {
    static func mock(
        assets: [AssetData] = [],
        perpetuals: [PerpetualData] = [],
        collections: [NFTData] = [],
        lists: [AssetList] = [],
    ) -> WalletSearchResult {
        WalletSearchResult(assets: assets, perpetuals: perpetuals, collections: collections, lists: lists)
    }
}

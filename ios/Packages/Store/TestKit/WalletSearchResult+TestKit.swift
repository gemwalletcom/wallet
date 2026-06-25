// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Store

public extension WalletSearchResult {
    static func mock(
        assets: [AssetData] = [],
        perpetuals: [PerpetualData] = [],
        lists: [AssetList] = [],
    ) -> WalletSearchResult {
        WalletSearchResult(assets: assets, perpetuals: perpetuals, lists: lists)
    }
}

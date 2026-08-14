// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import Store

public extension TransactionsRequestFilter {
    static var activityDefaults: [TransactionsRequestFilter] {
        [.assetRankGreaterThan(AssetScore.defaultScore)]
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAssetConfigService
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store

public extension TransactionsRequestFilter {
    static var activityDefaults: [TransactionsRequestFilter] {
        [.assetRankGreaterThan(GemAssetConfigService.shared.defaultTokenRank().asInt)]
    }
}

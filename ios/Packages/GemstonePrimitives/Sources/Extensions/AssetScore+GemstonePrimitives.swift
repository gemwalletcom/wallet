// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAssetConfigService
import Primitives

public extension AssetScore {
    /// default score of a token asset, not assigned
    static var defaultScore: Int {
        GemAssetConfigService.shared.defaultTokenRank().asInt
    }
}

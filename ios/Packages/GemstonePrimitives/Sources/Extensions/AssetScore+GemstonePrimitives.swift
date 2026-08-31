// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import Foundation
import Primitives

private let assetConfig = GemAssetConfigService()

public extension AssetScore {
    /// default score of a token asset, not assigned
    static var defaultScore: Int {
        assetConfig.defaultTokenRank().asInt
    }

    static func defaultScore(chain: Chain) -> AssetScore {
        AssetScore(
            rank: AssetScore.defaultRank(chain: chain).asInt32,
        )
    }

    /// from 0 to 100. anything below is 0 is not good
    static func defaultRank(chain: Chain) -> Int {
        assetConfig.defaultRank(assetId: chain.assetId.identifier).asInt
    }
}

// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.assetDefaultRank
import Primitives

public extension AssetScore {
    /// default score of an asset, not assigned
    static let defaultScore = 15

    static func defaultScore(chain: Chain) -> AssetScore {
        AssetScore(
            rank: AssetScore.defaultRank(chain: chain).asInt32,
        )
    }

    /// from 0 to 100. anything below is 0 is not good
    static func defaultRank(chain: Chain) -> Int {
        Gemstone.assetDefaultRank(chain: chain.rawValue).asInt
    }
}

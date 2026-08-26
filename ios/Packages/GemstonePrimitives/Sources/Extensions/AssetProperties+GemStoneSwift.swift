// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.assetDefaultRank
import func Gemstone.assetIsSwapable
import Primitives

public extension AssetProperties {
    static func defaultValue(assetId: AssetId) -> AssetProperties {
        let isEnabled = AssetScore.defaultValue(assetId: assetId).rank >= 0
        let isStakeable = switch assetId.type {
        case .native: isEnabled && assetId.chain.isStakeSupported
        case .token: false
        }
        return AssetProperties(
            isEnabled: isEnabled,
            isBuyable: false,
            isSellable: false,
            isSwapable: Gemstone.assetIsSwapable(assetId: assetId.identifier),
            isStakeable: isStakeable,
            stakingApr: .none,
            isEarnable: false,
            earnApr: .none,
            hasImage: false,
        )
    }
}

public extension AssetScore {
    static func defaultValue(assetId: AssetId) -> AssetScore {
        AssetScore(rank: Gemstone.assetDefaultRank(assetId: assetId.identifier))
    }
}

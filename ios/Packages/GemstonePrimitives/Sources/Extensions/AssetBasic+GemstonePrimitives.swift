// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAssetConfigService
import Foundation
import class Gemstone.Config
import Primitives

private let assetConfig = GemAssetConfigService()

public extension AssetBasic {
    static func native(_ asset: Asset) -> AssetBasic {
        let chain = asset.chain
        let score = AssetScore.defaultScore(chain: chain)
        let isEnabled = score.rank >= 0
        let config = Config.shared.getChainConfig(chain: chain.rawValue)
        return AssetBasic(
            asset: asset,
            properties: AssetProperties(
                isEnabled: isEnabled,
                isBuyable: score.rank >= 40,
                isSellable: false,
                isSwapable: assetConfig.isSwapable(assetId: asset.id.identifier),
                isStakeable: isEnabled && config.isStakeSupported,
                stakingApr: .none,
                isEarnable: false,
                earnApr: .none,
                hasImage: isEnabled,
            ),
            score: score,
            price: nil,
        )
    }

    static func seed(_ asset: Asset) -> AssetBasic {
        AssetBasic(
            asset: asset,
            properties: AssetProperties(
                isEnabled: true,
                isBuyable: false,
                isSellable: false,
                isSwapable: false,
                isStakeable: false,
                stakingApr: .none,
                isEarnable: false,
                earnApr: .none,
                hasImage: false,
            ),
            score: AssetScore.defaultValue(assetId: asset.id),
            price: nil,
        )
    }
}

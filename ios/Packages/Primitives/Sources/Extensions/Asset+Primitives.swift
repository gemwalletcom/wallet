// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension Asset: Identifiable {}

public extension Asset {
    var chain: Chain {
        id.chain
    }

    var tokenId: String? {
        id.tokenId
    }
}

public extension AssetFull {
    var basic: AssetBasic {
        AssetBasic(
            asset: asset,
            properties: properties,
            score: score,
            price: price,
        )
    }
}

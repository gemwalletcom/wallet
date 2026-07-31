// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension AssetFull {
    static func mock(
        asset: Asset = .mock(),
        properties: AssetProperties = .mock(),
        score: AssetScore = .mock(),
        tags: [String] = [],
        links: [AssetLink] = [],
        associations: [AssetAssociation] = [],
        perpetuals: [PerpetualBasic] = [],
        price: Price? = nil,
        market: AssetMarket? = nil,
    ) -> Self {
        AssetFull(
            asset: asset,
            properties: properties,
            score: score,
            tags: tags,
            links: links,
            associations: associations,
            perpetuals: perpetuals,
            price: price,
            market: market,
        )
    }
}

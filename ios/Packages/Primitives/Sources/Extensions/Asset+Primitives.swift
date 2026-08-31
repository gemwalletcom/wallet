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

    func getTokenId() throws -> String {
        try id.getTokenId()
    }
}

public extension [Asset] {
    var ids: [String] {
        map(\.id.identifier)
    }

    var assetIds: [AssetId] {
        map(\.id)
    }
}

public extension [Chain] {
    var ids: [AssetId] {
        compactMap(\.assetId)
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
